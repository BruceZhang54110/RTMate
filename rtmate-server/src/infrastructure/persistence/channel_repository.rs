use async_trait::async_trait;
use std::sync::Arc;

use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use rtmate_common::dao::DataSource;
use rtmate_common::models::{NewRtChannel, RtChannel};
use rtmate_common::schema::rt_channel::dsl as channel_dsl;

use crate::domain::repositories::channel_repository_trait::ChannelRepositoryTrait;

pub struct ChannelRepository {
    data_source: Arc<DataSource>,
}

impl ChannelRepository {
    pub fn new(data_source: Arc<DataSource>) -> Self {
        ChannelRepository { data_source }
    }
}

#[async_trait]
impl ChannelRepositoryTrait for ChannelRepository {
    async fn find_by_app_and_name(
        &self,
        app_id_value: i64,
        name_value: &str,
    ) -> anyhow::Result<Option<RtChannel>> {
        let pg_connection = self.data_source.get_connection().await?;
        let name_value = name_value.to_owned();
        let result = pg_connection
            .interact(move |conn| {
                channel_dsl::rt_channel
                    .filter(channel_dsl::app_id.eq(app_id_value))
                    .filter(channel_dsl::name.eq(name_value))
                    .select(RtChannel::as_select())
                    .first::<RtChannel>(conn)
                    .optional()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Query channel failed: {}", e))??;
        Ok(result)
    }

    async fn create(&self, new_channel: NewRtChannel) -> anyhow::Result<RtChannel> {
        let pg_connection = self.data_source.get_connection().await?;
        let app_id = new_channel.app_id;
        let name = new_channel.name.clone();
        pg_connection
            .interact(move |conn| {
                diesel::insert_into(channel_dsl::rt_channel)
                    .values(&new_channel)
                    .execute(conn)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Insert channel failed: {}", e))??;

        self.find_by_app_and_name(app_id, &name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Channel not found after insert"))
    }
}
