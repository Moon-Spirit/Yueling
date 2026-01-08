use axum::{extract::{State}, response::Json, routing::{post}, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::storage::DbPool;
use crate::error::AppError;
use std::collections::hash_map::Entry;
use hex;

// 共享应用状态
use super::AppState;

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
    // 使用私有算法和公有算法加密目标用户名
    let encrypted_bytes = state.crypto_service.encrypt(req.to_username.as_bytes())
        .map_err(|e| AppError::Internal(format!("加密失败: {}", e)))?;
    let encrypted_hex = hex::encode(encrypted_bytes);
    // 解密回原始用户名
    let decrypted_bytes = state.crypto_service.decrypt(&hex::decode(&encrypted_hex).unwrap())
        .map_err(|e| AppError::Internal(format!("解密失败: {}", e)))?;
    let decrypted_username = String::from_utf8(decrypted_bytes)
        .map_err(|e| AppError::Internal(format!("UTF-8解码失败: {}", e)))?;
    
    let result = state.db_pool.send_friend_request(&req.from_user_id, &decrypted_username)
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
    let notify = json! ({
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

        let notify = json! ({
            "type": "friend_added",
            "user_id": friendship.friend_id,
            "friend_id": friendship.user_id,
            "friend_username": from_username,
            "message": "您已成为好友"
        })
        .to_string();

        let reverse_notify = json! ({
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

/// 注册好友相关路由
pub fn register_routes() -> Router<AppState> {
    Router::new()
        // 好友功能路由
        .route("/search-users", post(search_users_handler))
        .route("/send-friend-request", post(send_friend_request_handler))
        .route("/friends/add", post(send_friend_request_handler))
        .route("/get-friend-requests", post(get_friend_requests_handler))
        .route("/respond-to-friend-request", post(respond_to_friend_request_handler))
        .route("/get-friends", post(get_friends_handler))
        .route("/remove-friend", post(remove_friend_handler))
}