use async_trait::async_trait;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use rtmate_common::dao::DataSource;
use rtmate_common::models::{NewRtClientConnection, RtClientConnection};
use rtmate_common::schema::rt_client_connection::dsl as conn_dsl;
use std::sync::Arc;

use crate::domain::repositories::client_connection_repository_trait::ClientConnectionRepositoryTrait;

pub struct ClientConnectionRepository {
    data_source: Arc<DataSource>,
}

impl ClientConnectionRepository {
    pub fn new(data_source: Arc<DataSource>) -> Self {
        ClientConnectionRepository { data_source }
    }
}

#[async_trait]
impl ClientConnectionRepositoryTrait for ClientConnectionRepository {
    async fn get_rt_client_connection_by_token(
        &self,
        query_connect_token: &str,
    ) -> anyhow::Result<Option<RtClientConnection>> {
        let pg_connection = self.data_source.get_connection().await?;
        let connect_token_query = query_connect_token.to_owned();
        let result = pg_connection
            .interact(move |conn: &mut diesel::PgConnection| {
                conn_dsl::rt_client_connection
                    .filter(conn_dsl::connect_token.eq(connect_token_query))
                    .filter(conn_dsl::used.eq(false))
                    .limit(1)
                    .select(RtClientConnection::as_select())
                    .first::<RtClientConnection>(conn)
                    .optional()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Query rt_client_connection failed: {}", e))??;
        Ok(result)
    }

    async fn mark_connection_token_used(&self, connect_token: &str) -> anyhow::Result<()> {
        let pg_connection = self.data_source.get_connection().await?;
        let connect_token_value = connect_token.to_owned();
        pg_connection
            .interact(move |conn: &mut diesel::PgConnection| {
                diesel::update(conn_dsl::rt_client_connection.filter(conn_dsl::connect_token.eq(connect_token_value)))
                    .set(conn_dsl::used.eq(true))
                    .execute(conn)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Update rt_client_connection failed: {}", e))??;
        Ok(())
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
            .map_err(|e| anyhow::anyhow!("Insert rt_client_connection failed: {}", e))??;
        Ok(())
    }

    async fn delete_rt_client_connection_by_connect_token(
        &self,
        connect_token: &str,
    ) -> anyhow::Result<()> {
        let pg_connection = self.data_source.get_connection().await?;
        let connect_token_value = connect_token.to_owned();
        pg_connection
            .interact(move |conn: &mut diesel::PgConnection| {
                diesel::delete(conn_dsl::rt_client_connection.filter(conn_dsl::connect_token.eq(connect_token_value)))
                    .execute(conn)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Delete rt_client_connection failed: {}", e))??;
        Ok(())
    }
}
