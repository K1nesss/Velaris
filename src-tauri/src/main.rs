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
use api::games::{
    create_manual_game, delete_game, delete_game_session, game_detail, games_list,
    hidden_games_list, hide_game, ignored_games_list, restore_hidden_game, restore_ignored_game,
};
use api::steam::sync_steam_owned_game_icons;
use api::timeline::timeline_sessions;
use core::file_watcher::start_file_watcher;
use core::process_watcher::start_process_monitor;
use core::steam_scanner::steam_scan_print;
use steamlocate::SteamDir;
use storage::db::{export_database, import_database, init_database, print_database_tables};
use system::controls::{window_close, window_minimize, window_toggle_maximize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

#[tauri::command]
fn hello() -> String {
    "Hello from Rust".into()
}

fn tray_now_playing_label() -> String {
    match dashboard_current_playing() {
        Ok(Some(current)) => format!("正在游玩: {}", current.game_name),
        Ok(None) => "正在游玩: 无".to_string(),
        Err(_) => "正在游玩: 读取失败".to_string(),
    }
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let now_playing_item = MenuItem::with_id(
                app,
                "tray_now_playing",
                tray_now_playing_label(),
                false,
                None::<&str>,
            )?;
            let show_label = if cfg!(target_os = "windows") {
                "显示主窗口"
            } else {
                "Show Window"
            };
            let quit_label = if cfg!(target_os = "windows") {
                "退出"
            } else {
                "Quit"
            };

            let show_item = MenuItem::with_id(app, "tray_show", show_label, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "tray_quit", quit_label, true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&now_playing_item, &show_item, &quit_item])?;

            let icon = app.default_window_icon().cloned();

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event({
                    let now_playing_item = now_playing_item.clone();
                    move |tray, event| match event {
                        TrayIconEvent::DoubleClick { .. } => {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        TrayIconEvent::Click {
                            button: MouseButton::Right,
                            button_state: MouseButtonState::Down,
                            ..
                        } => {
                            let _ = now_playing_item.set_text(tray_now_playing_label());
                        }
                        _ => {}
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "tray_show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "tray_quit" => {
                        app.exit(0);
                    }
                    _ => {}
                });

            if let Some(icon) = icon {
                tray_builder = tray_builder.icon(icon);
            }

            let _tray = tray_builder.build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
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
            create_manual_game,
            game_detail,
            hide_game,
            hidden_games_list,
            restore_hidden_game,
            delete_game,
            delete_game_session,
            ignored_games_list,
            restore_ignored_game,
            sync_steam_owned_game_icons,
            timeline_sessions
        ])
        // 启动 App
        .run(tauri::generate_context!())
        // 错误处理
        .expect("error while running tauri application");
}
