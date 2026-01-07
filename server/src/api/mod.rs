use axum::{
    extract::{State, ws::WebSocketUpgrade, ws::Message, ws::WebSocket},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::storage::DbPool;
use crate::error::AppError;
use bcrypt::verify;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::collections::hash_map::Entry;
use tokio::sync::broadcast;
use uuid::Uuid;

// 共享应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    // client_id -> user_id 映射，用于在断开时清理 user_id 条目
    client_user_map: Arc<Mutex<HashMap<String, String>>>,
    broadcaster: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(db_pool: DbPool) -> Self {
        let (broadcaster, _) = broadcast::channel(100);
        Self {
            db_pool,
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_user_map: Arc::new(Mutex::new(HashMap::new())),
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

// 好友功能相关结构体

#[derive(Deserialize)]
pub struct SearchUsersRequest {
    pub query: String,
}

#[derive(Serialize)]
pub struct SearchUsersResponse {
    pub success: bool,
    pub message: String,
    pub users: Vec<SearchUser>,
}

#[derive(Serialize)]
pub struct SearchUser {
    pub id: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct SendFriendRequestRequest {
    pub from_user_id: String,
    pub to_username: String,
}

#[derive(Serialize)]
pub struct SendFriendRequestResponse {
    pub success: bool,
    pub message: String,
    pub request_id: Option<String>,
}

#[derive(Deserialize)]
pub struct GetFriendRequestsRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct GetFriendRequestsResponse {
    pub success: bool,
    pub message: String,
    pub requests: Vec<FriendRequestInfo>,
}

#[derive(Serialize)]
pub struct FriendRequestInfo {
    pub id: String,
    pub from_user_id: String,
    pub from_username: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct RespondToFriendRequestRequest {
    pub request_id: String,
    pub user_id: String,
    pub response: String, // "accepted" or "rejected"
}

#[derive(Serialize)]
pub struct RespondToFriendRequestResponse {
    pub success: bool,
    pub message: String,
    pub friendship: Option<FriendInfo>,
}

#[derive(Deserialize)]
pub struct GetFriendsRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct GetFriendsResponse {
    pub success: bool,
    pub message: String,
    pub friends: Vec<FriendInfo>,
}

#[derive(Serialize)]
pub struct FriendInfo {
    pub id: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct RemoveFriendRequest {
    pub user_id: String,
    pub friend_id: String,
}

#[derive(Serialize)]
pub struct RemoveFriendResponse {
    pub success: bool,
    pub message: String,
}

// 用户存在检查
#[derive(Deserialize)]
pub struct UserExistsRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct UserExistsResponse {
    pub success: bool,
    pub message: String,
    pub exists: bool,
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

// 注册处理器（核心API逻辑）
pub async fn register_handler(
    State(state): State<AppState>, // 注入共享状态
    Json(req): Json<RegisterRequest>, // 解析JSON请求体
) -> Result<Json<RegisterResponse>, AppError> {
    // 调用存储层注册用户
    let user = state.db_pool.register_user(&req.username, "", &req.password)
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(_, Some(msg)) if msg.contains("用户名已存在") =>
                AppError::UserExists(msg),
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

// 好友功能处理器

// 搜索用户
pub async fn search_users_handler(
    State(state): State<AppState>,
    Json(req): Json<SearchUsersRequest>,
) -> Result<Json<SearchUsersResponse>, AppError> {
    let users = state.db_pool.search_users(&req.query)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let search_users: Vec<SearchUser> = users.into_iter().map(|user| SearchUser {
        id: user.id,
        username: user.username,
    }).collect();

    Ok(Json(SearchUsersResponse {
        success: true,
        message: "搜索成功".into(),
        users: search_users,
    }))
}

// 发送好友请求
pub async fn send_friend_request_handler(
    State(state): State<AppState>,
    Json(req): Json<SendFriendRequestRequest>,
) -> Result<Json<SendFriendRequestResponse>, AppError> {
    let result = state.db_pool.send_friend_request(&req.from_user_id, &req.to_username)
        .map_err(|e| {
            match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::FriendOperation("目标用户不存在".into())
                }
                        rusqlite::Error::SqliteFailure(_, Some(msg)) if msg == "Already friends" => {
                            AppError::FriendOperation("已经是好友".into())
                        }
                        rusqlite::Error::SqliteFailure(_, Some(msg)) if msg == "Friend request already sent" => {
                            AppError::FriendOperation("好友请求已发送".into())
                        }
                        // 捕获 SQLite 的唯一约束错误并映射为友好提示
                        rusqlite::Error::SqliteFailure(_, Some(msg)) if msg.contains("UNIQUE constraint failed") => {
                            AppError::FriendOperation("好友请求已存在".into())
                        }
                _ => AppError::Database(e.to_string())
            }
        })?;

    // 尝试通知接收者（若其已通过 WebSocket 标识并连接）
    let notify = json!({
        "type": "friend_request",
        "request_id": result.id,
        "from_user_id": result.from_user_id,
        "to_user_id": result.to_user_id,
        "message": "您收到新的好友请求"
    })
    .to_string();

    if let Some(tx) = state.clients.lock().unwrap().get(&result.to_user_id) {
        let _ = tx.send(notify);
    }

    Ok(Json(SendFriendRequestResponse {
        success: true,
        message: "好友请求发送成功".into(),
        request_id: Some(result.id),
    }))
}

// 获取收到的好友请求
pub async fn get_friend_requests_handler(
    State(state): State<AppState>,
    Json(req): Json<GetFriendRequestsRequest>,
) -> Result<Json<GetFriendRequestsResponse>, AppError> {
    let requests = state.db_pool.get_received_friend_requests(&req.user_id)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut request_infos: Vec<FriendRequestInfo> = requests.into_iter().map(|req| {
        // 这里需获取发送者的用户名，联表查询并返回友好名称
        // 注意：在持有连接锁的情况下按顺序查询用户名，若性能成为问题可改为联表查询一次性获取
        FriendRequestInfo {
            id: req.id,
            from_user_id: req.from_user_id,
            from_username: "".to_string(), // 占位，后面会替换
            created_at: req.created_at,
        }
    }).collect();
    // 填充 from_username 字段（从 users 表查找用户名）
    if !request_infos.is_empty() {
        let conn = state.db_pool.0.lock().unwrap();
        for info in request_infos.iter_mut() {
            let uname: Result<String, rusqlite::Error> = conn.query_row(
                "SELECT username FROM users WHERE id = ?",
                [&info.from_user_id],
                |row| row.get(0),
            );
            if let Ok(name) = uname {
                info.from_username = name;
            } else {
                // 若查询失败，保留原始 user_id 以便前端回退显示
                info.from_username = info.from_user_id.clone();
            }
        }
    }
    Ok(Json(GetFriendRequestsResponse {
        success: true,
        message: "获取好友请求成功".into(),
        requests: request_infos,
    }))
}

// 响应好友请求
pub async fn respond_to_friend_request_handler(
    State(state): State<AppState>,
    Json(req): Json<RespondToFriendRequestRequest>,
) -> Result<Json<RespondToFriendRequestResponse>, AppError> {
    // 调用存储层并获取结果（如果被接受，会返回创建的 Friendship）
    let friendship = state.db_pool.respond_to_friend_request(&req.request_id, &req.user_id, &req.response)
        .map_err(|e| {
            match e {
                rusqlite::Error::SqliteFailure(_, Some(msg)) if msg == "Friend request already processed" => {
                    AppError::FriendOperation("好友请求已处理".into())
                }
                _ => AppError::Database(e.to_string())
            }
        })?;

    let mut friendship_info: Option<FriendInfo> = None;

    let message = if req.response == "accepted" {
        // 如果接受，查询双方用户名并通知双方刷新好友列表（若在线）
        let conn = state.db_pool.0.lock().unwrap();
        let from_username: String = conn.query_row(
            "SELECT username FROM users WHERE id = ?",
            [&friendship.user_id],
            |row| row.get(0),
        ).unwrap_or_else(|_| "".to_string());

        let to_username: String = conn.query_row(
            "SELECT username FROM users WHERE id = ?",
            [&friendship.friend_id],
            |row| row.get(0),
        ).unwrap_or_else(|_| "".to_string());

        let notify = json!({
            "type": "friend_added",
            "user_id": friendship.friend_id,
            "friend_id": friendship.user_id,
            "friend_username": from_username,
            "message": "您已成为好友"
        })
        .to_string();

        let reverse_notify = json!({
            "type": "friend_added",
            "user_id": friendship.user_id,
            "friend_id": friendship.friend_id,
            "friend_username": to_username,
            "message": "您已成为好友"
        })
        .to_string();

        // 尝试向发送者和接收者发送通知（如果他们通过 websocket 标识并连接）
        let mut clients = state.clients.lock().unwrap();
        if let Some(tx) = clients.get(&friendship.friend_id) {
            println!("Sending friend_added notify to {}: {}", friendship.friend_id, notify);
            let _ = tx.send(notify.clone());
        } else {
            println!("No websocket client for {} when sending notify", friendship.friend_id);
        }
        if let Some(tx) = clients.get(&friendship.user_id) {
            println!("Sending friend_added notify to {}: {}", friendship.user_id, reverse_notify);
            let _ = tx.send(reverse_notify.clone());
        } else {
            println!("No websocket client for {} when sending notify", friendship.user_id);
        }

        // 准备返回的好友信息（用于前端立即更新）——对调用者（接收者）返回对方信息
        friendship_info = Some(FriendInfo { id: friendship.user_id.clone(), username: from_username.clone() });
        "好友请求已接受".into()
    } else {
        "好友请求已拒绝".into()
    };

    Ok(Json(RespondToFriendRequestResponse {
        success: true,
        message,
        friendship: friendship_info,
    }))
}

// 获取好友列表
pub async fn get_friends_handler(
    State(state): State<AppState>,
    Json(req): Json<GetFriendsRequest>,
) -> Result<Json<GetFriendsResponse>, AppError> {
    let friends = state.db_pool.get_friends(&req.user_id)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let friend_infos: Vec<FriendInfo> = friends.into_iter().map(|friend| FriendInfo {
        id: friend.id,
        username: friend.username,
    }).collect();

    Ok(Json(GetFriendsResponse {
        success: true,
        message: "获取好友列表成功".into(),
        friends: friend_infos,
    }))
}

// 删除好友
pub async fn remove_friend_handler(
    State(state): State<AppState>,
    Json(req): Json<RemoveFriendRequest>,
) -> Result<Json<RemoveFriendResponse>, AppError> {
    state.db_pool.remove_friend(&req.user_id, &req.friend_id)
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(RemoveFriendResponse {
        success: true,
        message: "删除好友成功".into(),
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
        // 好友功能路由
        .route("/search-users", post(search_users_handler))
        .route("/send-friend-request", post(send_friend_request_handler))
        .route("/friends/add", post(send_friend_request_handler))
        .route("/get-friend-requests", post(get_friend_requests_handler))
        .route("/respond-to-friend-request", post(respond_to_friend_request_handler))
        .route("/get-friends", post(get_friends_handler))
        .route("/remove-friend", post(remove_friend_handler))
        // 用户存在性检查（前端启动时用于自动登录验证）
        .route("/user/exists", post(user_exists_handler))
        .with_state(app_state)
}

// 检查用户是否存在的处理器
pub async fn user_exists_handler(
    State(state): State<AppState>,
    Json(req): Json<UserExistsRequest>,
) -> Result<Json<UserExistsResponse>, AppError> {
    let exists = state.db_pool.user_exists_by_id(&req.user_id)
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(UserExistsResponse {
        success: true,
        message: "检查完成".into(),
        exists,
    }))
}