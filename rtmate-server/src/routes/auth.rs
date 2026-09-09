use std::sync::Arc;

use axum::{extract::State, Json};

use crate::common::AppError;
use crate::dto::{AppAuthResult, RtAppParam};
use crate::services::auth_service;
use crate::web_context::WebContext;

#[axum::debug_handler]
pub async fn auth_token_handler(
    State(web_context): State<Arc<WebContext>>,
    Json(payload): Json<RtAppParam>,
) -> Result<Json<rtmate_common::response_common::RtResponse<AppAuthResult>>, AppError> {
    auth_service::auth_token(State(web_context), Json(payload)).await
}
