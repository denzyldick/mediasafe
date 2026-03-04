use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::protocol::Message,
    tungstenite::Utf8Bytes,
};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    ice_transport::{ice_candidate::RTCIceCandidateInit, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncProgress {
    pub device_id: String,
    pub status: String,
    pub progress: f32, // 0.0 to 100.0
    pub bytes_per_second: u64,
    pub items_completed: u32,
    pub items_total: u32,
}

// Use the same SignalMessage structure as the axum server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalMessage {
    #[serde(rename = "join")]
    Join { device_id: String },
    #[serde(rename = "offer")]
    Offer { payload: String, target: String },
    #[serde(rename = "answer")]
    Answer { payload: String, target: String },
    #[serde(rename = "ice_candidate")]
    IceCandidate { payload: String, target: String },
    #[serde(rename = "peer_disconnected")]
    PeerDisconnected { device_id: String },
    #[serde(rename = "error")]
    Error { message: String },
}

pub struct WebRtcClient {
    pub room_id: String,
    pub is_initiator: bool,
    pub signaling_url: String,
    pub app_handle: AppHandle,
}

use tauri::AppHandle;

impl WebRtcClient {
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.app_handle.emit("webrtc-state", "Connecting to signaling...");
        // 1. Establish WebSocket Connection to Signaling Server
        let url_str = format!("{}/ws/{}", self.signaling_url, self.room_id);
        println!("Connecting to signaling server: {}", url_str);

        let req = url_str.into_client_request()?;

        // Connect to WebSocket using tokio-tungstenite
        let (ws_stream, _) = match connect_async(req).await {
            Ok(s) => s,
            Err(e) => {
                let err_msg = format!("Signaling connection failed: {}", e);
                println!("{}", err_msg);
                let _ = self.app_handle.emit("webrtc-state", err_msg);
                return Err(e.into());
            }
        };
        println!("WebSocket connected!");
        let _ = self.app_handle.emit("webrtc-state", "Connected to signaling. Waiting for peer...");
        let (write, mut read) = ws_stream.split();

        let write = Arc::new(Mutex::new(write));

        // 2. Identify ourselves to the Signaling Server
        let my_device_id = uuid::Uuid::new_v4().to_string();
        let join_msg = SignalMessage::Join {
            device_id: my_device_id.clone(),
        };
        write
            .lock()
            .await
            .send(Message::Text(Utf8Bytes::from(serde_json::to_string(
                &join_msg,
            )?)))
            .await?;

        // 3. Configure WebRTC (STUN)
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut m = MediaEngine::default();
        m.register_default_codecs()?;
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)?;

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        // 4. Handle ICE Candidates Discovered Locally
        // When we discover a route to ourselves, send it to the other peer via the signaling server
        let write_ice = Arc::clone(&write);
        let my_id_ice = my_device_id.clone();
        peer_connection.on_ice_candidate(Box::new(
            move |c: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
                let write_ice = Arc::clone(&write_ice);
                let _my_id_ice = my_id_ice.clone();
                Box::pin(async move {
                    if let Some(c) = c {
                        if let Ok(json) = c.to_json() {
                            if let Ok(payload) = serde_json::to_string(&json) {
                                let msg = SignalMessage::IceCandidate {
                                    payload,
                                    target: "peer".to_string(),
                                };
                                if let Ok(msg_str) = serde_json::to_string(&msg) {
                                    write_ice
                                        .lock()
                                        .await
                                        .send(Message::Text(Utf8Bytes::from(msg_str)))
                                        .await;
                                }
                            }
                        }
                    }
                })
            },
        ));

        // 5. Setup Data Channel
        if self.is_initiator {
            let data_channel = peer_connection
                .create_data_channel("file_transfer", None)
                .await?;

            let app_handle_dc = self.app_handle.clone();
            let dc_clone = Arc::clone(&data_channel);
            data_channel.on_open(Box::new(move || {
                println!("Data channel opened! Ready to send files.");
                let _ = app_handle_dc.emit("webrtc-state", "Secure Data Channel Ready");
                let dc_inner = Arc::clone(&dc_clone);
                Box::pin(async move {
                    // TODO: Replace with dynamic file list passed from UI
                    let test_payload = bytes::Bytes::from_static(b"INIT_FILE_TRANSFER:example.txt");
                    if let Err(e) = dc_inner.send(&test_payload).await {
                        println!("Failed to send data: {}", e);
                    }
                })
            }));
            let app_handle_msg = self.app_handle.clone();
            data_channel.on_message(Box::new(
                move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                    let content = String::from_utf8_lossy(&msg.data);
                    println!("Sender received ack: {}", content);
                    let _ = app_handle_msg.emit("webrtc-state", format!("Received: {}", content));
                    Box::pin(async move {})
                },
            ));

            // Create Offer
            let offer = peer_connection.create_offer(None).await?;
            peer_connection.set_local_description(offer.clone()).await?;

            let offer_msg = SignalMessage::Offer {
                payload: serde_json::to_string(&offer)?,
                target: "peer".to_string(),
            };
            write
                .lock()
                .await
                .send(Message::Text(Utf8Bytes::from(serde_json::to_string(
                    &offer_msg,
                )?)))
                .await?;
            println!("Sent WebRTC Offer");
        } else {
            // Receiver waits for Data Channel
            let app_handle_dc = self.app_handle.clone();
            peer_connection.on_data_channel(Box::new(move |d: Arc<webrtc::data_channel::RTCDataChannel>| {
                println!("Data channel created by initiator!");
                let _ = app_handle_dc.emit("webrtc-state", "Data Channel Established");

                let d_clone = Arc::clone(&d);
                let app_handle_msg = app_handle_dc.clone();
                d.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                    let content = String::from_utf8_lossy(&msg.data);
                    println!("Receiver got data chunk (len: {})", msg.data.len());
                    let _ = app_handle_msg.emit("webrtc-state", format!("Received: {}", content));
                    // If it's the transfer init signal...
                    if msg.data.starts_with(b"INIT_FILE_TRANSFER:") {
                       println!("Begin receiving file: {}", content);

                       // Send ACK
                       let dc = Arc::clone(&d_clone);
                       return Box::pin(async move {
                           let ack_payload = bytes::Bytes::from_static(b"ACK");
                           let _ = dc.send(&ack_payload).await;
                       });
                    }

                    // Otherwise it's raw binary data
                    // TODO: Write chunk to disk sequentially
                    Box::pin(async move {})
                }));
                Box::pin(async move {})
            }));
        }

        // Maintain connection state
        let app_handle_state = self.app_handle.clone();
        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                let status = match s {
                    RTCPeerConnectionState::Connected => "Connected",
                    RTCPeerConnectionState::Connecting => "Connecting WebRTC...",
                    RTCPeerConnectionState::Disconnected => "Peer Disconnected",
                    RTCPeerConnectionState::Failed => "Connection Failed",
                    RTCPeerConnectionState::New => "Waiting for peer...",
                    _ => "Awaiting connection...",
                };
                println!("Peer Connection State has changed: {}", s);
                let _ = app_handle_state.emit("webrtc-state", status);
                Box::pin(async move {})
            },
        ));

        // 6. Handle Incoming Signaling Messages
        let pc = Arc::clone(&peer_connection);
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(signal) = serde_json::from_str::<SignalMessage>(&text) {
                        match signal {
                            SignalMessage::Offer { payload, .. } => {
                                println!("Received Offer");
                                let sdp: RTCSessionDescription = serde_json::from_str(&payload)?;
                                pc.set_remote_description(sdp).await?;
                                let answer = pc.create_answer(None).await?;
                                pc.set_local_description(answer.clone()).await?;

                                let answer_msg = SignalMessage::Answer {
                                    payload: serde_json::to_string(&answer)?,
                                    target: "peer".to_string(),
                                };
                                write
                                    .lock()
                                    .await
                                    .send(Message::Text(Utf8Bytes::from(serde_json::to_string(
                                        &answer_msg,
                                    )?)))
                                    .await?;
                                println!("Sent WebRTC Answer");
                            }
                            SignalMessage::Answer { payload, .. } => {
                                println!("Received Answer");
                                let sdp: RTCSessionDescription = serde_json::from_str(&payload)?;
                                pc.set_remote_description(sdp).await?;
                            }
                            SignalMessage::IceCandidate { payload, .. } => {
                                println!("Received remote ICE candidate");
                                let candidate: RTCIceCandidateInit = serde_json::from_str(&payload)?;
                                pc.add_ice_candidate(candidate).await?;
                            }
                            SignalMessage::PeerDisconnected { .. } => {
                                println!("Peer disconnected");
                                let _ = self.app_handle.emit("webrtc-state", "Peer disconnected");
                            }
                            SignalMessage::Error { message } => {
                                println!("Signaling error: {}", message);
                                let _ = self.app_handle.emit("webrtc-state", format!("Signaling error: {}", message));
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    println!("WebSocket error: {}", e);
                    let _ = self.app_handle.emit("webrtc-state", format!("WebSocket error: {}", e));
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
