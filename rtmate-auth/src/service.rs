use axum::{extract::State, Json};
use crate::{common::{AppError, RtResponse}, models::RtApp, web_context::{WebContext}};
use std::sync::Arc;

/// 使用app_id 和 app_key生成token
#[axum::debug_handler]
pub async fn auth_token(State(web_context): State<Arc<WebContext>>, Json(rt_app_param): Json<RtApp>) -> Result<Json<RtResponse<RtApp>>, AppError> {
    let token = "generated".to_string(); // 这里应该是生成token的逻辑
    // https://github.com/tokio-rs/axum/blob/main/examples/diesel-async-postgres/src/main.rs
    
    // 1. 使用app_id 和 app_key查询数据库
    let rt_app = web_context.dao.get_rt_app_by_app_id(&rt_app_param.app_id).await?;
    
    // 2. 生成 token

    todo!()
}

