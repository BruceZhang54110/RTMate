use axum::{
    extract::{Json, State},
    http::HeaderMap,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use rtmate_common::dto::Claims;
use rtmate_common::response_common::RtResponse;
use std::sync::Arc;

use crate::common::{AppError, BizError, ValidationErrorDetail};
use crate::dto::CreateChannelRequest;
use crate::services::channel_service::ChannelService;
use crate::web_context::WebContext;
use crate::dto::CreateChannelResponse;

/// 创建频道 HTTP Handler
pub async fn create_channel(
    State(web_context): State<Arc<WebContext>>,
    headers: HeaderMap,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Json<RtResponse<CreateChannelResponse>>, AppError> {
    // 1. 认证：解析并校验 JWT
    let claims = extract_and_validate_claims(&web_context, &headers).await?;

    // 2. 字段校验
    validate_request(&payload)?;

    // 3. 业务处理
    let (response, _is_new) = ChannelService::create_channel(
        web_context.rt_app_repository.clone(),
        web_context.channel_repository.clone(),
        &claims.app_id,
        &claims.client_id,
        payload,
    )
    .await?;

    Ok(Json(RtResponse::ok_with_data(response)))
}

/// 从 Authorization: Bearer <token> 中提取并校验 JWT
async fn extract_and_validate_claims(
    web_context: &WebContext,
    headers: &HeaderMap,
) -> Result<Claims, AppError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::from(BizError::Unauthorized))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
        .ok_or_else(|| AppError::from(BizError::Unauthorized))?;

    let app_id = parse_app_id_from_token(token).ok_or_else(|| AppError::from(BizError::Unauthorized))?;

    let rt_app = web_context
        .rt_app_repository
        .get_rt_app_by_app_id(&app_id)
        .await
        .map_err(|e| AppError::from(e))?
        .ok_or_else(|| AppError::from(BizError::AppNotFound))?;

    let token_data = decode_token(token, &rt_app.app_key)
        .map_err(|_e| AppError::from(BizError::Unauthorized))?;

    let claims = token_data.claims;
    if claims.app_id != app_id {
        return Err(AppError::from(BizError::Unauthorized));
    }

    let now = chrono::Local::now();
    if claims.exp < now {
        return Err(AppError::from(BizError::Unauthorized));
    }

    Ok(claims)
}

/// 不验证签名，仅从 JWT payload 中读取 app_id，用于先查数据库获取 app_key
fn parse_app_id_from_token(token: &str) -> Option<String> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(parts[1]))
        .ok()?;
    let claims: Claims = serde_json::from_slice(&payload).ok()?;
    Some(claims.app_id)
}

fn decode_token(token: &str, app_key: &str) -> anyhow::Result<TokenData<Claims>> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(app_key.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| anyhow::anyhow!("decode token failed: {}", e))
}

/// 请求字段校验
fn validate_request(request: &CreateChannelRequest) -> Result<(), AppError> {
    let mut errors = Vec::new();
    let name = request.name.trim();

    if name.is_empty() {
        errors.push(ValidationErrorDetail::new("name", "name is required"));
    } else if name.chars().count() > 64 {
        errors.push(ValidationErrorDetail::new(
            "name",
            "name must be 1-64 characters",
        ));
    }

    if let Some(desc) = &request.description {
        if desc.trim().chars().count() > 256 {
            errors.push(ValidationErrorDetail::new(
                "description",
                "description must be at most 256 characters",
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::with_data(
            400,
            "参数校验失败",
            errors,
        ))
    }
}
