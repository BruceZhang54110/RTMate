
use anyhow::Ok;

use crate::db;

async fn query() -> anyhow::Result<String> {
    let (client, collection) 
        = db::init_db_client().await?;
    Ok("value".to_string())

}