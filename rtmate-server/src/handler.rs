
use anyhow::Context;
use anyhow::Ok;
use hmac::Mac;

use crate::store::Store;
use crate::req::AuthPayload;
use crate::req::MessageSendPayload;
use crate::req::RequestEvent;
use crate::req::RequestParam;
use crate::req::SubscribePayload;
use hmac::Hmac;
use sha2::Sha256;

const APP_ID: &str = "app_id";
const TOKEN: &str = "token";

type HmacSha256 = Hmac<Sha256>;


/// 处理websocket  客户端传入的消息
pub fn handle_msg(websocket_msg: &str) -> anyhow::Result<()> {
    let request_param: RequestParam = serde_json::from_str(websocket_msg)
        .with_context(|| format!("解析 Websocket 消息失败:{}", websocket_msg))?;
    process_event(request_param.event, websocket_msg)
}

fn process_event(event: RequestEvent, websocket_msg: &str) -> anyhow::Result<()> {
    match event {
        RequestEvent::Subscribe(payload) => {
            handle_subscribe_msg(payload)
                .with_context(|| format!("处理订阅消息失败:{}", websocket_msg))
        }
        RequestEvent::MessageSend(payload) => {
            handle_send_msg(payload)
                .with_context(|| format!("处理发送消息失败:{}", websocket_msg))
        }
        RequestEvent::Auth(payload) => {
            handle_auth_app(payload, &Store::new())
                .with_context(|| format!("处理认证消息失败:{}", websocket_msg))
        }
    }
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
pub fn handle_auth_app(payload: AuthPayload, store: &Store) -> anyhow::Result<()> {
    // 从数据库中获取 appKey
    
    let app_key = store.get(&payload.app_id)
        .ok_or_else(|| anyhow::anyhow!("appId not found in store"))?;
    // 使用 HMAC-SHA256 进行签名验证
    let mut mac = HmacSha256::new_from_slice(app_key.as_bytes())
    .with_context(|| "fail to create HmacSha256")?;
    let auth_data = format!("{}:{}:{}", &payload.app_id, &payload.token, &payload.timestamp);
    mac.update(auth_data.as_bytes());
    let server_sign = mac.finalize().into_bytes();

    // 假设 signature 是十六进制字符串
    println!("client_sign_hex_decode: {:?}", hex::decode(&payload.signature));
    let client_sign = hex::decode(&payload.signature)
        .map_err(|_| anyhow::anyhow!("signature decode failed"))?;

    if server_sign.as_slice() == client_sign.as_slice() {
        tracing::debug!("auth signature success, server_sign: {}, client_sign: {}", hex::encode(server_sign), &payload.signature);
    } else {
        return Err(anyhow::anyhow!("auth failed, server_sign: {}, client_sign: {}", hex::encode(server_sign), &payload.signature));
    }
    
    Ok(())
}