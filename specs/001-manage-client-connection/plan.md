# Implementation Plan: Client Connection Management on Connect

**Branch**: `001-manage-client-connection` | **Date**: 2025-12-30 | **Spec**: `specs/001-manage-client-connection/spec.md`  
**Input**: 特性说明「客户端连接管理（建立连接阶段）」：围绕连接生命周期、认证上下文绑定，以及按应用维度的并发 / 空闲控制。

## Summary

本特性将在 `rtmate-server` 中基于现有的 `ClientConnection` 与 `ConnectionManager`，
为每条 WebSocket 连接建立明确的生命周期管理：WebSocket 连接建立后亍需管理（暂不应整上连接池），
在认证完成后整上连接池，并为后续的每租户并发限制提供基础支持。

连接建立后，会询门官对凭证进行认证，认证成功整上连接池并产起应用标识 `rt_app`，
未认证连接直接关闭。其后，连接管理器将成为频道订阅、连接清理与资源保护的集中入口。

## Technical Context

**Language/Version**: Rust stable 1.79（Tokio 异步运行时）  
**Primary Dependencies**: Axum（提供 HTTP / WebSocket 路由与升级）、Tokio（异步与 `mpsc`）、
DashMap / DashSet（线程安全的内存 Map/Set）、Serde（用于协议 JSON 序列化）  
**Storage**: 已有的 `Dao` 用于认证与业务数据访问；本特性中的连接状态仅保存在内存中的
`ConnectionManager` 中（单节点、非持久化）  
**Testing**: 使用工作区统一的 `cargo test`；为连接生命周期新增集成测试，放在
`rtmate-server/tests/integration/` 下，使用 Axum + WebSocket 客户端模拟真实连接行为  
**Target Platform**: 长时间运行的 Rust 后端服务（Linux/macOS），由浏览器和 CLI WebSocket 客户端消费  
**Project Type**: 多 crate 工作区后端服务：`rtmate-server` 作为入口，`rtmate-common` 承载共享 DTO，
`rtmate-auth` 提供认证相关逻辑  
**Performance Goals**: 在当前迭代中，目标是单节点至少支撑 1,000 条并发 WebSocket 连接，
连接注册与查找维持 O(1) 的时间复杂度；暂不设定精确延迟 SLO，只要求不明显劣于当前简单实现  
**Constraints**: WebSocket 处理流程中禁止阻塞 IO；不得改变现有 JSON Envelope 协议对外表现；
新的连接跟踪逻辑必须遵守多 crate 边界（不将重型类型泄露到 `rtmate-common`）  
**Scale/Scope**: 范围限定为「单节点」连接管理：生命周期跟踪、认证上下文绑定以及并发限制支持；
跨节点 Presence、分布式房间和高级限流明确不在本次范围之内。

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Crate 边界**: 连接管理逻辑仅实现于 `rtmate-server`（`manager/ws_connection.rs` 与
  `web_context.rs`），共享 DTO 与错误类型仍位于 `rtmate-common`，满足职责分离要求。
- **统一事件模型与 JSON Envelope**: 本特性聚焦内部连接管理，不新增对外消息类型；现有认证
  与业务消息继续使用 `rtmate-common` 中定义的统一 JSON Envelope 协议。
- **内核简洁性**: 将复杂业务流程隔离在 handler 之外与未来的 service 层中；
  `ConnectionManager` 仅负责生命周期、订阅与简单限流，符合“简洁内核、可演进扩展”的原则。
- **质量与可观测性**: 计划中包含对连接创建、认证绑定、关闭与策略拒绝等关键事件的结构化日志；
  同时会补充至少一组连接生命周期的集成测试。

**Gate Status**: PASS —— 目前设计未引入对 RTMate 宪法的明显违背；若后续迭代需要更复杂的跨切逻辑，
必须在更新计划时显式说明并给出理由。

## Project Structure

### Documentation (this feature)

```text
specs/001-manage-client-connection/
├── plan.md              # 本文件（/speckit.plan 命令输出）
├── research.md          # Phase 0：设计与决策记录
├── data-model.md        # Phase 1：数据模型与实体关系
├── quickstart.md        # Phase 1：快速验证指南
├── contracts/           # Phase 1：行为约束与非正式契约
└── tasks.md             # Phase 2：/speckit.tasks 输出（本命令不生成）
```

### Source Code (repository root)

```text
rtmate-server/
├── src/
│   ├── web_context.rs                  # 为 WebContext 注入 ConnectionManager
│   ├── handlers/
│   │   └── ws.rs                       # 在 ws_handler/process_websocket 中接入连接管理
│   └── manager/
│       └── ws_connection.rs            # 扩展 ClientConnection & ConnectionManager 行为
└── tests/
    └── integration/
        └── ws_connection_lifecycle.rs  # 连接生命周期与清理的集成测试（计划新增）
```

**Structure Decision**: 本特性完全在 `rtmate-server` 内实现，复用现有的
`manager/ws_connection.rs` 作为核心连接注册中心，并在 `handlers/ws.rs` 中与 Axum WebSocket
handler 集成；`WebContext` 则作为依赖注入入口，将共享的 `ConnectionManager` 暴露给各 handler 使用。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|---------------------------------------|

当前设计未引入额外项目、抽象或跨切逻辑，因此未产生需要单独论证的复杂度来源；
若后续在连接管理中加入更重的模式（例如新的服务层、分布式协调模块），
需要在此表中补充具体违例与原因说明。
