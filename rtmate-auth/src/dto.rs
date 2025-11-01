use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RtAppParam {

    // 租户的app_id
    pub app_id: String,
    // 请求状态随机字符串，防止重放攻击
    pub state: String,
    // 请求时间戳，单位秒
    pub timestamp: u64,
    // 请求签名
    pub signature: String,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppAuthResult {
    pub app_id: String,
    pub access_token: String,
    pub connect_token: String,
}

impl AppAuthResult {
    pub fn new(app_id: String, access_token: String, connect_token: String) -> Self {
        AppAuthResult { app_id, access_token, connect_token }
    }
}
