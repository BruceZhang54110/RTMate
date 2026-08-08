# Data Model: 认证服务整合进实时服务

本特性**不引入新的数据表或字段**，完全复用 `rtmate-common` 已有的模型。

## 实体

### RtApp（应用/租户）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Integer (PK) | 内部租户 id |
| app_id | String | 对外租户标识 |
| app_key | String | 用于 HMAC 签名验证与 JWT 编码的密钥 |
| ... | ... | 现有其它字段 |

### RtClientConnection（客户端连接凭证）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Integer (PK) | |
| app_id | Integer (FK) | 关联 RtApp.id |
| rt_app | String | 对外的 app_id 字符串 |
| client_id | String | 本次认证的终端 UUID |
| connect_token | String | 用于 WebSocket 建立时一次性校验的 UUID |
| used | Boolean | 是否已使用 |
| expire_time | Timestamp | 过期时间（默认在创建时设置为 1 分钟后） |
| ... | ... | 现有其它字段 |

## 关系

- `rt_app` 1 : N `rt_client_connection`：一个租户可以拥有多条待使用的连接凭证记录。

## 校验规则

- 请求中的 `app_id` 必须对应 `rt_app` 表中已存在的记录。
- 请求签名必须等于 `HMAC-SHA256(app_key, "{app_id}:{state}:{timestamp}")`。
- WebSocket 连接阶段，`connect_token` 必须存在、未被使用且未过期。

## 状态转换

1. **签发阶段**：HTTP `POST /api/auth/token` 成功后，向 `rt_client_connection` 插入 `used = false` 的新记录。
2. **使用阶段**：WebSocket 建立并完成 `auth` 事件后，将该 `connect_token` 标记为 `used = true`。

## 说明

- 数据模型与数据库表结构在本次整合中不做修改。
- 迁移重点仅在于：将原本由 `rtmate-auth` 执行的 `rt_client_connection` 插入操作，纳入 `rtmate-server` 的 `ClientConnectionRepository` 统一实现。
