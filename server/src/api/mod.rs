use axum::{
    extract::{State, ws::WebSocketUpgrade, ws::Message, ws::WebSocket},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

// 导入子模块
mod user;
mod friend;
mod message;

// 导出子模块的路由注册函数
pub use user::register_user_routes;
pub use friend::register_friend_routes;
pub use message::register_message_routes;

// 共享应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: crate::storage::DbPool,
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    // client_id -> user_id 映射，用于在断开时清理 user_id 条目
    client_user_map: Arc<Mutex<HashMap<String, String>>>,
    broadcaster: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(db_pool: crate::storage::DbPool) -> Self {
        let (broadcaster, _) = broadcast::channel(100);
        Self {
            db_pool,
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_user_map: Arc::new(Mutex::new(HashMap::new())),
            broadcaster,
        }
    }
}

// WebSocket处理器
async fn ws_handler(
    upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    upgrade.on_upgrade(|socket| handle_socket(socket, state))
}

// 处理WebSocket连接
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4().to_string();
    
    // 创建客户端专用通道
    let (client_tx, mut client_rx) = broadcast::channel(100);
    
    println!("New WebSocket client connected: {}", client_id);
    
    // 广播新客户端连接
    let _ = state.broadcaster.send(format!("Client {} joined", client_id));

    // 处理接收消息
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // 尝试解析为 JSON 以处理特殊类型（例如 identify）
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
                        match t {
                            "identify" => {
                                if let Some(user_id) = v.get("user_id").and_then(|x| x.as_str()) {
                                    // 将客户端通道映射到 user_id，方便推送定向通知
                                    let mut clients_map = state_clone.clients.lock().unwrap();
                                    clients_map.insert(user_id.to_string(), client_tx.clone());
                                    // 记录 client_id -> user_id 映射，便于断开时清理
                                    state_clone.client_user_map.lock().unwrap().insert(client_id_clone.clone(), user_id.to_string());
                                    println!("WebSocket client {} identified as user {}", client_id_clone, user_id);
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }
                }

                println!("Received from {}: {}", client_id_clone, text);
                // 广播消息给所有客户端（保持原有行为）
                let _ = state_clone.broadcaster.send(format!("{}: {}", client_id_clone, text));
            }
        }
    });
    
    // 处理发送消息
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = client_rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });
    
    // 等待任一任务结束
    tokio::select! {
        _ = recv_task => (),
        _ = send_task => (),
    }
    
    // 移除客户端映射（包括 client_id 以及可能存在的 user_id）
    {
        let mut clients = state.clients.lock().unwrap();
        // 先移除以 client_id 作为 key 的项（若存在）
        clients.remove(&client_id);
    }

    // 若之前识别过（有 client->user 映射），清理对应的 user_id 映射
    {
        let mut cmap = state.client_user_map.lock().unwrap();
        if let Some(user_id) = cmap.remove(&client_id) {
            let mut clients = state.clients.lock().unwrap();
            clients.remove(&user_id);
        }
    }

    println!("WebSocket client disconnected: {}", client_id);
    // 广播客户端断开连接
    let _ = state.broadcaster.send(format!("Client {} left", client_id));
}

// 导出路由
pub fn register_routes(db_pool: crate::storage::DbPool) -> Router {
    // 创建共享应用状态
    let app_state = AppState::new(db_pool);
    
    // 主路由器
    let app = Router::new()
        .route("/ws", get(ws_handler))
        // 用户相关路由
        .merge(user::register_user_routes())
        // 好友相关路由
        .merge(friend::register_friend_routes())
        // 消息相关路由
        .merge(message::register_message_routes())
        .with_state(app_state);
    
    app
}