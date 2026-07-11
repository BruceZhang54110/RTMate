use async_trait::async_trait;
use rtmate_common::models::{NewRtChannel, RtChannel};

#[async_trait]
pub trait ChannelRepositoryTrait: Send + Sync {
    /// 根据 app_id 与频道名称查询频道
    async fn find_by_app_and_name(
        &self,
        app_id: i64,
        name: &str,
    ) -> anyhow::Result<Option<RtChannel>>;

    /// 创建新频道
    async fn create(&self, new_channel: NewRtChannel) -> anyhow::Result<RtChannel>;
}
