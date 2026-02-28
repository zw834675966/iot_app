# Task Plan - Replace custom token strings with jsonwebtoken JWT

- Task: 使用 `jsonwebtoken` 开源库完善当前项目 JWT 生成与刷新校验
- Owner: Codex
- Started: 2026-02-27 11:17

## Steps
1. 梳理现有 auth token 流程与约束（Tauri 安全边界）
2. 先写失败测试，覆盖刷新令牌非法输入与签发语义
3. 引入 `jsonwebtoken` 并在 `auth/services.rs` 实现标准 JWT 签发/校验
4. 调整命令层刷新逻辑，使用 refresh token 解码后的 subject 重新签发
5. 更新文档与进度记录，执行验证命令

## Progress
- [11:17:35] 创建任务计划文件并记录实施步骤 (completed)
- [11:18:04] 在 auth/commands.rs 增加 refresh token 非法输入与 token 类型校验测试（RED） (completed)
- [11:19:14] 更新 Cargo.toml，引入 jsonwebtoken 依赖 (completed)
- [11:19:44] 在 auth/services.rs 实现 JWT claims、签发与 refresh token 验签函数 (completed)
- [11:19:59] 更新 auth_refresh_token：先验签 refresh token，再按 subject 重新签发 (completed)
- [11:21:44] 修复 jsonwebtoken v10 provider 配置，启用 rust_crypto feature (completed)
- [11:24:24] 运行 cargo test 全量回归，14 个测试全部通过 (completed)
- [11:24:40] 更新 auth/README，补充 JWT 签发/刷新校验与密钥配置说明 (completed)
- [11:25:12] 追加 docs/development-progress.md 任务记录并同步 Tauri 安全边界评估 (completed)
- [11:25:50] 执行 RED 测试：refresh_rejects_* 初次运行 2 项失败，符合预期 (completed)
