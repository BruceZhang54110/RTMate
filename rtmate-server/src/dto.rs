use serde::Serialize;
use serde::Deserialize;
use axum::extract::ws::Message;
use rtmate_common::response_common::RtResponse;

pub enum OutboundMessage {
    /// 业务响应，需要序列化为JSON
    Response(RtResponse<WsData>),
    /// 原生websocket消息
    Raw(Message),
}


#[derive(Serialize, Debug)]
pub struct AuthResponse {
    pub state: bool,
    pub client_id: String,
}

impl AuthResponse {
    pub fn new(state: bool, client_id: String) -> Self {
        AuthResponse { state, client_id }
    }
    
}


#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum WsData {

    Auth(AuthResponse),
    Connect(AuthResponse),
}

#[derive(Debug, Deserialize)]
pub struct QueryParam {
    // 连接 token
    #[serde(default, deserialize_with = "empty_to_none")]
    pub connect_token: Option<String>,
}

fn empty_to_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.and_then(|s| {
        let t = s.trim().to_string();
        if t.is_empty() { 
            None 
        } else { 
            Some(t) 
        }
    }))
}