use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::errors::ErrorKind;
use std::fmt;
use std::error::Error;

// 封装自己的AppError
pub struct RtWsError {
    pub code: i32,
    pub message: String,
    // 用于调试内部错误
    pub source: Option<anyhow::Error>,
}

impl From<anyhow::Error> for RtWsError {
    fn from(value: anyhow::Error) -> Self {
        tracing::error!("Internal error: {:?}", value);
        RtWsError {
            code: 500,
            message: "系统内部错误".to_string(),
            source: Some(value),
        }
    }
}

#[derive(Debug)]
pub enum BizError {
    AppNotFound,
    JwtError(JwtError)
}

impl From<JwtError> for BizError {
    fn from(value: JwtError) -> Self {
        BizError::JwtError(value)
    }
}

impl fmt::Display for BizError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BizError::AppNotFound => write!(f, "应用不存在（app_id未找到）"),
            BizError::JwtError(err) => write!(f, "JWT 错误: {}", err),
        }
    }
}

impl Error for BizError {}


impl From<BizError> for RtWsError {
    fn from(value: BizError) -> Self {
        match value {
            BizError::JwtError(err) => {
                // 根据 JwtError 的类型进行更细致的错误码映射
                let (code, message) = match err.kind() {
                    ErrorKind::ExpiredSignature => (401, "Token 已过期"),
                    ErrorKind::InvalidSignature => (1005, "签名验证失败，请检查您的请求是否合法"),
                    ErrorKind::InvalidToken => (401, "无效的token"),
                    _ => (500, "Token 解码失败"),
                };
                RtWsError {
                    code,
                    message: message.to_string(),
                    source: Some(anyhow::Error::new(err)),
                }
            }
            BizError::AppNotFound => RtWsError {
                code: 400,
                message: "app_id 未找到".to_string(),
                source: None,
            },
        }
    }
}

