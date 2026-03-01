-- 遗留数据清理：将用户表中包含外部网络链接的头像重置为空字符串
-- 目的：适应纯局域网/离线部署的要求，去除对外部图片CDN的依赖，提升安全性和加载速度
UPDATE users
SET avatar = ''
WHERE TRIM(avatar) != ''                       -- 仅处理 avatar 字段不为空的记录
  AND (
    LOWER(TRIM(avatar)) LIKE 'http://%'        -- 匹配 HTTP 链接
    OR LOWER(TRIM(avatar)) LIKE 'https://%'    -- 匹配 HTTPS 链接
    OR LOWER(TRIM(avatar)) LIKE '//%'          -- 匹配自适应协议的链接（如 //cdn.example.com/a.png）
    OR POSITION('://' IN LOWER(TRIM(avatar))) > 0 -- 匹配任何包含 :// 的协议前缀
  );

-- 遗留数据清理：将动态路由表中包含外部网络链接或不支持的在线图标格式重置为 NULL
-- 目的：同样是为了满足离线部署策略，确保前端启动不产生任何外部网络请求，且不因为加载失败导致 UI 异常
UPDATE routes
SET meta_icon = NULL
WHERE meta_icon IS NOT NULL                    -- 仅处理 meta_icon 字段不为空的记录
  AND TRIM(meta_icon) != ''
  AND (
    POSITION(':' IN meta_icon) > 0             -- 匹配包含冒号的复杂在线 Iconify 图标标识符
    OR LOWER(TRIM(meta_icon)) LIKE 'http://%'  -- 匹配 HTTP 外链图标
    OR LOWER(TRIM(meta_icon)) LIKE 'https://%' -- 匹配 HTTPS 外链图标
    OR LOWER(TRIM(meta_icon)) LIKE '//%'       -- 匹配自适应协议外链
    OR POSITION('://' IN LOWER(TRIM(meta_icon))) > 0 -- 匹配任何包含 :// 的协议前缀
  );
