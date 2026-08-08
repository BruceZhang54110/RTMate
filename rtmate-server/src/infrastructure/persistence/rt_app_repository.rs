use async_trait::async_trait;
use chrono::Local;
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

#[derive(diesel::Insertable)]
#[diesel(table_name = rtmate_common::schema::rt_app)]
struct NewRtApp {
    app_id: String,
    app_key: String,
    created_time: Option<chrono::DateTime<Local>>,
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

    async fn create_rt_app(&self, app_id: &str, app_key: &str) -> anyhow::Result<RtApp> {
        let pg_connection = self.data_source.get_connection().await?;
        let app_id = app_id.to_string();
        let app_key = app_key.to_string();
        Ok(pg_connection
            .interact(move |conn: &mut diesel::PgConnection| {
                diesel::insert_into(rt_app_dsl::rt_app)
                    .values(&NewRtApp {
                        app_id: app_id.clone(),
                        app_key: app_key.clone(),
                        created_time: Some(Local::now()),
                    })
                    .execute(conn)?;
                rt_app_dsl::rt_app
                    .filter(rt_app_dsl::app_id.eq(app_id))
                    .select(RtApp::as_select())
                    .first::<RtApp>(conn)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Create rt_app failed: {}", e))??)
    }

    async fn delete_rt_app_by_app_id(&self, app_id: &str) -> anyhow::Result<()> {
        let pg_connection = self.data_source.get_connection().await?;
        let app_id = app_id.to_string();
        pg_connection
            .interact(move |conn: &mut diesel::PgConnection| {
                diesel::delete(rt_app_dsl::rt_app.filter(rt_app_dsl::app_id.eq(app_id)))
                    .execute(conn)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Delete rt_app failed: {}", e))??;
        Ok(())
    }
}
