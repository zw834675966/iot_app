-- PostgreSQL 17 schema bootstrap

CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  nickname TEXT NOT NULL,
  avatar TEXT NOT NULL,
  is_active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS user_roles (
  user_id BIGINT NOT NULL,
  role TEXT NOT NULL,
  PRIMARY KEY (user_id, role),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS permissions (
  id BIGSERIAL PRIMARY KEY,
  code TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS user_permissions (
  user_id BIGINT NOT NULL,
  permission_id BIGINT NOT NULL,
  PRIMARY KEY (user_id, permission_id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS routes (
  id BIGINT PRIMARY KEY,
  parent_id BIGINT,
  path TEXT NOT NULL UNIQUE,
  name TEXT,
  component TEXT,
  meta_title TEXT NOT NULL,
  meta_icon TEXT,
  meta_rank INTEGER,
  FOREIGN KEY (parent_id) REFERENCES routes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS route_roles (
  route_id BIGINT NOT NULL,
  role TEXT NOT NULL,
  PRIMARY KEY (route_id, role),
  FOREIGN KEY (route_id) REFERENCES routes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS route_auths (
  route_id BIGINT NOT NULL,
  auth TEXT NOT NULL,
  PRIMARY KEY (route_id, auth),
  FOREIGN KEY (route_id) REFERENCES routes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS device_registry (
  id BIGSERIAL PRIMARY KEY,
  device_id TEXT NOT NULL UNIQUE,
  device_name TEXT NOT NULL,
  owner_username TEXT NOT NULL,
  registered_at BIGINT NOT NULL
);
