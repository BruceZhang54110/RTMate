
use anyhow::anyhow;
use jsonwebtoken::TokenData;
use rt_common::dao::Dao;
use crate::dao_query::DaoQuery;
use crate::dto::AuthResponse;
use crate::req::AuthPayload;
use crate::req::MessageSendPayload;
use crate::req::RequestEvent;
use crate::req::RequestParam;
use crate::req::SubscribePayload;
use jsonwebtoken::{DecodingKey, Validation, Algorithm};
use rt_common::dto::Claims;
use rt_common::response_common::RtResponse;
use crate::dto::WsData;
use std::sync::Arc;
use crate::web_context::WebContext;
use crate::common::BizError;

/// 处理websocket  客户端传入的消息
pub async fn handle_msg(web_context: Arc<WebContext>
        , websocket_msg: &str) -> anyhow::Result<RtResponse<WsData>> {
    let request_param: Result<RequestParam, RtResponse<WsData>>  = serde_json::from_str(websocket_msg)
        .map_err(|e| {
            RtResponse {
                code: 400,
                message: format!("解析 Websocket 消息失败: {}", e),
                data: None,
            }
        });
    match request_param {
        Err(err) => Ok(err),
        Ok(param) => process_event(web_context, param.event).await,
    }
}

async fn process_event(web_context: Arc<WebContext>, event: RequestEvent) -> anyhow::Result<RtResponse<WsData>> {
    match event {
        // RequestEvent::Subscribe(payload) => {
        //     let a = handle_subscribe_msg(payload);
        // }
        // RequestEvent::MessageSend(payload) => {
        //     handle_send_msg(payload)
        //         .with_context(|| format!("处理发送消息失败:{}", websocket_msg))
        // }
        RequestEvent::Auth(payload) => {
            match handle_auth_app(web_context, payload).await {
                Ok(data) => Ok(RtResponse::ok_with_data(WsData::Auth(data))),
                Err(e) => {
                    tracing::error!("认证失败: {}", e);
                    Ok(RtResponse::err(500, "系统异常"))
                }, // 认证错误建议用401
            }
        },
        _ => Err(anyhow!("不支持的事件类型")),
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

/// 处理应用认证，验证 jwt token
pub async fn handle_auth_app(web_context: Arc<WebContext>, payload: AuthPayload) -> anyhow::Result<AuthResponse> {
    // 从数据库中获取 appKey
    tracing::info!("收到认证请求: {:?}", payload);
    let token = payload.token;
    let app_id = payload.app_id;

    tracing::info!("认证请求 app_id: {}, token: {}", app_id, token);
    // 解码 token
    let rt_app = web_context.dao.get_rt_app_by_app_id(&app_id)
        .await?
        .ok_or(BizError::AppNotFound)?;
    let token_data= decode_token(&token, &rt_app.app_key)?;
    // 验证 app_id 是否匹配
    let claims = token_data.claims;
    if claims.app_id != app_id {
        return Err(anyhow!("app_id 不匹配"));
    }
    // 判断 token 是否过期
    let now = chrono::Utc::now();
    if claims.exp < now {
        return Err(anyhow!("token 已过期"));
    }
    let client_id = claims.client_id;
    tracing::info!("app_id: [{}], client_id: [{}]认证通过", claims.app_id, client_id);
    Ok(AuthResponse::new(true, client_id))
}

/// jwt token 解码
fn decode_token(token: &str, app_key: &str) -> anyhow::Result<TokenData<Claims>> {
    let token_data = jsonwebtoken::decode::<Claims>(&token
        , &DecodingKey::from_secret(app_key.as_ref())
        , &Validation::new(Algorithm::HS256))?;
    Ok(token_data)
}