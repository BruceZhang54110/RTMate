<!--
Sync Impact Report
- Version change: 1.0.0 → 1.1.0 (add docs/ learning documentation sync rule in development process)
- Modified principles (newly defined):
  - Principle 1: 内核与多 crate 架构优先
  - Principle 2: 统一标准事件模型与 JSON Envelope
  - Principle 3: 简洁内核与可演进扩展
  - Principle 4: 质量与可观测性优先
  - Principle 5: 治理与版本控制
- Added sections:
  - 架构与技术栈约束
  - 开发流程与质量保障
  - Governance（治理规则具体化）
- Removed sections:
  - None (template placeholders were materialized, no sections dropped)
- Templates requiring updates:
  - None
- Templates checked for alignment (no changes required):
  - .specify/templates/plan-template.md
  - .specify/templates/spec-template.md
  - .specify/templates/tasks-template.md
  - .specify/templates/checklist-template.md
  - .specify/templates/agent-file-template.md
- Commands templates:
  - ⚠ `.specify/templates/commands` directory not present; when command templates are added, they MUST be checked against this constitution.
- Runtime guidance docs checked:
  - README.md (already consistent with mission and protocol direction)
- Deferred TODOs:
  - None
-->

# RTMate Constitution

## Core Principles

### 内核与多 crate 架构优先

RTMate 是一个在 Rust 多 crate 架构下实现的 WebSocket as a Service 实时内核。
所有新功能 MUST 优先以可复用 crate / 模块形式实现，保持 `rtmate-server`（入口内核）、
`rtmate-common`（协议与 DTO）、`rtmate-auth`（认证领域实验与能力沉淀）边界清晰，
跨 crate 共享的数据结构和错误模型 MUST 定义在 `rtmate-common` 中。

### 统一标准事件模型与 JSON Envelope

本项目在 Rust 多 crate 架构下，以统一标准事件模型为核心约束，所有对外 HTTP/WS 行为
MUST 通过统一的 JSON Envelope 表达；事件 `type` 必须采用稳定的命名规范
（如 `auth.login`, `room.join`），并保持向后兼容扩展；错误信息 MUST 通过 Envelope 中
结构化的错误字段暴露，而不是任意文本或多种风格并存。

### 简洁内核与可演进扩展

连接内核 MUST 保持最小必要抽象：专注连接管理、认证钩子、事件路由与房间/频道模型，
避免在 handler 中堆叠复杂业务逻辑；扩展能力（频道、Presence、限流、Webhook、脚本等）
SHOULD 通过清晰的接口或事件扩展点接入，而不是直接侵入内核实现，从而支持按里程碑
渐进式演进而不破坏现有用户。

### 质量与可观测性优先

关键协议解析、事件路由与认证逻辑 MUST 具备单元测试或集成测试覆盖；任何影响外部接口
或协议的变更 MUST 附带相应的用例或示例更新。服务 MUST 使用结构化日志记录关键维度
（连接 ID、用户/租户、事件类型、错误码），并避免记录敏感信息（token、密码、秘钥）；
一旦出现可复现的协议/内核缺陷，修复 MUST 伴随监控或日志信号的增强。

### 治理与版本控制

宪法是项目的最高治理规则，对开发流程、协议变更和破坏性改动具有约束力；任何破坏性
协议或行为变更 MUST 经过显式设计评审与版本管理。仓库采用语义化版本管理：
MAJOR 表示治理或协议的非兼容调整，MINOR 表示新增原则或扩展指导，PATCH 表示措辞
澄清和非语义性修正；所有规范性变更 MUST 在本宪法中反映并更新版本信息。

## 架构与技术栈约束

- 项目 MUST 使用 Rust stable（当前假定 >=1.79）与 Tokio 异步运行时，核心 HTTP/WS 接入
  由 Axum 提供；JSON 序列化 MUST 使用 Serde 或与之兼容的方案。
- `rtmate-server` 负责 WebSocket 接入、握手、连接管理、事件路由和基础房间/频道模型；
  `rtmate-common` 负责统一响应结构、事件 DTO、错误码与共享 Claims 类型；
  `rtmate-auth` 负责认证相关领域逻辑与实验实现，并逐步沉淀为可复用组件。
- 任一 crate 引入新依赖时 MUST 评估其对其他 crate 的影响，并避免将重型依赖泄露到
  `rtmate-common` 中，以保持公共层的轻量与可移植性。
- 对外暴露的协议字段、错误码和事件类型 MUST 视为公共契约，变更前必须评估对现有
  客户端与未来扩展（例如 JS SDK、Webhook）的兼容性。

## 开发流程与质量保障

- 分支策略推荐使用长寿命主分支 `main` 加短生命周期的 `feature/*` 分支；任何进入
  `main` 的变更 MUST 通过 Pull Request 和 Code Review，不允许直接向主分支推送。
- 每项特性或修复 SHOULD 关联清晰的 Issue，至少包含：问题/目标描述、方案摘要、协议
  影响与验收标准；破坏性变更 MUST 在 Issue 中明确标注并在评审时重点讨论。
- 所有变更在合并前 MUST 至少通过 `cargo build`，涉及核心协议或连接内核的改动
  SHOULD 增加或更新相应测试；在发现现有测试覆盖不足时，应优先补齐关键路径测试。
- README 与后续的协议文档（例如 `docs/protocol.md` 一旦创建） MUST 与当前宪法保持
  一致：当协议行为或事件模型被更新时，应在同一变更集中同步文档。
- 在实现涉及 Rust 语言特性或内核行为的重要变更时 SHOULD 同步更新 `docs/` 目录下的
  学习文档（例如 `docs/rust-overview.md`），以保持项目的“可学习性”。

## Governance

本宪法优先于仓库中其他实践文档或历史习惯，一旦存在冲突，以本宪法为准；任何有意
偏离本宪法的实现 MUST 在设计说明或 Pull Request 描述中显式说明理由与影响。

宪法的修改分为三类：MAJOR（治理或协议规则的非兼容重写）、MINOR（新增原则或扩展
指导）、PATCH（措辞澄清和非语义性修正）；每次修改 MUST 在本文件中更新版本与
“Last Amended” 日期，并在 Sync Impact Report 中简要描述影响范围。

Code Review 过程 MUST 包含“宪法检查”环节：至少验证当前变更是否遵守模块边界、协议
约定、测试与日志要求；若存在合理的违例，则 MUST 在变更描述中记录原因，并在后续
迭代表达为新的原则或对现有原则的修订提案。

**Version**: 1.1.0 | **Ratified**: 2025-12-30 | **Last Amended**: 2025-12-30
