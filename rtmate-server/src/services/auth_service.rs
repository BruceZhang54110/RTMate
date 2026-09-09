use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use chrono::{Duration, Local};
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, EncodingKey, Header};
use rtmate_common::dto::Claims;
use rtmate_common::models::NewRtClientConnection;
use crate::manager::ClientConnection;
use rtmate_common::response_common::RtResponse;
use sha2::Sha256;
use uuid::Uuid;
use crate::common::{AppError, BizError};
use crate::dto::{AppAuthResult, OutboundMessage, RtAppParam};
use crate::web_context::WebContext;
use crate::domain::repositories::client_connection_repository_trait::ClientConnectionRepositoryTrait;
use crate::manager::ConnectionManager;
use tokio::sync::mpsc::Sender;
use crate::dto::AuthResponse;
use crate::common::RtWsError;

type HmacSha256 = Hmac<Sha256>;

pub struct AuthService {
    client_connection_repository: Arc<dyn ClientConnectionRepositoryTrait>,
    connection_manager: Arc<ConnectionManager>
}

impl AuthService {
    pub fn new(client_connection_repository: Arc<dyn ClientConnectionRepositoryTrait>
        , connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            client_connection_repository,
            connection_manager
        }
    }


    /// 根据 connect_token 注册终端
    pub async fn register_client(&self, connect_token: &str, ws_sender: Sender<OutboundMessage>) 
        -> Result<AuthResponse, RtWsError> {
        let rt_connection = 
            self.client_connection_repository.get_rt_client_connection_by_token(connect_token).await
            .map_err(|e| RtWsError::system("数据库查询失败", e))?
            .ok_or_else(|| RtWsError::biz(crate::common::WsBizCode::InvalidConnectToken))?;
        // 注册终端
        let client_id = rt_connection.client_id.clone();
        let rt_app = rt_connection.rt_app.clone();
        let conn = ClientConnection {
                rt_app: rt_connection.rt_app,
                client_id: Arc::new(client_id.clone()),
                connect_token: None,
                sender: ws_sender          
            };
            self.connection_manager.add_connection(conn);
            let app_connections_count = self.connection_manager.get_app_connections_count(Arc::new(rt_app));
            tracing::info!("app:{}, client 连接数:{}", rt_connection.app_id, app_connections_count);
        Ok(AuthResponse::new(true, client_id))
    }
}

/// 使用 app_id 和 app_key 签发 access token 与 connect token。
#[axum::debug_handler]
pub async fn auth_token(
    State(web_context): State<Arc<WebContext>>,
    Json(rt_app_param): Json<RtAppParam>,
) -> Result<Json<RtResponse<AppAuthResult>>, AppError> {
    let app_id = &rt_app_param.app_id;
    let state = &rt_app_param.state;
    let timestamp = rt_app_param.timestamp;

    let rt_app = web_context
        .rt_app_repository
        .get_rt_app_by_app_id(app_id)
        .await?
        .ok_or(BizError::AppNotFound)?;
    let app_key_param = &rt_app.app_key;

    // 校验签名
    let data = format!("{}:{}:{}", app_id, state, timestamp);
    let mut mac = HmacSha256::new_from_slice(app_key_param.as_bytes())?;
    mac.update(data.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    tracing::debug!("Generated signature: {}, data: {}", signature, data);

    if signature != rt_app_param.signature {
        return Err(AppError::from(BizError::InvalidSignature));
    }

    let client_id = Uuid::new_v4().as_simple().to_string();
    let jwt_token = generate_jwt_token(&rt_app.app_id, app_key_param, &client_id)?;
    let connect_token = Uuid::new_v4().as_simple().to_string();

    let new_conn = NewRtClientConnection {
        app_id: rt_app.id,
        rt_app: rt_app.app_id.clone(),
        client_id: client_id.clone(),
        connect_token: connect_token.clone(),
        used: false,
        expire_time: Some(Local::now() + Duration::minutes(1)),
    };

    web_context
        .client_connection_repository
        .save_connect_token(new_conn)
        .await?;

    let result = AppAuthResult::new(
        rt_app.app_id,
        jwt_token,
        connect_token,
        client_id,
    );
    Ok(Json(RtResponse::ok_with_data(result)))
}

fn generate_jwt_token(app_id: &str, app_key: &str, client_id: &str) -> anyhow::Result<String> {
    let now = Local::now();
    let exp = now + Duration::hours(2);
    let jti = Uuid::new_v4().as_simple().to_string();

    let claims = Claims::new(
        app_id.to_string(),
        client_id.to_string(),
        jti,
        now.to_utc(),
        exp.to_utc(),
    );
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_key.as_ref()),
    )?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::Mac;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    #[test]
    fn test_signature_matches_expected() {
        let app_key = "secret-key";
        let data = "demo-app:state:1234567890";
        let mut mac = HmacSha256::new_from_slice(app_key.as_bytes()).unwrap();
        mac.update(data.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_jwt_token_generation() {
        let token = generate_jwt_token("demo-app", "secret-key", "client-123").unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_jwt_token_can_be_decoded_by_websocket_auth() {
        let app_id = "demo-app";
        let app_key = "secret-key";
        let client_id = "client-123";
        let token = generate_jwt_token(app_id, app_key, client_id).unwrap();

        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(app_key.as_ref()),
            &validation,
        )
        .unwrap();

        assert_eq!(token_data.claims.app_id, app_id);
        assert_eq!(token_data.claims.client_id, client_id);
        assert!(!token_data.claims.jti.is_empty());
    }
}
