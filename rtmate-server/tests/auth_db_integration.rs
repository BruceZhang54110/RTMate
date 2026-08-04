//! 真实数据库访问的 auth 集成测试（004-integrate-auth-server）
//!
//! 本测试直接连接 PostgreSQL，要求：
//! - 环境变量 `DATABASE_URL` 指向已应用 migrations 的数据库；
//! - 数据库中已存在 `rt_app` 与 `rt_client_connection` 表；
//! - 本测试会插入临时测试数据并在测试结束时清理。
//!
//! 测试数据准备与清理全部通过 `rtmate-server` 的 repository 层完成，不直接执行 SQL。
//!
//! Run with: cargo test -p rtmate-server --test auth_db_integration -- --nocapture

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use hmac::Mac;
use sha2::Sha256;

use rtmate_server::dto::RtAppParam;
use rtmate_server::services::auth::auth_token;
use rtmate_server::web_context::WebContext;

fn generate_signature(app_id: &str, state: &str, timestamp: u64, app_key: &str) -> String {
    type HmacSha256 = hmac::Hmac<Sha256>;
    let data = format!("{}:{}:{}", app_id, state, timestamp);
    let mut mac = HmacSha256::new_from_slice(app_key.as_bytes()).unwrap();
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[tokio::test]
async fn test_auth_token_endpoint_with_real_database() {
    dotenvy::dotenv().ok();

    let ctx = Arc::new(WebContext::new().await.expect("failed to build WebContext"));

    let app_id = format!("test-app-{}", uuid::Uuid::new_v4().as_simple());
    let app_key = "test-app-key-123456789012345678901234567890";

    // 1. 通过 repository 层插入测试应用
    ctx.rt_app_repository
        .create_rt_app(&app_id, app_key)
        .await
        .expect("create test rt_app failed");

    // 2. 构造认证请求
    let state = "test-state";
    let timestamp = chrono::Utc::now().timestamp() as u64;
    let signature = generate_signature(&app_id, state, timestamp, app_key);

    let payload = RtAppParam {
        app_id: app_id.clone(),
        state: state.to_string(),
        timestamp,
        signature,
    };

    // 3. 调用 auth_token 服务
    let response = auth_token(State(ctx.clone()), Json(payload))
        .await
        .expect("auth_token should succeed")
        .0;

    assert_eq!(response.code, 200);
    let data = response.data.expect("response data should not be None");
    assert_eq!(data.app_id, app_id);
    assert!(!data.access_token.is_empty());
    assert!(!data.connect_token.is_empty());
    assert!(!data.client_id.is_empty());

    // 4. 通过 repository 层验证数据库中存在 connect_token 记录
    let connect_token = data.connect_token.clone();
    let stored_conn = ctx
        .client_connection_repository
        .get_rt_client_connection_by_token(&connect_token)
        .await
        .expect("query rt_client_connection failed")
        .expect("connect_token should be persisted");
    assert_eq!(stored_conn.rt_app, app_id);
    assert_eq!(stored_conn.client_id, data.client_id);
    assert!(!stored_conn.used);

    // 5. 通过 repository 层清理测试数据
    ctx.client_connection_repository
        .delete_rt_client_connection_by_connect_token(&connect_token)
        .await
        .expect("delete rt_client_connection failed");
    ctx.rt_app_repository
        .delete_rt_app_by_app_id(&app_id)
        .await
        .expect("delete rt_app failed");
}

#[tokio::test]
async fn test_auth_token_invalid_signature_with_real_database() {
    dotenvy::dotenv().ok();

    let ctx = Arc::new(WebContext::new().await.expect("failed to build WebContext"));

    let app_id = format!("test-app-{}", uuid::Uuid::new_v4().as_simple());
    let app_key = "test-app-key-123456789012345678901234567890";

    ctx.rt_app_repository
        .create_rt_app(&app_id, app_key)
        .await
        .expect("create test rt_app failed");

    let payload = RtAppParam {
        app_id: app_id.clone(),
        state: "test-state".to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
        signature: "invalid-signature".to_string(),
    };

    let result = auth_token(State(ctx.clone()), Json(payload)).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 1005);

    // 清理
    ctx.rt_app_repository
        .delete_rt_app_by_app_id(&app_id)
        .await
        .expect("delete rt_app failed");
}
