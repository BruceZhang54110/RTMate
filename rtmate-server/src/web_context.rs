use std::sync::Arc;

use rtmate_common::dao::DataSource;

use crate::domain::repositories::channel_repository_trait::ChannelRepositoryTrait;
use crate::domain::repositories::client_connection_repository_trait::ClientConnectionRepositoryTrait;
use crate::domain::repositories::rt_app_repository_trait::RtAppRepositoryTrait;
use crate::infrastructure::persistence::{
    ChannelRepository, ClientConnectionRepository, RtAppRepository,
};
use crate::manager::{BroadcastManager, ConnectionManager};

#[derive(Clone)]
pub struct WebContext {
    pub rt_app_repository: Arc<dyn RtAppRepositoryTrait>,
    pub channel_repository: Arc<dyn ChannelRepositoryTrait>,
    pub client_connection_repository: Arc<dyn ClientConnectionRepositoryTrait>,
    pub connection_manager: Arc<ConnectionManager>,
    pub broadcast_manager: Arc<BroadcastManager>,
}

impl WebContext {
    pub async fn new() -> anyhow::Result<Self> {
        let data_source = Arc::new(DataSource::new().await?);
        let rt_app_repository = Arc::new(RtAppRepository::new(data_source.clone()));
        let channel_repository = Arc::new(ChannelRepository::new(data_source.clone()));
        let client_connection_repository =
            Arc::new(ClientConnectionRepository::new(data_source));
        let connection_manager = Arc::new(ConnectionManager::new());
        // 从环境变量读取 broadcast channel 容量，默认 1024
        let broadcast_capacity = std::env::var("BROADCAST_CHANNEL_CAPACITY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1024);
        let broadcast_manager = Arc::new(BroadcastManager::new(broadcast_capacity));
        Ok(WebContext {
            rt_app_repository,
            channel_repository,
            client_connection_repository,
            connection_manager,
            broadcast_manager,
        })
    }
}
