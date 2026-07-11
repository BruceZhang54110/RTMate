use std::sync::Arc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rtmate_server::manager::BroadcastManager;
use rtmate_server::dto::OutboundMessage;
use axum::extract::ws::Message;
use tokio::sync::mpsc;

const SUBSCRIBER_COUNTS: [usize; 3] = [10, 50, 100];
const MESSAGE_SIZES: [usize; 3] = [100, 1024, 10240]; // bytes

/// 生成指定大小的测试消息体
fn make_payload(size: usize) -> String {
    "x".repeat(size)
}

/// 模拟旧的遍历发送方式：逐个向每个订阅者的 mpsc 通道发送消息
async fn broadcast_by_iteration(
    senders: &[mpsc::Sender<OutboundMessage>],
    message: OutboundMessage,
) -> usize {
    let mut delivered = 0;
    for sender in senders {
        if sender.send(message.clone()).await.is_ok() {
            delivered += 1;
        }
    }
    delivered
}

/// 基准：对比不同订阅者数量、不同消息体大小下的广播延迟
fn bench_broadcast_with_size(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("broadcast_latency_by_size");

    for &count in &SUBSCRIBER_COUNTS {
        for &size in &MESSAGE_SIZES {
            let param = (count, size);

            group.bench_with_input(
                BenchmarkId::new("broadcast_manager", format!("{}_subs_{}_bytes", count, size)),
                &param,
                |b, &(count, size)| {
                    b.to_async(&rt).iter_with_setup(
                        || {
                            let manager = BroadcastManager::new(1024);
                            let channel_id = Arc::new("bench_ch".to_string());
                            let mut receivers = Vec::with_capacity(count);
                            for i in 0..count {
                                let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
                                manager.subscribe(&channel_id, &Arc::new(format!("client_{}", i)), tx).unwrap();
                                receivers.push(rx);
                            }
                            (manager, channel_id, receivers, make_payload(size))
                        },
                        |(manager, channel_id, mut receivers, payload)| async move {
                            let message = OutboundMessage::Raw(Message::Text(payload.into()));
                            let delivered = manager.publish(&channel_id, message).unwrap();
                            black_box(delivered);

                            for rx in &mut receivers {
                                let _ = rx.recv().await;
                            }
                            black_box(receivers);
                        },
                    );
                },
            );

            group.bench_with_input(
                BenchmarkId::new("iteration_broadcast", format!("{}_subs_{}_bytes", count, size)),
                &param,
                |b, &(count, size)| {
                    b.to_async(&rt).iter_with_setup(
                        || {
                            let mut senders = Vec::with_capacity(count);
                            let mut receivers = Vec::with_capacity(count);
                            for _ in 0..count {
                                let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
                                senders.push(tx);
                                receivers.push(rx);
                            }
                            (senders, receivers, make_payload(size))
                        },
                        |(senders, mut receivers, payload)| async move {
                            let message = OutboundMessage::Raw(Message::Text(payload.into()));
                            let delivered = broadcast_by_iteration(&senders, message).await;
                            black_box(delivered);

                            for rx in &mut receivers {
                                let _ = rx.recv().await;
                            }
                            black_box(receivers);
                        },
                    );
                },
            );
        }
    }

    group.finish();
}

/// 基准：存在一个慢订阅者时，其他订阅者的接收延迟
fn bench_slow_subscriber_impact(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("slow_subscriber_impact");
    let count = 50;

    group.bench_function("broadcast_manager_with_slow_subscriber", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let manager = BroadcastManager::new(1024);
                let channel_id = Arc::new("slow_bench_ch".to_string());
                let mut fast_receivers = Vec::with_capacity(count - 1);

                for i in 0..(count - 1) {
                    let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
                    manager.subscribe(&channel_id, &Arc::new(format!("fast_{}", i)), tx).unwrap();
                    fast_receivers.push(rx);
                }

                // 慢订阅者：buffer 容量为 1，且不消费消息
                let (slow_tx, _slow_rx) = mpsc::channel::<OutboundMessage>(1);
                manager.subscribe(&channel_id, &Arc::new("slow".to_string()), slow_tx).unwrap();

                (manager, channel_id, fast_receivers)
            },
            |(manager, channel_id, mut fast_receivers)| async move {
                let message = OutboundMessage::Raw(Message::Text("benchmark".into()));
                let start = std::time::Instant::now();
                let delivered = manager.publish(&channel_id, message).unwrap();
                black_box(delivered);

                // 只等待 fast receivers
                for rx in &mut fast_receivers {
                    let _ = rx.recv().await;
                }
                black_box(start.elapsed());
                black_box(fast_receivers);
            },
        );
    });

    group.bench_function("iteration_broadcast_with_slow_subscriber", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let mut senders = Vec::with_capacity(count);
                let mut fast_receivers = Vec::with_capacity(count - 1);

                for _ in 0..(count - 1) {
                    let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
                    senders.push(tx);
                    fast_receivers.push(rx);
                }

                // 慢订阅者：buffer 容量为 1，且不消费消息
                let (slow_tx, _slow_rx) = mpsc::channel::<OutboundMessage>(1);
                senders.push(slow_tx);

                (senders, fast_receivers)
            },
            |(senders, mut fast_receivers)| async move {
                let message = OutboundMessage::Raw(Message::Text("benchmark".into()));
                let start = std::time::Instant::now();
                let delivered = broadcast_by_iteration(&senders, message).await;
                black_box(delivered);

                // 只等待 fast receivers
                for rx in &mut fast_receivers {
                    let _ = rx.recv().await;
                }
                black_box(start.elapsed());
                black_box(fast_receivers);
            },
        );
    });

    group.finish();
}

/// 基准：多频道并发发布场景
fn bench_multi_channel_contention(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("multi_channel_contention");
    let channel_count = 10;
    let subs_per_channel = 20;

    group.bench_function("broadcast_manager_multi_channel", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let manager = BroadcastManager::new(1024);
                let mut all_receivers = Vec::with_capacity(channel_count);
                for c in 0..channel_count {
                    let channel_id = Arc::new(format!("ch_{}", c));
                    let mut receivers = Vec::with_capacity(subs_per_channel);
                    for i in 0..subs_per_channel {
                        let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
                        manager.subscribe(&channel_id, &Arc::new(format!("c{}_sub{}", c, i)), tx).unwrap();
                        receivers.push(rx);
                    }
                    all_receivers.push((channel_id, receivers));
                }
                (manager, all_receivers)
            },
            |(manager, mut all_receivers)| async move {
                for (channel_id, _) in &all_receivers {
                    let message = OutboundMessage::Raw(Message::Text("multi-channel".into()));
                    manager.publish(channel_id, message).unwrap();
                }
                for (_, receivers) in &mut all_receivers {
                    for rx in receivers {
                        let _ = rx.recv().await;
                    }
                }
                black_box(all_receivers);
            },
        );
    });

    group.bench_function("iteration_broadcast_multi_channel", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let mut all_senders = Vec::with_capacity(channel_count);
                let mut all_receivers = Vec::with_capacity(channel_count);
                for _ in 0..channel_count {
                    let mut senders = Vec::with_capacity(subs_per_channel);
                    let mut receivers = Vec::with_capacity(subs_per_channel);
                    for _ in 0..subs_per_channel {
                        let (tx, rx) = mpsc::channel::<OutboundMessage>(1024);
                        senders.push(tx);
                        receivers.push(rx);
                    }
                    all_senders.push(senders);
                    all_receivers.push(receivers);
                }
                (all_senders, all_receivers)
            },
            |(all_senders, mut all_receivers)| async move {
                for senders in &all_senders {
                    let message = OutboundMessage::Raw(Message::Text("multi-channel".into()));
                    broadcast_by_iteration(senders, message).await;
                }
                for receivers in &mut all_receivers {
                    for rx in receivers {
                        let _ = rx.recv().await;
                    }
                }
                black_box(all_receivers);
            },
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_broadcast_with_size,
    bench_slow_subscriber_impact,
    bench_multi_channel_contention
);
criterion_main!(benches);
