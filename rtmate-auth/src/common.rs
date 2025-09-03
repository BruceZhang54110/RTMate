use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Serialize};

// 封装自己的AppError
pub struct AppError {
    pub code: i32,
    pub message: String,
    // 用于调试内部错误
    pub source: Option<anyhow::Error>,
}

impl <E> From<E> for AppError
where
    E: Into<anyhow::Error>
{
    fn from(value: E) -> Self {
        let source = value.into();
        tracing::error!("Internal error: {:?}", source);
        AppError {
            code: 500, // 500 表示服务器异常
            message: "系统内部错误".to_string(),
            source: Some(source),
        }
    }

}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {

    fn into_response(self) -> axum::response::Response {
         // 使用 () 作为 T，表示没有数据
        let response = RtResponse::<()> {
            code: self.code,
            message: self.message,
            data: None,
        };
        (StatusCode::OK, Json(response)).into_response()
    }

}

#[derive(Serialize, Debug)]
pub struct RtResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> RtResponse<T> {

    /// 创建一个带数据的业务成功响应
    pub fn ok_with_data(data: T) -> Self {
        RtResponse {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    /// 创建一个无数据的业务成功响应
    pub fn ok() -> Self {
        RtResponse {
            code: 200,
            message: "success".to_string(),
            data: None,
        }
    }

    /// 创建一个业务失败响应
    pub fn err(code: i32, message: &str) -> Self {
        RtResponse {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}


pub enum BizError {
    // 应用未找到
    AppNotFound,
    // 参数错误
    InvalidParams,
    // 非法签名
    InvalidSignature,
}

impl From<BizError> for AppError {
    fn from(value: BizError) -> Self {
        match value {
            BizError::AppNotFound => AppError {
                code: 1004,
                message: "您的app未找到，请检查appId".to_string(),
                source: None,
            },
            BizError::InvalidParams => AppError {
                code: 400,
                message: "参数错误".to_string(),
                source: None,
            },
            BizError::InvalidSignature => AppError {
                code: 1005,
                message: "签名验证失败，请检查您的请求是否合法".to_string(),
                source: None,
            },
        }
    }
}

