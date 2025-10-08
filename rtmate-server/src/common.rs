use rtmate_common::response_common::RtResponse;
use jsonwebtoken::errors::ErrorKind::*;

#[derive(Debug, Clone, Copy)]
pub enum WsBizCode {
    // 应用未找到
    AppNotFound,
    // 参数错误
    InvalidParams,
    // 无效token
    InvalidToken,
    // 过期token
    ExpiredToken,
    // 签名无效
    SignatureInvalid,
    // 认证失败（app_id不匹配）
    AuthMismatch,
    // 不支持的事件类型
    UnsupportedEvent,
}

impl WsBizCode {
    pub fn code(self) -> i32 {
        match self {
            WsBizCode::InvalidParams => 400,
            WsBizCode::AppNotFound => 400,
            WsBizCode::InvalidToken => 401,
            WsBizCode::ExpiredToken => 401,
            WsBizCode::SignatureInvalid => 1005,
            WsBizCode::AuthMismatch => 401,
            WsBizCode::UnsupportedEvent => 400,
        }
    }
    pub fn message(self) -> &'static str {
        match self {
            WsBizCode::InvalidParams => "参数错误",
            WsBizCode::AppNotFound => "app_id 未找到",
            WsBizCode::InvalidToken => "无效的 token",
            WsBizCode::ExpiredToken => "token 已过期",
            WsBizCode::SignatureInvalid => "签名验证失败",
            WsBizCode::AuthMismatch => "认证失败（app_id 不匹配）",
            WsBizCode::UnsupportedEvent => "不支持的事件类型",
        }
    }
}

#[derive(Debug)]
pub enum RtWsError {
    Business {
        code: i32,
        message: String,
        biz: WsBizCode,
    },
    System {
        code: i32,
        message: String,
        source: anyhow::Error,
    }
}

impl RtWsError {
    pub fn biz(b: WsBizCode) -> Self {
        RtWsError::Business {
            code: b.code(),
            message: b.message().to_string(),
            biz: b,
        }
    }
    pub fn system(msg: &str, err: impl Into<anyhow::Error>) -> Self {
        let src = err.into();
        tracing::error!("internal_error msg={msg} error={:?}", src);
        RtWsError::System {
            code: 500,
            // 保留调用方传入的简短说明，方便日志检索
            message: msg.to_string(),
            source: src,
        }

    }

    pub fn code(&self) -> i32 {
        match self {
            RtWsError::Business { code, .. } => *code,
            RtWsError::System { code, .. } => *code,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            RtWsError::Business { message, .. } => message,
            RtWsError::System { message, .. } => message,
        }
    }

}

impl std::fmt::Display for RtWsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RtWsError::Business { code, message, .. } => write!(f, "BusinessError(code={}, message={})", code, message),
            RtWsError::System { code, message, .. } => write!(f, "SystemError(code={}, message={})", code, message),
        }
    }
}

impl std::error::Error for RtWsError {}

// JWT 专用映射
impl From<jsonwebtoken::errors::Error> for RtWsError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        
        match e.kind() {
            ExpiredSignature => RtWsError::biz(WsBizCode::ExpiredToken),
            InvalidSignature => RtWsError::biz(WsBizCode::SignatureInvalid),
            InvalidToken => RtWsError::biz(WsBizCode::InvalidToken),
            _ => RtWsError::system("JWT解析失败", e),
        }
    }
}

// 统一错误转换为响应结构，便于在调用端直接 .map_err(|e| RtResponse::from(e))
impl<T> From<RtWsError> for RtResponse<T> {
    fn from(e: RtWsError) -> Self {
        RtResponse::err(e.code(), e.message())
    }
}

