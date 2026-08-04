# Contract: POST /api/auth/token

## 接口用途

应用客户端使用 `app_id`、`state`、`timestamp` 与签名申请访问令牌（JWT）和连接令牌（connect_token），用于后续 WebSocket 连接鉴权。

## 请求

- **Method**: `POST`
- **Path**: `/api/auth/token`
- **Content-Type**: `application/json`
- **Request Body**:

```json
{
  "appId": "demo-app",
  "state": "random-state-string",
  "timestamp": 1710000000,
  "signature": "hex-encoded-hmac-sha256-signature"
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| appId | string | 是 | 应用标识 |
| state | string | 是 | 随机字符串，防止重放 |
| timestamp | integer | 是 | 请求时间戳，单位秒 |
| signature | string | 是 | 签名：`HMAC-SHA256(app_key, "appId:state:timestamp")` |

## 成功响应（200 OK）

统一响应封套：

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "appId": "demo-app",
    "accessToken": "eyJhbGciOiJIUzI1NiIs...",
    "connectToken": "uuid-connect-token",
    "clientId": "uuid-client-id"
  }
}
```

`data` 字段说明：

| 字段 | 类型 | 说明 |
|------|------|------|
| appId | string | 应用标识 |
| accessToken | string | JWT，用于 WebSocket `auth` 事件 |
| connectToken | string | 一次性连接凭证，用于 WebSocket 握手后校验 |
| clientId | string | 本次分配的终端 id |

## 错误响应

所有错误均使用统一响应封套，HTTP 状态码为 200，业务错误码在 `code` 字段中体现：

```json
{
  "code": 1004,
  "message": "您的app未找到，请检查appId",
  "data": null
}
```

常见错误码（需与现有 `rtmate-auth` 实现保持一致）：

| 错误码 | 含义 | 触发场景 |
|--------|------|----------|
| 1004 | 应用未找到 | app_id 不存在于 `rt_app` 表 |
| 1005 | 签名无效 | HMAC 签名不匹配（具体码以现有实现为准） |
| 500 | 系统内部错误 | 数据库异常等 |

## 兼容性

- 请求路径、请求体字段、成功响应结构、错误响应结构必须与整合前 `rtmate-auth` 保持完全一致。
- 客户端无需修改任何代码即可继续调用。

## 相关契约

- WebSocket 连接与 `auth` 事件契约维持不变，参见项目既有文档与测试。
