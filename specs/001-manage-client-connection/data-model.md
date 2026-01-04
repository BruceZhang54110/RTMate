# Data Model: Client Connection Management on Connect

## Entities

### ClientConnection

代表由 `ConnectionManager` 管理的一条 WebSocket 客户端连接。连接池通过 `client_id` 作为唯一键管理具体的连接。

**关键字段（概念层面）:**

- `client_id` (Arc<String>): 节点内唯一的连接标识，用作连接池映射表的键。由 Auth 认证成功后生成或从认证信息中提取。
- `rt_app` (String): 租户标识（应用标识），通过 Auth 事件认证后获得。用于：
  - 关联连接到特定租户
  - 按租户维度统计并发连接数，支持按租户限制最大连接数
- `connect_token` (String): 本次 WebSocket 连接建立时使用的一次性 token；仅在必要时用于审计 / 调试，
  不允许在日志中以明文形式出现。
- `sender` (mpsc::Sender): 指向客户端的消息发送通道句柄，用于服务器向该客户端推送消息。
- `authenticated_at` (Timestamp)【可选字段】: 认证成功并加入连接池的时间点。

**设计说明:**
- 连接池中的连接已经完成认证，**不需要状态枚举字段**
- 未认证的连接不会进入连接池，直接在 ws.rs 中关闭
- 关闭原因通过日志记录，不需要在结构体中持久化

### ConnectionManager

内存中的连接注册与订阅管理器，用于追踪所有活跃客户端连接及其频道订阅关系。

**主要职责:**

- 维护从 `client_id` 到 `ClientConnection` 的映射。
- 提供添加 / 移除连接的接口，并在移除连接时清理其对应的频道订阅关系。
- 基于 `rt_app` 统计每个应用下的当前连接数，以支持按应用维度的并发限制策略。

### BusinessEntityIdentifier（应用级）

表示本特性中用来做并发限制与资源隔离的业务维度。

**在本迭代中的定义:**

- Business entity identifier = `rt_app`（应用标识），由 connect token / 认证过程推导而来。
- 并发限制以“每个 `rt_app` 能够建立的最大连接数”为单位进行配置与控制。

## Relationships

- **ClientConnection ↔ ConnectionManager**: 每个 `ClientConnection` 由 `ConnectionManager`
  管理，负责将其插入 / 移除内部映射表。
- **ClientConnection ↔ BusinessEntityIdentifier**: 每条连接都关联到一个业务实体
  （在本迭代中为一个 `rt_app`），连接管理器基于该字段统计并发数并执行限制策略。

## Validation Rules

- `client_id` 在同一运行节点内必须唯一；由 Auth 事件认证成功后生成（通常从 JWT token 的 claims 中提取）。
- 连接必须先完成 Auth 事件认证，获得合法的 `rt_app`，才能创建 `ClientConnection` 并加入连接池。
- 认证失败、超时或第一条消息不是 Auth 事件的连接，直接关闭，不进入连接池。
- 当连接从连接池移除时，其所有关联的频道订阅信息必须同时从 `channels` 与 `subscriptions`
  映射中清理，避免内存泄漏与订阅状态错乱。
- 当为某个 `rt_app` 配置了最大并发连接数时，超过该上限的新认证请求必须被拒绝，
  并在日志中记录原因，供后续排查与审计使用。
