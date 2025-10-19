use crate::domain::repositories::rt_app_repository_trait::RtAppRepositoryTrait;
use rtmate_common::dao::DataSource;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use std::sync::Arc;

use rtmate_common::schema::rt_app::dsl as rt_app_dsl;
use rtmate_common::schema::rt_client_connection::dsl as conn_dsl;
use async_trait::async_trait;
use rtmate_common::models::{RtApp, NewRtClientConnection};

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
        let result = pg_connection.interact(move |conn: &mut diesel::PgConnection| {
            rt_app_dsl::rt_app
                .filter(rt_app_dsl::app_id.eq(query_app_id))
                .select(RtApp::as_select())
                .first::<RtApp>(conn)
                .optional()
        }).await.map_err(|e| anyhow::anyhow!("Query failed: {}", e))??;
        Ok(result)
    }
    
    async fn save_connect_token(&self, new_connection: NewRtClientConnection) -> anyhow::Result<()> {
        let pg_connection = self.data_source.get_connection().await?;
        pg_connection
            .interact(move |conn: &mut diesel::PgConnection| {
                diesel::insert_into(conn_dsl::rt_client_connection)
                    .values(&new_connection)
                    .execute(conn)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Insert rt app new_connection failed: {}", e))??;
        Ok(())
    }
}