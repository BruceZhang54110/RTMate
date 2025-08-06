use axum::routing::post;
use axum::Router;
use axum::routing::get;
use axum::routing::MethodRouter;
use crate::service::auth_token;




async fn root() -> &'static str {
    "RTMate Auth Service test"
}


pub async fn startup() {
    // 初始化日志
    // 创建路由
    let app = init_router();
    // 创建API路由
    let app = create_api_router(app
        , MethodRouterExtension::new("/api/auth/token"
            , post(auth_token)));

    println!("RTMate Auth Service started");    // 启动服务
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

// 
fn init_router() -> Router {
    Router::new()
    .route("/", get(|| async { "RTMate Auth Service" }))
}

struct MethodRouterExtension<'a> {
    path: &'a str,
    method_router: MethodRouter,
}

impl<'a> MethodRouterExtension<'a> {
    fn new(path: &'a str, method_router: MethodRouter) -> Self {
        MethodRouterExtension { path, method_router }
    }  
}

fn create_api_router(router: Router, method_router_extension: MethodRouterExtension) ->Router {
    router.route(method_router_extension.path, method_router_extension.method_router)
}