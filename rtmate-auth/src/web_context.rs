use std::sync::Arc;

use axum::routing::post;
use axum::Router;
use axum::routing::get;
use axum::routing::MethodRouter;
use crate::dao::Dao;
use crate::service::auth_token;

#[derive(Clone)]
pub struct WebContext {

    // 数据源
    pub dao: Dao,
}

impl WebContext {
    pub async fn new() -> anyhow::Result<Self> {
        let dao = Dao::new().await?;
        Ok(WebContext { dao })
    }

}


async fn root() -> &'static str {
    "RTMate Auth Service test"
}

/// web服务初始化
pub async fn startup() {
    // 初始化日志
    let web_context = WebContext::new().await.unwrap().into();
    // 创建路由
    let app = init_router();
    // 创建API路由
    let app = create_api_router(app
        , MethodRouterExtension::new("/api/auth/token"
            , post(auth_token)));
    let app = app.with_state(web_context);
    println!("RTMate Auth Service started");    // 启动服务
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

// 
fn init_router() -> Router<Arc<WebContext>> {
    Router::new()
    .route("/", get(|| async { "RTMate Auth Service" }))
}

struct MethodRouterExtension<'a> {
    path: &'a str,
    method_router: MethodRouter<Arc<WebContext>>,
}

impl<'a> MethodRouterExtension<'a> {
    fn new(path: &'a str, method_router: MethodRouter<Arc<WebContext>>) -> Self {
        MethodRouterExtension { path, method_router }
    }  
}

fn create_api_router(router: Router<Arc<WebContext>>, method_router_extension: MethodRouterExtension) ->Router<Arc<WebContext>> {
    router.route(method_router_extension.path, method_router_extension.method_router)
}