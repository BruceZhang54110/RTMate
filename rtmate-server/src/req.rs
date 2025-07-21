use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // 将 Rust 的 snake_case 字段映射到 JSON 的 camelCase
pub struct SubscribePayload {
    // 频道id
    pub channel_id: String,
    // 主题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // 将 Rust 的 snake_case 字段映射到 JSON 的 camelCase
pub struct MessageSendPayload {
    // 频道id
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    // 主题
    pub topic: Option<String>,
    // 消息
    pub text: serde_json::Value,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RequestParam {

    // 事件
    #[serde(flatten)]
    pub event: RequestEvent,
    // 元数据
    #[serde(default = "HashMap::new")]
    #[serde(skip_serializing_if = "HashMap::is_empty")] 
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", rename_all = "camelCase")]
pub enum RequestEvent {
    Subscribe(SubscribePayload),
    MessageSend(MessageSendPayload),
}