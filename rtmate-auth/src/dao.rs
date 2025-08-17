
use anyhow::Ok;
use diesel::sql_types::Text;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use serde::Deserialize;
use config::ConfigError;
use deadpool_postgres::Runtime;
use thiserror::Error;
use deadpool_diesel::postgres::BuildError;
use deadpool_diesel::postgres::Manager;
use deadpool_diesel::postgres::Pool;
use deadpool_diesel::Timeouts;
use diesel::dsl::sql;
use diesel::QueryResult;
use diesel::SelectableHelper;

use crate::models::RtApp;
use crate::schema::rt_app::dsl::*;



#[derive(Debug, Deserialize)]
struct DbConfig {

    #[serde()]
    pub database_url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,

    #[serde(rename = "test_query", default = "default_test_query")]
    pub test_query: String,

    #[serde(flatten)]
    pub timeouts: Timeouts,
}

fn default_max_connections() -> usize {
    5
}

fn default_connect_timeout() -> u64 {
    10
}

fn default_test_query() -> String {
    "SELECT 1".to_string()
}


impl DbConfig {

    pub fn from_env() -> anyhow::Result<Self, ConfigError> {

        config::Config::builder()
            .add_source(config::Environment::default())
            .build()
            .unwrap()
            .try_deserialize()

    }

}

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("unable to load .env file")]
    Env(dotenvy::Error),

    #[error("missing DATABASE_URL")]
    DatabaseURL,

    #[error("unable to build pool")]
    PoolBuildError(BuildError),
}

pub struct DataSource {
    pool: deadpool_diesel::Pool<deadpool_diesel::Manager<diesel::PgConnection>>,
}


impl DataSource {
    pub async fn new() -> anyhow::Result<Self> {
        dotenvy::dotenv().map_err(PoolError::Env)?;

        let db_config= DbConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load database configuration: {}", e))?;
        
        
        let manager = Manager::new(db_config.database_url, Runtime::Tokio1);
        // Create a pool from the configuration

        let pool: deadpool_diesel::Pool<deadpool_diesel::Manager<diesel::PgConnection>> = Pool::builder(manager)
        .max_size(db_config.max_connections)
        .timeouts(db_config.timeouts)
        .build()
        .map_err(PoolError::PoolBuildError)?;

        Ok(DataSource { pool })

    }

}

pub struct Dao {
    data_source: DataSource,
}

impl Dao {
    pub async fn new() -> anyhow::Result<Self> {
        let data_source = DataSource::new().await?;
        Ok(Dao { data_source })
    }

    pub async fn query(&self) -> anyhow::Result<String> {
        let pg_connection = self.data_source.pool.get().await?;
        
        let query_result = pg_connection.interact(|conn: &mut diesel::PgConnection| -> QueryResult<String> {
            sql::<Text>("SELECT 'hello world'").get_result(conn)
        }).await;
        query_result.map_err(|e| anyhow::anyhow!("Query failed: {}", e))?
        .map_err(|e| anyhow::anyhow!("Query failed: {}", e))
    }

    pub async fn query_all_rt_app(&self) -> anyhow::Result<()> {
        let pg_connection = self.data_source.pool.get().await?;
        let _r = pg_connection.interact(|conn: &mut diesel::PgConnection| {
            let results = rt_app.limit(5)
                .filter(app_id.eq("dd"))
                .select(RtApp::as_select())
                .load(conn).expect("Error loading rt_app");
            println!("Number of apps: {}", results.len());
            for app in results {
                println!("App ID: {}, App Key: {}", app.app_id, app.app_key);
            }
        }).await;
        
        Ok(())


    }

}