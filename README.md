<div align="center">

# RTMate

Minimal WebSocket Service Core (MVP) – Rust + Axum

</div>

> Status: Experimental (NOT production ready). Broadcast / channel / presence logic not yet implemented. Only `auth` event is functional.

## 目录 (Table of Contents)
1. [What](#1-what)
2. [Features](#2-features-implemented-vs-planned)
3. [Quick Start](#3-quick-start)
4. [Request Schema](#4-request-schema)
5. [Response Schema](#5-response-schema)
6. [Error Codes](#6-error-codes-wsbizcode)
7. [Roadmap](#7-roadmap-short-term)
8. [Project Structure](#8-project-structure)
9. [Limitations](#9-limitations)
10. [License](#10-license)
11. [Auth Token 说明](#11-auth-token-说明)
12. [Metadata / TraceId](#12-metadata--traceid)
13. [开发调试](#13-开发调试)
14. [贡献 / Contributing](#14-贡献--contributing)

## 1. What
RTMate 是一个探索型实时服务内核，目标演进为 “WebSocket as a Service / Realtime BaaS”。当前仅包含：基础 WebSocket 握手、Auth 事件处理、统一错误响应模型。

## 2. Features (Implemented vs Planned)
| 功能 | 状态 | 说明 |
|------|------|------|
| /ws 握手 | ✅ | 建立 WebSocket 连接 |
| Auth 事件 | ✅ | JWT 校验 app_id / exp / 签名 |
| 统一错误模型 | ✅ | WsBizCode + RtWsError + RtResponse |
| Subscribe / MessageSend | 🕑 | 仅占位，无逻辑 |
| 广播 / 频道管理 | ❌ | 计划中 |
| Presence 在线状态 | ❌ | 计划中 |
| 限流 / 用量统计 | ❌ | 计划中 |
| Webhook / 外部事件 | ❌ | 计划中 |
| 脚本扩展 (Wasm/Lua) | ❌ | 计划中 |

## 3. Quick Start
前置依赖：已安装 Rust（建议 stable 最新版本，例如 1.79+）。

```bash
cd rtmate-server
cargo run
```
默认监听：`ws://127.0.0.1:3000/ws`

浏览器测试：
```javascript
const ws = new WebSocket('ws://127.0.0.1:3000/ws');
ws.onopen = () => {
	ws.send(JSON.stringify({
		event: 'auth',
		payload: { appId: 'demo-app', token: 'FAKE_OR_REAL_JWT' }
	}));
};
ws.onmessage = e => console.log('response:', e.data);
```

## 4. Request Schema
```jsonc
{
	"event": "auth",
	"payload": { "appId": "demo-app", "token": "<JWT>" },
	"metadata": { "traceId": "optional" }
}
```

## 5. Response Schema
Envelope 统一格式：
```jsonc
{
	"code": <number>,        // 200 表示成功；4xx 业务错误；500 系统错误
	"message": "<string>",  // 简短描述
	"data": { ... } | null   // 业务数据或 null
}
```

成功示例：
```json
{"code":200,"message":"success","data":{"state":true,"client_id":"CLIENT_ID"}}
```
失败示例（Token 过期）：
```json
{"code":401,"message":"token 已过期","data":null}
```
失败示例（不支持的事件）：
```json
{"code":400,"message":"不支持的事件类型","data":null}
```
失败示例（参数错误 / JSON 结构不符）：
```json
{"code":400,"message":"参数错误","data":null}
```
系统错误（内部异常，message 会较通用；详细堆栈仅记录在服务端日志）：
```json
{"code":500,"message":"internal error","data":null}
```

## 6. Error Codes (WsBizCode)
| 枚举 | code | message |
|------|------|---------|
| InvalidParams | 400 | 参数错误 |
| AppNotFound | 400 | app_id 未找到 |
| InvalidToken | 401 | 无效的 token |
| ExpiredToken | 401 | token 已过期 |
| SignatureInvalid | 1005 | 签名验证失败 |
| AuthMismatch | 401 | 认证失败（app_id 不匹配） |
| UnsupportedEvent | 400 | 不支持的事件类型 |

系统错误：`code=500`，message 为简短描述，详细堆栈记录在服务器日志。

## 7. Roadmap (Short Term)
Phase 1 (MVP++)
- [ ] Channel Registry & 广播
- [ ] Presence (在线成员 / 计数)
- [ ] Subscribe / MessageSend 逻辑实现

Phase 2 (可观测 & 控制)
- [ ] Rate Limit / Usage 采集
- [ ] TraceId 注入与日志统一
- [ ] Prometheus 指标 (连接数 / QPS / 错误分布)

Phase 3 (生态 & 扩展)
- [ ] Webhook 骨架 (connect / disconnect / message)
- [ ] JS SDK v0 (connect / subscribe / send / onMessage)
- [ ] Token 签发/刷新 辅助工具

Phase 4 (进阶特性)
- [ ] Lua / Wasm 脚本扩展沙箱
- [ ] 多租户限额 / 计费 (usage aggregation)
- [ ] 灰度 / 分片 / 水平扩展策略

## 8. Project Structure
```
rt-common/        # 共享 DTO / Response / Claims
rtmate-server/    # WebSocket 服务入口 (当前主要逻辑)
rtmate-auth/      # 认证相关探索 (后续可能整合)
```

## 9. Limitations
- 无广播 / 频道 / Presence
- 未做限流 / 用量 / 防滥用
- 事件 subscribe / messageSend 未实现逻辑
- 不做消息持久化 / 重放
- 不建议用于生产环境

## 10. License
Apache-2.0 （见根目录 LICENSE）

---
Future Goal: 高性能、多租户、可扩展（Webhook / 脚本 / 用量计费）的实时消息内核。欢迎 Issue / PR。