//! Integration tests for basic broadcast behavior (003-message-broadcast US1)
//!
//! Run with: cargo test --test broadcast_basic

use std::sync::Arc;
use rtmate_server::manager::BroadcastManager;
use rtmate_server::dto::OutboundMessage;
use axum::extract::ws::Message;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_single_subscriber_receives_message() {
    let manager = BroadcastManager::new(16);
    let channel_id = Arc::new("room_001".to_string());
    let client_id = Arc::new("client_1".to_string());
    let (tx, mut rx) = mpsc::channel::<OutboundMessage>(10);

    manager.subscribe(&channel_id, &client_id, tx).unwrap();

    let outbound = OutboundMessage::Raw(Message::Text("hello".into()));
    let count = manager.publish(&channel_id, outbound).unwrap();

    assert_eq!(count, 1);
    assert!(rx.recv().await.is_some());
}

#[tokio::test]
async fn test_multiple_subscribers_receive_same_message() {
    let manager = BroadcastManager::new(16);
    let channel_id = Arc::new("room_001".to_string());

    let (tx1, mut rx1) = mpsc::channel::<OutboundMessage>(10);
    let (tx2, mut rx2) = mpsc::channel::<OutboundMessage>(10);
    let (tx3, mut rx3) = mpsc::channel::<OutboundMessage>(10);

    manager.subscribe(&channel_id, &Arc::new("client_1".to_string()), tx1).unwrap();
    manager.subscribe(&channel_id, &Arc::new("client_2".to_string()), tx2).unwrap();
    manager.subscribe(&channel_id, &Arc::new("client_3".to_string()), tx3).unwrap();

    let outbound = OutboundMessage::Raw(Message::Text("broadcast".into()));
    let count = manager.publish(&channel_id, outbound).unwrap();

    assert_eq!(count, 3);
    assert!(rx1.recv().await.is_some());
    assert!(rx2.recv().await.is_some());
    assert!(rx3.recv().await.is_some());
}

#[tokio::test]
async fn test_publish_to_empty_channel_does_not_panic() {
    let manager = BroadcastManager::new(16);
    let channel_id = Arc::new("empty_room".to_string());

    let outbound = OutboundMessage::Raw(Message::Text("hello".into()));
    let count = manager.publish(&channel_id, outbound).unwrap();

    assert_eq!(count, 0);
}
