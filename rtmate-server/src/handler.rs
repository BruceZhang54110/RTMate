
use jsonwebtoken::TokenData;
use crate::common::{RtWsError, WsBizCode};
use crate::dao_query::DaoQuery;
use crate::dto::AuthResponse;
use crate::req::{AuthPayload, RequestEvent, RequestParam};
use jsonwebtoken::{DecodingKey, Validation, Algorithm};
use rt_common::dto::Claims;
use rt_common::response_common::RtResponse;
use crate::dto::WsData;
use std::sync::Arc;
use crate::web_context::WebContext;

/// 处理websocket  客户端传入的消息
pub async fn handle_msg(web_context: Arc<WebContext>, websocket_msg: &str) -> Result<RtResponse<WsData>, RtWsError> {
    // 1. 解析 JSON -> 业务错误
    let param: RequestParam = serde_json::from_str(websocket_msg)
        .map_err(|_| RtWsError::biz(WsBizCode::InvalidParams))?;
    // 2. 分发事件并得到领域结果
    let ws_data = process_event(web_context, param.event).await?;
    // 3. 成功统一包装
    Ok(RtResponse::ok_with_data(ws_data))
}

async fn process_event(web_context: Arc<WebContext>, event: RequestEvent) -> Result<WsData, RtWsError> {
    match event {
        RequestEvent::Auth(payload) => {
            let data = handle_auth_app(web_context, payload).await?;
            Ok(WsData::Auth(data))
        }
        // TODO: 未来新增事件，在此直接返回 WsData
        _ => Err(RtWsError::biz(WsBizCode::UnsupportedEvent)),
    }
}


/// 处理应用认证，验证 jwt token
pub async fn handle_auth_app(web_context: Arc<WebContext>, payload: AuthPayload)
         -> Result<AuthResponse, RtWsError> {
    // 从数据库中获取 appKey
    tracing::info!("收到认证请求: {:?}", payload);
    let token = payload.token;
    let app_id = payload.app_id;

    tracing::info!("认证请求 app_id: {}, token: {}", app_id, token);
    // 解码 token
    let rt_app = web_context.dao.get_rt_app_by_app_id(&app_id)
        .await
        .map_err(|e| RtWsError::system("数据库查询失败", e))?
        .ok_or_else(|| RtWsError::biz(WsBizCode::AppNotFound))?;
    let token_data= decode_token(&token, &rt_app.app_key)?;
    // 验证 app_id 是否匹配
    let claims = token_data.claims;
    if claims.app_id != app_id {
        return Err(RtWsError::biz(WsBizCode::AuthMismatch));
    }
    // 判断 token 是否过期
    let now = chrono::Utc::now();
    if claims.exp < now {
        return Err(RtWsError::biz(WsBizCode::ExpiredToken));
    }
    let client_id = claims.client_id;
    tracing::info!("app_id: [{}], client_id: [{}]认证通过", claims.app_id, client_id);
    Ok(AuthResponse::new(true, client_id))
}

/// jwt token 解码
fn decode_token(token: &str, app_key: &str) -> Result<TokenData<Claims>, RtWsError> {
    let token_data = jsonwebtoken::decode::<Claims>(&token
        , &DecodingKey::from_secret(app_key.as_ref())
        , &Validation::new(Algorithm::HS256))?;
    Ok(token_data)
}