# SQLite Tools MCP 配置与使用（本仓库）

本仓库已将数据库 MCP 从 `dbhub` 替换为 **SQLite Tools MCP**，服务名为：

- `sqlite_tools`

该服务用于本地 SQLite 数据库（Tauri 桌面端数据）相关的结构检查与 SQL 验证。

## 1. 当前推荐配置（Codex）

在 `%USERPROFILE%\.codex\config.toml` 中使用：

```toml
[mcp_servers.sqlite_tools]
type = "stdio"
command = "C:/Program Files/nodejs/node.exe"
args = ["C:/Users/zw/.codex/scripts/sqlite-tools-stdio-bridge.cjs"]
env = { SQLITE_DEFAULT_PATH = "C:/Users/zw/AppData/Roaming/com.pureadmin.thin/db", SQLITE_ALLOW_ABSOLUTE_PATHS = "true" }
startup_timeout_sec = 30
```

## 2. 为什么需要 bridge

`mcp-sqlite-tools` 默认使用行级 JSON I/O，而部分客户端（包括当前 Codex 环境）使用 Content-Length framed MCP。

因此这里通过：

- `C:/Users/zw/.codex/scripts/sqlite-tools-stdio-bridge.cjs`

做 framed <-> line 协议转换，保证握手与工具调用兼容。

## 3. 常用工具（sqlite_tools）

替换后重点使用以下工具进行数据库任务：

- 只读查询：`execute_read_query`
- 写入查询：`execute_write_query`
- 结构查询：`execute_schema_query`
- 结构检查：`list_tables`、`describe_table`
- 事务控制：`begin_transaction`、`commit_transaction`、`rollback_transaction`

默认优先只读查询；写操作要最小化、明确化。

## 4. 验证方法

1. 重启 Codex CLI（让新 MCP 配置生效）。
2. 在会话中检查 `sqlite_tools` 可用性（握手通过后可列工具并调用）。
3. 针对本仓库数据库执行最小查询验证（如表列表、单表 schema）。

## 5. 故障排查

- 启动失败：先在终端确认 `npx -y mcp-sqlite-tools` 可正常拉起。
- MCP 不生效：确认已重启 Codex，且 `config.toml` 中服务名为 `sqlite_tools`。
- 目录/路径问题：确认 `SQLITE_DEFAULT_PATH` 指向存在目录，数据库文件在该目录下可读写。

## 6. 本仓库回退策略

如果当前会话 MCP 不可用，先使用仓库测试进行变更验证：

- `cargo test --manifest-path src-tauri/Cargo.toml`

待 `sqlite_tools` 恢复后，再补做 schema/SQL MCP 校验。
