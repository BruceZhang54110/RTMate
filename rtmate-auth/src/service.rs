use axum::{extract::State, Json};
use rtmate_common::response_common::RtResponse;
use std::sync::Arc;
use crate::dto::{RtAppParam, AppAuthResult};
use crate::common::BizError;
use crate::common::AppError;
use crate::web_context::WebContext;
use jsonwebtoken::{encode, Header, EncodingKey};
use rtmate_common::dto::Claims;
use chrono::Duration;
use chrono::Local;
use uuid::Uuid;
use hmac::Hmac;
use sha2::Sha256;
use hmac::Mac;


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

    let rt_app = web_context
        .rt_app_repository
        .get_rt_app_by_app_id(&rt_app_param.app_id)
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

    // 2. 生成统一的 client_id
    let client_id = Uuid::new_v4().as_simple().to_string();
    // 3. 生成 jwt token
    let jwt_token = generate_jwt_token(&rt_app.app_id, &rt_app.app_key, &client_id)?;
    // 4. 生成 connect_token
    let connect_token = Uuid::new_v4().as_simple().to_string();
    // 5. 保存 connect_token 到数据库
    use rtmate_common::models::NewRtClientConnection;
    let new_conn = NewRtClientConnection {
        app_id: rt_app.id,
        // 克隆 app_id 避免后续仍需使用 rt_app.app_id 时发生所有权移动
        rt_app: rt_app.app_id.clone(),
        client_id: client_id.clone(),
        connect_token: connect_token.clone(),
        used: false,
        expire_time: Some(Local::now() + Duration::minutes(1)), // connect_token 2小时后过期
    };
    web_context
        .rt_app_repository
        .save_connect_token(new_conn)
        .await?;
    // 5. 返回结果
    // 这里构造响应时再克隆一次 app_id；如果后续不再使用 rt_app.app_id，可以直接移动
    let result = AppAuthResult::new(rt_app.app_id, jwt_token, connect_token, client_id);
    Ok(Json(RtResponse::ok_with_data(result)))
}

fn generate_jwt_token(app_id: &str, app_key_param: &str, client_id: &str) -> anyhow::Result<String> {
    let now = Local::now();
    let exp = now + Duration::hours(2); // token 2小时后过期

    // 1. 生成 jti
    let jti = Uuid::new_v4().as_simple().to_string();
    // 2. 生成 claims payload
    let claims = Claims::new(app_id.to_string(), client_id.to_string(), jti, now.to_utc(), exp.to_utc());
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(app_key_param.as_ref()))?;
    Ok(token)
}

