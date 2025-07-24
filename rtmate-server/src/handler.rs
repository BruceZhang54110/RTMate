use std::collections::HashMap;

use anyhow::Context;
use anyhow::Ok;

use crate::req::AuthPayload;
use crate::req::MessageSendPayload;
use crate::req::RequestEvent;
use crate::req::RequestParam;
use crate::req::SubscribePayload;

const APP_ID: &str = "app_id";
const TOKEN: &str = "token";

/// 处理websocket  客户端传入的消息
pub fn handle_msg(websocket_msg: &str) -> anyhow::Result<()> {
    let request_param: RequestParam = serde_json::from_str(websocket_msg)
        .with_context(|| format!("解析 Websocket 消息失败:{}", websocket_msg))?;
    match request_param.event {
        RequestEvent::Subscribe(payload) => handle_subscribe_msg(payload),
        RequestEvent::MessageSend(payload) => handle_send_msg(payload),
        RequestEvent::Auth(payload) => {
            handle_auth_app(payload)
                .with_context(|| format!("处理认证消息失败:{}", websocket_msg))?;
        }
    }
    anyhow::Ok(())
}

fn handle_subscribe_msg(payload: SubscribePayload) -> anyhow::Result<()> {
    println!("{:?}", payload);
    Ok(())
}

fn handle_send_msg(payload: MessageSendPayload) -> anyhow::Result<()> {
    println!("{:?}", payload);
    Ok(())
}

/// 处理应用认证，使用 hmac_sha256 算法进行签名验证
fn handle_auth_app(authPayload: AuthPayload) -> anyhow::Result<()> {
    authPayload.app_id;
    authPayload.token;
    // 使用冒号分割token
    // 校验
    Ok(())
}