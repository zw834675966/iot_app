# 数据库迁移脚本 (Migrations) 开发者指南

> 本文档详细介绍 `src-tauri/src/db/migrations/` 文件夹中的 SQL 迁移脚本，旨在帮助开发者深入理解当前系统（基于 PostgreSQL 17）的数据库结构设计、数据演进历程及扩展规范。

## 目录

- [概述](#概述)
- [文件清单](#文件清单)
- [迁移脚本详解](#迁移脚本详解)
  - [0001_schema.sql - 表结构初始化](#0001_schema-sql---表结构初始化)
  - [0002_seed.sql - 初始种子数据](#0002_seed-sql---初始种子数据)
  - [0003_legacy_offline_cleanup.sql - 遗留数据清理](#0003_legacy_offline_cleanupsql---遗留数据清理)
  - [0004_user_registration_extension.sql - 用户注册与生命周期扩展](#0004_user_registration_extensionsql---用户注册与生命周期扩展)
  - [0005_permission_page_to_user_registration.sql - 路由节点调整](#0005_permission_page_to_user_registrationsql---路由节点调整)
  - [0006_hide_button_permission_route.sql - 清理冗余功能](#0006_hide_button_permission_routesql---清理冗余功能)
- [数据库架构图](#数据库架构图)
- [开发指南](#开发指南)
  - [迁移命名与注册规范](#迁移命名与注册规范)
  - [如何安全地添加新表/新字段](#如何安全地添加新表新字段)
- [离线环境与安全设计原则](#离线环境与安全设计原则)

---

## 概述

本项目由于业务复杂度和性能要求，底座数据库已从 SQLite 升级为 **PostgreSQL 17**。所有初始化建表和后期结构/数据调整，都依赖当前目录下的 SQL 迁移脚本实现。

### 迁移设计目标

1. **版本化与幂等性**：使用编号排序确保执行顺序。对于建表或特定写入必须具有幂等性（利用 `IF NOT EXISTS` 或 `ON CONFLICT DO NOTHING`），确保重启不会导致数据损坏或报错。
2. **代码即文档**：迁移文件内的 SQL 语句全部添加了详尽的逐行/段落级中文注释，为接手的开发者提供了最直接的业务参考。
3. **隔离与自动化**：Rust 后端在连接到 PostgreSQL 实例后，会自动在引导阶段（Bootstrap）获取咨询锁并比对当前迁移状态，自动应用未执行的新脚本。

---

## 文件清单

| 序号 | 文件名                                          | 核心内容/目标                                       |
| ---- | ----------------------------------------------- | --------------------------------------------------- |
| 0001 | `0001_schema.sql`                               | 创建所有基础数据表结构 (含用户、权限、路由、设备等) |
| 0002 | `0002_seed.sql`                                 | 插入系统运行所必须的初始账号、基础路由及RBAC策略    |
| 0003 | `0003_legacy_offline_cleanup.sql`               | (遗留) 将不支持离线环境的外链图片、图标清洗为安全值 |
| 0004 | `0004_user_registration_extension.sql`          | 为用户表 (`users`) 添加手机号、账号有效期等扩展列   |
| 0005 | `0005_permission_page_to_user_registration.sql` | 修改数据库中的硬编码路由名称以贴合最新业务场景      |
| 0006 | `0006_hide_button_permission_route.sql`         | 移除不需要的前端演示级权限验证子菜单                |

---

## 迁移脚本详解

### 0001_schema.sql - 表结构初始化

基于 PostgreSQL 17，搭建了整个应用关系型数据库的地基。

#### 1. 用户与鉴权体系

核心包含了用户基础信息以及与其绑定的角色、权限。

- **`users`**: 用户基本信息表。主键为 `BIGSERIAL`，包含 `username`（唯一）、`password`、`nickname` 和 `avatar` 等。
- **`permissions`**: 系统的可用操作权限表，`code` 保持唯一标识。
- **`user_roles`** 与 **`user_permissions`**: 多对多关系连接表，通过联合主键防止重复关联。外键全配置为 `ON DELETE CASCADE`，保证删除用户时关联干净。
- **`casbin_rule`**: 后端 RBAC 引擎（Casbin）的策略存储表。涵盖了 `ptype`, `v0`~`v5` 的组合存储。

#### 2. 动态路由体系

前端菜单与页面权限直接由数据库驱动。

- **`routes`**: 页面路由树表，利用 `parent_id` 形成自引用的树形层级。保存菜单标题（`meta_title`）、图标（`meta_icon`）及前端组件路径等。
- **`route_roles`** 与 **`route_auths`**: 决定了哪些角色可以看见哪些路由，以及进入此路由页面需要具备哪些细粒度权限。

#### 3. 业务扩展示例

- **`device_registry`**: 记录硬件设备的设备 ID、名称和所属用户，展示了如何在此体系中挂载具体的业务实体。

### 0002_seed.sql - 初始种子数据

这是保证系统能够在空白数据库首次启动即可正常运作的基石数据。

- 注入了超级管理员 `admin` 和普通演示账号 `common`（密码皆为 `admin123`）。
- 定义了最基础的按钮级操作权限 (`permission:btn:add`, `edit`, `delete`)。
- 填充了“权限管理”、“用户注册管理”等一套完整的前端菜单（路由表）及角色关联配置。
- **最后特别引入了 `setval` 指令**，将序列值同步到当前最大 ID 以防未来新插入数据发生主键冲突。

### 0003_legacy_offline_cleanup.sql - 遗留数据清理

系统为了适应“纯局域网 / 断网环境”下的内网部署，必须切断对外部网络资源的请求。

- 通过正则风格 `LIKE` 语句查找头像(`avatar`) 或菜单图标(`meta_icon`) 字段中的 `http://`, `https://` 或自适应 `//`。
- 将这些带有外部 CDN 依赖的数据清空或置为 `NULL`。

### 0004_user_registration_extension.sql - 用户注册与生命周期扩展

系统业务迭代时的典型表结构升级脚本。

- **增加字段**: 为 `users` 表通过 `ALTER TABLE ADD COLUMN IF NOT EXISTS` 追加了 `phone`、`account_is_permanent`、`account_valid_days` 等账号有效期与追踪相关字段。
- **补充默认值**: 使用 `COALESCE` 与 `EXTRACT(EPOCH FROM NOW())` 赋予老数据安全的默认值（老账号统筹为永久有效）。
- **优化性能**: 为新增的 `phone` 和 `account_expire_at` 字段增加了索引 `CREATE INDEX`，加速手机号查询与后台定时清理过期账号的任务。

### 0005 & 0006 - 路由树的微调清理

- **0005**：通过特定的 `id` 和 `path` 对准某个路由行，把其英文的抽象概念名称重命名为业务性的 “用户注册管理”。
- **0006**：直接利用 `DELETE` 和 `LIKE` 删除了不需要的示例层级。基于 PostgreSQL 的外键级联删除特性，那些多对多关系中的角色绑定记录也随之安全消失，不会留下孤儿数据。

---

## 数据库架构图

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                              users                                      │
│  id | username | password | nickname | avatar | is_active | created_at... │
└─────────────────────────────────────────────────────────────────────────┘
         │                              │
         │ 1:N                          │ N:M (ON DELETE CASCADE)
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
         │ 1:N                          │ N:M (ON DELETE CASCADE)
         ▼                              ▼
┌──────────────────┐          ┌─────────────────┐
│    route_roles   │          │   route_auths   │
│ route_id |  role │          │ route_id | auth │
└──────────────────┘          └─────────────────┘
```

---

## 开发指南

随着需求增加，修改数据库结构不可避免。为保持系统稳定，请遵循以下规范。

### 迁移命名与注册规范

1. **新建迁移脚本文件**: 必须使用 `000{N}_{下划线分隔的英文描述}.sql` 命名（保证按字母序恰好是你希望的执行顺序）。
2. **在 Rust 代码中注册**: 脚本仅放在文件夹里不会自动生效。请前往 `src-tauri/src/db/mod.rs`（或对应负责拉起迁移的入口文件中），将其引入到执行数组内：
   ```rust
   // 例如
   const MIGRATIONS: &[&str] = &[
       include_str!("migrations/0001_schema.sql"),
       include_str!("migrations/0002_seed.sql"),
       // ...
       include_str!("migrations/0007_new_feature_added.sql"),
   ];
   ```

### 如何安全地添加新表/新字段

**严禁**直接回退修改 `0001_schema.sql` (这会导致已上线的旧版本节点不更新)。
所有变动必须是以往后追加新脚本文件 (如 `0007_xxx.sql`) 的形式出现。

- **新建表**时：一定要带上 `CREATE TABLE IF NOT EXISTS`。
- **新增字段**时：PostgreSQL 支持 `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`。
- **涉及业务逻辑迁移**时：在添加新字段后，紧跟 `UPDATE` 语句给老数据分配恰当的默认值。

---

## 离线环境与安全设计原则

因为项目常部署于安全隔离网络（离线环境）：

1. **外部链接阻断**：永远不向数据库中写入例如 Google Fonts、外部 CDN Icon 或云存储上的图像 URL，除非你在后续步骤中有独立的镜像缓存代理层。
2. **逻辑软删除与生命周期**：尽可能利用 `is_active` 或 `account_expire_at` 字段做判断限制，而非物理删除，以保留重要操作审计。
3. **安全主键策略**：PostgreSQL 推荐使用 `BIGSERIAL` (对应 Rust `i64`) 作为自增的主键，它有着超长的整数跨度，能抵御极高频次的插拔写入。
