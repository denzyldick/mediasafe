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

use crate::database::{Database, PhotoSyncInfo};
use std::collections::HashMap;
use std::path::Path;
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncProgress {
    pub device_id: String,
    pub status: String,
    pub progress: f32, // 0.0 to 100.0
    pub bytes_per_second: u64,
    pub items_completed: u32,
    pub items_total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    ManifestRequest,
    ManifestResponse {
        photos: Vec<PhotoSyncInfo>,
    },
    FileRequest {
        id: String,
    },
    FileHeader {
        id: String,
        filename: String,
        size: u64,
        created: String,
    },
    FileChunk {
        id: String,
        data: Vec<u8>,
    },
    FileEnd {
        id: String,
    },
}

// Use the same SignalMessage structure as the axum server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalMessage {
    #[serde(rename = "join")]
    Join { device_id: String },
    #[serde(rename = "joined")]
    Joined {
        device_id: String,
        room_id: String,
        peer_count: usize,
    },
    #[serde(rename = "offer")]
    Offer { payload: String, target: String },
    #[serde(rename = "answer")]
    Answer { payload: String, target: String },
    #[serde(rename = "ice_candidate")]
    IceCandidate { payload: String, target: String },
    #[serde(rename = "peer_disconnected")]
    PeerDisconnected { device_id: String },
    #[serde(rename = "peer_joined")]
    PeerJoined { device_id: String },
    #[serde(rename = "error")]
    Error { message: String },
}

pub struct WebRtcClient {
    pub room_id: String,
    pub is_initiator: bool,
    pub signaling_url: String,
    pub app_handle: Option<AppHandle>,
    pub config_path: String,
}

use tauri::AppHandle;

struct IncomingFile {
    id: String,
    filename: String,
    size: u64,
    received: u64,
    created: String,
    file: tokio::fs::File,
}

use warp::Filter;

pub struct MediaServerState {
    pub port: u16,
}

pub fn start_media_server(_config_path: String) -> u16 {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (addr, server) = warp::serve(
                warp::path("media")
                    .and(warp::fs::dir("/"))
                    .with(warp::cors().allow_any_origin()),
            )
            .bind_ephemeral(([127, 0, 0, 1], 0));

            let port = addr.port();
            let _ = tx.send(port);
            server.await;
        });
    });

    rx.recv().unwrap_or(0)
}

impl WebRtcClient {
    fn emit(&self, event: &str, payload: impl Serialize + Clone) {
        if let Some(app) = &self.app_handle {
            let _ = app.emit(event, payload);
        }
    }

    async fn send_sync_message(
        dc: &Arc<webrtc::data_channel::RTCDataChannel>,
        msg: &SyncMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(msg)?;
        dc.send(&bytes::Bytes::from(json)).await?;
        Ok(())
    }

    async fn send_file(
        dc: Arc<webrtc::data_channel::RTCDataChannel>,
        photo_id: String,
        file_path: String,
        created: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&file_path);
        if !path.exists() {
            return Err(format!("File not found: {file_path}").into());
        }

        let mut file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let size = metadata.len();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        // 1. Send Header
        Self::send_sync_message(
            &dc,
            &SyncMessage::FileHeader {
                id: photo_id.clone(),
                filename,
                size,
                created,
            },
        )
        .await?;

        // 2. Send Chunks
        let mut buffer = vec![0u8; 65536]; // 64KB chunks
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            Self::send_sync_message(
                &dc,
                &SyncMessage::FileChunk {
                    id: photo_id.clone(),
                    data: buffer[..n].to_vec(),
                },
            )
            .await?;

            // Flow control: Wait if buffer is too full (e.g. > 1MB)
            while dc.buffered_amount().await > 1_000_000 {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }

        // 3. Send End
        Self::send_sync_message(&dc, &SyncMessage::FileEnd { id: photo_id }).await?;

        Ok(())
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Attempting to connect to signaling at {} for room {}",
            self.signaling_url, self.room_id
        );
        self.emit("webrtc-state", "Connecting to signaling...");
        let url_str = format!("{}/{}", self.signaling_url, self.room_id);
        let req = url_str.into_client_request()?;

        let (ws_stream, _) = match connect_async(req).await {
            Ok(s) => {
                println!("Successfully connected to signaling server!");
                s
            }
            Err(e) => {
                let err_msg = format!("Signaling connection failed: {e}");
                println!("Signaling connection failed: {e}");
                self.emit("webrtc-state", err_msg);
                return Err(e.into());
            }
        };
        self.emit(
            "webrtc-state",
            "Connected to signaling. Waiting for peer...",
        );
        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        let my_device_id = uuid::Uuid::new_v4().to_string();
        println!("Joining network with device ID: {my_device_id}");
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

        let write_ice = Arc::clone(&write);
        peer_connection.on_ice_candidate(Box::new(
            move |c: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
                let write_ice = Arc::clone(&write_ice);
                Box::pin(async move {
                    if let Some(c) = c {
                        if let Ok(json) = c.to_json() {
                            if let Ok(payload) = serde_json::to_string(&json) {
                                let msg = SignalMessage::IceCandidate {
                                    payload,
                                    target: "peer".to_string(),
                                };
                                if let Ok(msg_str) = serde_json::to_string(&msg) {
                                    let _ = write_ice
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

        let incoming_files: Arc<Mutex<HashMap<String, IncomingFile>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let config_path_clone = self.config_path.clone();
        let _app_handle_outer = self.app_handle.clone();

        if self.is_initiator {
            let data_channel = peer_connection
                .create_data_channel("file_transfer", None)
                .await?;
            let dc_clone = Arc::clone(&data_channel);
            let app_handle_opt = self.app_handle.clone();

            data_channel.on_open(Box::new(move || {
                println!("Data channel opened! Requesting manifest...");
                if let Some(app) = &app_handle_opt {
                    let _ = app.emit("webrtc-state", "Secure Data Channel Ready");
                }
                let dc_inner = Arc::clone(&dc_clone);
                Box::pin(async move {
                    let _ = Self::send_sync_message(&dc_inner, &SyncMessage::ManifestRequest).await;
                })
            }));

            let dc_clone_msg = Arc::clone(&data_channel);
            let incoming_files_clone = Arc::clone(&incoming_files);
            let config_path_dc = config_path_clone.clone();
            let app_handle_opt_msg = self.app_handle.clone();

            data_channel.on_message(Box::new(
                move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                    let dc = Arc::clone(&dc_clone_msg);
                    let incoming_files = Arc::clone(&incoming_files_clone);
                    let config_path = config_path_dc.clone();
                    let app_handle = app_handle_opt_msg.clone();

                    Box::pin(async move {
                        let text = String::from_utf8_lossy(&msg.data);
                        if let Ok(sync_msg) = serde_json::from_str::<SyncMessage>(&text) {
                            match sync_msg {
                                SyncMessage::ManifestResponse { photos } => {
                                    let db = Database::new(&config_path);
                                    let my_manifest = db.get_photo_sync_info();
                                    for peer_photo in photos {
                                        if !my_manifest.iter().any(|p| p.id == peer_photo.id) {
                                            let _ = Self::send_sync_message(
                                                &dc,
                                                &SyncMessage::FileRequest { id: peer_photo.id },
                                            )
                                            .await;
                                        }
                                    }
                                }
                                SyncMessage::FileHeader {
                                    id,
                                    filename,
                                    size,
                                    created,
                                } => {
                                    let save_path =
                                        Path::new(&config_path).join("sync_temp").join(&filename);
                                    let _ = tokio::fs::create_dir_all(save_path.parent().unwrap())
                                        .await;
                                    if let Ok(file) = tokio::fs::File::create(&save_path).await {
                                        let mut incoming = incoming_files.lock().await;
                                        incoming.insert(
                                            id.clone(),
                                            IncomingFile {
                                                id,
                                                filename,
                                                size,
                                                received: 0,
                                                created,
                                                file,
                                            },
                                        );
                                    }
                                }
                                SyncMessage::FileChunk { id, data } => {
                                    let mut incoming = incoming_files.lock().await;
                                    if let Some(file_state) = incoming.get_mut(&id) {
                                        let _ = file_state.file.write_all(&data).await;
                                        file_state.received += data.len() as u64;
                                        if let Some(app) = &app_handle {
                                            let progress = (file_state.received as f32
                                                / file_state.size as f32)
                                                * 100.0;
                                            let _ = app.emit(
                                                "sync-progress",
                                                SyncProgress {
                                                    device_id: "peer".to_string(),
                                                    status: format!(
                                                        "Receiving {}",
                                                        file_state.filename
                                                    ),
                                                    progress,
                                                    bytes_per_second: 0,
                                                    items_completed: 0,
                                                    items_total: 0,
                                                },
                                            );
                                        }
                                    }
                                }
                                SyncMessage::FileEnd { id } => {
                                    let mut incoming = incoming_files.lock().await;
                                    if let Some(file_state) = incoming.remove(&id) {
                                        let temp_path = Path::new(&config_path)
                                            .join("sync_temp")
                                            .join(&file_state.filename);
                                        let db = Database::new(&config_path);
                                        let dirs = db.list_directories();
                                        let target_dir = if !dirs.is_empty() {
                                            Path::new(&dirs[0])
                                        } else {
                                            &Path::new(&config_path).join("Siegu")
                                        };
                                        let _ = tokio::fs::create_dir_all(&target_dir).await;
                                        let final_path = target_dir.join(&file_state.filename);
                                        if let Ok(_) =
                                            tokio::fs::rename(&temp_path, &final_path).await
                                        {
                                            db.import_photo(
                                                &file_state.id,
                                                &final_path.to_string_lossy(),
                                                &file_state.created,
                                            );
                                        }
                                    }
                                }
                                SyncMessage::ManifestRequest => {
                                    let db = Database::new(&config_path);
                                    let photos = db.get_photo_sync_info();
                                    let _ = Self::send_sync_message(
                                        &dc,
                                        &SyncMessage::ManifestResponse { photos },
                                    )
                                    .await;
                                }
                                SyncMessage::FileRequest { id } => {
                                    let db = Database::new(&config_path);
                                    if let Ok((path, created)) = db.connection.query_row(
                                        "SELECT location, created FROM photo WHERE id = ?1",
                                        [&id],
                                        |row| {
                                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                                        },
                                    ) {
                                        let dc_send = Arc::clone(&dc);
                                        tokio::spawn(async move {
                                            let _ =
                                                Self::send_file(dc_send, id, path, created).await;
                                        });
                                    }
                                }
                            }
                        }
                    })
                },
            ));
        } else {
            let app_handle_opt = self.app_handle.clone();
            let incoming_files_clone = Arc::clone(&incoming_files);
            let config_path_dc = config_path_clone.clone();

            peer_connection.on_data_channel(Box::new(move |d: Arc<webrtc::data_channel::RTCDataChannel>| {
                let dc_clone = Arc::clone(&d);
                let incoming_files = Arc::clone(&incoming_files_clone);
                let config_path = config_path_dc.clone();
                let app_handle = app_handle_opt.clone();

                d.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                    let dc = Arc::clone(&dc_clone);
                    let incoming_files = Arc::clone(&incoming_files);
                    let config_path = config_path.clone();
                    let app_handle = app_handle.clone();

                    Box::pin(async move {
                        let text = String::from_utf8_lossy(&msg.data);
                        if let Ok(sync_msg) = serde_json::from_str::<SyncMessage>(&text) {
                            match sync_msg {
                                SyncMessage::ManifestRequest => {
                                    let db = Database::new(&config_path);
                                    let photos = db.get_photo_sync_info();
                                    let _ = Self::send_sync_message(&dc, &SyncMessage::ManifestResponse { photos }).await;
                                }
                                SyncMessage::ManifestResponse { photos } => {
                                    let db = Database::new(&config_path);
                                    let my_manifest = db.get_photo_sync_info();
                                    for peer_photo in photos {
                                        if !my_manifest.iter().any(|p| p.id == peer_photo.id) {
                                            let _ = Self::send_sync_message(&dc, &SyncMessage::FileRequest { id: peer_photo.id }).await;
                                        }
                                    }
                                }
                                SyncMessage::FileRequest { id } => {
                                    let db = Database::new(&config_path);
                                    if let Ok((path, created)) = db.connection.query_row("SELECT location, created FROM photo WHERE id = ?1", [&id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
                                        let dc_send = Arc::clone(&dc);
                                        tokio::spawn(async move { let _ = Self::send_file(dc_send, id, path, created).await; });
                                    }
                                }
                                SyncMessage::FileHeader { id, filename, size, created } => {
                                    let save_path = Path::new(&config_path).join("sync_temp").join(&filename);
                                    let _ = tokio::fs::create_dir_all(save_path.parent().unwrap()).await;
                                    if let Ok(file) = tokio::fs::File::create(&save_path).await {
                                        let mut incoming = incoming_files.lock().await;
                                        incoming.insert(id.clone(), IncomingFile { id, filename, size, received: 0, created, file });
                                    }
                                }
                                SyncMessage::FileChunk { id, data } => {
                                    let mut incoming = incoming_files.lock().await;
                                    if let Some(file_state) = incoming.get_mut(&id) {
                                        let _ = file_state.file.write_all(&data).await;
                                        file_state.received += data.len() as u64;
                                        if let Some(app) = &app_handle {
                                            let progress = (file_state.received as f32 / file_state.size as f32) * 100.0;
                                            let _ = app.emit("sync-progress", SyncProgress {
                                                device_id: "peer".to_string(), status: format!("Receiving {}", file_state.filename),
                                                progress, bytes_per_second: 0, items_completed: 0, items_total: 0,
                                            });
                                        }
                                    }
                                }
                                SyncMessage::FileEnd { id } => {
                                    let mut incoming = incoming_files.lock().await;
                                    if let Some(file_state) = incoming.remove(&id) {
                                        let temp_path = Path::new(&config_path).join("sync_temp").join(&file_state.filename);
                                        let db = Database::new(&config_path);
                                        let dirs = db.list_directories();
                                        let target_dir = if !dirs.is_empty() { Path::new(&dirs[0]) } else { &Path::new(&config_path).join("Siegu") };
                                        let _ = tokio::fs::create_dir_all(&target_dir).await;
                                        let final_path = target_dir.join(&file_state.filename);
                                        if let Ok(_) = tokio::fs::rename(&temp_path, &final_path).await {
                                            db.import_photo(&file_state.id, &final_path.to_string_lossy(), &file_state.created);
                                        }
                                    }
                                }
                            }
                        }
                    })
                }));
                Box::pin(async move {})
            }));
        }

        let app_handle_opt_state = self.app_handle.clone();
        let config_path_state = self.config_path.clone();
        let room_id_state = self.room_id.clone();

        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                println!("Peer Connection State changed to: {s:?}");
                let status = match s {
                    RTCPeerConnectionState::Connected => "Connected",
                    RTCPeerConnectionState::Connecting => "Connecting WebRTC...",
                    RTCPeerConnectionState::Disconnected => "Peer Disconnected",
                    RTCPeerConnectionState::Failed => "Connection Failed",
                    RTCPeerConnectionState::New => "Waiting for peer...",
                    _ => "Awaiting connection...",
                };

                if let Some(app) = &app_handle_opt_state {
                    let _ = app.emit("webrtc-state", status);

                    if s == RTCPeerConnectionState::Connected {
                        // Save device to database when connected
                        let db = Database::new(&config_path_state);
                        let peer_name = format!("Peer ({})", &room_id_state[..8]);
                        let _ = db.connection.execute(
                            "INSERT OR REPLACE INTO device(ip, name) VALUES(?1, ?2)",
                            (&room_id_state, &peer_name),
                        );
                        let _ = app.emit("refresh-devices", ());
                    }
                }
                Box::pin(async move {})
            },
        ));

        let pc = Arc::clone(&peer_connection);
        let pending_ice_candidates = Arc::new(Mutex::new(Vec::new()));

        if self.is_initiator {
            println!("Initiator: Waiting 1s before sending WebRTC Offer...");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            println!("Initiator: Sending WebRTC Offer...");
            let offer = match pc.create_offer(None).await {
                Ok(o) => o,
                Err(e) => {
                    println!("Error creating offer: {e}");
                    return Err(e.into());
                }
            };
            if let Err(e) = pc.set_local_description(offer.clone()).await {
                println!("Error setting local description: {e}");
                return Err(e.into());
            }
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
            println!("Initiator: WebRTC Offer sent");
        }

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received signaling message: {text}");
                    match serde_json::from_str::<SignalMessage>(&text) {
                        Ok(signal) => {
                            match signal {
                                SignalMessage::Joined { peer_count, .. } => {
                                    println!("Confirmed room entry! Peer count: {peer_count}");
                                    if self.is_initiator && peer_count == 2 {
                                        println!("Initiator: Room full on entry. Creating WebRTC Offer...");
                                        let offer = match pc.create_offer(None).await {
                                            Ok(o) => o,
                                            Err(e) => {
                                                println!("Error creating offer: {e}");
                                                continue;
                                            }
                                        };
                                        if let Err(e) =
                                            pc.set_local_description(offer.clone()).await
                                        {
                                            println!("Error setting local description: {e}");
                                            continue;
                                        }
                                        let offer_msg = SignalMessage::Offer {
                                            payload: serde_json::to_string(&offer)?,
                                            target: "peer".to_string(),
                                        };
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(Utf8Bytes::from(
                                                serde_json::to_string(&offer_msg)?,
                                            )))
                                            .await?;
                                        println!("Initiator: WebRTC Offer sent");
                                    }
                                }
                                SignalMessage::PeerJoined { .. } => {
                                    println!("Peer joined the room!");
                                    if self.is_initiator {
                                        println!("Initiator: Creating WebRTC Offer...");
                                        let offer = match pc.create_offer(None).await {
                                            Ok(o) => o,
                                            Err(e) => {
                                                println!("Error creating offer: {e}");
                                                continue;
                                            }
                                        };
                                        if let Err(e) =
                                            pc.set_local_description(offer.clone()).await
                                        {
                                            println!("Error setting local description: {e}");
                                            continue;
                                        }
                                        let offer_msg = SignalMessage::Offer {
                                            payload: serde_json::to_string(&offer)?,
                                            target: "peer".to_string(),
                                        };
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(Utf8Bytes::from(
                                                serde_json::to_string(&offer_msg)?,
                                            )))
                                            .await?;
                                        println!("Initiator: WebRTC Offer sent");
                                    }
                                }
                                SignalMessage::Offer { payload, .. } => {
                                    println!("Received WebRTC Offer");
                                    let sdp: RTCSessionDescription =
                                        serde_json::from_str(&payload)?;
                                    if let Err(e) = pc.set_remote_description(sdp).await {
                                        println!("Error setting remote description: {e}");
                                        continue;
                                    }

                                    // Add pending ICE candidates
                                    let mut pending = pending_ice_candidates.lock().await;
                                    for candidate in pending.drain(..) {
                                        let _ = pc.add_ice_candidate(candidate).await;
                                    }

                                    let answer = match pc.create_answer(None).await {
                                        Ok(a) => a,
                                        Err(e) => {
                                            println!("Error creating answer: {e}");
                                            continue;
                                        }
                                    };
                                    if let Err(e) = pc.set_local_description(answer.clone()).await {
                                        println!("Error setting local description (answer): {e}");
                                        continue;
                                    }
                                    let answer_msg = SignalMessage::Answer {
                                        payload: serde_json::to_string(&answer)?,
                                        target: "peer".to_string(),
                                    };
                                    println!("Sending WebRTC Answer");
                                    write
                                        .lock()
                                        .await
                                        .send(Message::Text(Utf8Bytes::from(
                                            serde_json::to_string(&answer_msg)?,
                                        )))
                                        .await?;
                                }
                                SignalMessage::Answer { payload, .. } => {
                                    println!("Received WebRTC Answer");
                                    let sdp: RTCSessionDescription =
                                        serde_json::from_str(&payload)?;
                                    if let Err(e) = pc.set_remote_description(sdp).await {
                                        println!(
                                            "Error setting remote description (answer): {e}"
                                        );
                                        continue;
                                    }

                                    // Add pending ICE candidates
                                    let mut pending = pending_ice_candidates.lock().await;
                                    for candidate in pending.drain(..) {
                                        let _ = pc.add_ice_candidate(candidate).await;
                                    }
                                }
                                SignalMessage::IceCandidate { payload, .. } => {
                                    println!("Received Ice Candidate");
                                    let candidate: RTCIceCandidateInit =
                                        serde_json::from_str(&payload)?;

                                    if pc.remote_description().await.is_none() {
                                        println!("Buffering ICE candidate as remote description is not set yet");
                                        pending_ice_candidates.lock().await.push(candidate);
                                    } else if let Err(e) = pc.add_ice_candidate(candidate).await {
                                        println!("Error adding ICE candidate: {e}");
                                    }
                                }
                                SignalMessage::PeerDisconnected { .. } => {
                                    println!("Peer disconnected via signaling");
                                    self.emit("webrtc-state", "Peer disconnected");
                                }
                                SignalMessage::Error { message } => {
                                    println!("Signaling Error: {message}");
                                    self.emit(
                                        "webrtc-state",
                                        format!("Signaling error: {message}"),
                                    );
                                }
                                SignalMessage::Join { .. } => {
                                    // Ignore Join messages from other peers in this loop
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse signaling message: {text}. Error: {e}");
                        }
                    }
                }
                Err(e) => {
                    println!("WebSocket Error in read loop: {e}");
                    self.emit("webrtc-state", format!("WebSocket error: {e}"));
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
