use async_trait::async_trait;
use rtmate_common::models::RtApp;

#[async_trait]
pub trait RtAppRepositoryTrait: Send + Sync {
    /// 根据 app_id 查询 RtApp
    async fn get_rt_app_by_app_id(&self, query_app_id: &str) -> anyhow::Result<Option<RtApp>>;

    /// 创建新的 RtApp 记录（主要用于测试与后台管理）
    async fn create_rt_app(&self, app_id: &str, app_key: &str) -> anyhow::Result<RtApp>;

    /// 根据 app_id 删除 RtApp 记录（主要用于测试清理）
    async fn delete_rt_app_by_app_id(&self, app_id: &str) -> anyhow::Result<()>;
}
