# Research: 认证服务整合进实时服务

## 待解问题

1. 如何在单一服务中同时暴露 HTTP 认证接口与 WebSocket 实时接口？
2. 认证所需的数据库表是否已存在于 `rtmate-server` 的仓库中？
3. 是否需要统一响应封套？
4. 如何移除 `rtmate-auth` 而不破坏现有协议？
5. 代码迁移后如何保证高内聚、低耦合？

## 调研结果

### 1. HTTP 与 WebSocket 接口共存

`rtmate-server` 已使用 Axum `Router` 同时挂载 HTTP 路由（`/api/channels/*`）与 WebSocket 升级路径（`/ws`）。新增 `POST /api/auth/token` 只需在 `routes/mod.rs` 中追加 `.route("/api/auth/token", post(auth_token_handler))`，与现有机制完全一致。

### 2. 数据库表与 Repository 覆盖

- `rt_app` 表：`rtmate-server` 已有 `RtAppRepository` 提供 `get_rt_app_by_app_id`。
- `rt_client_connection` 表：`rtmate-server` 已有 `ClientConnectionRepository` 提供查询与使用标记。
- `rtmate-auth` 还需要向 `rt_client_connection` 插入新记录。因此只需在 `ClientConnectionRepositoryTrait` 增加 `save_connect_token` 方法，并在实现中统一插入逻辑。

### 3. 统一响应封套

`rtmate-common` 提供 `AppError` / `BizError` / `RtResponse`：`rtmate-server` 的 HTTP 路由（`channel_service`）已使用 `AppError`；`rtmate-auth` 的 `auth_token` handler 也返回 `Result<Json<RtResponse<AppAuthResult>>, AppError>`。两者可直接合并到同一响应封套下，无需新抽象。

### 4. 移除 rtmate-auth

`rtmate-auth` 仅包含：
- `auth_token` HTTP endpoint
- HMAC-SHA256 签名验证
- JWT 生成
- 插入 `rt_client_connection` 的 connect_token
- 启动 web 服务所需的最小配置

迁移后：
- endpoint 移到 `rtmate-server/src/routes/auth.rs`。
- 签名验证与 JWT 生成移到 `rtmate-server/src/services/auth.rs`。
- 数据库插入复用 `rtmate-server/src/infrastructure/persistence/client_connection_repository.rs`。
- `Cargo.toml` workspace 删除 `rtmate-auth` member；原目录与文件删除。

### 5. 高内聚、低耦合保证

- 所有数据库访问通过 `domain/repositories` 的 trait 定义，`infrastructure/persistence` 的 struct 实现。
- `routes` 只负责解析请求与构造响应；`services` 只负责领域逻辑；`repositories` 只负责持久化。
- 不新增跨模块私有耦合；`rtmate-server` 对 `rtmate-common` 的依赖是公开契约依赖。

## 决策

**选择方案**：最小迁移方案——在 `rtmate-server` 现有分层结构内新增 route/service，复用现有 `WebContext` 与 repository，删除 `rtmate-auth` 整个 crate。

## 理由

- 符合规格要求：单一服务、统一 repository、统一响应封套。
- 符合宪法最小抽象原则：不引入新 crate 或通用 trait 层。
- 保持客户端零改动：对外协议与数据模型不变。
- 直接消除重复配置与重复依赖。

## 备选方案及拒绝原因

| 备选方案 | 拒绝原因 |
|----------|----------|
| 保留 `rtmate-auth` 作为库 crate，`rtmate-server` 依赖它 | 会保留两个可部署单元与两份配置，违反 P1 用户故事“单一服务” |
| 提取新的 `rtmate-auth-core` 共享 crate 供两者使用 | 仅服务于一个使用方，违反 YAGNI，且增加抽象层 |
