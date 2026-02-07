use axum::{
    extract::{
        State,
        ws::{
            WebSocketUpgrade, 
            Message, 
            WebSocket
        }
    },
    routing::get,
    Router
};
use serde_json::{
    Value
};
use std::collections::HashMap;
use std::sync::{
    Arc, 
    Mutex
};
use futures_util::{
    SinkExt, 
    StreamExt
};
use tokio::sync::broadcast;
use uuid::Uuid;

/// 共享应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: crate::storage::DbPool,
    /// 用户ID到WebSocket广播通道的映射
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    /// 客户端ID到用户ID的映射，用于断开连接时清理资源
    client_user_map: Arc<Mutex<HashMap<String, String>>>,
    /// 全局广播通道，用于向所有客户端发送消息
    broadcaster: broadcast::Sender<String>,
    pub group_chat_broadcast_channel_map: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(db_pool: crate::storage::DbPool) -> Self {
        let (broadcaster, _) = broadcast::channel(100);
        Self {
            db_pool,
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_user_map: Arc::new(Mutex::new(HashMap::new())),
            broadcaster,
            group_chat_broadcast_channel_map: Arc::new(Mutex::new(HashMap::new()))
        }
    }
    
    /// 获取客户端映射（用于消息推送）
    pub fn get_clients(&self) -> &Arc<Mutex<HashMap<String, broadcast::Sender<String>>>> {
        &self.clients
    }
}

/// WebSocket连接升级处理器
async fn ws_handler(
    upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    upgrade.on_upgrade(|socket| handle_websocket(socket, state))
}

/// 处理WebSocket连接
async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4().to_string();
    
    // 创建客户端专用广播通道
    let (self_tx, mut self_rx) = broadcast::channel(100);
    
    println!("新WebSocket客户端连接: {}", client_id);
    
    // 广播新客户端连接消息
    let _ = state.broadcaster.send(format!("Client {} joined", client_id));
//---------------------------------------------------------------------------------------------------------------------------------------------------------------------
    // 初始化或获取用户加入的所有群聊的订阅广播通道
    let head = if let Some(Ok(Message::Text(text))) = receiver.next().await{
        let head: Value=serde_json::from_str(&text).unwrap(); //注意unwrap后续修复 
        println!("调试打印: {{来自ws的消息: {head}}}");
        if let Some(list_of_group_chats)=head["list_of_group_chats"].as_array() {
            println!("调试打印: {{群聊功能初始化: 此用户存在群}}");
            for group_id_value in list_of_group_chats { //为每个群聊创建一个广播通道
                if let Value::String(group_id)=group_id_value {
                    let mut group_chat_broadcast_channel_map= state.group_chat_broadcast_channel_map.lock().unwrap(); //注意unwrap后续修复
                    // 当前群广播通道已经创建过了直接克隆订阅端通道,并开启群消息接收任务
                    if group_chat_broadcast_channel_map.contains_key(group_id) {
                        let mut rx=group_chat_broadcast_channel_map[group_id].subscribe();
                        let self_tx=self_tx.clone();
                        tokio::spawn(async move {
                            while let Ok(msg) = rx.recv().await {
                                if self_tx.send(msg).is_err() {
                                    break;
                                }
                            }
                        });
                        continue;
                    }
                    // 当前群没有创建过群聊广播通道则创建,并开启消息接收任务
                    let (tx, mut rx) = broadcast::channel::<String>(100);
                    group_chat_broadcast_channel_map.insert(group_id.clone(),tx);
                    let self_tx=self_tx.clone();
                    tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await {
                            if self_tx.send(msg).is_err() {
                                break;
                            }   
                        }
                    });
                }
            }
        }else{
            println!("调试打印: {{群聊功能初始化: 此用户没有群}}");
        }
        head
    }else {
        Value::Null
    };
//-----------------------------------------------------------------------------------------------------------------------------------------------------------------
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    // 身份初始化
    if let Some(msg_type) = head.get("type").and_then(|x| x.as_str()) {
        match msg_type {
            "identify" => {
                // 处理客户端身份标识
                if let Some(user_id) = head.get("user_id").and_then(|x| x.as_str()) {
                    // 将客户端通道映射到用户ID，方便推送定向通知
                    let mut clients_map = state_clone.clients.lock().unwrap();
                    clients_map.insert(user_id.to_string(), self_tx.clone());
                    // 记录客户端ID到用户ID的映射，便于断开时清理
                    state_clone.client_user_map.lock().unwrap().insert(client_id_clone.clone(), user_id.to_string());
                    println!("WebSocket客户端 {} 标识为用户 {}", client_id_clone, user_id);
                }
            },
            _=>()
        }
    }
//----------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 身份初始化和群聊初始化先后顺序好像搞反了但不影响运行

    // 处理接收消息的任务
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // 尝试解析为JSON以处理特殊类型消息
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                println!("调试打印: {{来自ws的消息: {v}}}");
                if let Some(msg_type) = v.get("type").and_then(|x| x.as_str()) {
                    match msg_type {
                        // 好友消息分支
                        "friend_Message"=>{

                        },
                        // 群聊消息分支
                        "group_chat"  => {
                            if let Some(group_id)=v.get("group_id").and_then(|x| x.as_str()) {
                                let group_chat_broadcast_channel_map=state_clone.group_chat_broadcast_channel_map.lock().unwrap();
                                let content=if let Some(msg)= v.get("content").and_then(|x| x.as_str()) {
                                    msg
                                }else{
                                    ""
                                };
                                group_chat_broadcast_channel_map[group_id].send(content.to_string()).unwrap();
                            }
                        },
                        _ => {}
                    }
                }
            }
            println!("从客户端 {} 收到消息: {}", client_id_clone, text);
            // 广播消息给所有客户端
            let _ = state_clone.broadcaster.send(format!("{}: {}", client_id_clone, text));
        }
    });
    
    // 处理发送消息的任务
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = self_rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });
    
    // 等待任一任务结束
    tokio::select! {
        _ = recv_task => (),
        _ = send_task => (),
    }
    
    // 清理资源：移除客户端映射
    {
        let mut clients = state.clients.lock().unwrap();
        clients.remove(&client_id);
    }

    // 清理用户ID映射（如果存在）
    {
        let mut client_user_map = state.client_user_map.lock().unwrap();
        if let Some(user_id) = client_user_map.remove(&client_id) {
            let mut clients = state.clients.lock().unwrap();
            clients.remove(&user_id);
        }
    }

    println!("WebSocket客户端断开连接: {}", client_id);
    // 广播客户端断开连接消息
    let _ = state.broadcaster.send(format!("Client {} left", client_id));
}

/// 注册WebSocket路由
pub fn register_ws_route() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}
