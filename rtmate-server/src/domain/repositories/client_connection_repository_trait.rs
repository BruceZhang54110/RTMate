use async_trait::async_trait;
use rtmate_common::models::{NewRtClientConnection, RtClientConnection};

#[async_trait]
pub trait ClientConnectionRepositoryTrait: Send + Sync {
    /// 根据 connect_token 查询还未使用的客户端连接请求记录
    async fn get_rt_client_connection_by_token(
        &self,
        query_connect_token: &str,
    ) -> anyhow::Result<Option<RtClientConnection>>;

    /// 标记 connect_token 为已使用
    async fn mark_connection_token_used(&self, connect_token: &str) -> anyhow::Result<()>;

    /// 保存新的 connect_token 记录
    async fn save_connect_token(&self, new_connection: NewRtClientConnection) -> anyhow::Result<()>;

    /// 根据 connect_token 删除连接记录（主要用于测试清理）
    async fn delete_rt_client_connection_by_connect_token(
        &self,
        connect_token: &str,
    ) -> anyhow::Result<()>;
}
