# 数据库迁移脚本开发者指南

> 本文档详细介绍 `src-tauri/src/db/migrations/` 文件夹中的 SQL 迁移脚本，帮助开发者理解数据库结构设计和扩展开发。

## 目录

- [概述](#概述)
- [文件清单](#文件清单)
- [迁移脚本详解](#迁移脚本详解)
  - [0001_schema.sql - 表结构初始化](#0001_schema-sql---表结构初始化)
  - [0002_seed.sql - 初始种子数据](#0002_seed-sql---初始种子数据)
  - [0003_legacy_offline_cleanup.sql - 遗留数据清理](#0003_legacy_offline_cleanupsql---遗留数据清理)
- [数据库架构图](#数据库架构图)
- [开发指南](#开发指南)
  - [添加新表](#添加新表)
  - [修改现有表结构](#修改现有表结构)
  - [添加新迁移](#添加新迁移)
- [离线安全设计原则](#离线安全设计原则)
- [常见问题](#常见问题)

---

## 概述

本项目的数据库采用 **SQLite** 作为嵌入式存储，通过 Rust 端的迁移脚本在应用首次启动时自动初始化。迁移脚本存放在 `migrations/` 目录下，采用**顺序编号**的命名方式确保执行顺序。

### 设计目标

1. **离线优先**：所有数据采用本地存储，无外部网络依赖
2. **自动迁移**：应用启动时自动执行迁移，无需手动干预
3. **幂等性**：迁移脚本可安全重复执行，不会造成数据损坏
4. **可扩展**：支持后续业务功能扩展

---

## 文件清单

| 文件名                            | 用途               | 执行时机           |
| --------------------------------- | ------------------ | ------------------ |
| `0001_schema.sql`                 | 创建所有数据表结构 | 首次启动           |
| `0002_seed.sql`                   | 插入初始种子数据   | 首次启动           |
| `0003_legacy_offline_cleanup.sql` | 清理遗留外链数据   | 首次启动（一次性） |

---

## 迁移脚本详解

### 0001_schema.sql - 表结构初始化

本文件创建了系统的核心数据表，涵盖以下领域：

#### 1. 用户与鉴权体系

| 表名               | 说明                | 关键字段                                                        |
| ------------------ | ------------------- | --------------------------------------------------------------- |
| `users`            | 用户基本信息        | `username`(唯一), `password`, `nickname`, `avatar`, `is_active` |
| `user_roles`       | 用户-角色多对多关联 | `user_id`, `role`                                               |
| `permissions`      | 权限定义            | `code`(唯一)                                                    |
| `user_permissions` | 用户-权限直接关联   | `user_id`, `permission_id`                                      |

#### 2. 动态路由体系

| 表名          | 说明                     | 关键字段                                           |
| ------------- | ------------------------ | -------------------------------------------------- |
| `routes`      | 前端路由配置（树形结构） | `parent_id`(自引用), `path`, `component`, `meta_*` |
| `route_roles` | 路由-角色访问控制        | `route_id`, `role`                                 |
| `route_auths` | 路由-操作权限控制        | `route_id`, `auth`                                 |

#### 3. 设备管理（业务扩展）

| 表名              | 说明             | 关键字段                                                            |
| ----------------- | ---------------- | ------------------------------------------------------------------- |
| `device_registry` | 物联网设备注册表 | `device_id`(唯一), `device_name`, `owner_username`, `registered_at` |

#### 外键策略

所有关联表均使用 `ON DELETE CASCADE`，确保：

- 删除用户时自动清理关联的角色和权限
- 删除路由时自动清理子路由和权限关联
- 数据一致性由数据库层自动维护

---

### 0002_seed.sql - 初始种子数据

本文件为系统注入初始数据，包含以下内容：

#### 1. 默认用户

| 用户名   | 密码       | 角色   | 权限               |
| -------- | ---------- | ------ | ------------------ |
| `admin`  | `admin123` | admin  | 全部权限 (`*:*:*`) |
| `common` | `admin123` | common | btn:add, btn:edit  |

#### 2. 权限定义

```sql
*:*:*                     -- 超级管理员全部权限
permission:btn:add       -- 按钮新增权限
permission:btn:edit      -- 按钮编辑权限
permission:btn:delete    -- 按钮删除权限
```

#### 3. 示例路由结构

```
权限管理 (/permission)
├── 用户注册管理 (/permission/page/index)
└── 按钮权限 (/permission/button)
    ├── 路由返回按钮权限 (/permission/button/router)
    └── 登录接口返回按钮权限 (/permission/button/login)
```

#### 4. 设备注册

预注册了一个开发设备 `device-localhost-001`，用于演示设备管理功能。

---

### 0003_legacy_offline_cleanup.sql - 遗留数据清理

本脚本用于从旧版本（纯前端网络版）升级时的数据清洗：

#### 清理目标

1. **用户头像外链**：将 `http://`、`https://`、`//` 开头的头像 URL 清空
2. **路由图标外链**：将包含冒号的 Iconify 图标格式和外链图标设置为 NULL

#### 执行条件

- 仅在首次启动时执行一次
- 通过 `app_migrations` 表记录执行状态
- 具有幂等性，重复执行不会造成数据丢失

---

## 数据库架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              users                                      │
│  id | username | password | nickname | avatar | is_active            │
└─────────────────────────────────────────────────────────────────────────┘
         │                              │
         │ 1:N                          │ N:M
         ▼                              ▼
┌──────────────────┐          ┌─────────────────────────┐
│    user_roles    │          │   user_permissions      │
│ user_id |  role  │          │ user_id | permission_id │
└──────────────────┘          └─────────────────────────┘
                                       │
                                       │ N:1
                                       ▼
                              ┌───────────────┐
                              │  permissions  │
                              │ id |   code   │
                              └───────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                              routes                                     │
│  id | parent_id | path | name | component | meta_title | meta_icon   │
└─────────────────────────────────────────────────────────────────────────┘
         │                              │
         │ 1:N                          │ N:M
         ▼                              ▼
┌──────────────────┐          ┌─────────────────┐
│    route_roles   │          │   route_auths   │
│ route_id |  role │          │ route_id | auth │
└──────────────────┘          └─────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                         device_registry                                 │
│  id | device_id | device_name | owner_username | registered_at        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 开发指南

### 添加新表

1. 在 `0001_schema.sql` 中添加 `CREATE TABLE` 语句
2. 使用 `IF NOT EXISTS` 确保幂等性
3. 合理设计外键关联，使用 `ON DELETE CASCADE`

**示例：添加系统配置表**

```sql
CREATE TABLE IF NOT EXISTS system_config (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  key TEXT NOT NULL UNIQUE,      -- 配置键
  value TEXT NOT NULL,            -- 配置值
  description TEXT,              -- 说明
  created_at INTEGER NOT NULL,   -- 创建时间
  updated_at INTEGER NOT NULL    -- 更新时间
);
```

---

### 修改现有表结构

**不要直接修改已有表结构**，而是创建新的迁移脚本。

1. 创建新文件 `0004_xxx.sql`
2. 使用 `ALTER TABLE` 添加新字段（SQLite 支持有限）
3. 如需复杂变更，可使用 `CREATE TABLE ... AS` 重建表

**示例：添加配置表种子数据**

```sql
-- 0004_seed_system_config.sql

-- 添加系统配置
INSERT OR IGNORE INTO system_config (id, key, value, description, created_at, updated_at) VALUES
  (1, 'app_name', '能源管理系统', '应用名称', 1772150000000, 1772150000000),
  (2, 'version', '1.0.0', '系统版本', 1772150000000, 1772150000000);
```

---

### 添加新迁移

#### 步骤 1：创建迁移文件

在 `migrations/` 目录下创建新文件，命名格式：`000{N+1}_{描述}.sql`

#### 步骤 2：在 Rust 端注册

编辑 `src-tauri/src/db/mod.rs`，在迁移列表中添加新文件：

```rust
// 示例代码
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/0001_schema.sql"),
    include_str!("migrations/0002_seed.sql"),
    include_str!("migrations/0003_legacy_offline_cleanup.sql"),
    // 添加新迁移
    include_str!("migrations/0004_xxx.sql"),
];
```

#### 步骤 3：执行验证

运行应用，检查数据库是否正确创建：

```bash
pnpm tauri:dev
```

---

## 离线安全设计原则

为确保系统在纯内网环境下正常运行，遵循以下设计原则：

### 1. 头像与图片

- 用户头像存储为空字符串，触发前端使用本地默认头像
- 禁止在数据库中存储外链 URL

### 2. 菜单图标

- 使用本地 Iconify 图标（需预先打包到前端）
- 禁止使用 CDN 图标

### 3. 静态资源

- 所有前端资源通过 Vite 打包到应用内
- 无需外部网络请求

### 4. 数据清理

- `0003_legacy_offline_cleanup.sql` 确保旧数据外链被清理

---

## 常见问题

### Q1: 如何重置数据库？

删除 SQLite 数据库文件，应用重启后会自动重新创建：

- Windows: `%APPDATA%\com.pureadmin.thin\db\pure-admin-thin.sqlite3`
- 开发环境: `<项目目录>/db/pure-admin-thin.sqlite3`

### Q2: 如何查看当前数据库状态？

使用 SQLite 工具连接数据库：

```bash
sqlite3 pure-admin-thin.sqlite3
```

常用命令：

```sql
-- 查看所有表
.tables

-- 查看表结构
.schema users

-- 查看用户数据
SELECT * FROM users;

-- 查看迁移记录
SELECT * FROM app_migrations;
```

### Q3: 迁移脚本执行失败怎么办？

1. 检查 SQLite 版本是否兼容
2. 查看 Rust 控制台错误日志
3. 删除数据库文件重新尝试

### Q4: 如何扩展设备管理功能？

`device_registry` 表是业务扩展的基石，可添加以下字段：

```sql
-- 扩展设备表
ALTER TABLE device_registry ADD COLUMN device_type TEXT;
ALTER TABLE device_registry ADD COLUMN firmware_version TEXT;
ALTER TABLE device_registry ADD COLUMN last_heartbeat INTEGER;
```

---

## 相关文档

- [数据库模块 README](../README.md) - 数据库模块整体介绍
- [Tauri 框架约束](../docs/tauri-framework-constraints.md) - Tauri 开发规范
- [开发进度文档](../docs/development-progress.md) - 项目进度跟踪
