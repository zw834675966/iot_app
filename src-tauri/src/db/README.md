# 数据库模块 (PostgreSQL) - 开发者指南

> 本文档详细介绍 `src-tauri/src/db` 数据库模块的设计、架构和使用指南。

## 目录

- [模块概述](#模块概述)
- [目录结构](#目录结构)
- [核心职责](#核心职责)
- [数据访问层（Repository）](#数据访问层repository)
- [初始化与迁移机制](#初始化与迁移机制)
- [管理员功能](#管理员功能)
- [开发规范](#开发规范)
- [相关文档](#相关文档)

---

## 模块概述

本模块负责管理应用的 PostgreSQL 数据库基础设施，为能源管理桌面端提供数据持久化支持。采用**仓储模式（Repository Pattern）**封装数据访问逻辑，为上层服务提供清晰的数据接口。

### 设计目标

1. **类型安全**：使用 Rust 强类型系统封装查询结果
2. **双 ORM 支持**：同时支持 SeaORM（ORM）和 SQLx（原生 SQL）
3. **解耦设计**：业务逻辑与数据访问完全分离
4. **可扩展**：支持后续业务功能扩展

---

## 目录结构

```
src-tauri/src/db/
├── mod.rs                          # 数据库连接管理、初始化、迁移入口
├── path_store.rs                   # 数据库 URL 存储管理
├── bootstrap.rs                    # 数据库初始化引导
├── migrations.rs                   # 数据库迁移管理
├── auth_repository.rs              # 鉴权数据查询仓储
├── admin_repository.rs              # 管理员数据仓储
│   ├── seaorm_users.rs             # SeaORM 用户管理实现
│   └── sqlx_reports.rs            # SQLx 报表查询实现
├── entities/                       # SeaORM 实体模型
│   ├── mod.rs                     # 实体模块入口
│   ├── prelude.rs                  # 实体预导出
│   ├── users.rs                    # 用户实体
│   └── user_roles.rs              # 用户角色关联实体
├── migrations/                     # SQL 迁移脚本目录
│   ├── 0001_schema.sql            # 表结构初始化
│   ├── 0002_seed.sql              # 初始种子数据
│   ├── 0003_legacy_offline_cleanup.sql      # 遗留数据清理
│   ├── 0004_user_registration_extension.sql # 用户注册扩展
│   ├── 0005_permission_page_to_user_registration.sql # 路由重命名
│   └── 0006_hide_button_permission_route.sql # 隐藏按钮权限
└── tests.rs                        # 数据库测试模块
```

### 各文件职责

| 文件                  | 职责                                     |
| --------------------- | ---------------------------------------- |
| `mod.rs`              | 数据库连接管理、URL 配置、迁移执行入口   |
| `path_store.rs`       | 数据库连接 URL 的存储和获取              |
| `bootstrap.rs`        | 数据库初始化引导（获取锁、执行迁移）     |
| `migrations.rs`       | 迁移状态管理、执行逻辑                   |
| `auth_repository.rs`  | 鉴权相关的数据查询（用户档案、动态路由） |
| `admin_repository.rs` | 管理员数据仓储（用户 CRUD、账号管理）    |
| `entities/*.rs`       | SeaORM 实体定义                          |
| `migrations/*.sql`    | 表结构和数据的 SQL 脚本                  |

---

## 核心职责

### 1. 用户档案与权限

提供针对以下表的关联查询：

- `users` - 用户基本信息
- `user_roles` - 用户角色关联
- `user_permissions` - 用户权限关联
- `permissions` - 权限定义

### 2. 动态路由

基于以下表构建适配前端的树状路由配置：

- `routes` - 路由定义（树形结构）
- `route_roles` - 路由角色访问控制
- `route_auths` - 路由操作权限控制

### 3. 设备注册表

通过 `device_registry` 表管理硬件设备信息，作为业务扩展的基石。

---

## 数据访问层（Repository）

### 1. auth_repository.rs 模块

提供用户认证相关的数据查询功能：

| 函数                | 功能说明                   | 返回类型                                |
| ------------------- | -------------------------- | --------------------------------------- |
| `find_user_profile` | 根据用户名密码查询用户档案 | `Result<Option<UserProfile>, AppError>` |
| `find_async_routes` | 查询并构建动态路由树       | `Result<Vec<Value>, AppError>`          |

### 2. admin_repository.rs 模块

提供管理员功能的数据访问接口：

| 函数                       | 功能说明         |
| -------------------------- | ---------------- |
| `create_user`              | 创建新用户       |
| `update_user`              | 更新用户信息     |
| `delete_user`              | 删除用户         |
| `renew_user_account`       | 续期用户账号     |
| `list_users`               | 获取用户列表     |
| `is_admin_user`            | 检查是否为管理员 |
| `deactivate_expired_users` | 停用过期用户     |

### 查询流程

```
services.rs 调用
       │
       ▼
auth_repository.rs / admin_repository.rs
       │
       ├── 连接数据库（connect_async / connect_orm_async）
       ├── SQL 查询（多表关联）
       ├── 映射结果到结构体
       └── 返回结果
```

### 内部数据结构

#### RouteRow（路由行数据）

从数据库查询的原始数据结构，包含：

- 路由基本信息（id, path, name, component）
- 路由元数据（title, icon, rank）
- 关联的角色和权限列表

#### RouteNode（路由节点）

内存中构建树形结构的数据，包含：

- RouteRow 的所有字段
- `children: Vec<RouteNode>` - 子路由列表

---

## 初始化与迁移机制

### 迁移执行流程

```
应用启动
    │
    ▼
db::init_database()
    │
    ├── 1. 建立数据库连接
    │
    ├── 2. 获取 PostgreSQL 咨询锁（防止并发初始化）
    │
    ├── 3. 执行迁移
    │    ├── init_schema (0001_schema.sql)
    │    ├── init_seed_data (0002_seed.sql)
    │    ├── apply_one_time_data_fix (0003)
    │    ├── apply_user_registration_extension (0004)
    │    ├── apply_permission_route_rename (0005)
    │    └── apply_hide_button_permission_route (0006)
    │
    ├── 4. 释放咨询锁
    │
    └── 5. 返回初始化结果
```

### 迁移设计原则

- **幂等性**：所有迁移脚本可安全重复执行
- **版本记录**：每次执行记录到 `app_migrations` 表
- **咨询锁**：使用 PostgreSQL 咨询锁防止并发初始化

---

## 管理员功能

### 用户账号生命周期

- **永久账号**：`account_is_permanent = 1`，无过期时间
- **有时限账号**：`account_is_permanent = 0`，设置 `account_expire_at` 过期时间

### 账号自动停用

系统启动时自动检查并停用已过期的用户账号：

```rust
// 停用条件
is_active = 1 AND account_is_permanent = 0 AND account_expire_at IS NOT NULL AND account_expire_at <= now_millis
```

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
       db::block_on(async move {
           let mut connection = db::connect_async().await?;
           // SQL 查询逻辑
       })
   }
   ```

3. **导出模块**
   在 `mod.rs` 中添加 `pub mod new_feature_repository;`

### 双 ORM 选择

- **SeaORM**：适合 CRUD 操作、实体关联查询
- **SQLx**：适合复杂聚合查询、报表数据提取

### SQL 编写规范

- 使用参数化查询（防止 SQL 注入）
- 使用 `LEFT JOIN` 处理可选关联
- 使用 `COALESCE` 处理 NULL 值
- 使用 `STRING_AGG` 聚合多对多关系

### 错误处理

- 所有数据库错误封装为 `AppError::Database`
- 使用 `?` 操作符链式传递错误
- 避免暴露原始数据库错误信息给前端

---

## 相关文档

- [鉴权模块](../auth/README.md)
- [数据库迁移脚本](migrations/README.md)
- [Tauri 框架约束](../../docs/tauri-framework-constraints.md)
- [项目开发进度](../../docs/development-progress.md)
