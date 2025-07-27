use axum::Router;
use axum::routing::get;

pub async fn startup() {
    // 初始化日志
    // 创建路由
    let app = create_router();
    println!("RTMate Auth Service started");
    // 启动服务
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

// 
fn create_router() -> Router {
    Router::new()
    .route("/", get(|| async { "RTMate Auth Service" }))
}