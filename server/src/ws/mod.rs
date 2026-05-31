use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::info;
use uuid::Uuid;

use crate::AppState;

pub type RoomMap = Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>;

#[derive(Clone)]
pub struct WsState {
    pub rooms: RoomMap,
}

const CHAT_MAX_LENGTH: usize = 500;
const RATE_LIMIT_WINDOW_SECS: u64 = 10;
const RATE_LIMIT_MAX_MSGS: u32 = 30;

struct RateLimiter {
    timestamps: std::collections::VecDeque<u64>,
}

impl RateLimiter {
    fn new() -> Self {
        Self { timestamps: std::collections::VecDeque::new() }
    }

    fn check(&mut self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Remove old timestamps outside the window
        while let Some(&t) = self.timestamps.front() {
            if now - t > RATE_LIMIT_WINDOW_SECS {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
        if self.timestamps.len() >= RATE_LIMIT_MAX_MSGS as usize {
            return false;
        }
        self.timestamps.push_back(now);
        true
    }
}

fn validate_origin(headers: &HeaderMap) -> bool {
    let allowed_origins = ["http://localhost:3000", "http://127.0.0.1:3000"];
    match headers.get("origin").and_then(|v| v.to_str().ok()) {
        Some(origin) => allowed_origins.iter().any(|&a| origin == a),
        // Allow requests without Origin header (e.g. direct websocket clients)
        None => true,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    seed: u32,
    #[serde(default)]
    id: String,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default)]
    z: f64,
    #[serde(default)]
    yaw: f32,
    #[serde(default)]
    pitch: f32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlayerState {
    id: String,
    name: String,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
struct ChatMessage {
    player_id: String,
    player_name: String,
    text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    players: Vec<PlayerState>,
    #[serde(default)]
    your_id: String,
    #[serde(default)]
    count: u32,
    #[serde(default)]
    chat: ChatMessage,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !validate_origin(&headers) {
        info!("WebSocket connection rejected: invalid origin");
        return (axum::http::StatusCode::FORBIDDEN, "Origin not allowed").into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, app.ws_state))
}

async fn handle_socket(mut socket: WebSocket, state: WsState) {
    let player_id = Uuid::new_v4().to_string();
    let player_name = format!("Player_{}", &player_id[..6]);
    let mut room_key: Option<String> = None;
    let mut rx: Option<broadcast::Receiver<String>> = None;

    let (pong_tx, mut pong_rx) = tokio::sync::mpsc::channel::<()>(8);
    let mut rate_limiter = RateLimiter::new();

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let s: String = text.to_string();
                        if let Ok(cmsg) = serde_json::from_str::<ClientMessage>(&s) {
                            match cmsg.msg_type.as_str() {
                                "join" => {
                                    let seed = cmsg.seed;
                                    let key = format!("seed:{}", seed);
                                    let sender = state.rooms.read().await.get(&key).cloned();
                                    let sender = if let Some(s) = sender {
                                        s
                                    } else {
                                        let (tx, _) = broadcast::channel(100);
                                        state.rooms.write().await.insert(key.clone(), tx.clone());
                                        tx
                                    };
                                    rx = Some(sender.subscribe());
                                    room_key = Some(key.clone());

                                    let welcome = ServerMessage {
                                        msg_type: "welcome".into(),
                                        players: vec![],
                                        your_id: player_id.clone(),
                                        count: sender.receiver_count() as u32,
                                        chat: ChatMessage { player_id: String::new(), player_name: String::new(), text: String::new() },
                                    };
                                    let _ = socket.send(Message::Text(serde_json::to_string(&welcome).unwrap().into())).await;

                                    info!("Player {} joined room {} ({} players)", &player_id[..6], key, sender.receiver_count());
                                }
                                "pos" => {
                                    if let Some(ref key) = room_key {
                                        let rooms = state.rooms.read().await;
                                        if let Some(sender) = rooms.get(key) {
                                            let player = PlayerState {
                                                id: player_id.clone(),
                                                name: player_name.clone(),
                                                x: cmsg.x,
                                                y: cmsg.y,
                                                z: cmsg.z,
                                                yaw: cmsg.yaw,
                                                pitch: cmsg.pitch,
                                            };
                                            let msg = ServerMessage {
                                                msg_type: "pos".into(),
                                                players: vec![player],
                                                your_id: String::new(),
                                                count: 0,
                                                chat: ChatMessage { player_id: String::new(), player_name: String::new(), text: String::new() },
                                            };
                                            let _ = sender.send(serde_json::to_string(&msg).unwrap());
                                        }
                                    }
                                }
                                "chat" => {
                                    if !rate_limiter.check() {
                                        let warn = ServerMessage {
                                            msg_type: "chat".into(),
                                            players: vec![],
                                            your_id: String::new(),
                                            count: 0,
                                            chat: ChatMessage {
                                                player_id: String::new(),
                                                player_name: "Sistema".into(),
                                                text: "Rate limit exceeded. Please slow down.".into(),
                                            },
                                        };
                                        let _ = socket.send(Message::Text(serde_json::to_string(&warn).unwrap().into())).await;
                                        continue;
                                    }
                                    if let Some(ref key) = room_key {
                                        let rooms = state.rooms.read().await;
                                        if let Some(sender) = rooms.get(key) {
                                            let mut chat_text = cmsg.text.trim().to_string();
                                            if chat_text.is_empty() { continue; }
                                            if chat_text.len() > CHAT_MAX_LENGTH {
                                                chat_text.truncate(CHAT_MAX_LENGTH);
                                            }
                                            if chat_text.starts_with('/') {
                                                match chat_text.split_once(' ') {
                                                    Some(("/whisper", rest)) => {
                                                        // whisper — broadcast with whisper prefix
                                                        let msg = ServerMessage {
                                                            msg_type: "chat".into(),
                                                            players: vec![],
                                                            your_id: String::new(),
                                                            count: 0,
                                                            chat: ChatMessage {
                                                                player_id: player_id.clone(),
                                                                player_name: player_name.clone(),
                                                                text: format!("[whisper] {}", rest),
                                                            },
                                                        };
                                                        let _ = sender.send(serde_json::to_string(&msg).unwrap());
                                                    }
                                                    _ => {
                                                        let help = format!("Comandos: /list, /whisper <msg>");
                                                        let msg = ServerMessage {
                                                            msg_type: "chat".into(),
                                                            players: vec![],
                                                            your_id: String::new(),
                                                            count: 0,
                                                            chat: ChatMessage {
                                                                player_id: String::new(),
                                                                player_name: "Sistema".into(),
                                                                text: help,
                                                            },
                                                        };
                                                        let _ = sender.send(serde_json::to_string(&msg).unwrap());
                                                    }
                                                }
                                            } else {
                                                let msg = ServerMessage {
                                                    msg_type: "chat".into(),
                                                    players: vec![],
                                                    your_id: String::new(),
                                                    count: 0,
                                                    chat: ChatMessage {
                                                        player_id: player_id.clone(),
                                                        player_name: player_name.clone(),
                                                        text: chat_text.clone(),
                                                    },
                                                };
                                                let _ = sender.send(serde_json::to_string(&msg).unwrap());
                                            }
                                        }
                                    }
                                }
                                "pong" => {
                                    let _ = pong_tx.send(()).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
            result = async {
                rx.as_mut()?.recv().await.ok()
            } => {
                if let Some(text) = result {
                    if let Ok(smsg) = serde_json::from_str::<ServerMessage>(&text) {
                        if smsg.players.iter().any(|p| p.id == player_id) {
                            continue;
                        }
                    }
                    let _ = socket.send(Message::Text(text.into())).await;
                }
            }
            _ = pong_rx.recv() => {}
        }
    }

    if let Some(ref key) = room_key {
        let mut rooms = state.rooms.write().await;
        if let Some(sender) = rooms.get(key) {
            if sender.receiver_count() <= 1 {
                rooms.remove(key);
                info!("Room {} removed (empty)", key);
            }
        }
    }
    info!("Player {} disconnected", &player_id[..6]);
}
