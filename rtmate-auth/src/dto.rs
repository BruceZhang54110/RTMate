use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


#[derive(Deserialize, Serialize, Debug)]
pub struct RtAppParam {

    pub app_id: String,
    pub state: String,
    pub signature: String,
    pub timestamp: u64,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppAuthResult {
    app_id: String,
    token: String,
}

impl AppAuthResult {
    pub fn new(app_id: String, token: String) -> Self {
        AppAuthResult {
            app_id,
            token,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {

    // 租户ID
    app_id: String,
    // 客户端ID
    client_id: String,
    // 签发时间
    #[serde(with = "chrono::serde::ts_seconds")]
    iat: DateTime<Utc>,
    // 过期时间
    #[serde(with = "chrono::serde::ts_seconds")]
    exp: DateTime<Utc>,

}

impl Claims {
    pub fn new(app_id: String, client_id: String, iat: DateTime<Utc>, exp: DateTime<Utc>) -> Self {
        Claims {
            app_id,
            client_id,
            iat,
            exp,
        }
    }
    
}
