-- ==========================================================================================
-- Hide button-permission route tree
-- File: 0006_hide_button_permission_route.sql
-- Purpose: remove legacy button-permission menu/routes from dynamic routing
-- ==========================================================================================

PRAGMA foreign_keys = ON;

DELETE FROM routes
WHERE path = '/permission/button'
   OR path LIKE '/permission/button/%';
