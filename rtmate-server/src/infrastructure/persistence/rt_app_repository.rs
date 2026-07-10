use async_trait::async_trait;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use rtmate_common::dao::DataSource;
use rtmate_common::models::RtApp;
use rtmate_common::schema::rt_app::dsl as rt_app_dsl;
use std::sync::Arc;

use crate::domain::repositories::rt_app_repository_trait::RtAppRepositoryTrait;

pub struct RtAppRepository {
    data_source: Arc<DataSource>,
}

impl RtAppRepository {
    pub fn new(data_source: Arc<DataSource>) -> Self {
        RtAppRepository { data_source }
    }
}

#[async_trait]
impl RtAppRepositoryTrait for RtAppRepository {
    async fn get_rt_app_by_app_id(&self, query_app_id: &str) -> anyhow::Result<Option<RtApp>> {
        let pg_connection = self.data_source.get_connection().await?;
        let query_app_id = query_app_id.to_owned();
        let result = pg_connection
            .interact(move |conn: &mut diesel::PgConnection| {
                rt_app_dsl::rt_app
                    .filter(rt_app_dsl::app_id.eq(query_app_id))
                    .select(RtApp::as_select())
                    .first::<RtApp>(conn)
                    .optional()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Query rt_app failed: {}", e))??;
        Ok(result)
    }
}
