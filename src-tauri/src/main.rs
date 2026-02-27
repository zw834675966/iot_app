// 在 Windows Release 模式下隐藏控制台窗口，切勿删除此属性！
// 该属性仅在非 debug 构建时生效，将子系统设置为 "windows" 以避免弹出黑色终端。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 桌面应用程序入口点。
///
/// 委托给 [`pure_admin_thin_tauri_lib::run`] 启动 Tauri 运行时。
fn main() {
    pure_admin_thin_tauri_lib::run();
}
