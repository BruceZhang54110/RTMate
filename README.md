<div align="center">

# RTMate
Minimal realtime WebSocket core (Rust + Axum). Single service with HTTP auth + WebSocket channels.

</div>

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/BruceZhang54110/RTMate)

> Status: Early preview (NOT production ready).

## 为什么 / Why
RTMate 致力于成为一个轻量、可读、可扩展的实时服务内核：
- 轻量：最少抽象，易于二次开发 / 探索
- 统一：单一 JSON Envelope + 错误码
- 可演进：后续补齐频道 / Presence / Webhook / 指标 / 脚本扩展

## Features
✅ WebSocket `/ws` 握手 & auth 事件  
✅ HTTP `POST /api/auth/token` 应用认证令牌签发  
✅ 统一响应结构 (code / message / data)  
🛠 Subscribe / MessageSend (占位)  
🛠 Channels & Broadcast  
🛠 Presence  
🛠 Rate limit & Usage metrics  
🛠 Webhook & JS SDK  

图例: ✅ 已实现 | 🛠 规划中

## Quick Start
前置：Rust stable (>=1.79)。

```bash
cargo run -p rtmate-server
```

默认地址：`ws://127.0.0.1:3000/ws`

浏览器最小示例：
```javascript
const ws = new WebSocket('ws://127.0.0.1:3000/ws');
ws.onopen = () => ws.send(JSON.stringify({
  event: 'auth',
  payload: { appId: 'demo-app', token: '<JWT>' }
}));
ws.onmessage = e => console.log('resp:', e.data);
```

CLI (wscat)：
```bash
npx wscat -c ws://127.0.0.1:3000/ws
> {"event":"auth","payload":{"appId":"demo-app","token":"<JWT>"}}
```

## Minimal Protocol
Request:
```json
{"event":"auth","payload":{"appId":"demo-app","token":"<JWT>"}}
```
Success:
```json
{"code":200,"message":"success","data":{"state":true,"client_id":"..."}}
```
Error (token 过期示例):
```json
{"code":401,"message":"token 已过期","data":null}
```
更多字段 / 错误码：将迁移到 `docs/protocol.md`（尚未创建）。

## Short Roadmap
- Channels & broadcast
- Presence tracking
- JS SDK (connect / subscribe / send)
- Rate limit & metrics
- Webhook skeleton

## Crates
`rtmate-server` (入口 + 认证 + 实时通信) · `rtmate-common` (DTO/响应/Claims)

## Configuration

- `BROADCAST_CHANNEL_CAPACITY`：每个频道广播 channel 的缓冲容量，默认 `1024`。容量满时旧消息会被丢弃，慢客户端可能触发 `Lagged`。

## Limitations
Not production ready: Presence / 限流 / 持久化 / 安全强化 仍在规划中。

## Contributing
欢迎 Issue / PR。后续将补充 `CONTRIBUTING.md`。

## License
Apache-2.0（见根目录 LICENSE）

---
Future: 多租户、频道 / Presence、Webhook、用量计费、脚本扩展。If this interests you, star & follow the roadmap.