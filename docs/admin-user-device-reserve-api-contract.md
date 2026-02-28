# Admin 用户注册与设备配置预留 - API 契约（仅接口保留）

## 1. 文档目的

本文件定义“设备配置功能预留”的接口契约，以及 admin 用户注册/续期涉及的管理接口契约。

说明：
- 本期仅定义契约，不实现设备配置业务逻辑。
- 设备配置接口返回“预留态”响应，供前端对接与联调占位。

---

## 2. 通用约定

- 通信方式：Tauri IPC（`invoke`）
- 响应包裹：`{ success: boolean, data: T }` 或抛出 `AppError`
- 时间字段：Unix 毫秒时间戳
- 角色枚举：`operator` | `tenant` | `maintainer`

---

## 3. 用户管理接口（admin）

## 3.1 注册用户

- Command: `auth_admin_register_user`
- 权限：仅 `admin`

Request
```json
{
  "payload": {
    "username": "tenant_a01",
    "password": "P@ssw0rd",
    "nickname": "张三",
    "phone": "13800138000",
    "roles": ["tenant", "operator"],
    "accountTermType": "days",
    "accountValidDays": 180
  }
}
```

字段说明：
- `phone`：可选
- `roles`：多选，至少 1 个
- `accountTermType`：`permanent` | `days`
- `accountValidDays`：当 `days` 时必填，且 `> 0`

Response
```json
{
  "success": true,
  "data": {
    "userId": 12,
    "username": "tenant_a01",
    "roles": ["tenant", "operator"],
    "isActive": true,
    "accountIsPermanent": false,
    "accountExpireAt": 1779999999999
  }
}
```

错误码语义（建议）：
- `FORBIDDEN`: 非 admin 调用
- `VALIDATION_ERROR`: 参数非法
- `USERNAME_EXISTS`: 用户名已存在

## 3.2 账号续期

- Command: `auth_admin_renew_user_account`
- 权限：仅 `admin`

Request
```json
{
  "payload": {
    "userId": 12,
    "renewMode": "days",
    "renewDays": 90
  }
}
```

`renewMode`：
- `permanent`：设置永久有效
- `days`：延长指定天数

Response
```json
{
  "success": true,
  "data": {
    "userId": 12,
    "accountIsPermanent": false,
    "accountExpireAt": 1787777777777,
    "isActive": true
  }
}
```

---

## 4. 设备配置预留接口（本期不实现业务）

## 4.1 查询用户设备配置（预留）

- Command: `user_device_scope_get`
- 权限：`admin`（后续可扩展）

Request
```json
{
  "payload": {
    "userId": 12
  }
}
```

Response（预留态）
```json
{
  "success": true,
  "data": {
    "implemented": false,
    "message": "RESERVED_API_NOT_IMPLEMENTED",
    "scope": {
      "allAreas": false,
      "allFloors": false,
      "allDevices": false,
      "areas": [],
      "floors": [],
      "devices": []
    }
  }
}
```

## 4.2 保存用户设备配置（预留）

- Command: `user_device_scope_upsert`
- 权限：`admin`（后续可扩展）

Request
```json
{
  "payload": {
    "userId": 12,
    "allAreas": true,
    "allFloors": false,
    "allDevices": false,
    "areas": ["A01", "A02"],
    "floors": ["F03", "F05"],
    "devices": ["dev-001", "dev-002"]
  }
}
```

Response（预留态）
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "RESERVED_API_NOT_IMPLEMENTED",
    "message": "user device scope API is reserved for next phase"
  }
}
```

---

## 5. 前端联调建议（本期）

1. 设备配置页面可先接入上述两个 command。
2. 当收到 `RESERVED_API_NOT_IMPLEMENTED` 时展示“功能预留”提示，不阻断其他页面流程。
3. 注册与续期接口可按正式逻辑联调，设备配置保持占位。
