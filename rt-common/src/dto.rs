use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {

    // 租户ID
    pub app_id: String,
    // 客户端ID
    pub client_id: String,
    // JWT ID, 用于防止重放攻击
    pub jti: String,
    // 签发时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    // 过期时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>,

}

impl Claims {
    pub fn new(app_id: String, client_id: String, jti: String, iat: DateTime<Utc>, exp: DateTime<Utc>) -> Self {
        Claims {
            app_id,
            client_id,
            jti,
            iat,
            exp,
        }
    }
    
}
