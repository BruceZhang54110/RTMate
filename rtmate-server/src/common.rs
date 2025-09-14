use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::errors::ErrorKind;


// 封装自己的AppError
pub struct RtWsError {
    pub code: i32,
    pub message: String,
    // 用于调试内部错误
    pub source: Option<anyhow::Error>,
}

impl <E> From<E> for RtWsError
where
    E: Into<anyhow::Error>
{
    fn from(value: E) -> Self {
        let source = value.into();
        tracing::error!("Internal error: {:?}", source);
        RtWsError {
            code: 500, // 500 表示服务器异常
            message: "系统内部错误".to_string(),
            source: Some(source),
        }
    }

}

pub enum BizError {
    JwtError(JwtError)
}

impl From<JwtError> for BizError {
    fn from(value: JwtError) -> Self {
        BizError::JwtError(value)
    }
}

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
        }
    }
}

