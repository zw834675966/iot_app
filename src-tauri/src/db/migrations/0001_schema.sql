-- PostgreSQL 17 schema bootstrap
-- PostgreSQL 17 数据库结构初始化脚本

-- 创建用户基本信息表，存储用户的登录名、密码、昵称等基础信息
CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,            -- 自增主键 ID
  username TEXT NOT NULL UNIQUE,       -- 登录用户名，不允许为空且必须唯一
  password TEXT NOT NULL,              -- 登录密码，通常存储加密后的哈希值
  nickname TEXT NOT NULL,              -- 用户昵称/显示名称
  avatar TEXT NOT NULL,                -- 用户头像（本地路径或 Base64）
  is_active INTEGER NOT NULL DEFAULT 1 -- 账号状态：1 表示启用，0 表示禁用，默认为启用
);

-- 创建用户-角色多对多关联表，记录用户被分配的角色
CREATE TABLE IF NOT EXISTS user_roles (
  user_id BIGINT NOT NULL,                     -- 关联的用户 ID
  role TEXT NOT NULL,                          -- 关联的角色名称（如 admin, common）
  PRIMARY KEY (user_id, role),                 -- 联合主键，确保一个用户不会重复绑定同一个角色
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE -- 外键约束，删除用户时级联删除其角色关联记录
);

-- 创建系统权限表，定义系统中存在的所有操作权限代码
CREATE TABLE IF NOT EXISTS permissions (
  id BIGSERIAL PRIMARY KEY,                    -- 自增主键 ID
  code TEXT NOT NULL UNIQUE                    -- 权限代码，不允许为空且必须唯一（例如 *:*:* 或 permission:btn:add）
);

-- 创建用户-权限多对多关联表，实现直接向用户分配特定操作权限（跳过角色）
CREATE TABLE IF NOT EXISTS user_permissions (
  user_id BIGINT NOT NULL,                     -- 关联的用户 ID
  permission_id BIGINT NOT NULL,               -- 关联的权限 ID
  PRIMARY KEY (user_id, permission_id),        -- 联合主键，确保一个用户不会重复绑定同一个权限
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE, -- 外键约束，删除用户时级联删除对应的权限关联
  FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE -- 外键约束，删除权限时级联删除对应的用户关联
);

-- 创建前端动态路由配置表，采用树形结构存储应用的页面路由信息
CREATE TABLE IF NOT EXISTS routes (
  id BIGINT PRIMARY KEY,                       -- 路由记录 ID（需手动指定或由程序控制以保持稳定）
  parent_id BIGINT,                            -- 父路由 ID，为 NULL 时表示顶级路由
  path TEXT NOT NULL UNIQUE,                   -- 路由的 URL 路径，必须唯一
  name TEXT,                                   -- 路由名称（通常与前端 Vue 组件的 name 属性一致）
  component TEXT,                              -- 路由对应的组件物理路径
  meta_title TEXT NOT NULL,                    -- 路由元数据：菜单中显示的标题
  meta_icon TEXT,                              -- 路由元数据：菜单中显示的图标
  meta_rank INTEGER,                           -- 路由元数据：决定菜单显示顺序的排序权重
  FOREIGN KEY (parent_id) REFERENCES routes(id) ON DELETE CASCADE -- 外键自引用约束，实现树形结构，删除父路由时级联删除所有子孙路由
);

-- 创建路由-角色关联表，实现前端页面的粗粒度访问控制（页面级鉴权）
CREATE TABLE IF NOT EXISTS route_roles (
  route_id BIGINT NOT NULL,                    -- 关联的路由 ID
  role TEXT NOT NULL,                          -- 允许访问该路由的角色名称
  PRIMARY KEY (route_id, role),                -- 联合主键
  FOREIGN KEY (route_id) REFERENCES routes(id) ON DELETE CASCADE -- 外键约束，删除路由时级联删除对应的角色关联
);

-- 创建路由-操作权限关联表，用于定义进入该路由页面需要哪些细粒度的操作权限
CREATE TABLE IF NOT EXISTS route_auths (
  route_id BIGINT NOT NULL,                    -- 关联的路由 ID
  auth TEXT NOT NULL,                          -- 进入该路由或在路由内操作所需的具体权限代码
  PRIMARY KEY (route_id, auth),                -- 联合主键
  FOREIGN KEY (route_id) REFERENCES routes(id) ON DELETE CASCADE -- 外键约束，删除路由时级联删除对应的操作权限关联
);

-- 创建物联网设备注册表，用于管理系统下的所有硬件设备信息 (业务扩展)
CREATE TABLE IF NOT EXISTS device_registry (
  id BIGSERIAL PRIMARY KEY,                    -- 自增主键 ID
  device_id TEXT NOT NULL UNIQUE,              -- 设备的全局唯一标识符，不允许为空
  device_name TEXT NOT NULL,                   -- 设备的友好显示名称
  owner_username TEXT NOT NULL,                -- 设备所属用户的用户名（绑定关系）
  registered_at BIGINT NOT NULL                -- 设备注册成功的时间戳（通常为毫秒）
);

-- 创建 Casbin RBAC 策略规则表，用于通过 Casbin 引擎进行强大的后端接口与操作权限控制
CREATE TABLE IF NOT EXISTS casbin_rule (
  id BIGSERIAL PRIMARY KEY,                    -- 自增主键 ID
  ptype VARCHAR NOT NULL,                      -- 策略类型 (例如 'p' 代表 policy/策略规则, 'g' 代表 grouping/角色继承规则)
  v0 VARCHAR NOT NULL,                         -- 规则项 0（通常代表主体 Subject：用户或角色）
  v1 VARCHAR NOT NULL,                         -- 规则项 1（通常代表资源 Object：如 API 路径、设备类型等）
  v2 VARCHAR NOT NULL,                         -- 规则项 2（通常代表操作 Action：如 read, write, create 等）
  v3 VARCHAR NOT NULL,                         -- 规则项 3 (备用扩展字段)
  v4 VARCHAR NOT NULL,                         -- 规则项 4 (备用扩展字段)
  v5 VARCHAR NOT NULL,                         -- 规则项 5 (备用扩展字段)
  CONSTRAINT unique_key_sqlx_adapter UNIQUE (ptype, v0, v1, v2, v3, v4, v5) -- 联合唯一约束，防止在数据库中写入完全相同的重复策略
);
