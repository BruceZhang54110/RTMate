use anyhow::Context;

use crate::req::MessageSendPayload;
use crate::req::RequestEvent;
use crate::req::RequestParam;
use crate::req::SubscribePayload;

/// 处理websocket  客户端传入的消息
pub fn handle_msg(websocket_msg: &str) -> anyhow::Result<()> {
    let request_param: RequestParam = serde_json::from_str(websocket_msg)
        .with_context(|| format!("解析websocket 消息失败:{}", websocket_msg))?;
    match request_param.event {
        RequestEvent::Subscribe(payload) => handle_subscribe_msg(payload),
        RequestEvent::MessageSend(payload) => handle_send_msg(payload),
    }
    anyhow::Ok(())
}

fn handle_subscribe_msg(payload: SubscribePayload) {
    println!("{:?}", payload);
}

fn handle_send_msg(payload: MessageSendPayload) {
    println!("{:?}", payload);

}