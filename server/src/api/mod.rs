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
use server::utils::crypto::CryptoService;

// 导入子模块
mod user;
mod friend;
mod message;

/// 共享应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: crate::storage::DbPool,
    /// 用户ID到WebSocket广播通道的映射
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    /// 客户端ID到用户ID的映射，用于断开连接时清理资源
    client_user_map: Arc<Mutex<HashMap<String, String>>>,
    /// 全局广播通道，用于向所有客户端发送消息
    broadcaster: broadcast::Sender<String>,
    /// 加密服务，用于私有算法和公有算法加密
    pub crypto_service: CryptoService,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(db_pool: crate::storage::DbPool) -> Self {
        let (broadcaster, _) = broadcast::channel(100);
        // 初始化加密服务（使用随机生成的私钥）
        let crypto_service = CryptoService::generate();
        Self {
            db_pool,
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_user_map: Arc::new(Mutex::new(HashMap::new())),
            broadcaster,
            crypto_service,
        }
    }
}

/// WebSocket连接升级处理器
async fn ws_handler(
    upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    upgrade.on_upgrade(|socket| handle_websocket(socket, state))
}

/// 处理WebSocket连接
async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4().to_string();
    
    // 创建客户端专用广播通道
    let (client_tx, mut client_rx) = broadcast::channel(100);
    
    println!("新WebSocket客户端连接: {}", client_id);
    
    // 广播新客户端连接消息
    let _ = state.broadcaster.send(format!("Client {} joined", client_id));

    // 处理接收消息的任务
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // 尝试解析为JSON以处理特殊类型消息
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if let Some(msg_type) = v.get("type").and_then(|x| x.as_str()) {
                        match msg_type {
                            "identify" => {
                                // 处理客户端身份标识
                                if let Some(user_id) = v.get("user_id").and_then(|x| x.as_str()) {
                                    // 将客户端通道映射到用户ID，方便推送定向通知
                                    let mut clients_map = state_clone.clients.lock().unwrap();
                                    clients_map.insert(user_id.to_string(), client_tx.clone());
                                    // 记录客户端ID到用户ID的映射，便于断开时清理
                                    state_clone.client_user_map.lock().unwrap().insert(client_id_clone.clone(), user_id.to_string());
                                    println!("WebSocket客户端 {} 标识为用户 {}", client_id_clone, user_id);
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }
                }

                println!("从客户端 {} 收到消息: {}", client_id_clone, text);
                // 广播消息给所有客户端
                let _ = state_clone.broadcaster.send(format!("{}: {}", client_id_clone, text));
            }
        }
    });
    
    // 处理发送消息的任务
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
    
    // 清理资源：移除客户端映射
    {
        let mut clients = state.clients.lock().unwrap();
        clients.remove(&client_id);
    }

    // 清理用户ID映射（如果存在）
    {
        let mut client_user_map = state.client_user_map.lock().unwrap();
        if let Some(user_id) = client_user_map.remove(&client_id) {
            let mut clients = state.clients.lock().unwrap();
            clients.remove(&user_id);
        }
    }

    println!("WebSocket客户端断开连接: {}", client_id);
    // 广播客户端断开连接消息
    let _ = state.broadcaster.send(format!("Client {} left", client_id));
}

/// 注册所有API路由
pub fn register_routes(db_pool: crate::storage::DbPool) -> Router {
    // 创建共享应用状态
    let app_state = AppState::new(db_pool);
    
    // 主路由器配置
    Router::new()
        .route("/ws", get(ws_handler))
        // 用户相关路由
        .merge(user::register_routes())
        // 好友相关路由
        .merge(friend::register_routes())
        // 消息相关路由
        .merge(message::register_routes())
        .with_state(app_state)
}