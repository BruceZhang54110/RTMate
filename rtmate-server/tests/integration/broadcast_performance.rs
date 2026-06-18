//! Performance tests for high-performance broadcast (003-message-broadcast US3)
//!
//! Run with: cargo test --test broadcast_performance

use std::sync::Arc;
use std::time::Instant;
use rtmate_server::manager::BroadcastManager;
use rtmate_server::dto::OutboundMessage;
use axum::extract::ws::Message;
use tokio::sync::mpsc;

const SUBSCRIBER_COUNT: usize = 100;
const BURST_COUNT: usize = 100;
const LATENCY_THRESHOLD_MS: u128 = 50;

#[tokio::test]
async fn test_latency_for_100_subscribers() {
    let manager = BroadcastManager::new(1024);
    let channel_id = Arc::new("perf_room".to_string());
    let mut receivers = Vec::with_capacity(SUBSCRIBER_COUNT);

    for i in 0..SUBSCRIBER_COUNT {
        let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
        manager.subscribe(&channel_id, &Arc::new(format!("client_{}", i)), tx).unwrap();
        receivers.push(rx);
    }

    let outbound = OutboundMessage::Raw(Message::Text("perf-test".into()));
    let start = Instant::now();
    let count = manager.publish(&channel_id, outbound).unwrap();
    assert_eq!(count, SUBSCRIBER_COUNT);

    // Wait for all receivers to get the message
    for rx in &mut receivers {
        rx.recv().await.expect("Each subscriber should receive the message");
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < LATENCY_THRESHOLD_MS,
        "Broadcast latency {}ms exceeds threshold {}ms",
        elapsed.as_millis(),
        LATENCY_THRESHOLD_MS
    );
}

#[tokio::test]
async fn test_burst_100_messages_no_loss_no_disorder() {
    let manager = BroadcastManager::new(1024);
    let channel_id = Arc::new("burst_room".to_string());
    let mut receivers = Vec::with_capacity(SUBSCRIBER_COUNT);

    for i in 0..SUBSCRIBER_COUNT {
        let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
        manager.subscribe(&channel_id, &Arc::new(format!("client_{}", i)), tx).unwrap();
        receivers.push(rx);
    }

    let start = Instant::now();
    for seq in 0..BURST_COUNT {
        let outbound = OutboundMessage::Raw(Message::Text(format!("msg-{}", seq).into()));
        manager.publish(&channel_id, outbound).unwrap();
    }

    // Collect all messages for each subscriber
    for rx in &mut receivers {
        let mut received = Vec::with_capacity(BURST_COUNT);
        for _ in 0..BURST_COUNT {
            let msg = rx.recv().await.expect("Message should not be lost");
            received.push(msg);
        }

        // Verify order by checking message text sequence
        for (i, msg) in received.iter().enumerate() {
            let expected = format!("msg-{}", i);
            match msg {
                OutboundMessage::Raw(Message::Text(text)) => {
                    assert_eq!(text.as_ref(), expected, "Messages arrived out of order");
                }
                _ => panic!("Unexpected message type"),
            }
        }
    }

    let elapsed = start.elapsed();
    println!("Burst test completed in {}ms", elapsed.as_millis());
}
