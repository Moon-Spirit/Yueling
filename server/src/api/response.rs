use serde::Serialize;

/// 统一API响应格式
/// 标准化所有API接口的响应结构
#[derive(Serialize)]
pub struct ApiResponse<T> {
    /// 操作是否成功
    pub success: bool,
    /// 响应消息
    pub message: String,
    /// 响应数据
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    /// 
    /// # 参数
    /// - `data`: 响应数据
    /// - `message`: 响应消息
    /// 
    /// # 返回
    /// - `ApiResponse<T>`: 成功响应对象
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    /// 创建失败响应
    /// 
    /// # 参数
    /// - `message`: 错误消息
    /// 
    /// # 返回
    /// - `ApiResponse<T>`: 失败响应对象
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }

    /// 创建空成功响应
    /// 
    /// # 参数
    /// - `message`: 响应消息
    /// 
    /// # 返回
    /// - `ApiResponse<T>`: 空成功响应对象
    pub fn success_empty(message: &str) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }
}
