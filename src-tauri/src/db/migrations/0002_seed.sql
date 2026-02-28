-- PostgreSQL 17 seed data
INSERT INTO users (id, username, password, nickname, avatar, is_active) VALUES
  (1, 'admin', 'admin123', '小铭', '', 1),
  (2, 'common', 'admin123', '小林', '', 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role) VALUES
  (1, 'admin'),
  (2, 'common')
ON CONFLICT (user_id, role) DO NOTHING;

INSERT INTO permissions (id, code) VALUES
  (1, '*:*:*'),
  (2, 'permission:btn:add'),
  (3, 'permission:btn:edit'),
  (4, 'permission:btn:delete')
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_permissions (user_id, permission_id) VALUES
  (1, 1),
  (2, 2),
  (2, 3)
ON CONFLICT (user_id, permission_id) DO NOTHING;

INSERT INTO routes (id, parent_id, path, name, component, meta_title, meta_icon, meta_rank) VALUES
  (1, NULL, '/permission', NULL, NULL, '权限管理', 'ri/information-line', 10),
  (2, 1, '/permission/page/index', 'PermissionPage', NULL, '用户注册管理', NULL, NULL),
  (3, 1, '/permission/button', NULL, NULL, '按钮权限', NULL, NULL),
  (4, 3, '/permission/button/router', 'PermissionButtonRouter', 'permission/button/index', '路由返回按钮权限', NULL, NULL),
  (5, 3, '/permission/button/login', 'PermissionButtonLogin', 'permission/button/perms', '登录接口返回按钮权限', NULL, NULL)
ON CONFLICT (id) DO NOTHING;

INSERT INTO route_roles (route_id, role) VALUES
  (2, 'admin'),
  (2, 'common'),
  (3, 'admin'),
  (3, 'common')
ON CONFLICT (route_id, role) DO NOTHING;

INSERT INTO route_auths (route_id, auth) VALUES
  (4, 'permission:btn:add'),
  (4, 'permission:btn:edit'),
  (4, 'permission:btn:delete')
ON CONFLICT (route_id, auth) DO NOTHING;

INSERT INTO device_registry (device_id, device_name, owner_username, registered_at) VALUES
  ('device-localhost-001', 'Desktop Development Device', 'admin', 1772150000000)
ON CONFLICT (device_id) DO NOTHING;

SELECT setval('users_id_seq', GREATEST((SELECT COALESCE(MAX(id), 1) FROM users), 1), true);
SELECT setval('permissions_id_seq', GREATEST((SELECT COALESCE(MAX(id), 1) FROM permissions), 1), true);
