use std::sync::Arc;
use crate::common::{RtWsError, WsBizCode};
use crate::dto::OutboundMessage;
use crate::manager::{ConnectionManager, BroadcastManager, ChannelId, ClientId};
use rtmate_common::response_common::RtResponse;
use serde_json::json;
use tokio::sync::mpsc::Sender;

/// 发布订阅服务：封装频道订阅、取消订阅和广播分发的业务逻辑
pub struct PubSubService;

/// 订阅结果
#[derive(Debug)]
pub struct SubscribeResult {
    pub channel_id: String,
    pub client_id: String,
}

/// 取消订阅结果
#[derive(Debug)]
pub struct UnsubscribeResult {
    pub channel_id: String,
}

/// 发布结果
#[derive(Debug)]
pub struct PublishResult {
    pub channel_id: String,
    pub delivered_count: usize,
    pub failed_count: usize,
}

impl PubSubService {
    /// 客户端订阅频道
    /// 
    /// 流程：
    /// 1. 验证频道是否已注册（由管理员预置）
    /// 2. 检查客户端是否已订阅（幂等）
    /// 3. 调用 ConnectionManager.subscribe 建立订阅关系
    /// 4. 调用 BroadcastManager.subscribe 创建广播接收器
    pub fn subscribe(
        connection_manager: &ConnectionManager,
        broadcast_manager: &BroadcastManager,
        client_id: &str,
        channel_id: &str,
        ws_sender: Sender<OutboundMessage>,
    ) -> Result<SubscribeResult, RtWsError> {
        let cid: ClientId = Arc::new(client_id.to_string());
        let chid: ChannelId = Arc::new(channel_id.to_string());

        // 1. 验证频道存在
        if !connection_manager.is_channel_exists(&chid) {
            return Err(RtWsError::biz(WsBizCode::ChannelNotFound));
        }

        // 2. 检查是否已订阅（幂等）
        if connection_manager.is_subscribed(&cid, &chid) {
            return Ok(SubscribeResult {
                channel_id: channel_id.to_string(),
                client_id: client_id.to_string(),
            });
        }

        // 3. 执行订阅关系管理
        connection_manager.subscribe(cid.clone(), chid.clone())?;

        // 4. 创建广播接收器并启动转发任务
        broadcast_manager.subscribe(&chid, &cid, ws_sender)?;

        tracing::info!(
            client_id = %client_id,
            channel_id = %channel_id,
            "Client subscribed to channel"
        );

        Ok(SubscribeResult {
            channel_id: channel_id.to_string(),
            client_id: client_id.to_string(),
        })
    }

    /// 客户端取消订阅频道
    ///
    /// 流程：
    /// 1. 检查客户端是否已订阅
    /// 2. 调用 ConnectionManager.un_subscribe 移除订阅关系
    /// 3. 调用 BroadcastManager.unsubscribe 清理广播接收器
    pub fn unsubscribe(
        connection_manager: &ConnectionManager,
        broadcast_manager: &BroadcastManager,
        client_id: &str,
        channel_id: &str,
    ) -> Result<UnsubscribeResult, RtWsError> {
        let cid: ClientId = Arc::new(client_id.to_string());
        let chid: ChannelId = Arc::new(channel_id.to_string());

        // 1. 检查是否已订阅
        if !connection_manager.is_subscribed(&cid, &chid) {
            return Err(RtWsError::biz(WsBizCode::NotSubscribed));
        }

        // 2. 执行取消订阅关系管理
        connection_manager.un_subscribe(cid.clone(), chid.clone())?;

        // 3. 清理广播接收器
        broadcast_manager.unsubscribe(&chid, &cid);

        tracing::info!(
            client_id = %client_id,
            channel_id = %channel_id,
            "Client unsubscribed from channel"
        );
        Ok(UnsubscribeResult {
            channel_id: channel_id.to_string(),
        })
    }

    /// 后端服务向频道发布消息
    ///
    /// 流程：
    /// 1. 验证频道存在
    /// 2. 构造推送消息（channel.message 格式）
    /// 3. 调用 BroadcastManager.publish 进行高性能广播
    pub async fn publish(
        connection_manager: &ConnectionManager,
        broadcast_manager: &BroadcastManager,
        channel_id: &str,
        data: serde_json::Value,
    ) -> Result<PublishResult, RtWsError> {
        let chid: ChannelId = Arc::new(channel_id.to_string());

        // 1. 验证频道存在
        if !connection_manager.is_channel_exists(&chid) {
            return Err(RtWsError::biz(WsBizCode::ChannelNotFound));
        }

        // 2. 构造 channel.message 推送消息
        let payload = json!({
            "channel_id": channel_id,
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let response = RtResponse::ok_with_data(crate::dto::WsData::Message(payload));
        let message = OutboundMessage::Response(response);

        // 3. 高性能广播投递
        let delivered = broadcast_manager.publish(&chid, message).unwrap_or(0);

        tracing::info!(
            channel_id = %channel_id,
            delivered = delivered,
            "Message broadcast completed"
        );

        Ok(PublishResult {
            channel_id: channel_id.to_string(),
            delivered_count: delivered,
            failed_count: 0,
        })
    }
}
