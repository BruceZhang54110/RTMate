use std::sync::Arc;
use axum::{Router, routing::{any, post}};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use crate::web_context::WebContext;
use crate::handlers::{ws_handler, handle_404, create_channel, publish};

pub fn build_router(web_context: Arc<WebContext>) -> Router {
    Router::new()
        .fallback(handle_404)
        .route("/ws", any(ws_handler))
        .route("/api/channels", post(create_channel))
        .route("/api/channels/{name}/publish", post(publish))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(web_context)
}
