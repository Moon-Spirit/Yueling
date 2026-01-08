use axum::{extract::{State}, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use crate::storage::{DbPool, Message};
use crate::error::AppError;
use hex;

// 共享应用状态
use super::AppState;

// 消息请求体
#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub sender_id: String,
    pub receiver_id: String,
    pub content: String,
    pub message_type: String, // "private"或"group"
}

// 消息响应体
#[derive(Serialize)]
pub struct SendMessageResponse {
    pub success: bool,
    pub message: String,
    pub message_id: Option<String>,
}

// 获取未读消息请求
#[derive(Deserialize)]
pub struct GetUnreadMessagesRequest {
    pub user_id: String,
}

// 获取未读消息响应
#[derive(Serialize)]
pub struct GetUnreadMessagesResponse {
    pub success: bool,
    pub message: String,
    pub messages: Vec<Message>,
}

// 标记消息为已读请求
#[derive(Deserialize)]
pub struct MarkMessagesAsReadRequest {
    pub message_ids: Vec<String>,
}

// 标记消息为已读响应
#[derive(Serialize)]
pub struct MarkMessagesAsReadResponse {
    pub success: bool,
    pub message: String,
}

// 发送消息处理器
pub async fn send_message_handler(
    State(state): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, AppError> {
    // 使用私有算法和公有算法加密消息内容
    let encrypted_bytes = state.crypto_service.encrypt(req.content.as_bytes())
        .map_err(|e| AppError::Internal(format!("加密失败: {}", e)))?;
    let encrypted_hex = hex::encode(encrypted_bytes);
    
    let message = state.db_pool.send_message(
        &req.sender_id,
        &req.receiver_id,
        &encrypted_hex,
        &req.message_type,
    )
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(SendMessageResponse {
        success: true,
        message: "消息发送成功".into(),
        message_id: Some(message.id),
    }))
}

// 获取未读消息处理器
pub async fn get_unread_messages_handler(
    State(state): State<AppState>,
    Json(req): Json<GetUnreadMessagesRequest>,
) -> Result<Json<GetUnreadMessagesResponse>, AppError> {
    let mut messages = state.db_pool.get_unread_messages(&req.user_id)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    // 对每条消息的内容进行解密
    for msg in messages.iter_mut() {
        let encrypted_bytes = hex::decode(&msg.content)
            .map_err(|e| AppError::Internal(format!("十六进制解码失败: {}", e)))?;
        let decrypted_bytes = state.crypto_service.decrypt(&encrypted_bytes)
            .map_err(|e| AppError::Internal(format!("解密失败: {}", e)))?;
        // 将解密后的字节转换为字符串
        let decrypted_content = String::from_utf8(decrypted_bytes)
            .map_err(|e| AppError::Internal(format!("解密内容UTF-8解码失败: {}", e)))?;
        msg.content = decrypted_content;
    }

    Ok(Json(GetUnreadMessagesResponse {
        success: true,
        message: "获取未读消息成功".into(),
        messages,
    }))
}

// 标记消息为已读处理器
pub async fn mark_messages_as_read_handler(
    State(state): State<AppState>,
    Json(req): Json<MarkMessagesAsReadRequest>,
) -> Result<Json<MarkMessagesAsReadResponse>, AppError> {
    state.db_pool.mark_messages_as_read(&req.message_ids)
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(MarkMessagesAsReadResponse {
        success: true,
        message: "消息已标记为已读".into(),
    }))
}

/// 注册消息相关路由
pub fn register_routes() -> Router<AppState> {
    Router::new()
        .route("/send-message", post(send_message_handler))
        .route("/messages/unread", post(get_unread_messages_handler))
        .route("/messages/read", post(mark_messages_as_read_handler))
}