use std::sync::Arc;
use axum::{Router, routing::{any, post}, extract::{Path, State}, Json, http::StatusCode};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use crate::web_context::WebContext;
use crate::handlers::{ws_handler, handle_404};
use serde_json::Value;

/// 测试用：后端发布消息到频道（自动注册不存在的频道）
async fn test_publish(
    State(web_context): State<Arc<WebContext>>,
    Path(channel_id): Path<String>,
    Json(payload): Json<Value>,
) -> StatusCode {
    // 自动注册频道（方便测试，避免手动预置）
    web_context.connection_manager.register_channel(Arc::new(channel_id.clone()));
    
    let data = payload.get("data").cloned().unwrap_or(Value::Null);
    match crate::services::pubsub::PubSubService::publish(
        &web_context.connection_manager,
        &channel_id,
        data,
    ).await {
        Ok(result) => {
            tracing::info!(channel_id = %channel_id, delivered = result.delivered_count, "Test publish ok");
            StatusCode::OK
        }
        Err(e) => {
            tracing::warn!(channel_id = %channel_id, error = %e, "Test publish failed");
            StatusCode::BAD_REQUEST
        }
    }
}

pub fn build_router(web_context: Arc<WebContext>) -> Router {
    Router::new()
        .fallback(handle_404)
        .route("/ws", any(ws_handler))
        .route("/api/channels/{channel_id}/publish", post(test_publish))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(web_context)
}
