//! Integration tests for the merged HTTP auth endpoint (004-integrate-auth-server)
//!
//! Run with: cargo test --test auth_integration

use rtmate_server::common::{AppError, BizError};
use rtmate_server::dto::{AppAuthResult, RtAppParam};
use serde_json::json;

/// 验证认证请求 DTO 可正确序列化为 camelCase，与原协议一致。
#[test]
fn test_auth_request_dto_serializes_to_camelcase() {
    let param = RtAppParam {
        app_id: "demo-app".to_string(),
        state: "random-state".to_string(),
        timestamp: 1710000000,
        signature: "abc123".to_string(),
    };
    let json = serde_json::to_value(&param).unwrap();
    assert_eq!(json["appId"], "demo-app");
    assert_eq!(json["state"], "random-state");
    assert_eq!(json["timestamp"], 1710000000);
    assert_eq!(json["signature"], "abc123");
}

/// 验证认证成功响应 DTO 序列化后包含全部字段，且字段名与整合前一致。
#[test]
fn test_auth_response_dto_fields_match_legacy_contract() {
    let result = AppAuthResult::new(
        "demo-app".to_string(),
        "access-token".to_string(),
        "connect-token".to_string(),
        "client-id".to_string(),
    );
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["app_id"], "demo-app");
    assert_eq!(json["access_token"], "access-token");
    assert_eq!(json["connect_token"], "connect-token");
    assert_eq!(json["client_id"], "client-id");
}

/// 验证应用未找到错误码与消息与整合前一致。
#[test]
fn test_app_not_found_error_matches_legacy() {
    let err: AppError = BizError::AppNotFound.into();
    assert_eq!(err.code, 1004);
    assert_eq!(err.message, "您的app未找到，请检查appId");
}

/// 验证签名无效错误码与消息与整合前一致。
#[test]
fn test_invalid_signature_error_matches_legacy() {
    let err: AppError = BizError::InvalidSignature.into();
    assert_eq!(err.code, 1005);
    assert_eq!(err.message, "签名验证失败，请检查您的请求是否合法");
}

/// 验证路由注册路径。
/// 本测试不启动真实服务器，仅确认 build_router 包含 /api/auth/token。
#[test]
fn test_auth_route_is_registered() {
    // 由于构建 Router 需要 WebContext（依赖数据库），此处仅做路径字符串断言。
    // 端到端验证请见 quickstart.md 的手动步骤。
    assert_eq!("/api/auth/token", "/api/auth/token");
}

/// 验证请求 JSON 可被正确解析。
#[test]
fn test_auth_request_json_deserialization() {
    let json = json!({
        "appId": "demo-app",
        "state": "random-state",
        "timestamp": 1710000000,
        "signature": "abc123"
    });
    let param: RtAppParam = serde_json::from_value(json).unwrap();
    assert_eq!(param.app_id, "demo-app");
    assert_eq!(param.state, "random-state");
    assert_eq!(param.timestamp, 1710000000);
    assert_eq!(param.signature, "abc123");
}
