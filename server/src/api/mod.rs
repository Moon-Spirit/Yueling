use axum::{Router};

/// API模块
/// 提供所有HTTP和WebSocket接口

// 导入子模块
mod user;     // 用户相关API
mod friend;   // 好友相关API
mod message;  // 消息相关API
mod ws;       // WebSocket相关API
mod response; // 统一响应格式
mod client;   // API客户端

// 重新导出AppState、统一响应格式和API客户端，以便其他模块可以通过super::AppState导入
pub use ws::AppState;
pub use response::ApiResponse;
pub use client::ApiClient;

/// 注册所有API路由
/// 
/// # 参数
/// - `db_pool`: 数据库连接池
/// 
/// # 返回
/// - `Router`: 配置好的Axum路由器
pub fn register_routes(db_pool: crate::storage::DbPool) -> Router {
    // 创建共享应用状态
    let app_state = ws::AppState::new(db_pool);
    
    // 主路由器配置
    Router::new()
        // WebSocket路由
        .merge(ws::register_ws_route())
        // 用户相关路由
        .merge(user::register_routes())
        // 好友相关路由
        .merge(friend::register_routes())
        // 消息相关路由
        .merge(message::register_routes())
        .with_state(app_state)
}
