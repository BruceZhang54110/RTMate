use dashmap::DashMap;
use std::sync::Arc;
use crate::common::RtWsError;
use crate::common::WsBizCode;


type ClientId = Arc<String>;
type ChannelId = Arc<String>;

/// 客户端连接
pub struct ClientConnection {
    /// app_id
    pub rt_app: String,
    /// client_id
    pub client_id: Arc<String>,
    /// 连接ws 的connect token
    pub connect_token: String,

}


/// ConnectionManager: 全局的连接管理中心
pub struct ConnectionManager {
    /// 连接
    connections: DashMap<ClientId, Arc<ClientConnection>>,

    /// 频道
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
            client_id,
            rt_app,
            connect_token,
        } = conn; // conn 变量在这里被消费，没有部分移动的歧义。

        let client_id_for_key = client_id.clone();
        let cc_arc = Arc::new(
            ClientConnection {
            client_id,
            rt_app,
            connect_token,
        });
        self.connections.insert(client_id_for_key.clone(), cc_arc);
        client_id_for_key
    }

    fn remove_connection(&self, client_id: &ClientId) {
        if self.connections.remove(client_id).is_some() {
            tracing::info!("Client {} disconnected and removed from connections.", client_id);
            // 移除连接后，移除所有频道中的这个client_id
            for channel_entry in self.channels.iter() {
                let connection_map = channel_entry.value();
                connection_map.remove(client_id);
                if connection_map.is_empty() {
                    let channel_id = channel_entry.key(); 
                    self.channels.remove(channel_entry.key());
                    tracing::debug!("Channel {} is now empty and removed.", channel_id);
                }
            }

        } else {
            tracing::warn!("Attempted to remove non-existent client: {}", client_id);
        }
    }

    /// 订阅频道
    fn subscribe(&self, client_id: ClientId, channel_id: ChannelId) -> Result<(), RtWsError> {
        let conn_arc = match self.connections.get(&client_id) {
            Some(entry) => entry.value().clone(),
            None => return Err(RtWsError::biz(WsBizCode::ClientNotActive)),
        };
        let channel_map_entry = self.channels.entry(channel_id.clone())
            .or_insert_with(|| {
                tracing::debug!("Creating new channel map for: {}", channel_id.clone());
                DashMap::new()
            });
        let channel_map = channel_map_entry.value();
        channel_map.insert(client_id.clone(), conn_arc);
        Ok(())
    }

    /// 取消订阅
    fn un_subscribe(&self, client_id: ClientId, channel_id: ChannelId) -> Result<(), RtWsError> {
        let should_cleanup = {
            let mut inner_map_entry = match self.channels.get_mut(&channel_id) {
                Some(entry) => entry,
                None => {
                    return Err(RtWsError::biz(WsBizCode::ChannelNotFound));
                }
            };
            let inner_map = inner_map_entry.value_mut();
            if inner_map.remove(&client_id).is_none() {
                drop(inner_map_entry); // // 明确释放锁
                return Err(RtWsError::biz(WsBizCode::NotSubscribed));
            }
            let is_empty = inner_map.is_empty();
            is_empty
        };
        if should_cleanup {
            self.channels.remove(&channel_id);
            tracing::info!("Channel '{}' is empty and has been removed.", channel_id);
        }
        tracing::info!("Client '{}' unsubscribed from channel '{}'.", client_id, channel_id);
        Ok(())
    }
}