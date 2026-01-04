# Research: Client Connection Management on Connect

## Decision 1: Business entity for concurrency limits

- **Decision**: 在当前迭代中，以租户标识（`rt_app`）作为并发限制和资源隔离的业务实体维度。
- **Rationale**: 
  - 每条连接通过 `client_id` 作为唯一键进入连接池，由 `ConnectionManager` 管理具体的 WebSocket 连接
  - 连接同时携带 `rt_app` 字段标识所属租户
  - 按 `rt_app` 分组统计并发连接数，支持按租户限制最大连接数
  - 符合多租户 SaaS 的资源隔离需求，租户之间的连接被清晰地隔离
- **Alternatives considered**:
  - **按用户（userId）**: 粒度更细，但需要完整的用户身份模型，并在连接记录中传递 userId；
    当前设计更关注应用级的 auth，用户模型尚未在所有路径中稳定存在。
  - **无限制**: 不对并发连接做限制，可能导致资源耗尽。

## Decision 2: Connection pool only stores authenticated connections

- **Decision**: `ConnectionManager` 只存放已完成认证且完全就绪的连接；认证失败或未认证的连接不会进入连接池，直接关闭。
- **Rationale**: 
  - **简洁性**: 避免引入状态枚举和复杂的状态转换逻辑，连接池中的连接天然就是"就绪"的。
  - **认证门控**: WebSocket 连接建立后，必须等待客户端发送 Auth 事件并通过验证，才会创建 `ClientConnection` 并加入连接池。
  - **清晰语义**: `get_connection()` 返回 Some 表示已认证连接，返回 None 表示未认证或已断开。
- **Alternatives considered**:
  - **引入状态枚举**（Connecting/Authenticating/Ready/Closed）: 更细粒度的状态管理，但增加了实现复杂度和维护成本；对于当前的单节点场景，收益不明显。
  - **允许未认证连接进入连接池**: 需要在每次消息处理时检查状态，容易出现遗漏和安全问题。

## Decision 3: Logging and traceability

- **Decision**: 对连接创建、认证成功 / 失败、关闭、策略性拒绝等关键生命周期事件打结构化日志，
  日志中必须包含连接标识和应用标识（`client_id`、`rt_app`），但不得记录原始 token 或凭证。
- **Rationale**: 这与项目宪法要求的“质量与可观测性”一致，便于在连接规模较大时进行问题排查，
  同时不泄露敏感信息。
- **Alternatives considered**:
  - 记录完整报文 / token：信息更丰富，但与安全最佳实践冲突。
  - 几乎不记录日志：安全性高，但在生产环境排查连接问题会非常困难。

## Summary

本研究阶段明确了以下核心设计决策：

1. **并发限制维度**: 以应用标识（`rt_app`）为单位限制并发连接数
2. **连接池语义**: 只存放已认证的就绪连接，未认证连接不进入连接池
3. **日志与追踪**: 记录关键生命周期事件但不暴露敏感凭证

后续实现将默认：以应用为单位限制并发连接数，采用简洁的连接池设计，并记录清晰的审计日志。
