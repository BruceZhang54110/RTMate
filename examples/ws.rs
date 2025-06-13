use axum::{
    extract::{
        ws::{self, WebSocketUpgrade},
        State,
    }, http::{StatusCode, Version}, response::{Html, IntoResponse}, routing::any, Router
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::broadcast;
use tower_http::{services::ServeDir, trace::{DefaultMakeSpan, TraceLayer}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");


    // build our application with some routes
    let app = Router::new()
        .fallback(handle_404)
        //.fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", any(ws_handler))
        // logging so we can see what's going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(broadcast::channel::<String>(16).0);
    // run it with hyper

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());


    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    version: Version,
    State(sender): State<broadcast::Sender<String>>,
) -> axum::response::Response {
    tracing::debug!("accepted a WebSocket using {version:?}");
    let mut receiver = sender.subscribe();
    ws.on_upgrade(|mut ws| async move {
        loop {
            tokio::select! {
                // Since `ws` is a `Stream`, it is by nature cancel-safe.
                res = ws.recv() => {
                    match res {
                        Some(Ok(ws::Message::Text(s))) => {
                            tracing::debug!("accepted a WebSocket message {s:?}");
                            let _ = sender.send(s.to_string());
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => tracing::debug!("client disconnected abruptly: {e}"),
                        None => break,
                    }
                }
                // Tokio guarantees that `broadcast::Receiver::recv` is cancel-safe.
                res = receiver.recv() => {
                    match res {
                        Ok(msg) => if let Err(e) = ws.send(ws::Message::Text(msg.into())).await {
                            tracing::debug!("client disconnected abruptly: {e}");
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    })
}

// 404 处理函数
async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Html("<h1>404: Not Found</h1>"),
    )
}