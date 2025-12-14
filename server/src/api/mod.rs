use axum::{
    extract::{State, ws::WebSocketUpgrade, ws::Message, ws::WebSocket},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use crate::storage::DbPool;
use crate::error::AppError;
use bcrypt::verify;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

// 共享应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    broadcaster: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(db_pool: DbPool) -> Self {
        let (broadcaster, _) = broadcast::channel(100);
        Self {
            db_pool,
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcaster,
        }
    }
}

// 注册请求体（前端提交数据）
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String, // 明文密码（后端哈希存储）
}

// 注册响应体（返回给前端）
#[derive(Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<String>, // 成功时返回用户ID
}

// 登录请求体（前端提交数据）
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String, // 明文密码（后端验证）
}

// 登录响应体（返回给前端）
#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<String>, // 成功时返回用户ID
    pub username: Option<String>, // 成功时返回用户名
}

// WebSocket处理器
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
    
    // 将客户端添加到状态
    state.clients.lock().unwrap().insert(client_id.clone(), client_tx.clone());
    println!("New WebSocket client connected: {}", client_id);
    
    // 广播新客户端连接
    let _ = state.broadcaster.send(format!("Client {} joined", client_id));

    // 处理接收消息
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Received from {}: {}", client_id_clone, text);
                // 广播消息给所有客户端
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
    
    // 移除客户端
    state.clients.lock().unwrap().remove(&client_id);
    println!("WebSocket client disconnected: {}", client_id);
    
    // 广播客户端断开连接
    let _ = state.broadcaster.send(format!("Client {} left", client_id));
}

// 注册处理器（核心API逻辑）
pub async fn register_handler(
    State(state): State<AppState>, // 注入共享状态
    Json(req): Json<RegisterRequest>, // 解析JSON请求体
) -> Result<Json<RegisterResponse>, AppError> {
    // 调用存储层注册用户
    let user = state.db_pool.register_user(&req.username, "", &req.password)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => 
                AppError::UserExists("用户名已被注册".into()),
            _ => AppError::Database(e.to_string()),
        })?;

    // 返回成功响应
    Ok(Json(RegisterResponse {
        success: true,
        message: "注册成功".into(),
        user_id: Some(user.id),
    }))
}

// 登录处理器（核心API逻辑）
pub async fn login_handler(
    State(state): State<AppState>, // 注入共享状态
    Json(req): Json<LoginRequest>, // 解析JSON请求体
) -> Result<Json<LoginResponse>, AppError> {
    // 调用存储层获取用户
    let conn = state.db_pool.0.lock().unwrap();
    let user = conn.query_row(
        "SELECT id, username, password_hash FROM users WHERE username = ?",
        [&req.username],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => 
            AppError::InvalidCredentials("用户名或密码错误".into()),
        _ => AppError::Database(e.to_string()),
    })?;

    // 验证密码
    let (id, username, password_hash) = user;
    if !verify(&req.password, &password_hash).map_err(|_| AppError::Internal("密码验证失败".into()))? {
        return Err(AppError::InvalidCredentials("用户名或密码错误".into()));
    }

    // 返回成功响应
    Ok(Json(LoginResponse {
        success: true,
        message: "登录成功".into(),
        user_id: Some(id),
        username: Some(username),
    }))
}

// 导出路由
pub fn register_routes(db_pool: DbPool) -> Router {
    // 创建共享应用状态
    let app_state = AppState::new(db_pool);
    
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/ws", get(ws_handler))
        .with_state(app_state)
}