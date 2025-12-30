# Data Model: Client Connection Management on Connect

## Entities

### ClientConnection

代表由 `ConnectionManager` 管理的一条 WebSocket 客户端连接。

**关键字段（概念层面）:**

- `client_id` (String): 节点内唯一的连接标识，用作连接映射表的键，也用于日志与追踪。
- `rt_app` (String): 由已验证的 connect token 推导出的应用标识；在本特性中也是按应用维度
  限制并发连接的业务键。
- `connect_token` (String): 本次连接建立时使用的一次性 token；仅在必要时用于审计 / 调试，
  不允许在日志中以明文形式出现。
- `state` (Enum): 连接生命周期的逻辑状态，如 `Connecting`、`Authenticating`、`Ready`、
  `Closed(reason)` 等；现有代码中可能隐式存在，实施过程中建议逐步演进为显式枚举。
- `sender` (Channel): 指向客户端的消息发送通道句柄，用于服务器向客户端推送消息。
- `created_at` (Timestamp)【概念字段】: 连接注册的时间点。
- `closed_at` (Timestamp, optional)【概念字段】: 连接被关闭的时间点。
- `close_reason` (Enum/String, optional)【概念字段】: 连接被关闭的原因（如正常关闭、错误、
  空闲超时、并发限制等）。

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

- `client_id` 在同一运行节点内必须唯一；对于使用相同 `client_id` 注册新连接的场景，
  实现需要在“替换旧连接”与“拒绝新连接”之间进行策略选择，并保持行为一致。
- 在连接被视为 `Ready` 之前，必须已经获得合法的 `rt_app`（即来源于已通过验证的
  connect token），否则不允许作为“可用连接”参与业务消息处理。
- 当连接被移除时，其所有关联的频道订阅信息必须同时从 `channels` 与 `subscriptions`
  映射中清理，避免内存泄漏与订阅状态错乱。
- 当为某个 `rt_app` 配置了最大并发连接数时，超过该上限的新建连接尝试必须被拒绝
  或立即关闭，并记录对应的关闭原因，供后续排查与审计使用。
