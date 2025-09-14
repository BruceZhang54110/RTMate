use serde::Serialize;


#[derive(Serialize, Debug)]
pub struct RtResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
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
