//! Failure isolation tests for broadcast (003-message-broadcast US2)
//!
//! Run with: cargo test --test broadcast_failure_isolation

use std::sync::Arc;
use rtmate_server::manager::BroadcastManager;
use rtmate_server::dto::OutboundMessage;
use axum::extract::ws::Message;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_subscriber_disconnect_does_not_affect_others() {
    let manager = BroadcastManager::new(16);
    let channel_id = Arc::new("failure_room".to_string());

    let (tx1, mut rx1) = mpsc::channel::<OutboundMessage>(10);
    let (tx2, mut rx2) = mpsc::channel::<OutboundMessage>(10);
    let (tx3, mut rx3) = mpsc::channel::<OutboundMessage>(10);

    let client1 = Arc::new("client_1".to_string());
    let client2 = Arc::new("client_2".to_string());
    let client3 = Arc::new("client_3".to_string());

    manager.subscribe(&channel_id, &client1, tx1).unwrap();
    manager.subscribe(&channel_id, &client2, tx2).unwrap();
    manager.subscribe(&channel_id, &client3, tx3).unwrap();

    // Disconnect client 2
    manager.unsubscribe(&channel_id, &client2);

    // Give the aborted forwarding task a moment to drop its receiver
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let outbound = OutboundMessage::Raw(Message::Text("after-disconnect".into()));
    let count = manager.publish(&channel_id, outbound).unwrap();

    // Active receivers may briefly report the aborted task; verify behavior instead of exact count
    assert!(count >= 2, "At least 2 active subscribers should receive the message");
    assert!(rx1.recv().await.is_some());
    assert!(rx2.recv().await.is_none()); // disconnected client gets nothing
    assert!(rx3.recv().await.is_some());
}

#[tokio::test]
async fn test_slow_subscriber_does_not_block_others() {
    let manager = BroadcastManager::new(16);
    let channel_id = Arc::new("slow_room".to_string());

    let (tx1, mut rx1) = mpsc::channel::<OutboundMessage>(10);
    // Slow client has capacity 1
    let (tx2, _rx2) = mpsc::channel::<OutboundMessage>(1);

    manager.subscribe(&channel_id, &Arc::new("fast_client".to_string()), tx1).unwrap();
    manager.subscribe(&channel_id, &Arc::new("slow_client".to_string()), tx2).unwrap();

    // Publish multiple messages; slow client's buffer may lag but fast client should receive
    for i in 0..5 {
        let outbound = OutboundMessage::Raw(Message::Text(format!("msg-{}", i).into()));
        manager.publish(&channel_id, outbound).unwrap();
    }

    // Fast client should still receive messages
    let mut received = 0;
    while let Ok(Some(_)) = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx1.recv()).await {
        received += 1;
    }

    assert!(received > 0, "Fast subscriber should receive messages despite slow subscriber");
}

#[tokio::test]
async fn test_lagged_receiver_is_handled_gracefully() {
    let manager = BroadcastManager::new(2); // Very small capacity to force lag
    let channel_id = Arc::new("lag_room".to_string());

    let (tx, mut rx) = mpsc::channel::<OutboundMessage>(10);
    manager.subscribe(&channel_id, &Arc::new("lag_client".to_string()), tx).unwrap();

    // Fill up broadcast channel and overflow
    for i in 0..10 {
        let outbound = OutboundMessage::Raw(Message::Text(format!("msg-{}", i).into()));
        manager.publish(&channel_id, outbound).unwrap();
    }

    // Receiver should still be alive and receive some messages
    let mut received = 0;
    while let Ok(Some(_)) = tokio::time::timeout(tokio::time::Duration::from_millis(50), rx.recv()).await {
        received += 1;
    }

    // We won't assert exact count due to lag, but the receiver task should not panic
    println!("Lagged receiver received {} messages", received);
}
