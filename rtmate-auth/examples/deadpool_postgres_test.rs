use anyhow::Ok;
use config::ConfigError;
use serde::{Deserialize};
use tokio_postgres::NoTls;
use deadpool_postgres::Runtime;
use dotenvy::dotenv;
use tracing::warn;




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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = dotenv() {
        warn!("Failed to load .env file (may be intentional in production): {}", e);
    }
    let config = Config::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    // Create a pool from the configuration
    let pool = config
        .pg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .unwrap();
    // Get a connection from the pool
    let client = pool.get().await?;
    // Execute a simple query
    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    // And then check that we got back the same string we sent over.
    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");
    println!("finish, value: {}", value);
    Ok(())


}