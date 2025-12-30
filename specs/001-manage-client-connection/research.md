# Research: Client Connection Management on Connect

## Decision 1: Business entity for concurrency limits

- **Decision**: 在当前迭代中，以应用标识（`appId` / `rt_app`）作为并发限制使用的业务实体主维度。
- **Rationale**: 现有连接结构中已经携带 `rt_app`，天然适合作为归类与统计的键；
  实际压力通常也来自某个“吵闹”的应用实例，以应用为单位做限制既简单又符合 RTMate 作为“应用级实时内核”的定位。
- **Alternatives considered**:
  - **按用户（userId）**: 粒度更细，但需要完整的用户身份模型，并在连接记录中传递 userId；
    当前设计更关注应用级的 auth，用户模型尚未在所有路径中稳定存在。
  - **按租户（tenantId）**: 更贴合多租户 SaaS 的资源隔离，但目前代码中尚未显式建模租户概念，
    在此处引入会对未来的租户设计造成较强耦合。

## Decision 2: Connection lifecycle states

- **Decision**: 将连接生命周期抽象为最少包含以下状态：`Connecting`、`Authenticating`、
  `Ready`、`Closed(reason)`，并将当前 `ConnectionManager` 中活跃的 `ClientConnection`
  视为 `Ready` 连接；同时逐步向显式状态模型演进。
- **Rationale**: 现有 `ConnectionManager` 已经跟踪活跃连接，但缺乏明确状态；引入小而清晰的状态机，
  有利于在认证门控、消息处理和清理时保持行为一致。
- **Alternatives considered**:
  - 单一“active”状态 + 多个布尔标记：实现简单，但一旦引入认证、订阅、限流等功能，
    易出现状态组合爆炸与逻辑分支混乱的问题。
  - 更细粒度的状态（如 `Reconnecting`、`Suspended` 等）：适用于更复杂的 Presence / HA 场景，
    当前迭代不需要，留待未来扩展。

## Decision 3: Idle timeout and heartbeat behavior

- **Decision**: 引入可配置的空闲超时（例如默认 60 秒），当在该时间窗内没有应用层消息或有效 Ping/Pong
  交互时，将连接视为“空闲超时”，并主动关闭，记录关闭原因。
- **Rationale**: 当前 handler 已经处理 Ping/Pong，将其作为最简单的心跳信号可以快速提升对“僵尸连接”的防护，
  且实现成本较低。
- **Alternatives considered**:
  - 只依赖 TCP / WebSocket 层的断开：实现最简单，但在底层传输静默失败的情况下，
    连接管理器可能长期保留已经失效的连接。
  - 自定义更复杂的心跳协议与事件：灵活度更高，但更适合后续 Presence/健康检查等增强功能，
    当前迭代先采用保守实现。

## Decision 4: Logging and traceability

- **Decision**: 对连接创建、认证成功 / 失败、关闭、策略性拒绝等关键生命周期事件打结构化日志，
  日志中必须包含连接标识和应用标识（`client_id`、`rt_app`），但不得记录原始 token 或凭证。
- **Rationale**: 这与项目宪法要求的“质量与可观测性”一致，便于在连接规模较大时进行问题排查，
  同时不泄露敏感信息。
- **Alternatives considered**:
  - 记录完整报文 / token：信息更丰富，但与安全最佳实践冲突。
  - 几乎不记录日志：安全性高，但在生产环境排查连接问题会非常困难。

## Summary

本研究阶段明确了并发限制的主业务维度（按应用）、连接的状态机形态、空闲连接的处理策略以及日志与追踪的原则。

后续实现与测试将默认：以应用为单位限制并发连接数，使用小而清晰的状态模型，并采用保守的空闲超时机制与安全的日志策略。
