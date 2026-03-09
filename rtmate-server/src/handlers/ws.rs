use std::sync::Arc;
use axum::{extract::{Query, State, ws::{self, CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code}}, http::{HeaderMap, StatusCode, Version}, response::IntoResponse};
use axum::response::Response;
use tracing::debug;
use crate::web_context::WebContext;
use crate::dto::{QueryParam, WsData, OutboundMessage};
use rtmate_common::response_common::RtResponse;
use crate::handlers::auth;
use crate::req::{RequestParam, RequestEvent};
use crate::common::{RtWsError, WsBizCode};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};

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

    let _rt_client_connection = match auth::check_connect_token(web_context.clone(), &connect_token).await {
        Ok(conn) => conn,
        Err(e) => {
            let resp: RtResponse<WsData> = e.into();
            debug!("check_connect_token is fail: {:?}", resp);
            return if resp.code != 500 {
                StatusCode::FORBIDDEN.into_response()
            } else {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            };
        }
    };

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
        process_websocket(ws, web_context).await;
    })
}

async fn process_websocket(ws: WebSocket, web_context: Arc<WebContext>) {
    // 分离发送和接收, 以便同时处理, sink 独占写权限, stream 独占读权限
    let (mut sink, mut stream) = ws.split();
    let (tx, mut rx) = mpsc::channel::<OutboundMessage>(100);
    let mut authed_client_id: Option<Arc<String>> = None;
    // 后端消息发送到客户端
    let send_task = tokio::spawn(async move {
        while let Some(out_msg) = rx.recv().await {
            let ws_msg = match out_msg {
                OutboundMessage::Response(resp) => {
                    match  serde_json::to_string(&resp) {
                        Ok(json) => Message::Text(json.into()),
                        Err(_) => continue,   
                    }
                }
                OutboundMessage::Raw(raw) => raw,
                
            };
            // 发送消息给客户端
            if (sink.send(ws_msg)).await.is_err() {
                break;
            }
        }
    });
    // 接收websocket 消息
    let close_reason = loop {
        match stream.next().await {
            Some(Ok(ws::Message::Text(s))) => {
                let websocket_msg = s.to_string();
                let resp: RtResponse<WsData> = handle_msg(web_context.clone(), &websocket_msg, tx.clone())
                    .await
                    .unwrap_or_else(|e| e.into());

                if authed_client_id.is_none() {
                    if let Some(data) = &resp.data {
                        if resp.code == 200 {
                            if let WsData::Auth(auth_data) = data {
                                if auth_data.state {
                                    authed_client_id = Some(Arc::new(auth_data.client_id.clone()));
                                }
                            }
                        }
                    }
                }
                // 通过通道发送websocket消息给客户端
                if tx.send(OutboundMessage::Response(resp)).await.is_err() {
                    break "send_channel_closed";
                }
            }
            Some(Ok(ws::Message::Ping(ping_byte))) => {
                if let Err(e) = tx.send(OutboundMessage::Raw(Message::Pong(ping_byte))).await {
                    debug!("failed to send Pong message from server: {e}");
                    break "pong_send_failed";
                }
            }
            Some(Ok(ws::Message::Close(_))) => {
                debug!("Received close message from client. Connection will be closed.");
                break "client_close";
            }
            Some(Ok(_)) => {}
            Some(Err(e)) => {
                debug!("client disconnected abruptly: {e}");
                break "stream_error";
            }
            None => {
                break "stream_ended";
            }
        }
    };

    // 关通道
    drop(tx);
    // 停掉任务
    send_task.abort();

    if let Some(client_id) = authed_client_id {
        web_context.connection_manager.remove_connection(&client_id);
        tracing::info!(
            client_id = %client_id,
            close_reason = close_reason,
            "websocket closed and cleaned up"
        );
    } else {
        tracing::info!(
            close_reason = close_reason,
            "websocket closed without authenticated client"
        );
    }
}

/// 处理websocket  客户端传入的消息
pub async fn handle_msg(web_context: Arc<WebContext>, websocket_msg: &str, ws_sender: Sender<OutboundMessage>) -> Result<RtResponse<WsData>, RtWsError> {
    // 1. 解析 JSON -> 业务错误
    let param: RequestParam = serde_json::from_str(websocket_msg)
        .map_err(|_| RtWsError::biz(WsBizCode::InvalidParams))?;
    // 2. 分发事件并得到领域结果
    let ws_data = process_event(web_context, param.event, ws_sender).await?;
    // 3. 成功统一包装
    Ok(RtResponse::ok_with_data(ws_data))
}

async fn process_event(web_context: Arc<WebContext>
    , event: RequestEvent
    , ws_sender:Sender<OutboundMessage>) -> Result<WsData, RtWsError> {
    match event {
        RequestEvent::Auth(payload) => {
            let data = auth::handle_auth_and_register(web_context, payload, ws_sender).await?;
            Ok(WsData::Auth(data))
        }
        // TODO: 未来新增事件，在此直接返回 WsData
        _ => Err(RtWsError::biz(WsBizCode::UnsupportedEvent)),
    }
}
