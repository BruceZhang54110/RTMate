use std::net::SocketAddr;
use std::sync::Arc;
use rtmate_server::{bootstrap, routes, web_context::WebContext};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    // 初始化日志
    bootstrap::init_tracing();

    // 构建应用上下文
    let web_context: Arc<WebContext> = bootstrap::build_web_context().await?;

    // 构建路由
    let app = routes::build_router(web_context);

    // 监听地址（后续可改为配置）
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    // 启动服务 (当前未启用 TLS，可后续接入)
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
