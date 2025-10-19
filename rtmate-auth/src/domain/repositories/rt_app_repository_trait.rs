use rtmate_common::models::{RtApp, NewRtClientConnection};
use async_trait::async_trait;

#[async_trait]
pub trait RtAppRepositoryTrait: Send + Sync {
    /// 根据 app_id 查询 RtApp
    async fn get_rt_app_by_app_id(&self, query_app_id: &str) -> anyhow::Result<Option<RtApp>>;
    /// 保存 connect_token
    async fn save_connect_token(&self, new_connection: NewRtClientConnection) -> anyhow::Result<()>;
}