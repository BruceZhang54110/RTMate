use rtmate_common::models::RtApp;
use async_trait::async_trait;

#[async_trait]
pub trait RtAppRepositoryTrait: Send + Sync {
    /// 根据 app_id 查询 RtApp
    async fn get_rt_app_by_app_id(&self, query_app_id: &str) -> anyhow::Result<Option<RtApp>>;

}