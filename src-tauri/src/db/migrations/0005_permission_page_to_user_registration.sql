-- 将原先名为 "PermissionPage" (权限页面) 的路由节点更新为更符合业务含义的 "用户注册管理"
-- 依据路由 ID 和原本路径精确定位，避免误伤其他节点
UPDATE routes
SET meta_title = '用户注册管理'                  -- 更新前端菜单显示的标题为 '用户注册管理'
WHERE id = 2
  AND path = '/permission/page/index';         -- 仅修改 ID 为 2 且路径匹配的路由记录
