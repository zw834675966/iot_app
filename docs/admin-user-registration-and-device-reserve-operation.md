# Admin 用户注册与设备配置预留 - 操作文档（确认版）

## 1. 目标与范围

本方案用于满足以下页面逻辑优化目标（本阶段仅出文档，不改代码）：

1. 在 `admin`（管理员）权限下提供用户注册能力。
2. 注册字段包含：
   - 账号
   - 密码
   - 电话
   - 名称
   - 权限（`operator`/`tenant`/`maintainer`）
   - 账号期限（天数，到期自动注销，支持永久）
3. 用户页面增加“设备配置”入口（区域/楼层/设备，可全选），本期仅保留接口，待后续开发。

约束：
- 本期必须确保“已完成注册的数据”落库到 SQLite。
- 尽量复用当前数据库结构，减少侵入式改造与回滚成本。

---

## 2. 当前基线（已存在）

当前数据库（`pure-admin-thin.sqlite3`）已具备：

- 用户与鉴权
  - `users`
  - `user_roles`
  - `permissions`
  - `user_permissions`
- 菜单与路由权限
  - `routes`
  - `route_roles`
  - `route_auths`
- 设备基础表
  - `device_registry`

当前鉴权流程：
- 登录：`auth_login`（按 `users.username/password/is_active` 查询）
- 刷新：`auth_refresh_token`
- 异步路由：`auth_get_async_routes`

结论：
- 可直接复用 `users + user_roles` 实现注册与角色分配。
- 需扩展 `users` 字段以承载电话与账号期限。
- 设备配置“预留接口”建议先定义数据模型与命令契约，不实现业务写入。

---

## 3. 目标方案（最小侵入 + 可演进）

## 3.1 用户注册（Admin-only）

建议新增管理员命令（命名可在实现阶段微调）：

- `auth_admin_register_user`
- `auth_admin_list_users`（可选，便于管理页展示）
- `auth_admin_disable_expired_users`（可选，手动触发补偿）

注册字段映射（已确认）：

- 账号 -> `users.username`
- 密码 -> `users.password`（短期兼容现状；中期升级哈希）
- 电话 -> `users.phone`（新增，可选）
- 名称 -> `users.nickname`（复用）
- 权限 -> `user_roles.role`
  - 枚举值：`operator` / `tenant` / `maintainer`
  - 选择方式：多选（一个用户可绑定多个角色）
- 账号期限
  - 永久：`account_is_permanent = 1`
  - 非永久：保存 `account_valid_days` 与 `account_expire_at`

## 3.2 到期自动注销策略（已确认）

“自动注销”采用“惰性 + 启动补偿”双策略：

1. 惰性检查（强制）：
   - 登录前检查过期状态；
   - 刷新 token 前检查过期状态。
2. 启动补偿（建议）：
   - 应用启动后执行一次批量失效：将已过期用户 `is_active` 置 `0`。

说明：
- 现有架构中 access token 为无状态 JWT；若用户过期但 token 仍在有效期内，需在后续命令鉴权中补齐“用户状态二次校验”（实现阶段处理）。

## 3.3 用户设备配置（预留接口）

本期目标是“仅保留接口契约，不落地业务”：

- 前端页面可展示入口与表单框架（区域/楼层/设备、全选）。
- 后端命令保留契约，不实现业务逻辑（返回 `NOT_IMPLEMENTED` 或空结构）。

建议预留命令：

- `user_device_scope_get(userId)`
- `user_device_scope_upsert(payload)`（预留）

接口契约文档：
- `docs/admin-user-device-reserve-api-contract.md`

---

## 4. 数据库变更设计（确认方案）

## 4.1 变更 `users`（推荐）

在 `users` 上新增字段（迁移脚本建议：`0004_user_registration_extension.sql`）：

- `phone TEXT`
- `account_is_permanent INTEGER NOT NULL DEFAULT 0`
- `account_valid_days INTEGER`
- `account_expire_at INTEGER`（Unix 毫秒）
- `created_at INTEGER`
- `updated_at INTEGER`
- `created_by TEXT`（记录注册操作人，如 `admin`）

并增加索引：

- `idx_users_phone`（普通索引，不加唯一约束；电话为可选）
- `idx_users_account_expire_at`

说明：
- 复用 `nickname` 作为“名称”字段，不新增 `real_name`，减少改造面。
- 继续使用 `is_active` 作为启用/注销标志，避免新增状态机复杂度。

## 4.2 角色策略（复用）

不新增角色表，继续使用 `user_roles(role TEXT)`：

- 新增可选角色值：`operator` / `tenant` / `maintainer`
- `admin` 保持高权限，不允许被普通注册流程创建（实现时加校验）

## 4.3 设备配置预留（确认）

本期不新增设备配置相关数据表，仅保留接口契约与 API 文档。

收益：
- 避免占位表长期空置造成维护负担。
- 后续可根据真实业务一次性建模，降低返工风险。

---

## 5. 页面与交互逻辑（实现约束）

## 5.1 注册页（仅 admin 可见）

表单：
- `username`（必填，唯一）
- `password`（必填，长度策略待定）
- `phone`（可选，填了则做格式校验）
- `nickname`（必填）
- `roles`（多选：operator/tenant/maintainer）
- `account_term_type`（永久/按天）
- `account_valid_days`（按天时必填，>0）

校验规则：
- 用户名重复拒绝；
- 电话非空且格式非法时拒绝；
- `account_valid_days` 非法拒绝；
- 非 admin 调用注册接口拒绝。

## 5.2 用户页设备配置（预留）

页面可先展示：
- 区域选择（支持全选）
- 楼层选择（支持全选）
- 设备选择（支持全选）

本期行为：
- 保存按钮提示“功能预留，待开发”；
- 或仅保留 `get` 接口返回空配置，`upsert` 返回预留错误码。

---

## 6. 部署与回滚策略（推荐顺序）

## 6.1 部署顺序（分阶段）

1. 数据库迁移上线：
   - 新增 `users` 扩展字段与索引。
2. 后端命令上线：
   - admin 注册命令；
   - admin 续期命令；
   - 过期失效补偿命令/启动钩子；
   - 设备配置预留命令。
3. 前端页面上线：
   - admin 注册页与用户管理页；
   - 设备配置入口（预留态）。

## 6.2 回滚策略

- 若仅新增字段：
  - 业务回滚可通过禁用新命令与隐藏页面完成；
  - 数据不丢失，旧逻辑可继续工作。
- 若新增表：
  - 表可保留不使用，不建议立即 drop，避免回滚期间数据结构抖动。

---

## 7. 验收标准（本期）

必须满足：

1. `admin` 可成功注册用户，字段完整落库。
2. 可多选分配角色：`operator`/`tenant`/`maintainer`。
3. 非永久账号到期后不可登录（并可标记为注销/禁用）。
4. 支持续期（更新有效期）并落库。
5. 设备配置页面有入口与预留接口定义，并提供独立 API 契约文档，不影响现有功能。
6. 现有登录、刷新、动态路由逻辑不回归。

---

## 8. 已确认决策

1. 角色：多选。
2. 电话：可选，不做唯一约束。
3. 账号期限：允许续期。
4. 自动注销机制：采用“惰性 + 启动补偿”。
5. 设备配置：仅保留接口契约，并提供单独 API 文档。

技术风险提示：

- 当前密码字段为明文兼容模式；建议二期升级为哈希存储（如 Argon2）。
- `sqlite_tools` MCP 在本会话握手失败，SQL 最终落地前需在可用会话再次执行 MCP 校验。

---

## 9. 本文档执行结论

- 本文档已给出可执行的“先数据库、后命令、再页面”的最小风险部署路径。
- 当前阶段未修改任何业务代码，仅输出评审材料。
- 已具备进入代码实施与迁移脚本落地条件。
