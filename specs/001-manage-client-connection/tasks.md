# Tasks: 客户端连接管理（建立连接阶段）

**Input**: 基于 `specs/001-manage-client-connection/` 下的设计文档  
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**User Input**: 使用 ws_connection.rs 的连接池等功能管理 ws.rs 中创建的连接

**核心设计决策**: 
- ✅ **不使用状态枚举** - 连接池只存放已认证的就绪连接
- ✅ **认证门控** - 未认证连接不进入连接池，直接关闭
- ✅ **极简架构** - 移除了状态管理的复杂度

**Tests**: 测试任务为可选项，本特性未明确要求 TDD，因此测试任务标记为参考实现。

**Organization**: 任务按用户故事分组，使每个故事可以独立实现和测试。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行执行（不同文件，无依赖）
- **[Story]**: 任务所属用户故事（如 US1, US2, US3）
- 包含精确文件路径

## Path Conventions

本项目为多 crate 工作区，路径基于 `rtmate-server/` 目录：
- 源代码：`rtmate-server/src/`
- 测试：`rtmate-server/tests/`
- 配置：根目录 `.env` 和 `Cargo.toml`

---

## Phase 1: Setup（项目初始化）

**Purpose**: 确保依赖和基础配置就绪

- [ ] T001 验证 Rust 版本 >= 1.79，确认 Tokio、Axum、DashMap 依赖已在 rtmate-server/Cargo.toml 中
- [ ] T002 [P] 检查 .env 配置文件，确认 WebSocket 地址和端口配置存在
- [ ] T003 [P] 运行 `cargo build` 确保当前代码编译通过

---

## Phase 2: Foundational（基础设施 - 所有用户故事的前置条件）

**Purpose**: 完成核心连接管理基础设施，为所有用户故事提供支撑

**⚠️ CRITICAL**: 在此阶段完成之前，任何用户故事都无法开始实施

### 基础结构扩展（简化版）

- [ ] T004 在 rtmate-server/src/manager/ws_connection.rs 中为 ConnectionManager 的 remove_connection 方法改为 pub，供外部调用
- [ ] T005 [P] 在 rtmate-server/src/manager/ws_connection.rs 中为 ConnectionManager 添加 get_connection 方法，用于查询连接是否存在

### WebContext 依赖注入

- [ ] T006 在 rtmate-server/src/web_context.rs 中添加 ConnectionManager 字段（使用 Arc<ConnectionManager>）
- [ ] T007 在 rtmate-server/src/web_context.rs 的 new 方法中初始化 ConnectionManager 实例
- [ ] T008 在 rtmate-server/src/manager/mod.rs 中导出 ConnectionManager 和 ClientConnection 类型

**Checkpoint**: 基础设施完成 - 用户故事实现现在可以开始

---

## Phase 3: User Story 1 - 连接建立时创建并跟踪客户端连接 (Priority: P1) 🎯 MVP

**Goal**: WebSocket 连接建立后，等待客户端发送 Auth 事件并认证成功，才创建 ClientConnection 并加入连接池；认证失败直接关闭连接

**Independent Test**: 启动服务，用 WebSocket 客户端建立连接并发送合法的 Auth 事件，验证日志中有认证成功和连接创建记录；发送非法 Auth 验证连接被直接关闭

### Implementation for User Story 1

- [ ] T009 [US1] 在 rtmate-server/src/handler.rs 中新增 handle_auth_and_register 函数，合并认证与连接注册逻辑
- [ ] T010 [US1] 在 handle_auth_and_register 中，首先解析消息并验证是 Auth 事件，非 Auth 返回错误
- [ ] T011 [US1] 在 handle_auth_and_register 中，调用现有的 handle_auth_app 执行认证逻辑
- [ ] T012 [US1] 在 handle_auth_and_register 中，认证成功后创建 mpsc 通道（tx, rx）
- [ ] T013 [US1] 在 handle_auth_and_register 中，创建 ClientConnection 实例（包含 client_id, rt_app, connect_token, sender）
- [ ] T014 [US1] 在 handle_auth_and_register 中，调用 connection_manager.add_connection 注册连接并返回 client_id
- [ ] T015 [US1] 在 rtmate-server/src/handlers/ws.rs 的 process_websocket 函数开始时，等待第一条消息
- [ ] T016 [US1] 在 process_websocket 中，将第一条消息传给 handle_auth_and_register 进行认证
- [ ] T017 [US1] 在 process_websocket 中，认证成功后发送成功响应，并记录结构化日志（包含 client_id 和 rt_app）
- [ ] T018 [US1] 在 process_websocket 中，认证失败后发送错误响应，记录日志并直接 return 退出（不进入连接池）
- [ ] T019 [US1] 在 process_websocket 的正常消息处理循环中，处理业务消息（使用已有的 handle_msg）
- [ ] T020 [US1] 在 process_websocket 函数退出时（所有 break 分支），调用 connection_manager.remove_connection 清理连接
- [ ] T021 [US1] 在 process_websocket 退出时添加结构化日志记录连接关闭（包含 client_id、关闭原因）

### Tests for User Story 1（可选 - 参考实现）

- [ ] T022 [P] [US1] 在 rtmate-server/tests/integration/ 下创建 ws_connection_lifecycle.rs 集成测试文件
- [ ] T023 [P] [US1] 编写测试用例：建立 WebSocket 连接并认证成功，验证 ConnectionManager 中存在对应连接记录
- [ ] T024 [P] [US1] 编写测试用例：建立连接但认证失败，验证连接被关闭且未进入连接池

**Checkpoint**: 此时 User Story 1 功能完整且可独立测试

---

## Phase 4: User Story 2 - 基于连接记录维护认证与上下文 (Priority: P2)

**➡️ 注意**: 此用户故事已合并到 User Story 1 中实现（认证即注册）

在简化设计中，认证成功就意味着连接已经具备完整的上下文（client_id, rt_app）并加入连接池。
如果未来需要更丰富的上下文（如 user_id, tenant_id），可以在此阶段扩展。

**当前无需额外任务** - 直接跳过到 User Story 3

---

## Phase 5: User Story 3 - 按租户维度的并发连接限制 (Priority: P2) 🔒

**Goal**: 每个租户（`rt_app`）可以配置最大并发连接数限制。当某个租户的活跃连接数达到配置上限时，
新的认证请求被拒绝。实现方式：通过 `ConnectionManager` 按 `rt_app` 统计当前连接数，
在认证时（`handle_auth_and_register`）检查租户的连接数是否超限。

**设计说明**：
- 连接通过 `client_id` 作为唯一键进入连接池（`DashMap<ClientId, Arc<ClientConnection>>`）
- 按 `rt_app` 分组统计并计数（辅助的统计数据结构）
- 租户之间的限制相互独立

**Independent Test**: 配置租户 app_a 的最大并发连接数为 2，建立 3 条连接（同一租户），
验证第 3 条被拒绝；同时配置租户 app_b 的最大连接数为 1，验证租户间的限制是独立的（app_b 的第 2 条被拒绝，但不影响 app_a）。

### Implementation for User Story 3

- [ ] T025 [US3] 在 .env 或配置模块中添加 MAX_CONNECTIONS_PER_APP 配置项
- [ ] T026 [US3] 在 rtmate-server/src/manager/ws_connection.rs 中为 ConnectionManager 添加 app_connection_counts: DashMap<String, usize> 字段
- [ ] T027 [US3] 在 rtmate-server/src/manager/ws_connection.rs 的 add_connection 方法中，检查当前 rt_app 的连接数是否超过限制
- [ ] T028 [US3] 在 rtmate-server/src/manager/ws_connection.rs 的 add_connection 方法中，如果超限则返回错误
- [ ] T029 [US3] 在 rtmate-server/src/manager/ws_connection.rs 的 add_connection 方法成功时，递增 rt_app 计数
- [ ] T030 [US3] 在 rtmate-server/src/manager/ws_connection.rs 的 remove_connection 方法中，递减 rt_app 计数
- [ ] T031 [US3] 在 rtmate-server/src/handler.rs 的 handle_auth_and_register 中，当 add_connection 返回超限错误时，记录日志并返回错误

### Tests for User Story 3（可选 - 参考实现）

- [ ] T032 [P] [US3] 在 rtmate-server/tests/integration/ws_connection_lifecycle.rs 中添加并发限制测试

**Checkpoint**: 所有用户故事现在独立功能完整

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: 完善日志、文档和跨用户故事的优化

- [ ] T033 [P] 在 rtmate-server/src/manager/ws_connection.rs 中统一所有结构化日志格式，确保包含 client_id 和 rt_app
- [ ] T034 [P] 更新 docs/rust-overview.md，添加关于认证门控、连接池语义和所有权模式的说明
- [ ] T035 代码 review：确认所有连接管理逻辑符合项目宪法中的 crate 边界和简洁内核原则
- [ ] T036 运行 `cargo test` 验证所有测试通过
- [ ] T037 运行 `cargo clippy` 并修复所有警告
- [ ] T038 按照 specs/001-manage-client-connection/quickstart.md 验证功能

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖 - 可立即开始
- **Foundational (Phase 2)**: 依赖 Setup 完成 - **阻塞所有用户故事**
- **User Stories (Phase 3-5)**: 全部依赖 Foundational 完成
  - 用户故事可并行进行（如有多人协作）
  - 或按优先级顺序执行（P1 → P2 → P3）
- **Polish (Phase 6)**: 依赖所有期望的用户故事完成

### User Story Dependencies

- **User Story 1 (P1)**: 仅依赖 Foundational - 无其他用户故事依赖
- **User Story 2 (P2)**: 已合并到 User Story 1 中（认证即注册）
- **User Story 3 (P3)**: 仅依赖 Foundational 和 US1 - 建议在 US1 完成后实施

### Within Each User Story

- Tests（如有）必须在实现前编写并确保失败
- 模型/结构扩展优先于服务层
- 服务层优先于 handler 集成
- 核心实现完成后再做故事间集成
- 故事完成后验证独立性再进入下一优先级

### Parallel Opportunities

- Setup 阶段所有标记 [P] 的任务可并行
- Foundational 阶段所有标记 [P] 的任务可并行（在 Phase 2 内）
- Foundational 完成后，所有用户故事可并行开始（如团队容量允许）
- 每个用户故事内标记 [P] 的测试任务可并行
- 每个用户故事内标记 [P] 的模型任务可并行
- 不同用户故事可由不同团队成员并行实施

---

## Parallel Example: User Story 1

```bash
# 同时启动 User Story 1 的所有并行任务（如有测试）:
Task: "在 rtmate-server/tests/integration/ 下创建 ws_connection_lifecycle.rs"
Task: "编写测试用例：建立 WebSocket 连接，验证 ConnectionManager 中存在对应连接记录"
Task: "编写测试用例：关闭 WebSocket 连接，验证连接记录被正确清理"

# 同时启动 Foundational 阶段的并行模型扩展:
Task: "在 ClientConnection 添加时间戳字段"
Task: "在 ClientConnection 添加 close_reason 字段"
```

---

## Implementation Strategy

### MVP First (仅 User Story 1)

1. 完成 Phase 1: Setup
2. 完成 Phase 2: Foundational（关键 - 阻塞所有故事）
3. 完成 Phase 3: User Story 1
4. **停止并验证**: 独立测试 User Story 1
5. 如果准备好可部署/演示

### Incremental Delivery

1. 完成 Setup + Foundational → 基础就绪
2. 添加 User Story 1 → 独立测试 → 部署/演示（MVP!）
3. User Story 2 已合并到 US1
4. 添加 User Story 3 → 独立测试 → 部署/演示
5. 每个故事增加价值而不破坏之前故事

### Parallel Team Strategy

多开发者场景:

1. 团队一起完成 Setup + Foundational
2. Foundational 完成后:
   - 开发者 A: User Story 1（认证与连接池）
   - 开发者 B: User Story 3（并发限制与空闲超时）
3. 故事独立完成并集成

---

## Notes

- [P] 任务 = 不同文件，无依赖
- [Story] 标签将任务映射到具体用户故事，便于追溯
- 每个用户故事应可独立完成和测试
- 在实现前验证测试失败
- 在每个任务或逻辑组后提交
- 在任何检查点停止以独立验证故事
- 避免：模糊任务、相同文件冲突、破坏独立性的跨故事依赖

## Task Summary

- **总任务数**: 38
- **User Story 1**: 13 个实现任务 + 3 个测试任务（可选）
- **User Story 2**: 已合并到 User Story 1
- **User Story 3**: 7 个实现任务 + 1 个测试任务（可选）
- **Setup**: 3 个任务
- **Foundational**: 5 个任务
- **Polish**: 6 个任务
- **并行机会**: Foundational 阶段1 个，每个用户故事内多个
- **建议 MVP 范围**: Setup + Foundational + User Story 1（约 21 个任务）

### 与原设计的对比

| 项目 | 原设计 | 简化后 | 减少 |
|------|------|--------|------|
| 总任务数 | 52 | 38 | -14 (-27%) |
| Foundational | 8 | 5 | -3 |
| User Story 1 | 11+3 | 13+3 | +2 |
| User Story 2 | 8+2 | 0 | -10 |
| User Story 3 | 12+2 | 7+1 | -6 |

**个收获**:
- ✅ 移除了空闲超时检测相关任务（T006, T033-T035, T037）
- ✅ Foundational 中不再需要 `last_activity_at` 字段
- ✅ US3 从 10+2 需求简化为 7+1 需求
