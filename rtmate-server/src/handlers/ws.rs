use std::sync::Arc;
use axum::{extract::{Query, State, ws::{self, CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code}}, http::{HeaderMap, StatusCode, Version}, response::IntoResponse};
use axum::response::Response;
use tracing::debug;
use crate::web_context::WebContext;
use crate::dto::{QueryParam, WsData};
use rtmate_common::response_common::RtResponse;
use crate::handlers::auth;
use crate::req::{RequestParam, RequestEvent};
use crate::common::{RtWsError, WsBizCode};


enum ConnectKind {
    Connect(String),
    Reconnect(String)
}

pub async fn ws_handler(
    State(web_context): State<Arc<WebContext>>,
    ws: WebSocketUpgrade,
    _version: Version,
    _headers: HeaderMap,
    query_param: Query<QueryParam>,
) -> Response {
    debug!("4 accepted a WebSocket Connect Token using {:?}", query_param.connect_token);

    let connect_token = match query_param.connect_token.as_deref() {
        Some(t) => t.to_string(),
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if let Err(e) = auth::check_connect_token(web_context.clone(), &connect_token).await {
        let resp: RtResponse<WsData> = e.into();
        debug!("check_connect_token is fail: {:?}", resp);
        return if resp.code != 500 {
            StatusCode::FORBIDDEN.into_response()
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        };
    }

    ws.on_upgrade(move|mut ws| async move {
        debug!("WebSocket connection established");
        // 将 connection_token 更新为已使用
        if let Err(e) =  auth::mark_connect_token_used(web_context.clone(), &connect_token).await {
            // 如果报错就关闭websocket
            tracing::error!("mark_connect_token_used error:{}", e);
            let close_msg = Message::Close(Some(CloseFrame {
                code: close_code::ERROR,
                reason: e.message().into(),
            }));
            let _ = ws.send(close_msg).await;
            return ;
        }
        tracing::debug!("连接成功, connect_token:{}", connect_token);
        // 连接成功，下发一个 reconnect_token，用于后续的重连
        process_websocket(ws, web_context).await;
    })
}

async fn process_websocket(mut ws: WebSocket, web_context: Arc<WebContext>) {
    loop {
        match ws.recv().await {
            Some(Ok(ws::Message::Text(s))) => {
                let websocket_msg = s.to_string();
                let resp: RtResponse<WsData> = handle_msg(web_context.clone(), &websocket_msg)
                    .await
                    .unwrap_or_else(|e| e.into());

                match serde_json::to_string(&resp) {
                    Ok(text) => {
                        debug!("Sending ws response: {}", text);
                        if let Err(e) = ws.send(ws::Message::Text(text.into())).await {
                            debug!("failed to send ws response: {e}");
                        }
                    }
                    Err(e) => tracing::error!("serialize ws response failed: {}", e),
                }
            }
            Some(Ok(ws::Message::Ping(ping_byte))) => {
                if let Err(e) = ws.send(ws::Message::Pong(ping_byte)).await {
                    debug!("failed to send Pong message from server: {e}");
                    break;
                }
            }
            Some(Ok(ws::Message::Close(_))) => {
                debug!("Received close message from client. Connection will be closed.");
                break;
            }
            Some(Ok(_)) => {}
            Some(Err(e)) => {
                debug!("client disconnected abruptly: {e}");
                break;
            }
            None => break,
        }
    }
}

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
            let data = auth::handle_auth_app(web_context, payload).await?;
            Ok(WsData::Auth(data))
        }
        // TODO: 未来新增事件，在此直接返回 WsData
        _ => Err(RtWsError::biz(WsBizCode::UnsupportedEvent)),
    }
}