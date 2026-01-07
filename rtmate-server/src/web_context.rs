
use rtmate_common::dao::Dao;
use crate::manager::ConnectionManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct WebContext {

    // 数据源
    pub dao: Dao,

    // 连接管理器
    pub connection_manager: Arc<ConnectionManager>,
    
}

impl WebContext {
    pub async fn new() -> anyhow::Result<Self> {
        let dao = Dao::new().await?;
        let connection_manager = Arc::new(ConnectionManager::new());
        Ok(WebContext { dao, connection_manager })
    }

}