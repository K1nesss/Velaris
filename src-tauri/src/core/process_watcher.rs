use rusqlite::{Connection, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use sysinfo::System;

// 游戏会话状态#[derive(Debug, Clone)]
pub struct GameSession {
    pub game_id: i32,
    pub start_time: i64,
    pub session_id: Option<i32>,
}

// 进程监控器结构体
pub struct ProcessWatcher {
    db_path: PathBuf,
    game_paths: Arc<Mutex<HashMap<PathBuf, i32>>>, // 路径到游戏ID的映射
    active_sessions: Arc<Mutex<HashMap<i32, GameSession>>>, // 游戏ID到会话的映射
    scan_interval: Duration,
}

impl ProcessWatcher {
    fn open_connection(db_path: &Path) -> Result<Connection, String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| e.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| e.to_string())?;
        conn.busy_timeout(Duration::from_secs(3))
            .map_err(|e| e.to_string())?;
        Ok(conn)
    }

    // 创建新的进程监控器
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let _ = ProcessWatcher::open_connection(db_path)?;

        Ok(Self {
            db_path: db_path.to_path_buf(),
            game_paths: Arc::new(Mutex::new(HashMap::new())), // 游戏路径索引
            active_sessions: Arc::new(Mutex::new(HashMap::new())), // 正在进行的会话
            scan_interval: Duration::from_secs(5),            // 5秒扫描一次
        })
    }

    // 初始化游戏路径索引
    pub fn init_game_paths(&mut self) -> Result<(), String> {
        println!("Initializing game paths index...");

        let conn = ProcessWatcher::open_connection(&self.db_path)?;
        let mut game_paths = self.game_paths.lock().unwrap();
        game_paths.clear();

        // 从数据库加载游戏信息
        let mut stmt = conn
            .prepare("SELECT id, install_path FROM games")
            .map_err(|e| e.to_string())?;
        let game_iter = stmt
            .query_map([], |row| {
                let id: i32 = row.get(0)?;
                let install_path: String = row.get(1)?;
                Ok((id, install_path))
            })
            .map_err(|e| e.to_string())?;

        // 遍历查询结果
        for (id, install_path) in game_iter.flatten() {
            let path = PathBuf::from(install_path);
            game_paths.insert(path, id); // 路径 → 游戏ID
        }

        println!(
            "Game paths index initialized with {} games",
            game_paths.len()
        );
        Ok(())
    }

    // 恢复数据库中未结束的会话，避免重启程序后重复创建 session。
    pub fn recover_active_sessions(&mut self) -> Result<(), String> {
        println!("Recovering open game sessions...");

        let conn = ProcessWatcher::open_connection(&self.db_path)?;
        let mut active_sessions = self.active_sessions.lock().unwrap();
        active_sessions.clear();

        let mut stmt = conn
            .prepare(
                "SELECT id, game_id, start_time
                 FROM game_sessions
                 WHERE end_time IS NULL
                 ORDER BY game_id ASC, start_time ASC, id ASC",
            )
            .map_err(|e| e.to_string())?;

        let open_sessions = stmt
            .query_map([], |row| {
                Ok(GameSession {
                    session_id: Some(row.get(0)?),
                    game_id: row.get(1)?,
                    start_time: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for session in open_sessions {
            let session = session.map_err(|e| e.to_string())?;

            if active_sessions.contains_key(&session.game_id) {
                ProcessWatcher::close_duplicate_open_session(&conn, &session)?;
                println!(
                    "[Process Watcher] Closed duplicate open session: game_id={}, session_id={:?}",
                    session.game_id, session.session_id
                );
                continue;
            }

            println!(
                "[Process Watcher] Recovered open session: game_id={}, session_id={:?}",
                session.game_id, session.session_id
            );
            active_sessions.insert(session.game_id, session);
        }

        println!(
            "Recovered {} active sessions from database",
            active_sessions.len()
        );
        Ok(())
    }

    // 启动监控
    pub fn start_monitoring(&mut self) {
        println!("Starting process monitor...");

        let db_path = self.db_path.clone();
        let game_paths = self.game_paths.clone();
        let active_sessions = self.active_sessions.clone();
        let scan_interval = self.scan_interval;

        // 启动监控线程
        thread::spawn(move || {
            let mut system = System::new_all();

            loop {
                // 检查进程
                ProcessWatcher::check_processes(
                    &db_path,
                    &game_paths,
                    &active_sessions,
                    &mut system,
                );

                // 休眠指定时间
                thread::sleep(scan_interval);
            }
        });
    }

    // 检查进程
    fn check_processes(
        db_path: &Path,
        game_paths: &Arc<Mutex<HashMap<PathBuf, i32>>>,
        active_sessions: &Arc<Mutex<HashMap<i32, GameSession>>>,
        system: &mut System,
    ) {
        // 1. 获取所有运行中的进程
        system.refresh_all();

        let conn = match ProcessWatcher::open_connection(db_path) {
            Ok(conn) => conn,
            Err(error) => {
                println!("[Process Watcher] Failed to open database: {}", error);
                return;
            }
        };

        let indexed_paths: Vec<(PathBuf, i32)> = {
            let game_paths = game_paths.lock().unwrap();
            game_paths
                .iter()
                .map(|(path, game_id)| (path.clone(), *game_id))
                .collect()
        };

        // 收集当前运行的游戏进程
        let mut running_games = HashSet::new();

        // 2. 遍历进程，找出游戏进程
        for process in system.processes().values() {
            if let Some(exe_path) = process.exe() {
                // 检查进程路径是否属于某个游戏的安装目录
                for (game_path, game_id) in &indexed_paths {
                    if exe_path.starts_with(game_path) {
                        running_games.insert(*game_id);
                        break;
                    }
                }
            }
        }

        let (new_games, ended_game_ids) = {
            let active_sessions = active_sessions.lock().unwrap();
            let new_games = running_games
                .iter()
                .copied()
                .filter(|game_id| !active_sessions.contains_key(game_id))
                .collect::<Vec<_>>();

            let ended_game_ids = active_sessions
                .keys()
                .copied()
                .filter(|game_id| !running_games.contains(game_id))
                .collect::<Vec<_>>();

            (new_games, ended_game_ids)
        };

        // 检查新启动的游戏
        for game_id in new_games {
            // 启动新会话
            match ProcessWatcher::start_session(&conn, game_id) {
                Ok(session_id) => {
                    let start_time = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let mut active_sessions = active_sessions.lock().unwrap();
                    active_sessions.insert(
                        game_id,
                        GameSession {
                            game_id,
                            start_time: start_time as i64,
                            session_id: Some(session_id),
                        },
                    );
                }
                Err(e) => {
                    println!(
                        "[Process Watcher] Failed to save session (ID: {}) - {}",
                        game_id, e
                    );
                }
            }
        }

        // 从活跃会话中移除已结束游戏
        let ended_sessions = {
            let mut active_sessions = active_sessions.lock().unwrap();
            let mut ended_sessions = Vec::with_capacity(ended_game_ids.len());

            for game_id in ended_game_ids {
                if let Some(session) = active_sessions.remove(&game_id) {
                    ended_sessions.push(session);
                }
            }

            ended_sessions
        };

        for session in ended_sessions {
            ProcessWatcher::end_session(&conn, session);
        }
    }

    // 启动新会话
    fn start_session(conn: &Connection, game_id: i32) -> Result<i32, String> {
        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut stmt = conn
            .prepare("INSERT INTO game_sessions (game_id, start_time) VALUES (?1, ?2)")
            .map_err(|e| e.to_string())?;
        stmt.execute((game_id, start_time as i64))
            .map_err(|e| e.to_string())?;

        let session_id = conn.last_insert_rowid() as i32;
        Ok(session_id)
    }

    fn close_duplicate_open_session(
        conn: &Connection,
        session: &GameSession,
    ) -> Result<(), String> {
        if let Some(session_id) = session.session_id {
            conn.execute(
                "UPDATE game_sessions
                 SET end_time = ?1, duration_seconds = 0
                 WHERE id = ?2 AND end_time IS NULL",
                (session.start_time, session_id),
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    // 结束会话
    fn end_session(conn: &Connection, session: GameSession) {
        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;

        let duration_seconds = (end_time - session.start_time) as i32;

        // 更新会话
        if let Some(session_id) = session.session_id {
            let _ = conn.execute(
                "UPDATE game_sessions SET end_time = ?1, duration_seconds = ?2 WHERE id = ?3",
                (end_time, duration_seconds, session_id),
            );
        }

        // 更新游戏统计
        ProcessWatcher::update_game_stats(conn, session.game_id, duration_seconds, end_time);
    }

    // 更新游戏统计
    fn update_game_stats(
        conn: &Connection,
        game_id: i32,
        duration_seconds: i32,
        last_played_at: i64,
    ) {
        // 检查是否存在统计记录
        let mut stmt = conn
            .prepare("SELECT total_playtime_seconds FROM game_stats WHERE game_id = ?1")
            .unwrap();
        let result = stmt.query_row((game_id,), |row| row.get::<_, i32>(0)).ok();

        match result {
            Some(current_playtime) => {
                // 更新现有记录
                let new_playtime = current_playtime + duration_seconds;
                let _ = conn.execute(
                    "UPDATE game_stats SET total_playtime_seconds = ?1, last_played_at = ?2 WHERE game_id = ?3",
                    (new_playtime, last_played_at, game_id)
                );
            }
            None => {
                // 创建新记录
                let _ = conn.execute(
                    "INSERT INTO game_stats (game_id, total_playtime_seconds, last_played_at) VALUES (?1, ?2, ?3)",
                    (game_id, duration_seconds, last_played_at)
                );
            }
        }
    }
}

// Tauri 命令
#[tauri::command]
pub fn start_process_monitor() -> Result<(), String> {
    let db_path = PathBuf::from("./playtime-tracker.db");

    match ProcessWatcher::new(&db_path) {
        Ok(mut watcher) => {
            if let Err(e) = watcher.init_game_paths() {
                return Err(format!("Failed to initialize game paths: {}", e));
            }

            if let Err(e) = watcher.recover_active_sessions() {
                return Err(format!("Failed to recover active sessions: {}", e));
            }

            watcher.start_monitoring();
            Ok(())
        }
        Err(e) => Err(format!("Failed to start process monitor: {}", e)),
    }
}
