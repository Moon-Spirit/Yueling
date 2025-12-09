use rusqlite::{params, Connection, Result as SqliteResult}; 
use crate::core::models::User;

/// 初始化数据库表（首次运行需创建`users`表）
pub fn init_db(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建用户（插入数据库）
pub fn create_user(conn: &Connection, user: &User) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO users (id, username, email, password_hash) VALUES (?1, ?2, ?3, ?4)",
        params![user.id, user.username, user.email, user.password_hash],
    )?;
    Ok(())
}