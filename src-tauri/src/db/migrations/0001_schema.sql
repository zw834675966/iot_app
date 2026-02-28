-- ==========================================================================================
-- 数据库表结构初始化脚本
-- 文件: 0001_schema.sql
-- 用途: 创建能源管理系统的核心数据表结构
-- 执行时机: 应用首次启动时自动执行（通过 Rust 的 include_str! 机制加载）
-- 数据库: SQLite (pure-admin-thin.sqlite3)
-- ==========================================================================================

-- 【重要】启用外键约束
-- SQLite 默认不启用外键级联删除，此设置确保级联操作正常工作
-- 例如：删除用户时自动删除关联的用户角色和权限记录
PRAGMA foreign_keys = ON;

-- ==========================================================================================
-- 用户表 (users)
-- 用途: 存储系统用户的基础信息
-- 关键字段说明:
--   - id: 主键，自增整型
--   - username: 用户名，唯一且必填，用于登录认证
--   - password: 密码哈希值，必填（离线环境使用简单存储，生产环境建议加密）
--   - nickname: 用户显示名称，必填
--   - avatar: 头像URL，默认为空字符串（触发前端使用本地离线默认头像）
--   - is_active: 账户状态，1=启用，0=禁用
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,      -- 用户唯一标识，自增主键
  username TEXT NOT NULL UNIQUE,              -- 用户名（登录账号），全局唯一，禁止重复
  password TEXT NOT NULL,                     -- 密码（建议存储哈希值）
  nickname TEXT NOT NULL,                    -- 显示昵称
  avatar TEXT NOT NULL,                       -- 头像URL，空字符串表示使用本地默认头像
  is_active INTEGER NOT NULL DEFAULT 1       -- 账户启用状态：1=启用，0=禁用
);

-- ==========================================================================================
-- 用户角色关联表 (user_roles)
-- 用途: 实现用户与角色的多对多关系
-- 设计说明:
--   - 一个用户可以拥有多个角色（如：管理员 + 普通用户）
--   - 一个角色可以分配给多个用户
--   - 使用复合主键 (user_id, role) 确保用户-角色组合唯一
--   - 外键级联删除：删除用户时自动删除该用户的所有角色记录
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS user_roles (
  user_id INTEGER NOT NULL,                  -- 关联 users 表的用户ID
  role TEXT NOT NULL,                        -- 角色标识符（如：'admin', 'common', 'editor'）
  PRIMARY KEY (user_id, role),               -- 复合主键，防止重复分配
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE  -- 级联删除：用户删除时移除角色关联
);

-- ==========================================================================================
-- 权限表 (permissions)
-- 用途: 定义系统中所有可用的权限点
-- 设计说明:
--   - 权限是最细粒度的访问控制单元
--   - 权限代码采用冒号分隔的命名规范: 资源:操作:子操作
--   - 示例: 'permission:btn:add' 表示"权限按钮新增"权限
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS permissions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,      -- 权限唯一标识
  code TEXT NOT NULL UNIQUE                   -- 权限代码，全局唯一（如 '*:*:*' 表示超级管理员全部权限）
);


-- ==========================================================================================
-- 用户权限关联表 (user_permissions)
-- 用途: 实现用户与权限的直接多对多关系
-- 设计说明:
--   - 权限可以通过角色间接分配，也可以直接分配给用户
--   - 直接分配的权限优先级高于角色权限（业务逻辑中处理）
--   - 外键级联删除：删除用户或权限时自动清理关联记录
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS user_permissions (
  user_id INTEGER NOT NULL,                  -- 关联 users 表的用户ID
  permission_id INTEGER NOT NULL,            -- 关联 permissions 表的权限ID
  PRIMARY KEY (user_id, permission_id),      -- 复合主键，防止重复分配
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,        -- 级联删除用户时清理
  FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE  -- 级联删除权限时清理
);

-- ==========================================================================================
-- 动态路由表 (routes)
-- 用途: 存储前端动态路由配置，实现基于数据库的前端路由管理
-- 关键设计:
--   - parent_id 实现树形结构，支持无限层级嵌套
--   - component 字段指向前端 Vue 组件路径（用于动态加载）
--   - meta_* 字段存储路由的附加元数据（标题、图标、排序）
--   - 此表数据由后端读取并组装后返回给前端 Vue Router
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS routes (
  id INTEGER PRIMARY KEY,                     -- 路由唯一标识（手动指定，用于前端缓存键）
  parent_id INTEGER,                         -- 父路由ID，NULL表示顶级路由（根路由）
  path TEXT NOT NULL UNIQUE,                 -- 路由路径，全局唯一（如 '/permission/page'）
  name TEXT,                                  -- 路由名称（用于 keep-alive 和权限校验）
  component TEXT,                            -- 前端组件路径（如 'permission/page/index'）
  meta_title TEXT NOT NULL,                  -- 路由在菜单中的显示标题
  meta_icon TEXT,                             -- 菜单图标（离线环境使用本地 Iconify 图标名）
  meta_rank INTEGER,                         -- 菜单排序权重，数值越小越靠前
  FOREIGN KEY (parent_id) REFERENCES routes(id) ON DELETE CASCADE  -- 级联删除：删除父路由时删除子路由
);

-- ==========================================================================================
-- 路由角色关联表 (route_roles)
-- 用途: 控制哪些角色可以访问特定路由
-- 设计说明:
--   - 只有分配了角色的用户才能看到对应菜单
--   - 未分配角色的路由对所有用户可见（需配合前端权限指令）
--   - 路由级别的访问控制
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS route_roles (
  route_id INTEGER NOT NULL,                 -- 关联 routes 表的路由ID
  role TEXT NOT NULL,                        -- 角色标识符
  PRIMARY KEY (route_id, role),              -- 复合主键，防止重复分配
  FOREIGN KEY (route_id) REFERENCES routes(id) ON DELETE CASCADE  -- 级联删除路由时清理
);

-- ==========================================================================================
-- 路由权限关联表 (route_auths)
-- 用途: 控制路由内具体操作的权限（如：按钮级别的增删改查）
-- 设计说明:
--   - 用于前端页面内按钮/操作的权限显隐控制
--   - 每个路由可以关联多个权限代码
--   - 前端通过 v-permission 指令实现按钮级别控制
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS route_auths (
  route_id INTEGER NOT NULL,                 -- 关联 routes 表的路由ID
  auth TEXT NOT NULL,                        -- 权限代码（关联 permissions 表的 code 字段）
  PRIMARY KEY (route_id, auth),              -- 复合主键，防止重复分配
  FOREIGN KEY (route_id) REFERENCES routes(id) ON DELETE CASCADE  -- 级联删除路由时清理
);

-- ==========================================================================================
-- 设备注册表 (device_registry)
-- 用途: 管理已注册的物联网设备信息
-- 业务场景:
--   - 能源管理系统需要追踪和管理物理设备
--   - 设备与用户关联，实现设备所有权管理
--   - registered_at 使用 Unix 毫秒时间戳（JavaScript 兼容）
-- 扩展说明:
--   - 此表是业务扩展的基石，可根据实际需求添加更多字段
--   - 如：设备类型、型号、固件版本、在线状态、最后心跳时间等
-- ==========================================================================================
CREATE TABLE IF NOT EXISTS device_registry (
  id INTEGER PRIMARY KEY AUTOINCREMENT,      -- 设备记录唯一标识
  device_id TEXT NOT NULL UNIQUE,            -- 设备唯一标识符（UUID 或硬件序列号）
  device_name TEXT NOT NULL,                 -- 设备显示名称（便于用户识别）
  owner_username TEXT NOT NULL,              -- 设备所有者用户名（关联 users 表）
  registered_at INTEGER NOT NULL             -- 注册时间（Unix 毫秒时间戳）
);

-- ==========================================================================================
-- 表结构设计总结
-- ==========================================================================================
-- 1. 鉴权体系: users + user_roles + permissions + user_permissions
--    - 用户 -> 角色 -> 权限（间接）
--    - 用户 -> 权限（直接）
-- 
-- 2. 路由体系: routes + route_roles + route_auths
--    - 路由定义（树形结构）
--    - 路由访问角色控制
--    - 路由内操作权限控制
-- 
-- 3. 业务扩展: device_registry
--    - 设备管理基础表
--    - 可根据业务需求扩展字段
-- 
-- 4. 外键策略: 所有关联表均使用 ON DELETE CASCADE
--    - 确保数据一致性：删除主记录时自动清理关联数据
--    - 简化业务代码：无需手动维护关联数据的删除逻辑
-- ==========================================================================================
