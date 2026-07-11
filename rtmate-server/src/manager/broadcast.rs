use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing;
use crate::dto::{BroadcastMessage, OutboundMessage};
use crate::common::RtWsError;

pub type ChannelId = Arc<String>;
pub type ClientId = Arc<String>;

/// 高性能广播管理器
/// 
/// 每个频道维护一个 `tokio::sync::broadcast::Sender`，所有向该频道发布的消息
/// 只写入一次，由 Tokio 自动复制给多个 `Receiver`，避免逐个客户端遍历发送。
pub struct BroadcastManager {
    /// 每个频道的广播发送端
    channels: DashMap<ChannelId, broadcast::Sender<BroadcastMessage>>,
    /// 每个 (频道, 客户端) 对应的接收转发任务，用于取消订阅或断开时清理
    receiver_tasks: DashMap<(ChannelId, ClientId), JoinHandle<()>>,
    /// broadcast channel 默认容量
    default_capacity: usize,
    /// 全局序列号生成器，用于调试和顺序校验
    sequence_counter: DashMap<ChannelId, u64>,
}

impl BroadcastManager {
    /// 创建新的 BroadcastManager
    pub fn new(default_capacity: usize) -> Self {
        Self {
            channels: DashMap::new(),
            receiver_tasks: DashMap::new(),
            default_capacity,
            sequence_counter: DashMap::new(),
        }
    }

    /// 获取或创建指定频道的 broadcast sender
    fn get_or_create_sender(&self, channel_id: &ChannelId) -> broadcast::Sender<BroadcastMessage> {
        if let Some(entry) = self.channels.get(channel_id) {
            return entry.value().clone();
        }

        let (sender, _) = broadcast::channel(self.default_capacity);
        self.channels.insert(channel_id.clone(), sender.clone());
        tracing::debug!(channel_id = %channel_id, capacity = self.default_capacity, "Created broadcast channel");
        sender
    }

    /// 获取下一个序列号
    fn next_sequence(&self, channel_id: &ChannelId) -> u64 {
        let mut entry = self.sequence_counter.entry(channel_id.clone()).or_insert(0);
        *entry += 1;
        *entry
    }

    /// 客户端订阅频道
    /// 
    /// 为该客户端创建一个 `broadcast::Receiver` 并启动转发任务，将广播消息写入
    /// 客户端的 WebSocket 发送通道。
    pub fn subscribe(
        &self,
        channel_id: &ChannelId,
        client_id: &ClientId,
        ws_sender: tokio::sync::mpsc::Sender<OutboundMessage>,
    ) -> Result<(), RtWsError> {
        // 幂等：如果已经存在该 (channel, client) 的转发任务，先取消旧的
        self.unsubscribe(channel_id, client_id);

        let sender = self.get_or_create_sender(channel_id);
        let mut receiver = sender.subscribe();

        let ch_id = channel_id.clone();
        let cl_id = client_id.clone();

        let handle = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(msg) => {
                        tracing::debug!(
                            client_id = %cl_id,
                            channel_id = %ch_id,
                            sequence = msg.sequence,
                            "Forwarding broadcast message to client"
                        );
                        if ws_sender.send(msg.payload).await.is_err() {
                            tracing::debug!(client_id = %cl_id, "Client websocket sender closed, stopping broadcast receiver");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(
                            client_id = %cl_id,
                            channel_id = %ch_id,
                            skipped = skipped,
                            "Broadcast receiver lagged, messages dropped"
                        );
                        // 默认策略：继续接收最新消息
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::debug!(client_id = %cl_id, channel_id = %ch_id, "Broadcast channel closed, stopping receiver");
                        break;
                    }
                }
            }
        });

        self.receiver_tasks.insert((channel_id.clone(), client_id.clone()), handle);
        tracing::info!(client_id = %client_id, channel_id = %channel_id, "Client subscribed to broadcast channel");
        Ok(())
    }

    /// 客户端取消订阅频道
    /// 
    /// 中止对应的接收转发任务并清理状态。
    pub fn unsubscribe(&self, channel_id: &ChannelId, client_id: &ClientId) {
        let key = (channel_id.clone(), client_id.clone());
        if let Some((_, handle)) = self.receiver_tasks.remove(&key) {
            handle.abort();
            tracing::info!(client_id = %client_id, channel_id = %channel_id, "Client unsubscribed from broadcast channel");
        }

        // 如果该频道没有接收者了，可以考虑清理 sender
        if let Some(sender) = self.channels.get(channel_id) {
            if sender.receiver_count() == 0 {
                drop(sender);
                self.channels.remove(channel_id);
                self.sequence_counter.remove(channel_id);
                tracing::debug!(channel_id = %channel_id, "Broadcast channel empty, removed sender");
            }
        }
    }

    /// 向频道发布消息
    /// 
    /// 将消息写入广播 channel，返回实际接收者数量。如果频道当前没有订阅者，
    /// 不会报错，仅返回 0 并记录日志。
    pub fn publish(
        &self,
        channel_id: &ChannelId,
        payload: OutboundMessage,
    ) -> Result<usize, RtWsError> {
        let sender = self.get_or_create_sender(channel_id);
        let sequence = self.next_sequence(channel_id);
        let message = BroadcastMessage {
            channel_id: channel_id.to_string(),
            payload,
            published_at: chrono::Utc::now(),
            sequence,
        };

        match sender.send(message) {
            Ok(receiver_count) => {
                tracing::info!(
                    channel_id = %channel_id,
                    sequence = sequence,
                    subscriber_count = receiver_count,
                    "Broadcast message sent"
                );
                Ok(receiver_count)
            }
            Err(_) => {
                // 所有接收者都已关闭，但 sender 仍在；清理并返回 0
                tracing::warn!(channel_id = %channel_id, sequence = sequence, "No active receivers for broadcast message");
                Ok(0)
            }
        }
    }

    /// 获取频道当前订阅者数量
    pub fn receiver_count(&self, channel_id: &ChannelId) -> usize {
        self.channels
            .get(channel_id)
            .map(|s| s.receiver_count())
            .unwrap_or(0)
    }

    /// 清理某个客户端的所有广播接收任务
    /// 
    /// 通常在客户端断开连接时调用。
    pub fn cleanup_client(&self, client_id: &ClientId) {
        let keys_to_remove: Vec<_> = self
            .receiver_tasks
            .iter()
            .filter(|entry| entry.key().1 == *client_id)
            .map(|entry| entry.key().clone())
            .collect();

        for (channel_id, cl_id) in keys_to_remove {
            self.unsubscribe(&channel_id, &cl_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_publish_no_subscribers() {
        let manager = BroadcastManager::new(16);
        let outbound = OutboundMessage::Raw(axum::extract::ws::Message::Text("test".into()));
        let count = manager.publish(&Arc::new("ch1".to_string()), outbound).unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_subscribe_and_receive() {
        let manager = BroadcastManager::new(16);
        let channel_id = Arc::new("ch1".to_string());
        let client_id = Arc::new("c1".to_string());
        let (tx, mut rx) = mpsc::channel::<OutboundMessage>(10);

        manager.subscribe(&channel_id, &client_id, tx).unwrap();

        let outbound = OutboundMessage::Raw(axum::extract::ws::Message::Text("hello".into()));
        let count = manager.publish(&channel_id, outbound.clone()).unwrap();
        assert_eq!(count, 1);

        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let manager = BroadcastManager::new(16);
        let channel_id = Arc::new("ch1".to_string());

        let (tx1, mut rx1) = mpsc::channel::<OutboundMessage>(10);
        let (tx2, mut rx2) = mpsc::channel::<OutboundMessage>(10);

        manager.subscribe(&channel_id, &Arc::new("c1".to_string()), tx1).unwrap();
        manager.subscribe(&channel_id, &Arc::new("c2".to_string()), tx2).unwrap();

        let outbound = OutboundMessage::Raw(axum::extract::ws::Message::Text("broadcast".into()));
        let count = manager.publish(&channel_id, outbound).unwrap();
        assert_eq!(count, 2);

        assert!(rx1.recv().await.is_some());
        assert!(rx2.recv().await.is_some());
    }
}
