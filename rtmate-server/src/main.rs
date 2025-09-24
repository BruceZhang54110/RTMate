use axum::{
    http::HeaderMap,
    extract::{
        ws::{self, WebSocketUpgrade},
        }, http::{StatusCode, Version}, response::{Html, IntoResponse}, routing::any, Router
};
use std::{env, net::SocketAddr, path::PathBuf};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum_server::tls_rustls::RustlsConfig;
use rtmate_server::web_context::WebContext;
use rtmate_server::handler;
use std::sync::Arc;
use axum::extract::State;
use rt_common::response_common::RtResponse;
use rtmate_server::dto::WsData;

/// Websocket service main startup
#[tokio::main]
async fn main() {

    // 监控日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // configure certificate and private key used by https
    let cert_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("self_signed_certs")
        .join("cert.pem");

    let key = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("self_signed_certs")
        .join("key.pem");
    // 加载配置
    let _config = RustlsConfig::from_pem_file(cert_path, key).await.unwrap();
    let web_context: Arc<WebContext> = WebContext::new().await.unwrap().into();
    // 设置路由，也就是路径地址
    let app = Router::new()
        .fallback(handle_404)
        .route("/ws", any(ws_handler))
        // logging so we can see what's going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    let app = app.with_state(web_context);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    // let mut server = axum_server::bind_rustls(addr, config);
    let server = axum_server::bind(addr);

    // IMPORTANT: This is required to advertise our support for HTTP/2 websockets to the client.
    // If you use axum::serve, it is enabled by default.
    // server.http_builder().http2().enable_connect_protocol();

    server.serve(app.into_make_service()).await.unwrap();


}



async fn ws_handler(
    State(web_context): State<Arc<WebContext>>,
    ws: WebSocketUpgrade,
    version: Version,
    headers: HeaderMap
) -> axum::response::Response {
    tracing::debug!("accepted a WebSocket using {version:?}");
    tracing::debug!("accepted a WebSocket Header using {headers:?}");
    // 升级为 WebSocket 连接
    ws.on_upgrade(|mut ws| async move {
        loop {
            tokio::select! {
                // Since `ws` is a `Stream`, it is by nature cancel-safe.
                res = ws.recv() => {
                    match res {
                        Some(Ok(ws::Message::Text(s))) => {
                            let websocket_msg = s.to_string();
                            // 统一：领域调用 -> (Result) -> 转成统一响应 -> 序列化发送
                            let resp: RtResponse<WsData> =
                                handler::handle_msg(web_context.clone(), &websocket_msg)
                                    .await
                                    .unwrap_or_else(|e| e.into());

                            match serde_json::to_string(&resp) {
                                Ok(text) => {
                                    tracing::debug!("Sending ws response: {}", text);
                                    if let Err(e) = ws.send(ws::Message::Text(text.into())).await {
                                        tracing::debug!("failed to send ws response: {e}");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("serialize ws response failed: {}", e);
                                }
                            }
                        }
                        Some(Ok(ws::Message::Ping(ping_byte))) => {
                            if let Err(e) = ws.send(ws::Message::Pong(ping_byte)).await {
                                tracing::debug!("failed to send Pong message from server: {e}");
                                break; // 发送失败，认为连接已断开
                            }
                        }
                        Some(Ok(ws::Message::Close(_))) => {
                            tracing::debug!("Received close message from client. Connection will be closed.");
                            break;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => tracing::debug!("client disconnected abruptly: {e}"),
                        None => break,
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
