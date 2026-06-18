use serde::Serialize;
use serde::Deserialize;
use axum::extract::ws::Message;
use rtmate_common::response_common::RtResponse;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub enum OutboundMessage {
    /// 业务响应，需要序列化为JSON
    Response(RtResponse<WsData>),
    /// 原生websocket消息
    Raw(Message),
}

/// 内部广播 channel 中传输的消息单元。
/// 包装了最终要发送给客户端的 OutboundMessage，并附加广播所需的元数据。
#[derive(Clone)]
pub struct BroadcastMessage {
    pub channel_id: String,
    pub payload: OutboundMessage,
    pub published_at: DateTime<Utc>,
    pub sequence: u64,
}


#[derive(Serialize, Debug, Clone)]
pub struct AuthResponse {
    pub state: bool,
    pub client_id: String,
}

impl AuthResponse {
    pub fn new(state: bool, client_id: String) -> Self {
        AuthResponse { state, client_id }
    }
    
}


#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum WsData {
    Auth(AuthResponse),
    Connect(AuthResponse),
    Message(serde_json::Value),
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