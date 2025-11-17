use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use rtmate_common::dao::Dao;
use diesel::SelectableHelper;
use diesel::PgConnection;

use rtmate_common::models::RtApp;
use rtmate_common::models::RtClientConnection;
use rtmate_common::schema::rt_app::dsl::*;


#[allow(async_fn_in_trait)]
pub trait DaoQuery {
    /// 根据 app_id 查询 RtApp
    async fn get_rt_app_by_app_id(&self, query_app_id: &str) -> anyhow::Result<Option<RtApp>>;

    /// 根据 connect_token 查询客户端连接请求记录
    async fn get_rt_client_connection_by_token(&self, query_connect_token: &str) -> anyhow::Result<Option<RtClientConnection>>;

    /// 标记 connect_token 为已使用
    async fn mark_connection_token_used(&self, connect_token: &str) -> anyhow::Result<()>;

}

impl DaoQuery for Dao {

    async fn get_rt_app_by_app_id(&self, query_app_id: &str) -> anyhow::Result<Option<RtApp>> {
        let pg_connection = self.get_connection().await?;
        let query_app_id = query_app_id.to_owned();
        let result = pg_connection.interact(move |conn: &mut diesel::PgConnection| {
            rt_app
                .filter(app_id.eq(query_app_id))
                .select(RtApp::as_select())
                .first::<RtApp>(conn)
                .optional()
        }).await.map_err(|e| anyhow::anyhow!("Query failed: {}", e))??;
        Ok(result)
    }

    /// 根据 connect_token 查询还未创建成功的 RtClientConnection token
    async fn get_rt_client_connection_by_token(&self, query_connect_token: &str) -> anyhow::Result<Option<RtClientConnection>> {
        let pg_connection = self.get_connection().await?;
        let connect_token_query = query_connect_token.to_owned();
        use rtmate_common::schema::rt_client_connection::dsl::*;
        let result = pg_connection.interact(move |conn: &mut PgConnection| {
            rt_client_connection
                .filter(connect_token.eq(connect_token_query))
                .filter(used.eq(false))
                .select(RtClientConnection::as_select())
                .first::<RtClientConnection>(conn)
                .optional()
        }).await.map_err(|e| anyhow::anyhow!("Query failed: {}", e))??;
        Ok(result)
    }

    async fn mark_connection_token_used(&self, connect_token: &str) -> anyhow::Result<()> {
        let pg_connection = self.get_connection().await?;
        let connect_token_value = connect_token.to_owned();
        tracing::debug!("mark_connection_token_used connect_token_value: {}", connect_token_value);
        pg_connection.interact(move |conn: &mut PgConnection| {
            use rtmate_common::schema::rt_client_connection::dsl::*;
            diesel::update(rt_client_connection.filter(connect_token.eq(connect_token_value)))
                .set(used.eq(true))
                .execute(conn)
        }).await.map_err(|e| anyhow::anyhow!("Update failed: {}", e))??;
        Ok(())
    }
}

