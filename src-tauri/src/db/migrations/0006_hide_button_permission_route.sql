-- 清理/隐藏系统不再使用的 "按钮权限" 相关的演示路由菜单
-- 此操作不仅删除父节点，也会因为之前定义的外键 ON DELETE CASCADE 约束，自动清理关联的路由角色和操作权限关联表记录
DELETE FROM routes
WHERE path = '/permission/button'              -- 删除 "按钮权限" 目录本身
   OR path LIKE '/permission/button/%';        -- 批量删除所有挂载在 "按钮权限" 目录下的子路由 (如路由返回按钮权限、接口返回按钮权限等)
