# Contracts: Client Connection Management on Connect

本特性不新增新的 HTTP 对外端点，而是对现有 WebSocket 端点 `/ws` 的行为做更精细的约束，
在内部增加连接管理语义。

## WebSocket `/ws` 行为（非正式契约）

### 连接成功时

- 客户端使用合法的 `connect_token` 查询参数发起到 `ws://<host>/ws` 的 HTTP 升级请求。
- 服务端使用已有的认证逻辑对 `connect_token` 进行校验。
- 校验成功后，服务端必须：
  - 将该 `connect_token` 标记为已使用，避免重复使用；
  - 在 `ConnectionManager` 中创建新的 `ClientConnection` 条目，包含：
    - 节点内唯一的 `client_id`；
    - 由 token 推导出的应用标识 `rt_app`；
    - 指向客户端的发送通道句柄（sender）。

### 认证消息

- 认证消息与 JSON Envelope 的具体字段在协议层已有定义，本特性不修改这些外部字段。
- 本特性在行为上的核心约束是：
  - 认证成功后，需在 `ConnectionManager` 中整上 ClientConnection 记录，绑定租户标识 rt_app；
  - 在认证完成之前接受的业务消息不得按"已认证连接"的逻辑处理，
    必须返回错误或直接关闭连接，与规格中的需求保持一致。

### 连接关闭

- 当连接因客户端主动关闭、服务端主动关闭、网络异常、并发限制等原因关闭时，服务端必须：
  - 从 `ConnectionManager` 中移除对应的 `ClientConnection`；
  - 为该 `client_id` 清理所有频道订阅关系；
  - 在日志中记录合适的关闭原因，便于后续排查。

## JSON Envelope

- 所有业务与认证消息继续使用项目级别定义的统一 JSON Envelope 协议，本特性不改变消息字段结构。
- 本特性的目标是确保每一条活跃的 WebSocket 会话背后，都有一条可追踪的连接记录，
  而不是引入新的外部协议形态。
