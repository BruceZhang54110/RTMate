use std::sync::Arc;

use chrono::Local;
use rtmate_common::models::NewRtChannel;

use crate::common::{RtWsError, WsBizCode};
use crate::domain::repositories::channel_repository_trait::ChannelRepositoryTrait;
use crate::domain::repositories::rt_app_repository_trait::RtAppRepositoryTrait;
use crate::dto::{CreateChannelRequest, CreateChannelResponse};

pub struct ChannelService;

impl ChannelService {
    /// 创建频道（幂等）。
    ///
    /// 流程：
    /// 1. 根据 `app_id` 字符串查询 `rt_app`，确认租户存在。
    /// 2. 按 `(rt_app.id, name)` 查询是否已有频道。
    /// 3. 若有，直接返回已有频道（HTTP 200 语义）。
    /// 4. 若无，插入新频道并返回（HTTP 201 语义）。
    pub async fn create_channel(
        rt_app_repository: Arc<dyn RtAppRepositoryTrait>,
        channel_repository: Arc<dyn ChannelRepositoryTrait>,
        app_id: &str,
        client_id: &str,
        request: CreateChannelRequest,
    ) -> Result<(CreateChannelResponse, bool), RtWsError> {
        let rt_app = rt_app_repository
            .get_rt_app_by_app_id(app_id)
            .await
            .map_err(|e| RtWsError::system("数据库查询失败", e))?
            .ok_or_else(|| RtWsError::biz(WsBizCode::AppNotFound))?;

        let name = request.name.trim().to_string();
        let description = request.description.map(|d| d.trim().to_string());

        if let Some(existing) = channel_repository
            .find_by_app_and_name(rt_app.id, &name)
            .await
            .map_err(|e| RtWsError::system("查询频道失败", e))?
        {
            return Ok((CreateChannelResponse::from(existing), false));
        }

        let now = Some(Local::now());
        let new_channel = NewRtChannel {
            app_id: rt_app.id,
            app_id_str: rt_app.app_id,
            name,
            description,
            created_by: Some(client_id.to_string()),
            created_time: now,
            updated_time: None,
        };

        let created = channel_repository
            .create(new_channel)
            .await
            .map_err(|e| RtWsError::system("创建频道失败", e))?;

        Ok((CreateChannelResponse::from(created), true))
    }
}
