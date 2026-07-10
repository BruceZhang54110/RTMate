use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use rtmate_common::dto::Claims;
use std::sync::Arc;

use crate::dto::{
    CreateChannelRequest, ErrorResponse, FieldError, ValidationErrorResponse,
};
use crate::services::channel_service::ChannelService;
use crate::web_context::WebContext;

/// 创建频道 HTTP Handler
pub async fn create_channel(
    State(web_context): State<Arc<WebContext>>,
    headers: HeaderMap,
    Json(payload): Json<CreateChannelRequest>,
) -> Response {
    // 1. 认证：解析并校验 JWT
    let claims = match extract_and_validate_claims(&web_context, &headers).await {
        Ok(claims) => claims,
        Err(response) => return response,
    };

    // 2. 字段校验
    if let Err(response) = validate_request(&payload) {
        return response;
    }

    // 3. 业务处理
    match ChannelService::create_channel(
        web_context.rt_app_repository.clone(),
        web_context.channel_repository.clone(),
        &claims.app_id,
        &claims.client_id,
        payload,
    )
    .await
    {
        Ok((response, true)) => (StatusCode::CREATED, Json(response)).into_response(),
        Ok((response, false)) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "创建频道失败");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "创建频道失败".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 从 Authorization: Bearer <token> 中提取并校验 JWT
async fn extract_and_validate_claims(
    web_context: &WebContext,
    headers: &HeaderMap,
) -> Result<Claims, Response> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing Authorization header".to_string(),
                }),
            )
                .into_response()
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid Authorization format".to_string(),
                }),
            )
                .into_response()
        })?;

    let app_id = parse_app_id_from_token(token).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid token".to_string(),
            }),
        )
            .into_response()
    })?;

    let rt_app = match web_context
        .rt_app_repository
        .get_rt_app_by_app_id(&app_id)
        .await
    {
        Ok(Some(app)) => app,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "App not found".to_string(),
                }),
            )
                .into_response())
        }
        Err(e) => {
            tracing::warn!(error = %e, "查询 rt_app 失败");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "认证服务不可用".to_string(),
                }),
            )
                .into_response());
        }
    };

    let token_data = decode_token(token, &rt_app.app_key).map_err(|e| {
        tracing::warn!(error = %e, "JWT 解码失败");
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or expired token".to_string(),
            }),
        )
            .into_response()
    })?;

    let claims = token_data.claims;
    if claims.app_id != app_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Token app_id mismatch".to_string(),
            }),
        )
            .into_response());
    }

    let now = chrono::Local::now();
    if claims.exp < now {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Token expired".to_string(),
            }),
        )
            .into_response());
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
fn validate_request(request: &CreateChannelRequest) -> Result<(), Response> {
    let mut errors = Vec::new();
    let name = request.name.trim();

    if name.is_empty() {
        errors.push(FieldError {
            field: "name".to_string(),
            message: "name is required".to_string(),
        });
    } else if name.chars().count() > 64 {
        errors.push(FieldError {
            field: "name".to_string(),
            message: "name must be 1-64 characters".to_string(),
        });
    }

    if let Some(desc) = &request.description {
        if desc.trim().chars().count() > 256 {
            errors.push(FieldError {
                field: "description".to_string(),
                message: "description must be at most 256 characters".to_string(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(ValidationErrorResponse {
                error: "Validation failed".to_string(),
                details: errors,
            }),
        )
            .into_response())
    }
}
