# Database 模块 (SQLite) - 开发者指南

> 本文档详细介绍 `src-tauri/src/db` 数据库模块的设计、架构和使用指南。

## 目录

- [模块概述](#模块概述)
- [目录结构](#目录结构)
- [核心职责](#核心职责)
- [数据访问层（Repository）](#数据访问层repository)
- [初始化与迁移机制](#初始化与迁移机制)
- [离线安全设计](#离线安全设计)
- [遗留数据兼容](#遗留数据兼容)
- [开发规范](#开发规范)
- [常见问题](#常见问题)

---

## 模块概述

本模块负责管理应用的本地 SQLite 数据库基础设施，为能源管理桌面端提供离线数据支撑。采用**仓储模式（Repository Pattern）**封装数据访问逻辑，为上层服务提供清晰的数据接口。

### 设计目标

1. **离线优先**：所有数据存储在本地 SQLite，无需外部网络依赖
2. **类型安全**：使用 Rust 强类型系统封装查询结果
3. **解耦设计**：业务逻辑与数据访问完全分离
4. **可扩展**：支持后续业务功能扩展

---

## 目录结构

```
src-tauri/src/db/
├── mod.rs                     # 数据库连接管理、初始化、迁移入口
├── auth_repository.rs         # 鉴权域数据查询仓储
└── migrations/                # SQL 迁移脚本目录
    ├── 0001_schema.sql       # 表结构初始化
    ├── 0002_seed.sql         # 初始种子数据
    ├── 0003_legacy_offline_cleanup.sql  # 遗留数据清理
    └── 0004_user_registration_extension.sql # 用户注册与账号周期字段扩展
```

### 各文件职责

| 文件                 | 职责                                     |
| -------------------- | ---------------------------------------- |
| `mod.rs`             | 数据库连接管理、路径配置、迁移执行       |
| `auth_repository.rs` | 鉴权相关的数据查询（用户档案、动态路由） |
| `migrations/*.sql`   | 表结构和数据的 SQL 脚本                  |

---

## 核心职责

### 1. 用户档案与权限

提供针对以下表的关联查询：

- `users` - 用户基本信息
- `user_roles` - 用户角色关联
- `user_permissions` - 用户权限关联
- `permissions` - 权限定义

### 2. 异步路由

基于以下表构建适配前端的树状路由配置：

- `routes` - 路由定义（树形结构）
- `route_roles` - 路由角色访问控制
- `route_auths` - 路由操作权限控制

### 3. 设备注册表

通过 `device_registry` 表管理硬件设备信息，作为业务扩展的基石。

---

## 数据访问层（Repository）

### auth_repository.rs 模块

本模块采用**仓储模式**封装所有数据访问逻辑，是数据访问层（DAL）的核心组件。

#### 主要功能

| 函数                | 功能说明                   | 返回类型                                |
| ------------------- | -------------------------- | --------------------------------------- |
| `find_user_profile` | 根据用户名密码查询用户档案 | `Result<Option<UserProfile>, AppError>` |
| `find_async_routes` | 查询并构建动态路由树       | `Result<Vec<Value>, AppError>`          |

#### 查询流程

```
services.rs 调用
       │
       ▼
auth_repository.rs
       │
       ├── find_user_profile()
       │    │
       │    ├── 连接数据库
       │    ├── SQL 查询（多表关联）
       │    ├── 映射结果到结构体
       │    └── 返回用户档案
       │
       └── find_async_routes()
            │
            ├── 连接数据库
            ├── SQL 查询（多表关联）
            ├── 构建 HashMap 分组
            ├── 递归组装树形结构
            ├── 转换为 JSON
            └── 返回路由数组
```

#### 内部数据结构

##### RouteRow（路由行数据）

从数据库查询的原始数据结构，包含：

- 路由基本信息（id, path, name, component）
- 路由元数据（title, icon, rank）
- 关联的角色和权限列表

##### RouteNode（路由节点）

内存中构建树形结构的数据，包含：

- RouteRow 的所有字段
- `children: Vec<RouteNode>` - 子路由列表

#### 关键算法

##### 1. 字符串拆分 (split_csv)

将 SQL `GROUP_CONCAT` 返回的逗号分隔字符串转换为向量：

```rust
split_csv("admin,common,user")  // → ["admin", "common", "user"]
```

##### 2. 树形组装 (assemble_route_tree)

递归构建父子路由树形结构：

```text
parent_id = None (根节点)
    │
    ├── 路由1
    │    ├── 路由1.1
    │    │    └── 路由1.1.1
    │    └── 路由1.2
    │
    └── 路由2
         └── 路由2.1
```

##### 3. JSON 转换 (route_to_json)

将树形结构转换为 vue-router 兼容的 JSON 格式：

```json
{
  "path": "/permission",
  "meta": {
    "title": "权限管理",
    "icon": "ri/information-line",
    "roles": ["admin"]
  },
  "children": [...]
}
```

---

## 初始化与迁移机制

### 迁移执行流程

```
应用启动
    │
    ▼
db::init_database()
    │
    ├── 1. 创建 app_migrations 表（如不存在）
    │
    ├── 2. 读取 migrations/ 目录的 SQL 文件
    │    (使用 include_str! 宏静态编译)
    │
    ├── 3. 检查 app_migrations 表记录
    │
    ├── 4. 执行未执行的迁移脚本
    │    ├── 0001_schema.sql (表结构)
    │    ├── 0002_seed.sql (种子数据)
    │    ├── 0003_legacy_offline_cleanup.sql (清理)
    │    └── 0004_user_registration_extension.sql (注册扩展)
    │
    └── 5. 记录迁移执行状态
```

### 迁移设计原则

- **幂等性**：所有迁移脚本可安全重复执行
- **顺序执行**：按文件名数字顺序执行（0001, 0002, ...）
- **版本记录**：每次执行记录到 `app_migrations` 表

---

## 离线安全设计

### 核心原则

所有初始数据采用**离线安全（Offline-safe）**设计：

1. **用户头像**
   - 默认置为空字符串
   - 触发前端使用本地默认头像
   - 禁止存储外链 URL

2. **菜单图标**
   - 使用本地 Iconify 图标
   - 禁止使用 CDN 图标
   - 离线环境可用

3. **静态资源**
   - 所有资源打包到应用内
   - 无需外部网络请求

---

## 遗留数据兼容

### 清理脚本 (0003_legacy_offline_cleanup.sql)

为确保从旧版本升级时数据安全：

| 清理目标 | 清理规则                                    |
| -------- | ------------------------------------------- |
| 用户头像 | 清除 `http://`、`https://`、`//` 开头的 URL |
| 路由图标 | 清除包含 `:` 的 Iconify 格式图标            |

### 执行条件

- 仅在首次启动时执行
- 通过 `app_migrations` 表记录状态

### 注册扩展脚本 (0004_user_registration_extension.sql)

为管理员注册与账号生命周期功能补充用户扩展字段：
- `phone`：手机号（可选）
- `account_is_permanent`：是否永久账号
- `account_valid_days`：有效天数（非永久账号）
- `account_expire_at`：过期时间戳（毫秒）
- `created_at` / `updated_at` / `created_by`：审计字段

并新增索引：
- `idx_users_phone`
- `idx_users_account_expire_at`
- 重复执行不会造成数据丢失

---

## 开发规范

### 添加新查询

1. **创建新仓储文件**

   ```
   src-tauri/src/db/new_feature_repository.rs
   ```

2. **定义查询函数**

   ```rust
   pub fn find_xxx(...) -> Result<..., AppError> {
       let connection = db::connect()?;
       // SQL 查询逻辑
   }
   ```

3. **导出模块**
   在 `mod.rs` 中添加 `pub mod new_feature_repository;`

### SQL 编写规范

- 使用参数化查询（防止 SQL 注入）
- 使用 `LEFT JOIN` 处理可选关联
- 使用 `COALESCE` 处理 NULL 值
- 使用 `GROUP_CONCAT` 聚合多对多关系

### 错误处理

- 所有数据库错误封装为 `AppError::Database`
- 使用 `?` 操作符链式传递错误
- 避免暴露原始数据库错误信息给前端

---

## 常见问题

### Q1: 如何添加新的数据表？

1. 创建新的迁移脚本 `migrations/0004_xxx.sql`
2. 在迁移脚本中定义表结构
3. 应用启动时自动执行

### Q2: 如何查询自定义数据？

1. 在 `auth_repository.rs` 或新建 repository 文件
2. 编写 SQL 查询函数
3. 返回强类型结果

### Q3: 如何排查数据库问题？

1. 检查 SQLite 数据库文件是否存在
2. 使用 `sqlite3` 工具直接查询
3. 查看 Rust 控制台日志

### Q4: 如何重置数据库？

删除数据库文件，应用重启后自动重建：

- Windows: `%APPDATA%\com.pureadmin.thin\db\pure-admin-thin.sqlite3`

---

## 相关文档

- [鉴权模块](../auth/README.md)
- [数据库迁移脚本](migrations/README.md)
- [Tauri 框架约束](../../docs/tauri-framework-constraints.md)
- [项目开发进度](../../docs/development-progress.md)
