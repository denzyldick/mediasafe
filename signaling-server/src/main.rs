use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Standard WebSocket payload structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum SignalMessage {
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

// A single connected device has a transmitter channel to push messages to its WebSocket
struct Client {
    sender: mpsc::UnboundedSender<Message>,
}

// A room maps a unique hashed PIN to up to 2 connected devices (Initiator and Receiver)
struct Room {
    clients: HashMap<String, Client>,
}

type AppState = Arc<RwLock<HashMap<String, Room>>>;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "signaling_server=debug,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state: AppState = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/health", get(|| async { "Signaling Server OK" }))
        .route("/ws/:room_id", get(ws_handler))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "9489".to_string());
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>().unwrap();
    
    info!("Starting WebRTC Signaling Server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Spawn a task to forward messages from the mpsc channel to the actual WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut device_id = String::new();

    // Handle incoming messages from the WebSocket client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Parse JSON message
            if let Ok(signal) = serde_json::from_str::<SignalMessage>(&text) {
                match signal {
                    SignalMessage::Join { device_id: id } => {
                        device_id = id.clone();
                        info!("Device {} joining room {}", device_id, room_id);
                        
                        let mut rooms = state.write().await;
                        let room = rooms.entry(room_id.clone()).or_insert_with(|| Room {
                            clients: HashMap::new(),
                        });
                        
                        // Reject if room is full (2 devices max for P2P)
                        if room.clients.len() >= 2 {
                            let err = SignalMessage::Error { message: "Room is full".to_string() };
                            let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                            break;
                        }
                        
                        room.clients.insert(device_id.clone(), Client { sender: tx.clone() });
                    },
                    SignalMessage::Offer { payload, target } => {
                        info!("Relaying offer from {} to {}", device_id, target);
                        let msg = SignalMessage::Offer { payload, target: target.clone() };
                        relay_message(&state, &room_id, &target, msg).await;
                    },
                    SignalMessage::Answer { payload, target } => {
                        info!("Relaying answer from {} to {}", device_id, target);
                        let msg = SignalMessage::Answer { payload, target: target.clone() };
                        relay_message(&state, &room_id, &target, msg).await;
                    },
                    SignalMessage::IceCandidate { payload, target } => {
                        // Very noisy, keeping as debug
                        tracing::debug!("Relaying ICE candidate to {}", target);
                        let msg = SignalMessage::IceCandidate { payload, target: target.clone() };
                        relay_message(&state, &room_id, &target, msg).await;
                    },
                    _ => {}
                }
            }
        }
        
        // Cleanup when client disconnects
        if !device_id.is_empty() {
            info!("Device {} disconnected from {}", device_id, room_id);
            let mut rooms = state.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                room.clients.remove(&device_id);
                // Notify remaining peer
                for (_, client) in room.clients.iter() {
                    let msg = SignalMessage::PeerDisconnected { device_id: device_id.clone() };
                    let _ = client.sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }
                
                if room.clients.is_empty() {
                    rooms.remove(&room_id);
                    info!("Room {} deleted", room_id);
                }
            }
        }
    });

    // Wait for either the sender task or receiver task to conclude
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

// Helper to blind-forward a message to a specific target device in a room
async fn relay_message(state: &AppState, room_id: &str, target_id: &str, msg: SignalMessage) {
    let rooms = state.read().await;
    if let Some(room) = rooms.get(room_id) {
        if let Some(client) = room.clients.get(target_id) {
            let json = serde_json::to_string(&msg).unwrap();
            let _ = client.sender.send(Message::Text(json.into()));
        }
    }
}
