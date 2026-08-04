# Implementation Plan: 认证功能整合进实时服务

**Branch**: `004-integrate-auth-server` | **Date**: 2026-08-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-integrate-auth-server/spec.md`

## Summary

将独立的 `rtmate-auth` 认证服务合并进 `rtmate-server`，使单一可执行文件同时提供 HTTP 认证令牌签发接口与 WebSocket 实时发布/订阅能力。合并后工作空间只保留 `rtmate-common`（共享契约）和 `rtmate-server`（运行时 + 认证）。对外的 HTTP 认证协议、JWT 结构与 connect-token 持久化保持不变；内部要求所有数据库访问集中在统一的 repository 层，所有 HTTP/WebSocket 响应统一使用项目通用响应封套。

## Technical Context

**Language/Version**: Rust stable >= 1.79（workspace resolver = "2"，edition 2021）

**Primary Dependencies**: Axum 0.8.4（HTTP + WebSocket）、Tokio、deadpool-diesel + Diesel（PostgreSQL）、jsonwebtoken、hmac/sha2、uuid、serde、tracing、config/dotenvy

**Storage**: PostgreSQL，通过 `rtmate-common::dao::DataSource` 统一访问

**Testing**: `cargo test`、`cargo clippy`；新增 HTTP 认证端点集成测试，保留现有 WebSocket 集成测试

**Target Platform**: Linux 服务器（开发/测试在 macOS）

**Project Type**: web-service（实时 WebSocket + HTTP）

**Performance Goals**: 标准 web 服务期望；规格未给定具体指标

**Constraints**: 单一可部署二进制；统一 repository 层；统一响应封套；不重复配置/连接池；handler 不直接执行 SQL

**Scale/Scope**: 单进程运行；服务于当前测试与实际使用规模的并发客户端

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| 原则 | 初评 | 复评 | 备注 |
|------|------|------|------|
| I. Protocol-First & Unified Envelope | PASS | PASS | 保持统一 JSON 响应封套；HTTP 与 WebSocket 协议不变 |
| II. Workspace Clarity & Shared Contracts | VIOLATION（需论证） | PASS | 本特性将 `rtmate-auth` 并入 `rtmate-server`，使工作空间从 3 个 crate 变为 2 个；消除了 `rtmate-server` 对 `rtmate-auth` 私有内部的依赖，反而加强了契约边界。理由见 Complexity Tracking。 |
| III. Test-First Development | PASS | PASS | 既有测试覆盖认证与 WebSocket 流程；新增合并后集成测试 |
| IV. Minimal Abstractions / YAGNI | PASS | PASS | 不引入新 crate 或通用 trait 层；仅迁移代码到现有分层 |
| V. Observability & Structured Logging | PASS | PASS | 保留 `tracing` 结构化日志；不泄露密钥与完整令牌 |

**论证结论**：本特性导致的宪法原则 II 表面冲突，实质是消除跨模块私有耦合并降低部署单元。保留 `rtmate-auth` 作为库的方案无法达成 P1 用户故事（单一服务），因此偏离具有正当理由。复评时确认设计未引入新的私有依赖或抽象层，判定通过。

## Project Structure

### Documentation (this feature)

```text
specs/004-integrate-auth-server/
├── plan.md              # 本文件（/speckit-plan 输出）
├── research.md          # Phase 0 输出
├── data-model.md        # Phase 1 输出
├── quickstart.md        # Phase 1 输出
├── contracts/           # Phase 1 输出
└── tasks.md             # Phase 2 输出（/speckit-tasks 生成）
```

### Source Code (repository root)

```text
rtmate-server/src/
├── bootstrap.rs
├── common.rs
├── dto.rs
├── lib.rs
├── main.rs
├── req.rs
├── store.rs
├── web_context.rs
├── domain/
│   ├── mod.rs
│   └── repositories/
│       ├── channel_repository_trait.rs
│       ├── client_connection_repository_trait.rs   # ← 增加 save_connect_token
│       └── rt_app_repository_trait.rs
├── handlers/
│   ├── auth.rs              # ← 现有 WebSocket 鉴权逻辑，行为不变
│   ├── channel.rs
│   ├── errors.rs
│   ├── mod.rs
│   ├── publish.rs
│   └── ws.rs
├── infrastructure/
│   └── persistence/
│       ├── channel_repository.rs
│       ├── client_connection_repository.rs          # ← 实现 save_connect_token
│       └── rt_app_repository.rs
├── routes/
│   ├── mod.rs               # ← 注册 /api/auth/token
│   └── auth.rs              # ← 新增 HTTP 认证 endpoint handler
├── services/
│   ├── auth.rs              # ← 从 rtmate-auth 迁移：签名验证 + JWT + connect_token 生成
│   ├── channel_service.rs
│   ├── mod.rs
│   └── pubsub.rs
└── manager/
    └── ...

rtmate-server/tests/
├── integration/
│   ├── ws_pubsub.rs
│   ├── broadcast_basic.rs
│   ├── broadcast_performance.rs
│   └── broadcast_failure_isolation.rs
└── auth_integration.rs      # ← 新增：验证 HTTP 认证端点

# rtmate-auth 目录从工作空间移除
```

**Structure Decision**：认证能力直接融入 `rtmate-server` 现有分层结构。HTTP 令牌签发作为新增 route + service；WebSocket 令牌校验继续使用 `handlers/auth.rs`。`rt_app` 与 `rt_client_connection` 数据库访问复用 `infrastructure/persistence` 下的 repository。独立 `rtmate-auth` crate 从工作空间删除。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 工作空间从 3 个 crate 缩减为 2 个（宪法原则 II） | 本特性首要需求是单一可部署服务；保留 `rtmate-auth` 会维持两个二进制与两份配置，无法满足 P1 用户故事。 | 将 `rtmate-auth` 改为 `rtmate-server` 依赖的库可维持原则 II，但会留下两个部署单元、两份配置，违反用户“只需启动一个服务”的核心诉求。 |
| 使用 repository 层统一数据库访问（统一查询规范） | 规格要求统一数据库查询、高内聚低耦合；将 SQL 集中到 repository 可防止 handler/service 直接依赖数据库细节。 | 直接复制原 `rtmate-auth` 的 SQL 到 service 中代码量更少，但会制造多处直接访问数据库的实现，违反统一规范与可维护性。 |

## Phase 0: Research

见 `research.md`。核心结论：

- 认证 HTTP 接口挂载到 `POST /api/auth/token`，与 `rtmate-auth` 原路径一致。
- 现有 `ClientConnectionRepository` 已覆盖 connect_token 的查询与使用标记；只需新增 `save_connect_token` 插入方法即可替代原 `rtmate-auth` 的 repository 逻辑。
- 响应封套已统一为 `rtmate-common` 的 `RtResponse` / `AppError`。
- 选择最小迁移方案：在 `rtmate-server` 现有结构内新增 route/service，删除 `rtmate-auth`。

## Phase 1: Design

详细数据模型、接口契约与验证步骤分别见：

- `data-model.md`
- `contracts/auth_token.md`
- `quickstart.md`

设计要点：

- 数据模型无变更，复用 `rt_app` 与 `rt_client_connection`。
- HTTP 认证端点请求/响应协议保持原样；WebSocket `auth` 事件不变。
- 所有新增/迁移的 SQL 操作归入 `infrastructure/persistence` 的 repository 实现；handler 与 service 层只依赖 trait。
- 所有 HTTP 响应通过 `AppError`/`RtResponse` 统一返回；WebSocket 错误仍通过 `RtWsError` 统一 envelope。

## Done When

- [ ] `rtmate-auth` 从 `Cargo.toml` workspace 与文件系统中移除
- [ ] `rtmate-server` 暴露 `POST /api/auth/token`，行为与原 `rtmate-auth` 一致
- [ ] 所有数据库访问经由 repository 层，handler/service 无直接 SQL
- [ ] 所有 HTTP/WebSocket 响应使用统一响应封套
- [ ] `cargo test` 与 `cargo clippy` 全部通过
