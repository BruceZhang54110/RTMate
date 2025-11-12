use std::sync::Arc;
use axum::{extract::{ws::{WebSocketUpgrade, WebSocket, self}, Query, State}, http::{HeaderMap, StatusCode, Version}, response::IntoResponse};
use axum::response::Response;
use tracing::debug;
use crate::web_context::WebContext;
use crate::dto::{QueryParam, WsData};
use rtmate_common::response_common::RtResponse;
use crate::handler; // existing business handler functions

pub async fn ws_handler(
    State(web_context): State<Arc<WebContext>>,
    ws: WebSocketUpgrade,
    version: Version,
    headers: HeaderMap,
    query_param: Query<QueryParam>,
) -> Response {
    debug!("1 accepted a WebSocket using {:?}", version);
    debug!("2 accepted a WebSocket Header using {:?}", headers);
    debug!("3 accepted a WebSocket Query using {:?}", query_param);
    debug!("4 accepted a WebSocket Connect Token using {:?}", query_param.connect_token);

    let connect_token = match query_param.connect_token.as_deref() {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if let Err(e) = handler::check_connect_token(web_context.clone(), connect_token).await {
        let resp: RtResponse<WsData> = e.into();
        debug!("check_connect_token is fail: {:?}", resp);
        return if resp.code != 500 {
            StatusCode::FORBIDDEN.into_response()
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        };
    }

    ws.on_upgrade(|ws| async move {
        debug!("WebSocket connection established");
        process_websocket(ws, web_context).await;
    })
}

async fn process_websocket(mut ws: WebSocket, web_context: Arc<WebContext>) {
    loop {
        match ws.recv().await {
            Some(Ok(ws::Message::Text(s))) => {
                let websocket_msg = s.to_string();
                let resp: RtResponse<WsData> = handler::handle_msg(web_context.clone(), &websocket_msg)
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
