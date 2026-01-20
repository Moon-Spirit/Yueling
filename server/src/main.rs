use server::{
    register_routes,
    DbPool
};

use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

/// 主函数：启动聊天服务器
/// 
/// 1. 初始化数据库连接池
/// 2. 构建API路由和WebSocket服务
/// 3. 配置CORS
/// 4. 启动HTTP和WebSocket服务器
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化数据库连接池
    let db_pool = DbPool::new("server.db")?;

    // 配置跨域资源共享（CORS）策略
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // 构建API路由
    let app = register_routes(db_pool.clone()).layer(cors);

    // 启动服务器
    let port = 2025;
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("服务器正在监听 http://{} (HTTP) 和 ws://{} (WebSocket)", addr, addr);
    
    // 启动HTTP和WebSocket服务
    axum::serve(listener, app).await?;
    
    Ok(())
}
