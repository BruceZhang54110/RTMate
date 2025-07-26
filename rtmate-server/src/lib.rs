pub mod req;
pub mod handler;
pub mod store;


#[cfg(test)]
mod tests {
    use super::*;

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
                    "timestamp": 1753459770000
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

}