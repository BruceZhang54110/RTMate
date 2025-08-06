
use anyhow::Ok;
use serde::Deserialize;
use config::ConfigError;
use dotenvy::dotenv;
use tracing::warn;
use deadpool_postgres::Runtime;
use tokio_postgres::NoTls;

#[derive(Debug, Deserialize)]
struct Config {
    pg: deadpool_postgres::Config,
}

impl Config {

    pub fn from_env() -> anyhow::Result<Self, ConfigError> {

        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .unwrap()
            .try_deserialize()

    }

}

#[derive(Debug)]
pub struct Dao {
    pool: deadpool_postgres::Pool,
}

impl Dao {
    pub async fn new() -> anyhow::Result<Self> {
        if let Err(e) = dotenv() {
            warn!("Failed to load .env file (may be intentional in production): {}", e);
        }
        // Create a pool from the configuration
        let config = Config::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
        let pool = config
        .pg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .unwrap();

        Ok(Dao { pool })

    }

    pub async fn query(&self) -> anyhow::Result<String> {
        let client = &self.pool.get().await?;
        // Execute a simple query
        let rows = client
            .query("SELECT $1::TEXT", &[&"hello world"])
            .await?;

        // And then check that we got back the same string we sent over.
        let value: &str = rows[0].get(0);
        assert_eq!(value, "hello world");
        println!("finish, value: {}", value);
        Ok(value.to_string())
    }

}
