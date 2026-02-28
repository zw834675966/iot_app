# 鉴权模块 (Auth) - 开发者指南

本文档详细介绍 `src-tauri/src/auth/` 鉴权模块的设计原理、代码结构和扩展开发指南。

## 目录

- 模块概述
- 目录结构
- 架构设计
- 核心功能
- 数据模型
- JWT 机制详解
- 前端集成
- 扩展开发指南
- 测试策略
- 常见问题

---

## 模块概述

本模块是整个能源管理系统的鉴权核心，负责处理用户认证、权限校验和动态路由获取。采用领域驱动设计（DDD）思想，将接口层、业务层、模型层完全解耦，确保高可测试性和可维护性。

### 设计目标

1. 离线安全：无需外部网络依赖，所有数据存储在本地数据库
2. 无状态认证：使用 JWT 令牌实现无状态的身份验证
3. 解耦设计：业务逻辑完全独立于 Tauri 框架，可复用
4. 类型安全：使用 Rust 强类型系统确保数据安全

---

## 目录结构

```
src-tauri/src/auth/
├── mod.rs              # 模块声明和导出
├── commands.rs         # IPC 接口层（Adapter Layer）- 前端调用入口
├── admin_commands.rs   # 管理员 IPC 接口层
├── services.rs         # 业务逻辑层（Domain Layer）- 核心业务规则
├── admin_services.rs   # 管理员业务逻辑层
├── models.rs           # 数据模型层（DTO）- 数据传输对象
└── README.md           # 本文档
```

### 各层职责

| 文件                | 层级          | 职责                           | 特点               |
| ------------------- | ------------- | ------------------------------ | ------------------ |
| `commands.rs`       | Adapter Layer | 参数校验、结果封装、IPC 路由   | 薄层适配，仅做转发 |
| `admin_commands.rs` | Adapter Layer | 管理员命令处理                 | 薄层适配           |
| `services.rs`       | Domain Layer  | 业务规则、令牌管理、数据库查询 | 纯函数，无框架依赖 |
| `admin_services.rs` | Domain Layer  | 管理员业务规则                 | 纯函数             |
| `models.rs`         | DTO Layer     | 数据结构定义、序列化配置       | 仅包含数据字段     |

---

## 架构设计

### 分层架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              前端 (Vue/TypeScript)                           │
│                                                                             │
│   ┌─────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐   │
│   │  登录页面   │  │  路由守卫       │  │  权限指令 (v-permission)    │   │
│   └──────┬──────┘  └────────┬────────┘  └──────────────┬──────────────┘   │
│          │                   │                            │                  │
│          └───────────────────┼────────────────────────────┘                  │
│                              │ invoke()                                       │
└──────────────────────────────┼───────────────────────────────────────────────┘
                               │
                               ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                    Tauri IPC Bridge (commands.rs / admin_commands.rs)        │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  #[tauri::command] auth_login()                                    │   │
│   │  1. 参数反序列化 (serde)                                           │   │
│   │  2. 基础校验 (非空检查)                                            │   │
│   │  3. 调用 services 层                                               │   │
│   │  4. 封装响应结果                                                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                    业务逻辑层 (services.rs / admin_services.rs)              │
│                                                                             │
│   ┌──────────────────┐  ┌──────────────────┐  ┌────────────────────────┐   │
│   │ mint_token_pair  │  │ verify_refresh_  │  │ resolve_user_profile  │   │
│   │ - 生成令牌对     │  │   token          │  │ - 查询用户档案        │   │
│   │ - HS256 签名    │  │ - 验签           │  │ - 关联角色权限        │   │
│   └──────────────────┘  └──────────────────┘  └────────────────────────┘   │
│                                                                             │
│   ┌──────────────────┐  ┌──────────────────┐                              │
│   │ build_login_data │  │ build_async_     │                              │
│   │ - 组装登录数据   │  │   routes         │                              │
│   │ - 合并令牌       │  │ - 查询动态路由   │                              │
│   └──────────────────┘  └──────────────────┘                              │
└───────────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                    数据访问层 (db/auth_repository.rs / admin_repository.rs)   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  数据库                                                            │   │
│   │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐ ┌────────┐  │   │
│   │  │  users   │ │user_roles│ │permissions│ │ routes │ │device_ │  │   │
│   │  │          │ │          │ │          │ │        │ │registry│  │   │
│   │  └──────────┘ └──────────┘ └──────────┘ └────────┘ └────────┘  │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────────────────┘
```

### 数据流向

```
登录请求流程：

前端                          后端
  │                             │
  │ invoke("auth_login", {...}) │
  │ ─────────────────────────► │
  │                             │ commands.rs
  │                             │   ├─ 接收参数
  │                             │   ├─ 校验非空
  │                             │   └─ 调用 services
  │                             │
  │                             │ services.rs
  │                             │   ├─ 查询用户
  │                             │   ├─ 验证密码
  │                             │   ├─ 生成令牌
  │                             │   └─ 组装数据
  │                             │
  │ { success: true, data: {...} }
  │ ◄───────────────────────── │
  │                             │
  │ 保存令牌到本地存储          │
```

---

## 核心功能

### 1. 用户登录 (auth_login)

功能：验证用户名密码，返回用户档案和令牌对

流程：

1. 接收前端传入的 username 和 password
2. 校验参数非空
3. 查询数据库验证用户凭据
4. 生成 JWT 令牌对
5. 组装登录返回数据

错误处理：

- 用户名为空 → "username is required"
- 密码为空 → "password is required"
- 用户不存在或密码错误 → "invalid username or password"

### 2. 令牌刷新 (auth_refresh_token)

功能：使用刷新令牌获取新的令牌对

流程：

1. 接收前端传入的 refreshToken
2. 校验参数非空
3. 验证 JWT 签名和有效期
4. 校验令牌类型必须是 refresh
5. 生成新的令牌对

安全特性：

- 区分 access 和 refresh 令牌类型
- 防止令牌混淆攻击
- 每次刷新生成新的 refreshToken

### 3. 获取动态路由 (auth_get_async_routes)

功能：获取基于角色的动态路由配置

流程：

1. 查询数据库路由表
2. 组装为 vue-router 兼容格式
3. 返回路由数组

### 4. 管理员注册用户 (auth_admin_register_user)

功能：管理员创建新用户账号

### 5. 管理员续期用户账号 (auth_admin_renew_user_account)

功能：延长或设置用户账号有效期

### 6. 管理员列出用户 (auth_admin_list_users)

功能：获取所有用户列表

### 7. 管理员更新用户 (auth_admin_update_user)

功能：更新用户信息

### 8. 管理员删除用户 (auth_admin_delete_user)

功能：删除用户账号

### 9. 管理员修改密码 (auth_admin_change_user_password)

功能：重置用户密码

---

## 数据模型

### 模型分类

| 类型       | 结构体                   | 用途                     |
| ---------- | ------------------------ | ------------------------ |
| 请求体     | LoginPayload             | 登录请求参数             |
| 请求体     | RefreshTokenPayload      | 刷新令牌请求参数         |
| 响应体     | LoginData                | 登录成功返回数据         |
| 响应体     | RefreshTokenData         | 刷新令牌返回数据         |
| 内部模型   | UserProfile              | 用户档案（业务内部使用） |
| 内部模型   | TokenPair                | 令牌对（业务内部使用）   |
| 管理员请求 | AdminRegisterUserPayload | 注册用户请求             |
| 管理员响应 | AdminRegisteredUserData  | 注册用户返回             |

### 序列化约定

所有面向前端的结构体使用 #[serde(rename_all = "camelCase")]：

| Rust 字段     | JSON 字段 (前端) |
| ------------- | ---------------- |
| access_token  | accessToken      |
| refresh_token | refreshToken     |
| user_id       | userId           |

---

## JWT 机制详解

### 令牌类型

| 令牌类型      | 有效期 | 用途         |
| ------------- | ------ | ------------ |
| access_token  | 2 小时 | API 请求认证 |
| refresh_token | 7 天   | 令牌刷新     |

### JWT 载荷结构

```json
{
  "sub": "admin",
  "token_type": "access",
  "iat": 1704067200,
  "exp": 1704074400
}
```

### 安全特性

1. HS256 算法：HMAC-SHA256 签名
2. 密钥管理：环境变量 PURE_ADMIN_JWT_SECRET
3. 类型校验：区分 access/refresh 令牌
4. 时间校验：验证过期时间

---

## 前端集成

### 登录示例

```typescript
// 登录
const result = await invoke("auth_login", {
  payload: { username: "admin", password: "admin123" }
});

if (result.success) {
  const { accessToken, refreshToken, roles, permissions } = result.data;
  localStorage.setItem("token", accessToken);
  localStorage.setItem("refreshToken", refreshToken);
}
```

### 令牌刷新

```typescript
// 定时刷新或在令牌过期前刷新
const refreshToken = localStorage.getItem("refreshToken");
const result = await invoke("auth_refresh_token", {
  payload: { refreshToken }
});

if (result.success) {
  localStorage.setItem("token", result.data.accessToken);
  localStorage.setItem("refreshToken", result.data.refreshToken);
}
```

### 获取动态路由

```typescript
// 登录成功后获取
const result = await invoke("auth_get_async_routes");

if (result.success) {
  router.addRoutes(result.data);
}
```

---

## 扩展开发指南

### 添加新命令流程

1. 定义模型 (models.rs)
   - 新增请求结构体
   - 新增响应结构体

2. 实现业务 (services.rs 或 admin_services.rs)
   - 编写纯业务函数
   - 添加详细文档注释

3. 暴露命令 (commands.rs 或 admin_commands.rs)
   - 编写 #[tauri::command] 函数
   - 添加参数校验
   - 编写单元测试

4. 注册命令 (lib.rs)
   - 在 invoke_handler! 中注册

### 示例：添加用户信息接口

```rust
// 1. models.rs - 添加模型
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoData {
    pub username: String,
    pub nickname: String,
    pub avatar: String,
}

// 2. services.rs - 添加业务函数
pub fn get_user_info(username: &str) -> Result<UserInfoData, AppError> {
    // 实现...
}

// 3. commands.rs - 添加命令
#[tauri::command]
pub fn auth_get_user_info(username: String) -> AppResult<UserInfoData> {
    // 校验...
    Ok(ApiResponse::ok(services::get_user_info(&username)?))
}
```

---

## 测试策略

### 测试覆盖

- 参数校验：空值、格式验证
- 业务逻辑：用户查询、令牌生成
- 管理员操作：用户增删改查

### 运行测试

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib auth
```

---

## 常见问题

### Q1: 如何修改令牌有效期？

在 services.rs 中修改常量：

```rust
const ACCESS_TOKEN_LIFETIME_SECONDS: u64 = 2 * 60 * 60;  // 2小时
const REFRESH_TOKEN_LIFETIME_SECONDS: u64 = 7 * 24 * 60 * 60;  // 7天
```

### Q2: 如何修改 JWT 密钥？

设置环境变量：

```bash
# Linux/macOS
export PURE_ADMIN_JWT_SECRET="your-secret-key"

# Windows PowerShell
$env:PURE_ADMIN_JWT_SECRET="your-secret-key"
```

### Q3: 如何添加新的用户角色？

1. 在数据库 user_roles 表插入新角色
2. 在 route_roles 表分配角色到路由

### Q4: 如何排查登录失败？

1. 检查数据库用户是否存在
2. 验证密码是否匹配
3. 查看 Rust 控制台错误日志

---

## 相关文档

- 数据库迁移脚本：../db/migrations/README.md
- Tauri 框架约束：../../docs/tauri-framework-constraints.md
- 项目开发进度：../../docs/development-progress.md
