use base64::{engine::general_purpose, Engine as _};
use rusqlite::{Connection, Result};
use std::env;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::{read, remove_file, write};
use std::path::PathBuf;

const APP_DATA_DIR_NAME: &str = "Velaris";

#[tauri::command]
pub fn init_database() -> Result<(), String> {
    let db_path = get_db_path()?;
    println!("Database path: {}", db_path.display());

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    println!("Database connection established");

    // 设置 PRAGMA
    apply_connection_pragmas(&conn)
        .map_err(|e| format!("Failed to apply database PRAGMA settings: {}", e))?;
    println!("PRAGMA settings applied");

    // 检查数据库是否已经初始化
    let is_initialized = is_database_initialized(&conn)
        .map_err(|e| format!("Failed to check initialization status: {}", e))?;
    println!("Database initialized status: {}", is_initialized);

    if !is_initialized {
        // 读取并执行 schema.sql
        let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("storage")
            .join("schema.sql");
        println!("Schema path: {}", schema_path.display());

        let schema_sql =
            read_to_string(schema_path).map_err(|e| format!("Failed to read schema.sql: {}", e))?;
        println!("Schema SQL read successfully");

        conn.execute_batch(&schema_sql)
            .map_err(|e| format!("Failed to execute schema.sql: {}", e))?;
        println!("Database initialized successfully");
    }

    ensure_database_indexes(&conn)
        .map_err(|e| format!("Failed to ensure database indexes: {}", e))?;

    Ok(())
}

#[allow(dead_code)]
pub fn get_db_connection() -> Result<Connection, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    // 设置 PRAGMA
    apply_connection_pragmas(&conn).map_err(|e| e.to_string())?;

    Ok(conn)
}

#[tauri::command]
pub fn print_database_tables() {
    println!("=== Printing database tables ===");

    // 直接创建数据库连接
    let db_path = match get_db_path() {
        Ok(path) => path,
        Err(error) => {
            println!("Error getting database path: {}", error);
            return;
        }
    };

    match Connection::open(&db_path) {
        Ok(conn) => {
            // 设置 PRAGMA
            if let Err(error) = apply_connection_pragmas(&conn) {
                println!("Failed to apply PRAGMA settings: {}", error);
            }

            // 打印 games 表
            println!("\n--- Games Table ---");
            if let Ok(mut stmt) = conn.prepare("SELECT * FROM games;") {
                if let Ok(mut rows) = stmt.query([]) {
                    while let Ok(Some(row)) = rows.next() {
                        match (
                            row.get::<_, i32>(0),
                            row.get::<_, Option<i32>>(1),
                            row.get::<_, String>(2),
                            row.get::<_, String>(3),
                            row.get::<_, Option<String>>(4),
                            row.get::<_, i64>(5),
                            row.get::<_, i64>(6),
                        ) {
                            (
                                Ok(id),
                                Ok(appid),
                                Ok(name),
                                Ok(install_path),
                                Ok(cover_path),
                                Ok(created_at),
                                Ok(updated_at),
                            ) => {
                                println!("ID: {}, AppID: {:?}, Name: {}, Path: {}, Cover: {:?}, Created: {}, Updated: {}", 
                                         id, appid, name, install_path, cover_path, created_at, updated_at);
                            }
                            _ => println!("Error reading game row"),
                        }
                    }
                }
            }

            // 打印 game_sessions 表
            println!("\n--- Game Sessions Table ---");
            if let Ok(mut stmt) = conn.prepare("SELECT * FROM game_sessions;") {
                if let Ok(mut rows) = stmt.query([]) {
                    while let Ok(Some(row)) = rows.next() {
                        match (
                            row.get::<_, i32>(0),
                            row.get::<_, i32>(1),
                            row.get::<_, i64>(2),
                            row.get::<_, Option<i64>>(3),
                            row.get::<_, Option<i32>>(4),
                        ) {
                            (
                                Ok(id),
                                Ok(game_id),
                                Ok(start_time),
                                Ok(end_time),
                                Ok(duration_seconds),
                            ) => {
                                println!(
                                    "ID: {}, Game ID: {}, Start: {}, End: {:?}, Duration: {:?}s",
                                    id, game_id, start_time, end_time, duration_seconds
                                );
                            }
                            _ => println!("Error reading session row"),
                        }
                    }
                }
            }

            // 打印 game_stats 表
            println!("\n--- Game Stats Table ---");
            if let Ok(mut stmt) = conn.prepare("SELECT * FROM game_stats;") {
                if let Ok(mut rows) = stmt.query([]) {
                    while let Ok(Some(row)) = rows.next() {
                        match (
                            row.get::<_, i32>(0),
                            row.get::<_, i32>(1),
                            row.get::<_, Option<i64>>(2),
                        ) {
                            (Ok(game_id), Ok(total_playtime_seconds), Ok(last_played_at)) => {
                                println!(
                                    "Game ID: {}, Total Playtime: {}s, Last Played: {:?}",
                                    game_id, total_playtime_seconds, last_played_at
                                );
                            }
                            _ => println!("Error reading stats row"),
                        }
                    }
                }
            }

            println!("\n=== Database tables printed ===");
        }
        Err(e) => {
            println!("Error getting database connection: {}", e);
        }
    }
}

#[tauri::command]
pub fn export_database() -> Result<String, String> {
    let db_path = get_db_path()?;

    if !db_path.exists() {
        return Err("Database file not found".to_string());
    }

    if let Ok(conn) = Connection::open(&db_path) {
        let _ = apply_connection_pragmas(&conn);
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(FULL);");
    }

    let bytes = read(&db_path).map_err(|e| format!("Failed to read database file: {}", e))?;
    Ok(general_purpose::STANDARD.encode(bytes))
}

#[tauri::command]
pub fn import_database(base64_data: String) -> Result<(), String> {
    let db_path = get_db_path()?;
    let decoded = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Invalid database payload: {}", e))?;

    if decoded.is_empty() {
        return Err("Imported database payload is empty".to_string());
    }

    write(&db_path, decoded).map_err(|e| format!("Failed to write database file: {}", e))?;

    let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
    let shm_path = PathBuf::from(format!("{}-shm", db_path.display()));

    if wal_path.exists() {
        let _ = remove_file(wal_path);
    }
    if shm_path.exists() {
        let _ = remove_file(shm_path);
    }

    Ok(())
}

pub fn get_db_path() -> Result<PathBuf, String> {
    let base_dir = resolve_app_data_dir()?;
    let app_dir = base_dir.join(APP_DATA_DIR_NAME);

    create_dir_all(&app_dir).map_err(|e| format!("Failed to create app data directory: {}", e))?;

    Ok(app_dir.join("playtime-tracker.db"))
}

fn resolve_app_data_dir() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data));
        }
        if let Some(app_data) = env::var_os("APPDATA") {
            return Ok(PathBuf::from(app_data));
        }
    }

    if cfg!(target_os = "macos") {
        if let Some(home) = env::var_os("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support"));
        }
    }

    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".local").join("share"));
    }

    env::current_dir().map_err(|e| format!("Failed to resolve current directory: {}", e))
}

fn apply_connection_pragmas(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

fn is_database_initialized(conn: &Connection) -> Result<bool, String> {
    // 检查 games 表是否存在
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='games';")
        .map_err(|e| e.to_string())?;
    let exists = stmt.exists([]).map_err(|e| e.to_string())?;
    Ok(exists)
}

fn ensure_database_indexes(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_game_sessions_game_id_start_time ON game_sessions(game_id, start_time DESC);
         CREATE INDEX IF NOT EXISTS idx_game_sessions_start_time ON game_sessions(start_time DESC);
         CREATE INDEX IF NOT EXISTS idx_game_sessions_end_time ON game_sessions(end_time);
         CREATE INDEX IF NOT EXISTS idx_game_sessions_game_id_end_time ON game_sessions(game_id, end_time);
         CREATE INDEX IF NOT EXISTS idx_games_installed_name ON games(is_installed, name COLLATE NOCASE);
         CREATE INDEX IF NOT EXISTS idx_games_name_nocase ON games(name COLLATE NOCASE);
         CREATE INDEX IF NOT EXISTS idx_game_stats_last_played_at ON game_stats(last_played_at);",
    )
}
