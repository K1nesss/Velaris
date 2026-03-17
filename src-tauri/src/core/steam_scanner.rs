use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::time::SystemTime;
use steamlocate::SteamDir;

#[tauri::command]
pub fn steam_scan_print() -> Result<(), String> {
    println!("=== Starting Steam scan ===");

    let steam_dir = SteamDir::locate().map_err(|e| format!("Failed to locate Steam: {}", e))?;
    println!(
        "Steam installation found at - {}",
        steam_dir.path().display()
    );

    // 打开数据库连接
    let db_path = PathBuf::from("./playtime-tracker.db");
    println!("Database path: {}", db_path.display());
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    println!("Database connection established");

    // 设置 PRAGMA
    let _ = conn.execute("PRAGMA foreign_keys = ON;", []);
    println!("PRAGMA foreign_keys = ON");

    // 先将所有游戏标记为未安装，扫描时会重新标记为已安装
    conn.execute(
        "UPDATE games SET is_installed = 0, install_path = NULL;",
        [],
    )
    .map_err(|e| format!("Failed to mark games as not installed: {}", e))?;
    println!("Marked all games as not installed");

    let libraries = steam_dir
        .libraries()
        .map_err(|e| format!("Failed to get libraries: {}", e))?;

    let mut game_count = 0;
    let mut library_count = 0;

    for library in libraries {
        library_count += 1;
        let library = library.map_err(|e| format!("Failed to get library: {}", e))?;
        println!("Processing library - {}", library.path().display());

        let apps = library.apps();
        for app in apps {
            let app = app.map_err(|e| format!("Failed to get app: {}", e))?;
            let name = app.name.as_deref().unwrap_or("<unknown>");
            let install_path = library.resolve_app_dir(&app);
            let install_path_str = install_path.to_str().unwrap_or("");
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // 输出游戏详细信息
            println!(
                "  Found game: {} - {} - {}",
                app.app_id,
                name,
                install_path.display()
            );
            game_count += 1;

            // 插入或更新游戏信息到数据库
            let result = conn.execute(
                "INSERT INTO games (appid, name, install_path, is_installed, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, 1, ?4, ?5) 
                 ON CONFLICT(appid) DO UPDATE SET 
                 name = excluded.name, 
                 install_path = excluded.install_path, 
                 is_installed = excluded.is_installed,
                 updated_at = excluded.updated_at",
                (
                    app.app_id as i32,
                    name,
                    install_path_str,
                    timestamp as i64,
                    timestamp as i64,
                ),
            );

            if let Err(e) = result {
                println!("    ✗ Error saving to database: {}", e);
            }
            // match result {
            //     Ok(_) => println!("    ✓ Saved to database"),
            //     Err(e) => println!("    ✗ Error saving to database: {}", e),
            // }
        }
    }

    println!("=== Steam scan completed ===");
    println!("Found {} libraries and {} games\n", library_count, game_count);
    
    Ok(())
}
