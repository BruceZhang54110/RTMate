use dashmap::DashMap;
use std::sync::Arc;


type ClientId = Arc<String>;
type ChannelId = Arc<String>;

/// 客户端连接
pub struct ClientConnection {
    pub app_id: i64,
    pub rt_app: String,
    pub client_id: Arc<String>,
    pub connect_token: String,

}


/// ConnectionManager: 全局的连接管理中心
pub struct ConnectionManager {
    // 连接
    connections: DashMap<ClientId, Arc<ClientConnection>>,

    // 频道
    channels: DashMap<ChannelId, DashMap<ClientId, Arc<ClientConnection>>>
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager { 
            connections: DashMap::new(),
            channels: DashMap::new()
        }
    }

    pub fn add_connection(&self, conn: ClientConnection) -> ClientId {
        // 1. 【解构】消费 conn 的所有权，并将字段的所有权转移到局部变量。
        let ClientConnection {
            client_id, // Arc<String> 的所有权 moved 到此局部变量
            app_id: _app_id,
            rt_app: _rt_app,
            connect_token: _connect_token,
        } = conn; // conn 变量在这里被消费，没有部分移动的歧义。
        let client_id_for_key = client_id.clone();
        let cc_arc = Arc::new(
            ClientConnection {
            client_id, // Arc<String> 的所有权 moved 到此局部变量
            app_id: _app_id,
            rt_app: _rt_app,
            connect_token: _connect_token,
        });
        self.connections.insert(client_id_for_key.clone(), cc_arc);
        client_id_for_key
    }
}