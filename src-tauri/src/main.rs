// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod system;

use system::controls::{window_close, window_minimize};

#[tauri::command]
fn hello() -> String {
    "Hello from Rust".into()
}

fn main() {
    tauri::Builder::default()
        // 注册 hello 命令（核心新增）
        .invoke_handler(tauri::generate_handler![
            hello,
            window_minimize,
            window_close
        ])
        // 启动 App（替代原有的 app_lib::run()）
        .run(tauri::generate_context!())
        // 错误处理
        .expect("error while running tauri application");
    // app_lib::run();
}
