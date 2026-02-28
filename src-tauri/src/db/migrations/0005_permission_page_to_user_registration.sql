-- ==========================================================================================
-- Route title rename migration
-- File: 0005_permission_page_to_user_registration.sql
-- Purpose: rename permission page menu title to user registration management
-- ==========================================================================================

PRAGMA foreign_keys = ON;

UPDATE routes
SET meta_title = '用户注册管理'
WHERE id = 2
  AND path = '/permission/page/index';
