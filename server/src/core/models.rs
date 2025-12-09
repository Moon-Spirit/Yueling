use serde::{Deserialize, Serialize};

/// 用户模型（对应数据库`users`表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,          // UUID v4（唯一标识）
    pub username: String,    // 用户名（唯一）
    pub email: String,       // 邮箱（唯一）
    pub password_hash: String, // bcrypt哈希后的密码
}