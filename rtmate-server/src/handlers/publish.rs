use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
};
use rtmate_common::response_common::RtResponse;
use serde_json::Value;

use crate::common::{AppError, BizError};
use crate::dto::PublishResult;
use crate::handlers::channel::extract_and_validate_claims;
use crate::web_context::WebContext;

/// 服务端发布消息到频道
///
/// 已认证的应用只能向属于本应用的已存在频道发布消息。
/// 发布前通过数据库校验频道存在性与归属，然后将频道同步到运行时注册表，
/// 最后调用 PubSubService 进行广播。
pub async fn publish(
    State(web_context): State<Arc<WebContext>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<RtResponse<PublishResult>>, AppError> {
    // 1. JWT 认证，与 /api/channels 复用同一套逻辑
    let claims = extract_and_validate_claims(&web_context, &headers).await?;

    // 2. 校验频道存在且属于当前应用（数据库为权威来源）
    let _channel = web_context
        .channel_repository
        .find_by_app_id_str_and_name(&claims.app_id, &name)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::from(BizError::ChannelNotFound))?;

    // 3. 将已验证的频道同步到运行时注册表，确保广播层可识别
    web_context
        .connection_manager
        .register_channel(Arc::new(name.clone()));

    // 4. 提取业务数据
    let data = payload.get("data").cloned().unwrap_or(Value::Null);

    // 5. 执行广播
    let result = crate::services::pubsub::PubSubService::publish(
        &web_context.connection_manager,
        &web_context.broadcast_manager,
        &name,
        data,
    )
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        channel_id = %name,
        app_id = %claims.app_id,
        delivered = result.delivered_count,
        "Server publish ok"
    );

    Ok(Json(RtResponse::ok_with_data(PublishResult {
        channel_id: result.channel_id,
        delivered_count: result.delivered_count,
        failed_count: result.failed_count,
    })))
}
