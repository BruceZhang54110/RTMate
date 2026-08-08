# Tasks: 认证功能整合进实时服务

**Input**: Design documents from `/specs/004-integrate-auth-server/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: 本任务清单包含测试任务。根据项目宪法“Test-First Development”与规格中明确的独立测试要求，测试任务需先于对应实现编写并确保失败。

**Organization**: 按用户故事组织，保证每个故事可独立实现与测试。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可与其他无依赖任务并行执行
- **[Story]**: 所属用户故事（US1, US2, US3）
- 描述中包含精确文件路径

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 本特性无需新增项目脚手架；迁移复用现有 workspace 与依赖。

- 无独立 setup 任务。所有共享基础设施（DataSource、tracing、Axum Router）已在 `rtmate-server` 中存在。

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 完成用户故事前的公共基础：repository 能力补齐、DTO 补齐、认证端点失败测试。

**⚠️ CRITICAL**: 在所有用户故事开始前必须先完成本阶段，否则后续实现无法编译或测试。

- [x] T001 [P] 在 `rtmate-server/src/domain/repositories/client_connection_repository_trait.rs` 中新增 `save_connect_token` 方法，并在 `rtmate-server/src/infrastructure/persistence/client_connection_repository.rs` 中实现该方法。
- [x] T002 [P] 在 `rtmate-server/src/dto.rs` 中新增 `RtAppParam` 与 `AppAuthResult` 请求/响应 DTO。
- [x] T003 [P] 在 `rtmate-server/tests/auth_integration.rs` 中编写 `POST /api/auth/token` 集成测试（先失败），使用原始 JSON 验证端点存在与统一响应封套。

**Checkpoint**: Foundation ready - 数据库插入能力、认证 DTO、失败测试均已就绪，可进入用户故事实现。

---

## Phase 3: User Story 1 - 单一服务同时提供认证与实时通信 (Priority: P1) 🎯 MVP

**Goal**: 合并后的 `rtmate-server` 单一进程同时暴露 HTTP 认证接口与 WebSocket 实时接口。

**Independent Test**: 启动一个 `rtmate-server` 进程，先请求 `/api/auth/token` 获取令牌，再用该令牌在 `/ws` 完成 WebSocket `auth` 事件。

### Tests for User Story 1

- [x] T004 [US1] 运行 `rtmate-server/tests/auth_integration.rs` 直到通过；验证认证端点已挂载到统一服务。

### Implementation for User Story 1

- [x] T005 [P] [US1] 在 `rtmate-server/src/services/auth.rs` 中实现 HMAC 签名验证、JWT 生成与 connect_token 持久化逻辑（依赖 T001, T002）。
- [x] T006 [US1] 在 `rtmate-server/src/routes/auth.rs` 中实现 `POST /api/auth/token` handler，并在 `rtmate-server/src/routes/mod.rs` 中注册该路由（依赖 T002, T005）。
- [x] T007 [US1] 在 `rtmate-server/src/web_context.rs` 中确认 `ClientConnectionRepository` 与 `RtAppRepository` 已正确注入；无需新增上下文字段。

**Checkpoint**: 此时只启动一个 `rtmate-server` 进程即可同时完成 HTTP 认证与 WebSocket 连接。

---

## Phase 4: User Story 2 - 客户端认证方式保持兼容 (Priority: P2)

**Goal**: 客户端请求/响应协议与整合前完全一致；错误场景与 WebSocket 鉴权兼容。

**Independent Test**: 用相同请求分别调用整合前后的认证端点，逐字段对比成功响应与错误响应；用新端点签发的 JWT 通过 WebSocket 鉴权。

### Tests for User Story 2

- [x] T008 [P] [US2] 在 `rtmate-server/tests/auth_contract.rs` 中对比 `/api/auth/token` 成功响应字段与整合前样本（`appId`、`accessToken`、`connectToken`、`clientId` 及响应封套）。
- [x] T009 [P] [US2] 在 `rtmate-server/tests/auth_error.rs` 中覆盖签名无效与应用未找到两类错误场景，确认错误码与消息与整合前一致。

### Implementation for User Story 2

- [x] T010 [US2] 验证 `rtmate-server/src/handlers/auth.rs` 中的 WebSocket 鉴权逻辑仍接受新端点签发的 JWT（依赖 T006）。
- [x] T011 [US2] 确保 `rtmate-server/src/routes/auth.rs` 的错误转换统一使用 `AppError`/`RtResponse`，不返回裸领域类型或自定义 JSON。
- [x] T022 [US2] 在 `rtmate-server/tests/auth_db_integration.rs` 中新增真实数据库访问的集成测试，覆盖成功签发 token 与签名无效两类场景，并验证 `rt_client_connection` 表中已写入 connect_token 记录。

**Checkpoint**: User Story 2 完成后，客户端无需修改任何代码即可继续使用。

---

## Phase 5: User Story 3 - 代码库移除独立认证模块 (Priority: P3)

**Goal**: 工作空间中不再存在独立认证 crate；构建与测试全通过。

**Independent Test**: 全量 `cargo build`、`cargo test`、`cargo clippy` 通过；项目中无 `rtmate-auth` 残留引用。

### Tests for User Story 3

- [x] T012 [US3] 运行 `cargo test -p rtmate-server` 与 `cargo clippy -p rtmate-server -- -D warnings`，确认全部通过。

### Implementation for User Story 3

- [x] T013 [P] [US3] 从 `Cargo.toml` workspace 的 `members` 中移除 `rtmate-auth`。
- [x] T014 [P] [US3] 删除 `rtmate-auth/` 目录及其全部源码。
- [x] T015 [US3] 运行 `cargo update` 清理 `Cargo.lock` 中不再需要的条目。
- [x] T016 [US3] 在 `rtmate-server` 与 `rtmate-common` 中搜索 `rtmate_auth`、`rtmate-auth` 残留引用并清除。

**Checkpoint**: User Story 3 完成后，`rtmate-auth` 完全从仓库移除，只保留 `rtmate-common` 与 `rtmate-server` 两个 crate。

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: 文档更新、统一代码规范检查、端到端验证。

- [x] T017 [P] 更新 `README.md` 与 `docs/rust-overview.md`，移除“独立认证服务”描述，反映单一服务部署形态。
- [x] T018 [P] 更新 `progress.md` 与 `feature_list.json`，记录 004 特性状态。
- [x] T019 [P] 按 `quickstart.md` 手动执行端到端验证：HTTP 取 token → WebSocket `auth` → 发布/订阅。
- [x] T020 [P] 检查 `rtmate-server` 所有 handler/service，确保无直接 SQL、无裸 JSON 响应；必要时修复以符合统一 repository 与统一响应封套要求。
- [x] T021 [P] 检查并更新宪法（`.specify/memory/constitution.md`）中关于 crate 划分的描述，将工作空间从三 crate 结构改为两 crate 结构（如本次偏离需正式写入宪法）。

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无任务。
- **Foundational (Phase 2)**: 无前置依赖；阻塞所有用户故事。
- **User Stories (Phase 3-5)**: 均依赖 Foundational 完成；彼此之间可并行（但推荐按 P1 → P2 → P3 顺序降低风险）。
- **Polish (Phase 6)**: 依赖所有期望用户故事完成。

### User Story Dependencies

- **User Story 1 (P1)**: 仅依赖 Foundational。完成后即具备 MVP 验证能力。
- **User Story 2 (P2)**: 依赖 Foundational + T006（US1 端点实现完成即可开始）。
- **User Story 3 (P3)**: 依赖 Foundational + 所有 US1/US2 代码稳定（可在移除前确认无依赖）。

### Within Each User Story

- 测试先写并失败；再实现模型/服务/端点；最后集成验证。
- 服务依赖 repository 补齐；handler 依赖 DTO 与服务。

### Parallel Opportunities

- T001（repository 方法）、T002（DTO 补齐）、T003（失败测试）可并行。
- T005（service 实现）与 T006（handler/路由注册）需串行；T007 可并行于 T005/T006。
- T008 与 T009（US2 测试）可并行。
- T013、T014、T015、T016（US3 清理）可并行。
- T017-T021（Polish）可并行。

---

## Parallel Example: User Story 1

```bash
# 在 Foundational 完成后，可并行启动：
Task: "T001 repository 方法补齐"
Task: "T002 DTO 补齐"
Task: "T003 失败集成测试"

# 随后串行：
Task: "T005 service 实现" -> "T006 handler + 路由注册" -> "T004 测试通过"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. 完成 Phase 2 Foundational。
2. 完成 Phase 3 User Story 1。
3. **STOP and VALIDATE**: 手动运行 quickstart 验证单一服务即可同时完成 HTTP 认证与 WebSocket 连接。

### Incremental Delivery

1. Foundational → 完成公共基础。
2. User Story 1 → 单一服务同时提供 HTTP 认证 + WebSocket → 部署/演示 MVP。
3. User Story 2 → 协议兼容验证 → 客户端零改动。
4. User Story 3 → 移除 `rtmate-auth` crate → 全量构建测试通过。
5. Polish → 文档更新、代码规范检查、宪法更新。

### Parallel Team Strategy

- 开发者 A：T001 + T005 + T006（repository → service → handler）
- 开发者 B：T002 + T003 + T004（DTO + 测试）
- 开发者 C：T008 + T009 + T010 + T011（US2 协议兼容测试）
- 在 US1 稳定后，开发者 D：T013-T016（US3 清理）

---

## Notes

- `[P]` 任务表示可与其他无依赖任务并行。
- `[Story]` 标签将任务映射到具体用户故事，保证可追溯。
- 每个用户故事应能独立完成与测试；不要引入跨故事强依赖。
- 实现前请确认测试已失败（test-first）。
- 每完成一个任务或逻辑组后提交；可在任意 checkpoint 停下来独立验证。
- 避免：模糊任务、同一文件冲突、破坏故事独立性的跨故事依赖。
