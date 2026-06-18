use rtmate_common::dao::Dao;
use crate::manager::{ConnectionManager, BroadcastManager};
use std::sync::Arc;

#[derive(Clone)]
pub struct WebContext {

    // 数据源
    pub dao: Dao,

    // 连接管理器
    pub connection_manager: Arc<ConnectionManager>,

    // 广播管理器
    pub broadcast_manager: Arc<BroadcastManager>,
    
}

impl WebContext {
    pub async fn new() -> anyhow::Result<Self> {
        let dao = Dao::new().await?;
        let connection_manager = Arc::new(ConnectionManager::new());
        // 从环境变量读取 broadcast channel 容量，默认 1024
        let broadcast_capacity = std::env::var("BROADCAST_CHANNEL_CAPACITY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1024);
        let broadcast_manager = Arc::new(BroadcastManager::new(broadcast_capacity));
        Ok(WebContext { dao, connection_manager, broadcast_manager })
    }

}
