# Quickstart: Client Connection Management on Connect

本快速开始文档说明如何运行 RTMate，并验证「客户端连接管理（建立连接阶段）」特性带来的行为变化。

## 前置条件

- 已安装 Rust stable >= 1.79。
- 通过 Cargo 拉取并构建项目依赖（按工作区默认配置即可）。

## 启动服务

```bash
cd rtmate-server
cargo run
```

默认 WebSocket 端点为：

```text
ws://127.0.0.1:3000/ws
```

## 建立连接并观察连接管理行为

1. **获取或创建一个合法的 `connect_token`**，用于某个演示应用（具体获取方式取决于项目中现有的认证流程）。
2. **使用 WebSocket 客户端建立连接**，例如在浏览器控制台中：

   ```javascript
   const ws = new WebSocket('ws://127.0.0.1:3000/ws?connect_token=<TOKEN>');
   ws.onopen = () => console.log('connected');
   ws.onclose = (e) => console.log('closed', e.code, e.reason);
   ```

3. **查看服务端日志，验证以下信息**：
   - 出现一条新连接被注册的日志，包含唯一的连接标识（client_id）。
   - 日志中能够看到从 token 推导出的应用标识（rt_app）或等价信息。

4. **关闭连接**（例如在浏览器中调用 `ws.close()`），并验证：
   - 对应的连接已从连接管理器中移除；
   - 如有频道订阅，相关订阅信息已一并被清理。

## 下一步

- 在 `rtmate-server/tests/integration/ws_connection_lifecycle.rs` 下添加集成测试，
  自动验证连接的创建、认证、关闭与清理行为。
- 按特性说明与 research 文档中的决策，进一步在连接管理器中实现按租户维度（`rt_app`）的并发限制。
  （空闲超时检测目前不在本迭代范围内）
