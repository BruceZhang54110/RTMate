use jsonwebtoken::TokenData;
use rtmate_common::models::RtClientConnection;
use rtmate_common::response_common::RtResponse;
use tokio::sync::mpsc::Sender;
use crate::common::{RtWsError, WsBizCode};
use crate::dao_query::DaoQuery;
use crate::dto::{AuthResponse, WsData, OutboundMessage};
use crate::manager::ClientConnection;
use crate::req::{AuthPayload};
use jsonwebtoken::{DecodingKey, Validation, Algorithm};
use rtmate_common::dto::Claims;

use std::sync::Arc;
use crate::web_context::WebContext;
use crate::handlers::auth;

pub struct AuthResult {
    // 租户下的client
    pub client_id: String,
    // 租户标识
    pub app_id: String,
}


/// 处理应用认证，验证 jwt token
pub async fn handle_auth_and_register(web_context: Arc<WebContext>
    , payload: AuthPayload
    , ws_sender: Sender<OutboundMessage>)
         -> Result<AuthResponse, RtWsError> {
    let auth_result = validate_client(web_context.clone(), payload).await?;
    tracing::info!("app_id: [{}], client_id: [{}]认证通过", &auth_result.app_id, &auth_result.client_id);
    // Ok(AuthResponse::new(true, client_id))
    let client_id = auth_result.client_id.clone();
    auth::register_connection(web_context.clone(), auth_result, ws_sender).await?;
    if let Some(conn) = web_context.connection_manager.get_connection(&client_id) {
        let msg = RtResponse::ok_with_data(WsData::Auth(AuthResponse::new(false, client_id.clone())));
        let _ = conn.sender.send(OutboundMessage::Response(msg)).await.is_ok();
    }
    Ok(AuthResponse::new(true, client_id))
}

/// 验证 jwt token
async fn validate_client(web_context: Arc<WebContext>, payload: AuthPayload)
         -> Result<AuthResult, RtWsError> {
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
    let now = chrono::Local::now();
    if claims.exp < now {
        return Err(RtWsError::biz(WsBizCode::ExpiredToken));
    }
    let client_id = claims.client_id;
    tracing::info!("app_id: [{}], client_id: [{}]认证通过", claims.app_id, client_id);
    // Ok(AuthResponse::new(true, client_id))
    let auth_result: AuthResult = AuthResult {
        client_id,
        app_id
    };
    Ok(auth_result)

}


/// jwt token 解码
fn decode_token(token: &str, app_key: &str) -> Result<TokenData<Claims>, RtWsError> {
    let token_data = jsonwebtoken::decode::<Claims>(&token
        , &DecodingKey::from_secret(app_key.as_ref())
        , &Validation::new(Algorithm::HS256))?;
    Ok(token_data)
}

/// 校验 connect_token 的合法性
pub async fn check_connect_token(web_context: Arc<WebContext>, connect_token: &str) -> Result<RtClientConnection, RtWsError> {
    tracing::info!("校验 connect_token: {}", connect_token);
    // 从数据库中查询 connect_token 是否存在且未被使用
    let rt_client_connection = web_context.dao.get_rt_client_connection_by_token(connect_token)
        .await
        .map_err(|e| RtWsError::system("数据库查询失败", e))?
        .ok_or_else(|| RtWsError::biz(WsBizCode::InvalidConnectToken))?;
    // 判断是否过期
    let now = chrono::Local::now();
    tracing::debug!("connection_token 过期时间: {:?}, 当前时间: {:?}", rt_client_connection.expire_time, now);
    if let Some(expire_time) = rt_client_connection.expire_time {
        if expire_time < now {
            return Err(RtWsError::biz(WsBizCode::ExpiredConnectToken));
        }
    } else {
        return Err(RtWsError::biz(WsBizCode::InvalidConnectToken));
    }
    
    Ok(rt_client_connection)
}

pub async fn mark_connect_token_used(web_context: Arc<WebContext>, connect_token: &str) -> Result<(), RtWsError> {
    tracing::info!("标记 connect_token 为已使用: {}", connect_token);
    web_context.dao.mark_connection_token_used(connect_token)
        .await
        .map_err(|e| RtWsError::system("数据库更新失败", e))?;
    Ok(())
}

/// auth 通过后注册websocket 连接到连接池中
pub async fn register_connection(web_context: Arc<WebContext>
        , auth_result: AuthResult
        , ws_sender: Sender<OutboundMessage>) -> Result<(), RtWsError> {
            let connection_manager = web_context.connection_manager.clone();
            let client_id= Arc::new(auth_result.client_id);
            if connection_manager.get_connection(&client_id).is_some() {
                connection_manager.remove_connection(&client_id);
            }
            let conn = ClientConnection {
                client_id: client_id,
                rt_app: auth_result.app_id.clone(),
                connect_token: None,
                sender: ws_sender
            };
            connection_manager.add_connection(conn);
            let app_connections_count = connection_manager.get_app_connections_count(Arc::new(auth_result.app_id.clone()));
            tracing::info!("app:{}, client 连接数:{}", auth_result.app_id, app_connections_count);
            Ok(())

}