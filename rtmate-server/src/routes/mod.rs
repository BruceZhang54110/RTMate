use std::sync::Arc;
use axum::{Router, routing::{any, post}, extract::{Path, State}, Json};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use crate::common::AppError;
use crate::dto::PublishResult;
use crate::web_context::WebContext;
use crate::handlers::{ws_handler, handle_404, create_channel};
use rtmate_common::response_common::RtResponse;
use serde_json::Value;

/// 测试用：后端发布消息到频道（自动注册不存在的频道）
async fn test_publish(
    State(web_context): State<Arc<WebContext>>,
    Path(channel_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<RtResponse<PublishResult>>, AppError> {
    // 自动注册频道（方便测试，避免手动预置）
    web_context.connection_manager.register_channel(Arc::new(channel_id.clone()));

    let data = payload.get("data").cloned().unwrap_or(Value::Null);
    let result = crate::services::pubsub::PubSubService::publish(
        &web_context.connection_manager,
        &web_context.broadcast_manager,
        &channel_id,
        data,
    )
    .await
    .map_err(|e| {
        tracing::warn!(channel_id = %channel_id, error = %e, "Test publish failed");
        AppError::from(e)
    })?;

    tracing::info!(channel_id = %channel_id, delivered = result.delivered_count, "Test publish ok");
    Ok(Json(RtResponse::ok_with_data(PublishResult {
        channel_id: result.channel_id,
        delivered_count: result.delivered_count,
        failed_count: result.failed_count,
    })))
}

pub fn build_router(web_context: Arc<WebContext>) -> Router {
    Router::new()
        .fallback(handle_404)
        .route("/ws", any(ws_handler))
        .route("/api/channels", post(create_channel))
        .route("/api/channels/{channel_id}/publish", post(test_publish))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(web_context)
}
