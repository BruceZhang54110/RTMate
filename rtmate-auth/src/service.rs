use axum::{extract::State, Json};
use rt_common::response_common::RtResponse;
use std::sync::Arc;
use crate::dto::{RtAppParam, AppAuthResult};
use crate::common::BizError;
use crate::common::AppError;
use crate::web_context::WebContext;
use jsonwebtoken::{encode, Header, EncodingKey};
use rt_common::dto::Claims;
use chrono::{Utc, Duration};
use uuid::Uuid;
use hmac::Hmac;
use sha2::Sha256;
use hmac::Mac;
use crate::dao_query::DaoQuery;


type HmacSha256 = Hmac<Sha256>;

/// 使用app_id 和 app_key生成token
#[axum::debug_handler]
pub async fn auth_token(State(web_context): State<Arc<WebContext>>, Json(rt_app_param): Json<RtAppParam>)
     -> Result<Json<RtResponse<AppAuthResult>>, AppError> {
    // https://github.com/tokio-rs/axum/blob/main/examples/diesel-async-postgres/src/main.rs
    
    // 1. 使用app_id 和 app_key查询数据库
    let app_id = &rt_app_param.app_id;
    let state = &rt_app_param.state;
    let timestamp = rt_app_param.timestamp;

    let rt_app = web_context.dao.get_rt_app_by_app_id(&rt_app_param.app_id)
        .await?
        .ok_or(BizError::AppNotFound)?;
    let app_key_param = &rt_app.app_key;

    // 校验签名判断是否合法请求
    let data = format!("{}:{}:{}", app_id, state, &timestamp);
    // 使用 HMAC-SHA256 生成签名
    let mut mac = HmacSha256::new_from_slice(app_key_param.as_bytes())?;
    mac.update(data.as_bytes());
    
    let signature = hex::encode(mac.finalize().into_bytes());
    tracing::debug!("Generated signature: {}, data is:{}", signature, data);

    if signature != rt_app_param.signature {
        // 签名不匹配，返回错误
        return Err(AppError::from(BizError::InvalidSignature));
    }

    // 2. 生成 jwt token
    let token = generate_jwt_token(&rt_app.app_id, &rt_app.app_key)?;
    // 3. 返回结果
    let result = AppAuthResult::new(rt_app.app_id, token);
    Ok(Json(RtResponse::ok_with_data(result)))
}

fn generate_jwt_token(app_id: &str, app_key_param: &str) -> anyhow::Result<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(2); // token 2小时后过期

    // 1. 生成 client_id
    let client_id = Uuid::new_v4().as_simple().to_string();
    // 2. 生成 jti
    let jti = Uuid::new_v4().as_simple().to_string();
    // 2. 生成 claims paypoad
    let claims = Claims::new(app_id.to_string(), client_id, jti, now, exp);
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(app_key_param.as_ref()))?;
    Ok(token)
}

