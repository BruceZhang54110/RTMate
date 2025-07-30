use axum::Json;
use serde_json::{json, Value};

/// 使用app_id 和 app_key生成token
pub async fn auth_token() -> Json<Value> {
    let token = "generated".to_string(); // 这里应该是生成token的逻辑
    Json(json!({ "token": token }))
}