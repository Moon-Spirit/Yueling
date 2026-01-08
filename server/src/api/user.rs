use axum::{extract::{State}, response::Json, routing::{post}, Router};
use serde::{Deserialize, Serialize};
use crate::storage::DbPool;
use crate::error::AppError;
use bcrypt::verify;

// 共享应用状态
use super::AppState;

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

/// 注册用户相关路由
pub fn register_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/user/exists", post(user_exists_handler))
}