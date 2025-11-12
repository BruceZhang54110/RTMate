use axum::{http::StatusCode, response::{Html, IntoResponse}};

pub async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Html("<h1>404: Not Found</h1>"),
    )
}
