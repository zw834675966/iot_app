-- PostgreSQL 17 seed data
-- 插入系统初始化所需的种子数据

-- 插入默认的管理员和普通用户账号
INSERT INTO users (id, username, password, nickname, avatar, is_active) VALUES
  (1, 'admin', 'admin123', '小铭', '', 1),     -- 超级管理员账号，默认密码 admin123
  (2, 'common', 'admin123', '小林', '', 1)     -- 普通演示账号，默认密码 admin123
ON CONFLICT (id) DO NOTHING;                   -- 如果 ID 冲突则忽略，保证幂等性

-- 为默认用户分配相应的角色
INSERT INTO user_roles (user_id, role) VALUES
  (1, 'admin'),                                -- 用户 ID 1 分配 admin 角色
  (2, 'common')                                -- 用户 ID 2 分配 common 角色
ON CONFLICT (user_id, role) DO NOTHING;        -- 避免重复分配角色

-- 插入系统中预定义的权限标识符
INSERT INTO permissions (id, code) VALUES
  (1, '*:*:*'),                                -- 超级管理员的通配符权限，代表拥有所有权限
  (2, 'permission:btn:add'),                   -- 按钮级别权限：允许新增操作
  (3, 'permission:btn:edit'),                  -- 按钮级别权限：允许编辑操作
  (4, 'permission:btn:delete')                 -- 按钮级别权限：允许删除操作
ON CONFLICT (id) DO NOTHING;

-- 直接为用户分配特定的权限 (独立于角色)
INSERT INTO user_permissions (user_id, permission_id) VALUES
  (1, 1),                                      -- 为 admin 分配 '*:*:*' 全部权限
  (2, 2),                                      -- 为 common 分配 'permission:btn:add' 新增权限
  (2, 3)                                       -- 为 common 分配 'permission:btn:edit' 编辑权限
ON CONFLICT (user_id, permission_id) DO NOTHING;

-- 插入预定义的前端页面路由，构建左侧菜单树
INSERT INTO routes (id, parent_id, path, name, component, meta_title, meta_icon, meta_rank) VALUES
  (1, NULL, '/permission', NULL, NULL, '权限管理', 'ri/information-line', 10),                     -- 顶级菜单：权限管理目录
  (2, 1, '/permission/page/index', 'PermissionPage', NULL, '用户注册管理', NULL, NULL),           -- 子菜单：用户注册管理页面
  (3, 1, '/permission/button', NULL, NULL, '按钮权限', NULL, NULL),                                 -- 子菜单：按钮权限目录
  (4, 3, '/permission/button/router', 'PermissionButtonRouter', 'permission/button/index', '路由返回按钮权限', NULL, NULL),  -- 孙菜单：基于路由的按钮权限演示
  (5, 3, '/permission/button/login', 'PermissionButtonLogin', 'permission/button/perms', '登录接口返回按钮权限', NULL, NULL) -- 孙菜单：基于接口的按钮权限演示
ON CONFLICT (id) DO NOTHING;

-- 设置页面路由与角色的绑定关系 (配置哪些角色可以看到哪些菜单)
INSERT INTO route_roles (route_id, role) VALUES
  (2, 'admin'),                                -- admin 角色可访问用户注册管理
  (2, 'common'),                               -- common 角色也可访问用户注册管理
  (3, 'admin'),                                -- admin 角色可访问按钮权限目录
  (3, 'common')                                -- common 角色也可访问按钮权限目录
ON CONFLICT (route_id, role) DO NOTHING;

-- 配置进入路由所需的特定细粒度操作权限
INSERT INTO route_auths (route_id, auth) VALUES
  (4, 'permission:btn:add'),                   -- 访问路由返回按钮权限页面需要 add 权限
  (4, 'permission:btn:edit'),                  -- 需要 edit 权限
  (4, 'permission:btn:delete')                 -- 需要 delete 权限
ON CONFLICT (route_id, auth) DO NOTHING;

-- 注册一个用于本地开发和测试的默认物联网设备
INSERT INTO device_registry (device_id, device_name, owner_username, registered_at) VALUES
  ('device-localhost-001', 'Desktop Development Device', 'admin', 1772150000000) -- 设备归属于 admin 用户
ON CONFLICT (device_id) DO NOTHING;

-- 初始化 Casbin 后端接口/资源访问控制策略 (RBAC Policy)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5) VALUES
  ('p', 'admin', 'user', 'manage', '', '', ''),          -- 策略: admin 角色具有 user(用户资源) 的 manage(管理) 权限
  ('p', 'admin', 'device', 'create', '', '', ''),        -- 策略: admin 角色具有 device(设备资源) 的 create(创建) 权限
  ('p', 'admin', 'control', 'issue', '', '', ''),        -- 策略: admin 角色具有 control(控制指令) 的 issue(下发) 权限
  ('p', 'admin', 'dashboard', 'view', '', '', ''),       -- 策略: admin 角色具有 dashboard(仪表盘) 的 view(查看) 权限
  ('p', 'operator', 'control', 'issue', '', '', ''),     -- 策略: operator 角色具有 control 的 issue 权限
  ('p', 'guest', 'dashboard', 'view', '', '', ''),       -- 策略: guest 角色具有 dashboard 的 view 权限
  ('p', 'common', 'dashboard', 'view', '', '', '')       -- 策略: common 角色具有 dashboard 的 view 权限
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- 重新同步 PostgreSQL 中各表的序列自增值，确保后续插入新数据时 ID 不会与现有种子数据的 ID 冲突
SELECT setval('users_id_seq', GREATEST((SELECT COALESCE(MAX(id), 1) FROM users), 1), true);             -- 同步 users 表的序列
SELECT setval('permissions_id_seq', GREATEST((SELECT COALESCE(MAX(id), 1) FROM permissions), 1), true); -- 同步 permissions 表的序列
