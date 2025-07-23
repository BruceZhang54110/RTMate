use std::collections::HashMap;

use anyhow::Context;
use anyhow::Ok;

use crate::req::MessageSendPayload;
use crate::req::RequestEvent;
use crate::req::RequestParam;
use crate::req::SubscribePayload;

const APP_ID: &str = "app_id";
const TOKEN: &str = "token";

/// 处理websocket  客户端传入的消息
pub fn handle_msg(websocket_msg: &str) -> anyhow::Result<()> {
    let request_param: RequestParam = serde_json::from_str(websocket_msg)
        .with_context(|| format!("解析websocket 消息失败:{}", websocket_msg))?;
    match request_param.event {
        RequestEvent::Subscribe(payload) => handle_subscribe_msg(payload),
        RequestEvent::MessageSend(payload) => handle_send_msg(payload),
        RequestEvent::Auth() => {
            request_param.metadata.get(APP_ID);
        }
    }
    anyhow::Ok(())
}

fn handle_subscribe_msg(payload: SubscribePayload) {
    println!("{:?}", payload);
}

fn handle_send_msg(payload: MessageSendPayload) {
    println!("{:?}", payload);

}

/// 处理应用认证
fn handle_auth_app(metadata: &HashMap<String, serde_json::Value>) -> anyhow::Result<String> {
    let app_id = metadata.get(APP_ID)
        .and_then(|v| v.as_str())
        .context("应用ID(app_id)不能为空")?;
    let token = metadata.get(TOKEN)
        .and_then(|v| v.as_str())
        .context("应用令牌(token)不能为空")?;
    Ok("dd".to_string())
}