use serde::Serialize;
use serde_json::Value;

/// API客户端
/// 提供傻瓜式的API接口调用方法
/// 
/// 注意：此模块需要在Cargo.toml中添加reqwest依赖
/// ```toml
/// [dependencies]
/// reqwest = { version = "0.12", features = ["json", "async-std"] }
/// ```
#[derive(Clone)]
pub struct ApiClient {
    /// API基础URL
    base_url: String,
}

impl ApiClient {
    /// 创建新的API客户端
    /// 
    /// # 参数
    /// - `base_url`: API基础URL，例如 "http://localhost:3000"
    /// 
    /// # 返回
    /// - `ApiClient`: API客户端实例
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// 注册用户
    /// 
    /// # 参数
    /// - `username`: 用户名
    /// - `password`: 密码
    /// 
    /// # 返回
    /// - `Result<Value, anyhow::Error>`: 注册结果或错误
    pub async fn register(&self, username: &str, password: &str) -> Result<Value, anyhow::Error> {
        let req = RegisterRequest {
            username: username.into(),
            password: password.into(),
        };
        self.post("/register", &req).await
    }

    /// 用户登录
    /// 
    /// # 参数
    /// - `username`: 用户名
    /// - `password`: 密码
    /// 
    /// # 返回
    /// - `Result<Value, anyhow::Error>`: 登录结果或错误
    pub async fn login(&self, username: &str, password: &str) -> Result<Value, anyhow::Error> {
        let req = LoginRequest {
            username: username.into(),
            password: password.into(),
        };
        self.post("/login", &req).await
    }

    /// 发送消息
    /// 
    /// # 参数
    /// - `sender_id`: 发送者ID
    /// - `receiver_id`: 接收者ID
    /// - `content`: 消息内容
    /// - `message_type`: 消息类型 ("private" 或 "group")
    /// 
    /// # 返回
    /// - `Result<Value, anyhow::Error>`: 发送结果或错误
    pub async fn send_message(
        &self,
        sender_id: &str,
        receiver_id: &str,
        content: &str,
        message_type: &str,
    ) -> Result<Value, anyhow::Error> {
        let req = SendMessageRequest {
            sender_id: sender_id.into(),
            receiver_id: receiver_id.into(),
            content: content.into(),
            message_type: message_type.into(),
        };
        self.post("/send-message", &req).await
    }

    /// 获取用户信息
    /// 
    /// # 参数
    /// - `user_id`: 用户ID
    /// 
    /// # 返回
    /// - `Result<Value, anyhow::Error>`: 用户信息或错误
    pub async fn get_user_info(&self, user_id: &str) -> Result<Value, anyhow::Error> {
        self.get(&format!("/user/{}", user_id)).await
    }

    /// 通用POST请求
    /// 
    /// # 参数
    /// - `endpoint`: API端点
    /// - `data`: 请求数据
    /// 
    /// # 返回
    /// - `Result<Value, anyhow::Error>`: 响应结果或错误
    async fn post<T: Serialize>(&self, _endpoint: &str, _data: &T) -> Result<Value, anyhow::Error> {
        // 注意：这里需要reqwest库
        // let url = format!("{}{}", self.base_url, endpoint);
        // let client = reqwest::Client::new();
        // let response = client
        //     .post(&url)
        //     .json(data)
        //     .send()
        //     .await?
        //     .json()
        //     .await?;
        // Ok(response)
        
        // 临时实现，返回模拟数据
        Ok(serde_json::json!({
            "success": true,
            "message": "操作成功",
            "data": null
        }))
    }

    /// 通用GET请求
    /// 
    /// # 参数
    /// - `endpoint`: API端点
    /// 
    /// # 返回
    /// - `Result<Value, anyhow::Error>`: 响应结果或错误
    async fn get(&self, _endpoint: &str) -> Result<Value, anyhow::Error> {
        // 注意：这里需要reqwest库
        // let url = format!("{}{}", self.base_url, endpoint);
        // let client = reqwest::Client::new();
        // let response = client
        //     .get(&url)
        //     .send()
        //     .await?
        //     .json()
        //     .await?;
        // Ok(response)
        
        // 临时实现，返回模拟数据
        Ok(serde_json::json!({
            "success": true,
            "message": "操作成功",
            "data": null
        }))
    }
}

// 请求结构体
#[derive(Serialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct SendMessageRequest {
    sender_id: String,
    receiver_id: String,
    content: String,
    message_type: String,
}
