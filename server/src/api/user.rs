use axum::{
    extract::{
        State, 
        Multipart, 
        Path
    },
    response::{
        Json, 
        IntoResponse
    }, 
    routing::{
        post, 
        get
    }, 
    Router, 
    routing::put
};
use serde::{
    Deserialize, 
    Serialize
};
use crate::error::AppError;
use bcrypt::{
    verify
};
use std::fs;
use std::path::Path as FilePath;
use uuid::Uuid;
use http::{
    header::CONTENT_TYPE
};
use mime_guess::from_path;
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

// 头像上传响应体
#[derive(Serialize)]
pub struct AvatarUploadResponse {
    pub success: bool,
    pub message: String,
    pub avatar_url: Option<String>,
}

// 通用成功响应体
#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

// 更新用户信息请求体
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
    pub email: String,
}

// 用户信息响应体
#[derive(Serialize)]
pub struct UserInfoResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<serde_json::Value>,
}

// 注册处理器（核心API逻辑）
pub async fn register_handler(
    State(state): State<AppState>, // 注入共享状态
    Json(req): Json<RegisterRequest>, // 解析JSON请求体
) -> Result<Json<RegisterResponse>, AppError> {

    
    // 调用存储层注册用户（使用原始密码）
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
    // 使用私有算法和公有算法加密密码（与注册时相同）

    
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

    // 验证密码（使用解密后的原始密码）
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

// 获取用户信息处理器
pub async fn get_user_info_handler(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<UserInfoResponse>, AppError> {
    // 获取用户信息
    let user = state.db_pool.get_user_by_id(&user_id).map_err(|e| AppError::Database(e.to_string()))?;
    
    // 转换为JSON值，不包含敏感信息
    let user_json = serde_json::json!({
        "id": user.id,
        "username": user.username,
        "avatar_url": user.avatar_url,
        "created_at": user.created_at,
    });
    
    Ok(Json(UserInfoResponse {
        success: true,
        message: "获取用户信息成功".into(),
        user: Some(user_json),
    }))
}

// 头像上传处理器
pub async fn upload_avatar_handler(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<AvatarUploadResponse>, AppError> {
    // 创建上传目录
    let upload_dir = FilePath::new("./uploads/avatars");
    if !upload_dir.exists() {
        fs::create_dir_all(upload_dir).map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // 处理文件上传
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or("file");
        if name != "avatar" {
            continue;
        }

        let filename = field.file_name().unwrap_or("");
        let extension = FilePath::new(filename).extension().and_then(|ext| ext.to_str()).unwrap_or("png");
        
        // 生成唯一文件名
        let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
        let filepath = upload_dir.join(&unique_filename);
        
        // 读取文件内容
        let file_content = field.bytes().await.map_err(|e| AppError::Internal(e.to_string()))?;
        
        // 保存文件
        fs::write(&filepath, file_content).map_err(|e| AppError::Internal(e.to_string()))?;
        
        // 更新用户头像URL
        let avatar_url = format!("/uploads/avatars/{}", unique_filename);
        state.db_pool.update_user_avatar(&user_id, &avatar_url).map_err(|e| AppError::Database(e.to_string()))?;
        
        // 返回成功响应
        return Ok(Json(AvatarUploadResponse {
            success: true,
            message: "头像上传成功".into(),
            avatar_url: Some(avatar_url),
        }));
    }

    Err(AppError::Internal("未找到头像文件".into()))
}

// 获取头像处理器
pub async fn get_avatar_handler(
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let filepath = FilePath::new("./uploads/avatars").join(filename);
    
    if !filepath.exists() {
        return Err(AppError::NotFound("头像文件不存在".into()));
    }
    
    // 读取文件内容
    let file_content = fs::read(&filepath).map_err(|e| AppError::Internal(e.to_string()))?;
    
    // 猜测MIME类型
    let mime_type = from_path(&filepath).first_or_octet_stream().to_string();
    
    // 构建响应
    Ok((
        [(CONTENT_TYPE, mime_type)],
        file_content,
    ))
}

// 更新用户信息处理器
pub async fn update_user_info_handler(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<SuccessResponse>, AppError> {
    // 更新用户信息
    state.db_pool.update_user_info(&user_id, &req.username, &req.email)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    Ok(Json(SuccessResponse {
        success: true,
        message: "用户信息更新成功".into(),
    }))
}

/// 注册用户相关路由
pub fn register_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/user/exists", post(user_exists_handler))
        .route("/user/{user_id}", get(get_user_info_handler))
        .route("/user/{user_id}", put(update_user_info_handler))
        .route("/user/{user_id}/avatar", post(upload_avatar_handler))
        .route("/uploads/avatars/{filename}", get(get_avatar_handler))
}