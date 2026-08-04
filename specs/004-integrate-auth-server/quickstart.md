# Quickstart: 验证 004 认证服务整合

## 前置条件

- 已安装 Rust 工具链。
- PostgreSQL 已运行，并已创建 `rtmate` 数据库。
- 已应用 migrations（基线 migration 已包含 `rt_app` 与 `rt_client_connection` 表）。
- 工作目录为项目根目录。
- `.env` 中配置了数据库连接，例如：

```bash
DATABASE_URL=postgres://postgres:password@localhost/rtmate
```

## 验证步骤

### 1. 构建合并后的服务

```bash
cargo build -p rtmate-server
```

### 2. 启动服务

```bash
cargo run -p rtmate-server
```

服务将监听 `127.0.0.1:3000`。

### 3. 申请令牌

使用 `rt_app` 表中已存在的应用凭证构造请求（需先自行计算 HMAC-SHA256 签名）：

```bash
curl -X POST http://127.0.0.1:3000/api/auth/token \
  -H 'Content-Type: application/json' \
  -d '{
    "appId": "demo-app",
    "state": "test-state",
    "timestamp": 1710000000,
    "signature": "<your-hmac-signature>"
  }'
```

**预期结果**：响应码为 `200`，响应体包含 `code: 200`、`accessToken`、`connectToken` 与 `clientId`。

### 4. 建立 WebSocket 连接并鉴权

```bash
wscat -c 'ws://127.0.0.1:3000/ws'
> {"event":"auth","payload":{"appId":"demo-app","token":"<accessToken>"}}
```

**预期结果**：服务器返回认证成功事件，客户端可继续订阅/发布频道。

### 5. 对比兼容性（回归验证）

- 将相同请求分别发送到整合后的 `rtmate-server` 与旧 `rtmate-auth` 的响应逐字段比较，确认完全一致。
- 重点比较：成功响应的字段名、错误码、错误消息。

### 6. 运行自动检查

```bash
cargo test -p rtmate-server
cargo clippy -p rtmate-server -- -D warnings
```

**预期结果**：
- 测试全部通过。
- `clippy` 无警告（或项目允许的警告数不变）。
- 项目不再包含 `rtmate-auth` crate 的构建产物。

## 故障排查

- 若数据库连接失败，检查 `.env` 与 PostgreSQL 服务状态。
- 若签名验证失败，确认 HMAC 使用 `app_key` 对字符串 `"appId:state:timestamp"` 计算。
- 若 404，确认 `routes/mod.rs` 已正确注册 `/api/auth/token`。
