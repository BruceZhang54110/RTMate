use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;

/// 统一错误类型，用于 HTTP Handler 的错误传播。
/// 最终通过 `IntoResponse` 转换为统一响应信封 `RtResponse<Value>`。
#[derive(Debug)]
pub struct AppError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
    // 用于调试内部错误
    pub source: Option<anyhow::Error>,
}

impl AppError {
    /// 构造一个带结构化数据的业务错误响应（如字段级校验错误）。
    pub fn with_data(code: i32, message: impl Into<String>, data: impl Serialize) -> Self {
        AppError {
            code,
            message: message.into(),
            data: serde_json::to_value(data).ok(),
            source: None,
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        let source = value.into();
        tracing::error!("Internal error: {:?}", source);
        AppError {
            code: 500, // 500 表示服务器异常
            message: "系统内部错误".to_string(),
            data: None,
            source: Some(source),
        }
    }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use crate::response_common::RtResponse;
        let response = RtResponse {
            code: self.code,
            message: self.message,
            data: self.data,
        };
        (StatusCode::OK, Json(response)).into_response()
    }
}

/// 业务错误枚举，覆盖项目通用的业务失败场景。
#[derive(Debug)]
pub enum BizError {
    /// 应用未找到
    AppNotFound,
    /// 参数错误
    InvalidParams,
    /// 非法签名
    InvalidSignature,
    /// 未授权
    Unauthorized,
    /// 频道不存在
    ChannelNotFound,
}

impl From<BizError> for AppError {
    fn from(value: BizError) -> Self {
        match value {
            BizError::AppNotFound => AppError {
                code: 1004,
                message: "您的app未找到，请检查appId".to_string(),
                data: None,
                source: None,
            },
            BizError::InvalidParams => AppError {
                code: 400,
                message: "参数错误".to_string(),
                data: None,
                source: None,
            },
            BizError::InvalidSignature => AppError {
                code: 1005,
                message: "签名验证失败，请检查您的请求是否合法".to_string(),
                data: None,
                source: None,
            },
            BizError::Unauthorized => AppError {
                code: 401,
                message: "未授权，请检查认证信息".to_string(),
                data: None,
                source: None,
            },
            BizError::ChannelNotFound => AppError {
                code: 1006,
                message: "频道不存在".to_string(),
                data: None,
                source: None,
            },
        }
    }
}

/// 字段级校验错误详情，用于结构化返回参数校验失败信息。
#[derive(Debug, Serialize)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
}

impl ValidationErrorDetail {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        ValidationErrorDetail {
            field: field.into(),
            message: message.into(),
        }
    }
}
