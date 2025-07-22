pub mod req;

pub mod handler;


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

}