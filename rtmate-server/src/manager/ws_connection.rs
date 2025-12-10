use dashmap::DashMap;
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::common::RtWsError;
use crate::common::WsBizCode;
use crate::req::MessageSendPayload;
use dashmap::DashSet;


type ClientId = Arc<String>;
type ChannelId = Arc<String>;
type ChannelSet = DashSet<ChannelId>;

/// 客户端连接
pub struct ClientConnection {
    /// app_id
    pub rt_app: String,
    /// client_id
    pub client_id: Arc<String>,
    /// 连接ws 的connect token
    pub connect_token: String,

    // 发送消息
    pub sender: mpsc::Sender<MessageSendPayload>

}


/// ConnectionManager: 全局的连接管理中心
pub struct ConnectionManager {
    /// 连接
    connections: DashMap<ClientId, Arc<ClientConnection>>,

    /// 频道
    channels: DashMap<ChannelId, DashMap<ClientId, Arc<ClientConnection>>>,
    /// 每个客户端订阅的频道
    subscriptions: DashMap<ClientId, ChannelSet>

}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager { 
            connections: DashMap::new(),
            channels: DashMap::new(),
            subscriptions: DashMap::new()
        }
    }

    pub fn add_connection(&self, conn: ClientConnection) -> ClientId {
        // 1. 【解构】消费 conn 的所有权，并将字段的所有权转移到局部变量。
        let ClientConnection {
            client_id,
            rt_app,
            connect_token,
            sender,
        } = conn; // conn 变量在这里被消费，没有部分移动的歧义。

        let client_id_for_key = client_id.clone();
        let cc_arc = Arc::new(
            ClientConnection {
            client_id,
            rt_app,
            connect_token,
            sender,
        });
        self.connections.insert(client_id_for_key.clone(), cc_arc);
        client_id_for_key
    }

    /// 移除连接，并清理其所有订阅记录
    fn remove_connection(&self, client_id: &ClientId) {
        if self.connections.remove(client_id).is_none() {
            tracing::warn!("Attempted to remove non-existent client: {}", client_id);
            return;
        }
        tracing::info!("Client {} disconnected and removed from connections.", client_id);
        // 移除该客户端的订阅
        // type ChannelSet = DashSet<ChannelId>;

        let sub_entry: Option<(Arc<String>, DashSet<Arc<String>>)> = self.subscriptions.remove(client_id);
        if let Some((_, channel_set)) = sub_entry {
            for channel_id in channel_set.into_iter() {
                let should_cleanup = {
                    // 尝试获取频道内部 DashMap 的可变引用
                    let mut inner_map_entry = match self.channels.get_mut(&channel_id) {
                        Some(entry) => entry,
                        None => continue, // 频道可能已被其他线程或流程移除，跳过
                    };
                    // DashMap<ClientId, Arc<ClientConnection>>
                    let client_id_map = inner_map_entry.value_mut();
                    client_id_map.remove(client_id);
                    client_id_map.is_empty()
                };
                if should_cleanup {
                    self.channels.remove(&channel_id);
                    tracing::debug!("Channel {} is now empty and removed.", channel_id);
                }
                
            }
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
        let sub_set_entry = self.subscriptions.entry(client_id.clone())
            .or_insert_with(|| {
                tracing::debug!("Creating new subscription set for client: {}", client_id);
                DashSet::new()
            });
        sub_set_entry.value().insert(channel_id);
        
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
        // subscriptions: DashMap<ClientId, ChannelSet>
        if let Some(mut sub_set_entry)  = self.subscriptions.get_mut(&client_id) {
            sub_set_entry.value_mut().remove(&channel_id);
            if sub_set_entry.value().is_empty() {
                drop(sub_set_entry); // 显式释放 DashMapRefMut 的锁
                self.subscriptions.remove(&client_id);
            }
        }
        tracing::info!("Client '{}' unsubscribed from channel '{}'.", client_id, channel_id);
        Ok(())
    }
}