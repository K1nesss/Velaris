// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod core;
mod storage;
mod system;

use api::analytics::analytics_snapshot;
use api::dashboard::{
    dashboard_current_playing, dashboard_daily_chart, dashboard_donut_data,
    dashboard_recent_sessions, dashboard_snapshot, dashboard_today_total, dashboard_week_total,
};
use api::games::{game_detail, games_list};
use api::timeline::timeline_sessions;
use core::file_watcher::start_file_watcher;
use core::process_watcher::start_process_monitor;
use core::steam_scanner::steam_scan_print;
use steamlocate::SteamDir;
use storage::db::{export_database, import_database, init_database, print_database_tables};
use system::controls::{window_close, window_minimize, window_toggle_maximize};

#[tauri::command]
fn hello() -> String {
    "Hello from Rust".into()
}

fn main() {
    // 初始化数据库
    if let Err(e) = init_database() {
        println!("Error initializing database: {}", e);
    }

    // 程序启动时自动执行 Steam 扫描
    println!("=== Auto Steam Scan on Startup ===");
    if let Err(e) = steam_scan_print() {
        println!("Error during auto steam scan: {}", e);
    }

    // 程序启动时自动启动进程监控服务（常驻）
    println!("=== Starting Process Monitor Service ===");
    if let Err(e) = start_process_monitor() {
        println!("Error starting process monitor: {}", e);
    }

    // 程序启动时启动文件监听服务
    println!("=== Starting File Watcher Service ===");
    if let Ok(steam_dir) = SteamDir::locate() {
        if let Some(steam_path) = steam_dir.path().to_str() {
            start_file_watcher(steam_path.to_string());
        } else {
            println!("Failed to convert Steam path to string");
        }
    } else {
        println!("Failed to locate Steam installation");
    }

    tauri::Builder::default()
        // 注册命令
        .invoke_handler(tauri::generate_handler![
            hello,
            window_minimize,
            window_toggle_maximize,
            window_close,
            init_database,
            print_database_tables,
            export_database,
            import_database,
            dashboard_today_total,
            dashboard_week_total,
            dashboard_daily_chart,
            dashboard_current_playing,
            dashboard_donut_data,
            dashboard_recent_sessions,
            dashboard_snapshot,
            analytics_snapshot,
            games_list,
            game_detail,
            timeline_sessions
        ])
        // 启动 App
        .run(tauri::generate_context!())
        // 错误处理
        .expect("error while running tauri application");
}
