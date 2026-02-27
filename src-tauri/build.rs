/// Tauri 构建脚本入口。
///
/// 由 Cargo 在编译期自动调用，负责生成 Tauri 所需的平台相关绑定代码
/// （如 Windows 资源文件 `.rc`、macOS Info.plist 等）。
fn main() {
    tauri_build::build();
}
