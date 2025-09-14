pub mod req;
pub mod handler;
pub mod store;
pub mod dao_query;
pub mod common;
pub mod dto;


#[cfg(test)]
mod tests {
    use super::*;
    use hmac::Mac;
    use uuid::Uuid;
    use std::time::UNIX_EPOCH;
    use std::time::SystemTime;
    use hmac::Hmac;
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;



    /// 测试订阅payload json
    #[test]
    fn test_subscribe_payload_json() {
        let json = r#"
            {
                "event": "subscribe",
                "payload": {
                    "channelId": "chat_general"
                },
                "metadata": {
                    "requestId": "sub_req_001"
                }
            }
        "#;
        let param: req::RequestParam = serde_json::from_str(json).unwrap();
        println!("{:?}", param);
    }

    /// 测试订阅payload json
    #[test]
    fn test_message_send_payload_json() {
        let json = r#"
            {
                "event": "messageSend",
                "payload": {
                    "channelId": "chat_general",
                    "topic": "topicaaaa",
                    "text": {
                        "name":"bruce"
                    }
                },
                "metadata": {
                    "requestId": "sub_req_001"
                }
            }
        "#;
        let param: req::RequestParam = serde_json::from_str(json).unwrap();
        println!("{:?}", param);
    }

    /// 测试 auth payload json
    #[test]
    fn test_message_auth_payload_json() {
        let json = r#"
            {
                "event": "auth",
                "payload": {
                    "appId": "dd111d",
                    "token": "sdsdf",
                    "timestamp": 1753459770000,
                    "signature": "d4f5e6b7c8d9e0f1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x"
                },
                "metadata": {
                    "requestId": "sub_req_001"
                }
            }
        "#;
        let param: req::RequestParam = serde_json::from_str(json).unwrap();
        println!("{:?}", param);
    }

    #[test]
    fn test_store() {
        let mut store = store::Store::new();
        store.insert("app1".to_string(), "key1".to_string());
        assert_eq!(store.get("app1"), Some(&"key1".to_string()));
        assert_eq!(store.get("app2"), None);
    }

    #[test]
    fn test_auth_token() {
        use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

            tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

        // 数据库存储app_id -> app_key
        let mut store = store::Store::new();
        let app_key = Uuid::new_v4().to_string();
        store.insert("abc".to_string(), app_key);

        let app_id = "abc".to_string();
        let token = Uuid::new_v4().to_string();
        let timestamp= SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64;
        // 模拟客户端生成signature
        let auth_data = format!("{}:{}:{}", &app_id, &token, &timestamp);
        let app_key = store.get(&app_id)
        .ok_or_else(|| anyhow::anyhow!("appId not found in store")).unwrap();

        println!("app_key: {}", app_key);
        println!("auth_data: {}", auth_data);

        // 使用 HMAC-SHA256 生成签名
        let mut mac = HmacSha256::new_from_slice(app_key.as_bytes()).unwrap();
        mac.update(auth_data.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        println!("signature: {}", signature);

        // 模拟客户端传入的认证信息
        let payload = req::AuthPayload {
            app_id: app_id,
            token: token,
            signature: signature,
            timestamp: timestamp
        };

        // 调用处理函数
        let result = handler::handle_auth_app(payload, &store);
        println!("{:?}", result);
    }


}