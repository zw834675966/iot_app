# 2026-03-01-1046-notice-read-null-extra-fix

## Objective
- 修复 `notice_get_read_items` 在 `extra` 列为 NULL 时的解码失败。

## Scope
- `src-tauri/src/notice/repository.rs`
- `plan/2026-03-01-1046-notice-read-null-extra-fix.md`
- `docs/development-progress.md`

## Checklist
- [x] 定位根因并确认影响范围
- [x] 先补充失败用例复现 NULL 解码问题
- [x] 实施最小修复并保持接口兼容
- [x] 运行相关测试验证
- [x] 更新进度与开发文档

## Progress Timeline
- [10:46:00] Task started (in_progress)
- [10:46:18] 根因定位：`extra` 列可能为 NULL，但代码按 String 解码 (done)
- [10:46:52] 修改 notice 解码逻辑并新增 NULL extra 回归测试 (done)
- [10:48:23] 运行完整 `cargo test` 全量回归通过 (done)

## Verification
- command: `cargo test --manifest-path src-tauri/Cargo.toml notice::repository::tests::read_items_allow_null_extra_column -- --nocapture`
- result: passed (1 passed; 0 failed).
- command: `cargo test --manifest-path src-tauri/Cargo.toml`
- result: passed (44 passed; 0 failed; doctest 1 passed).

## Completion
- status: done
- follow-up: N/A
