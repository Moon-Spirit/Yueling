use rusqlite::{params, Connection, Result};
use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

// 用户模型（对应数据库表）
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,          // UUID主键
    pub username: String,    // 用户名（唯一）
    pub email: String,       // 邮箱（唯一）
    pub password_hash: String, // bcrypt哈希后的密码
    pub created_at: i64,     // 创建时间戳（Unix秒）
    pub avatar_url: String,  // 头像URL
}

// 消息模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,          // UUID主键
    pub sender_id: String,   // 发送者ID
    pub receiver_id: String, // 接收者ID（用户或群聊）
    pub content: String,     // 消息内容
    pub message_type: String, // 消息类型："private"或"group"
    pub created_at: i64,     // 创建时间戳
    pub is_read: bool,       // 是否已读
}

// 好友关系模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Friendship {
    pub id: String,          // UUID主键
    pub user_id: String,     // 用户ID
    pub friend_id: String,   // 好友ID
    pub status: String,      // 好友状态："pending"或"accepted"
    pub created_at: i64,     // 创建时间戳
}

// 群聊模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,          // UUID主键
    pub name: String,        // 群聊名称
    pub creator_id: String,  // 创建者ID
    pub created_at: i64,     // 创建时间戳
}

// 群聊成员模型
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,          // UUID主键
    pub group_id: String,    // 群聊ID
    pub user_id: String,     // 用户ID
    pub joined_at: i64,      // 加入时间戳
    pub role: String,        // 角色："owner"或"member"
}

// 好友请求响应
#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRequest {
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub status: String, // "pending", "accepted", "rejected"
    pub created_at: i64,
}

// 数据库连接池（线程安全）
#[derive(Clone)]
pub struct DbPool(pub Arc<Mutex<Connection>>);

impl DbPool {
    // 初始化数据库连接并创建所有表
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // 创建表（若不存在）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                avatar_url TEXT DEFAULT '',
                user_data TEXT
            )",
            [],
        )?;
        
        // 创建消息表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                sender_id TEXT NOT NULL,
                receiver_id TEXT NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                is_read INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(sender_id) REFERENCES users(id)
            )",
            [],
        )?;
        
        // 创建好友关系表（修改为支持双向好友关系）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS friendships (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                friend_id TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id),
                FOREIGN KEY(friend_id) REFERENCES users(id),
                UNIQUE(user_id, friend_id)
            )",
            [],
        )?;
        
        // 创建好友请求表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS friend_requests (
                id TEXT PRIMARY KEY,
                from_user_id TEXT NOT NULL,
                to_user_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at INTEGER NOT NULL,
                FOREIGN KEY(from_user_id) REFERENCES users(id),
                FOREIGN KEY(to_user_id) REFERENCES users(id),
                UNIQUE(from_user_id, to_user_id)
            )",
            [],
        )?;
        
        // 创建群聊表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                name TEXT NOT NULL,
                creator_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(creator_id) REFERENCES users(id),
                group_user_list: TEXT
            )",
            [],
        )?;
        
        // 创建群聊成员表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS group_members (
                id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                joined_at INTEGER NOT NULL,
                role TEXT NOT NULL,
                FOREIGN KEY(group_id) REFERENCES groups(id),
                FOREIGN KEY(user_id) REFERENCES users(id),
                UNIQUE(group_id, user_id)
            )",
            [],
        )?;
        
        Ok(Self(Arc::new(Mutex::new(conn))))
    }

    // 注册新用户（核心逻辑）
    pub fn register_user(
        &self,
        username: &str,
        _email: &str, // 保留参数但忽略，保持向后兼容
        password: &str,
    ) -> Result<User> {
        let conn = self.0.lock().unwrap();
        
        // 检查用户名是否已存在
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)",
            [username],
            |row| row.get(0),
        )?;
        
        if exists {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(0),
                Some("用户名已存在".to_string())
            ));
        }

        // 密码哈希（bcrypt）
        let password_hash = hash(password, DEFAULT_COST).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;

        // 插入数据库
        let user_id = Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        // 生成唯一占位邮箱（避免使用空字符串导致 UNIQUE 约束冲突）
        let email_placeholder = format!("{}@local", user_id);

        conn.execute(
            "INSERT INTO users (id, username, email, password_hash, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, username, &email_placeholder, &password_hash, created_at],
        )?;

        // 返回新用户（不含敏感信息）
        Ok(User {
            id: user_id,
            username: username.to_string(),
            email: email_placeholder,
            password_hash,
            created_at,
            avatar_url: String::new(),
        })
    }
    
    // 发送消息
    pub fn send_message(
        &self,
        sender_id: &str,
        receiver_id: &str,
        content: &str,
        message_type: &str,
    ) -> Result<Message> {
        let conn = self.0.lock().unwrap();
        
        let message_id = Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        conn.execute(
            "INSERT INTO messages (id, sender_id, receiver_id, content, message_type, created_at, is_read) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![message_id, sender_id, receiver_id, content, message_type, created_at, false],
        )?;
        
        Ok(Message {
            id: message_id,
            sender_id: sender_id.to_string(),
            receiver_id: receiver_id.to_string(),
            content: content.to_string(),
            message_type: message_type.to_string(),
            created_at,
            is_read: false,
        })
    }
    
    // 获取用户的未读消息
    pub fn get_unread_messages(&self, user_id: &str) -> Result<Vec<Message>> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sender_id, receiver_id, content, message_type, created_at, is_read 
             FROM messages 
             WHERE receiver_id = ? AND is_read = 0 AND message_type = 'private'"
        )?;
        
        let messages = stmt.query_map([user_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                sender_id: row.get(1)?,
                receiver_id: row.get(2)?,
                content: row.get(3)?,
                message_type: row.get(4)?,
                created_at: row.get(5)?,
                is_read: row.get(6)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
        
        Ok(messages)
    }
    
    // 将消息标记为已读
    pub fn mark_messages_as_read(&self, message_ids: &[String]) -> Result<()> {
        let conn = self.0.lock().unwrap();
        
        for message_id in message_ids {
            conn.execute(
                "UPDATE messages SET is_read = 1 WHERE id = ?",
                [message_id],
            )?;
        }
        
        Ok(())
    }
    
    // 获取用户好友列表
    pub fn get_friends(&self, user_id: &str) -> Result<Vec<User>> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT u.id, u.username, u.email, u.password_hash, u.created_at, u.avatar_url 
             FROM users u 
             JOIN friendships f ON u.id = f.friend_id 
             WHERE f.user_id = ? AND f.status = 'accepted'"
        )?;
        
        let friends = stmt.query_map([user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                password_hash: row.get(3)?,
                created_at: row.get(4)?,
                avatar_url: row.get(5)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
        
        Ok(friends)
    }

    // 添加好友功能相关方法

    // 搜索用户
    pub fn search_users(&self, query: &str) -> Result<Vec<User>> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, email, password_hash, created_at, avatar_url 
             FROM users 
             WHERE username LIKE ? OR id LIKE ? 
             LIMIT 10"
        )?;
        
        let search_pattern = format!("%{}%", query);
        let users = stmt.query_map(
            params![&search_pattern, &search_pattern],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    created_at: row.get(4)?,
                    avatar_url: row.get(5)?,
                })
            },
        )?
        .filter_map(Result::ok)
        .collect();
        
        Ok(users)
    }

    // 检查用户是否存在（根据用户ID）
    pub fn user_exists_by_id(&self, user_id: &str) -> Result<bool> {
        let conn = self.0.lock().unwrap();
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)",
            [user_id],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    // 发送好友请求
    pub fn send_friend_request(&self, from_user_id: &str, to_username: &str) -> Result<FriendRequest> {
        let conn = self.0.lock().unwrap();
        
        // 检查目标用户是否存在
        let to_user_id: String = conn.query_row(
            "SELECT id FROM users WHERE username = ?",
            [to_username],
            |row| row.get(0),
        )?;
        
        // 检查是否已经是好友
        let is_friend: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM friendships WHERE user_id = ? AND friend_id = ? AND status = 'accepted')",
            params![from_user_id, to_user_id],
            |row| row.get(0),
        )?;
        
        if is_friend {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some("Already friends".to_string())
            ));
        }
        
        // 检查是否已经发送过请求
        let has_pending_request: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM friend_requests WHERE from_user_id = ? AND to_user_id = ? AND status = 'pending')",
            params![from_user_id, to_user_id],
            |row| row.get(0),
        )?;
        
        if has_pending_request {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(2),
                Some("Friend request already sent".to_string())
            ));
        }
        
        // 创建好友请求
        let request_id = Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        conn.execute(
            "INSERT INTO friend_requests (id, from_user_id, to_user_id, status, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![request_id, from_user_id, to_user_id, "pending", created_at],
        )?;
        
        Ok(FriendRequest {
            id: request_id,
            from_user_id: from_user_id.to_string(),
            to_user_id,
            status: "pending".to_string(),
            created_at,
        })
    }

    // 获取收到的好友请求
    pub fn get_received_friend_requests(&self, user_id: &str) -> Result<Vec<FriendRequest>> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT fr.id, fr.from_user_id, fr.to_user_id, fr.status, fr.created_at
             FROM friend_requests fr
             WHERE fr.to_user_id = ? AND fr.status = 'pending'"
        )?;
        
        let requests = stmt.query_map([user_id], |row| {
            Ok(FriendRequest {
                id: row.get(0)?,
                from_user_id: row.get(1)?,
                to_user_id: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
        
        Ok(requests)
    }

    // 响应好友请求
    pub fn respond_to_friend_request(&self, request_id: &str, responder_id: &str, response: &str) -> Result<Friendship> {
        let conn = self.0.lock().unwrap();

        // 验证请求存在且由该接收方(responder)处理
        // 查询出原始发送者(from_user_id)和当前状态
        let (from_user_id, current_status): (String, String) = conn.query_row(
            "SELECT from_user_id, status FROM friend_requests WHERE id = ? AND to_user_id = ?",
            params![request_id, responder_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        if current_status != "pending" {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(3),
                Some("Friend request already processed".to_string())
            ));
        }

        // 更新好友请求状态
        conn.execute(
            "UPDATE friend_requests SET status = ? WHERE id = ?",
            params![response, request_id],
        )?;

        if response == "accepted" {
            // 创建双向好友关系（from_user_id <-> responder_id）
            let friendship_id = Uuid::new_v4().to_string();
            let created_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // 正向关系（发送者的好友是接收者）
            conn.execute(
                "INSERT INTO friendships (id, user_id, friend_id, status, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![friendship_id, from_user_id, responder_id, "accepted", created_at],
            )?;

            // 反向关系（接收者的好友是发送者）
            let reverse_friendship_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO friendships (id, user_id, friend_id, status, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![reverse_friendship_id, responder_id, from_user_id, "accepted", created_at],
            )?;

            Ok(Friendship {
                id: friendship_id,
                user_id: from_user_id.to_string(),
                friend_id: responder_id.to_string(),
                status: "accepted".to_string(),
                created_at,
            })
        } else {
            // 拒绝请求，只更新状态，不创建好友关系
            Ok(Friendship {
                id: request_id.to_string(),
                user_id: from_user_id.to_string(),
                friend_id: responder_id.to_string(),
                status: "rejected".to_string(),
                created_at: 0,
            })
        }
    }

    // 删除好友
    pub fn remove_friend(&self, user_id: &str, friend_id: &str) -> Result<()> {
        let conn = self.0.lock().unwrap();
        
        // 删除双向好友关系
        conn.execute(
            "DELETE FROM friendships WHERE user_id = ? AND friend_id = ?",
            params![user_id, friend_id],
        )?;
        
        conn.execute(
            "DELETE FROM friendships WHERE user_id = ? AND friend_id = ?",
            params![friend_id, user_id],
        )?;
        
        Ok(())
    }

    // 更新用户头像URL
    pub fn update_user_avatar(&self, user_id: &str, avatar_url: &str) -> Result<()> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "UPDATE users SET avatar_url = ? WHERE id = ?",
            params![avatar_url, user_id],
        )?;
        Ok(())
    }

    // 更新用户信息
    pub fn update_user_info(&self, user_id: &str, username: &str, email: &str) -> Result<()> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "UPDATE users SET username = ?, email = ? WHERE id = ?",
            params![username, email, user_id],
        )?;
        Ok(())
    }

    // 根据ID获取用户
    pub fn get_user_by_id(&self, user_id: &str) -> Result<User> {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT id, username, email, password_hash, created_at, avatar_url FROM users WHERE id = ?",
            [user_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    created_at: row.get(4)?,
                    avatar_url: row.get(5)?,
                })
            },
        )
    }
}