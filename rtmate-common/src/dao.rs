
use serde::Deserialize;
use config::ConfigError;
use thiserror::Error;
use deadpool_postgres::Runtime;
use deadpool_diesel::postgres::BuildError;
use deadpool_diesel::postgres::Manager;
use deadpool_diesel::postgres::Pool;
use deadpool_diesel::Timeouts;
use deadpool_diesel::postgres::Object;


#[derive(Clone)]
pub struct DaoContext {

    // 数据源
    pub dao: Dao,
}

impl DaoContext {
    pub async fn new() -> anyhow::Result<Self> {
        let dao = Dao::new().await?;
        Ok(DaoContext { dao })
    }

}


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

#[derive(Clone)]
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

    pub async fn get_connection(&self) -> anyhow::Result<Object> {
        let conn  = self.pool.get().await?;
        Ok(conn)
    }



}

#[derive(Clone)]
pub struct Dao {
    data_source: DataSource,
}

impl Dao {
    pub async fn new() -> anyhow::Result<Self> {
        let data_source = DataSource::new().await?;
        Ok(Dao { data_source })
    }

    pub async fn get_connection(&self) -> anyhow::Result<Object> {
        let conn  = self.data_source.pool.get().await?;
        Ok(conn)
    }

}