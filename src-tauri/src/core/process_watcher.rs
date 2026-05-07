use rusqlite::{Connection, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use sysinfo::System;

use crate::storage::db::get_db_path;

#[derive(Debug, Clone)]
pub struct GameSession {
    pub game_id: i32,
    pub start_time: i64,
    pub session_id: Option<i32>,
}

#[derive(Debug, Clone)]
struct GameProcessMatch {
    game_id: i32,
    path: PathBuf,
    exact: bool,
}

pub struct ProcessWatcher {
    db_path: PathBuf,
    game_matches: Arc<Mutex<Vec<GameProcessMatch>>>,
    active_sessions: Arc<Mutex<HashMap<i32, GameSession>>>,
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

    pub fn new(db_path: &Path) -> Result<Self, String> {
        let _ = ProcessWatcher::open_connection(db_path)?;

        Ok(Self {
            db_path: db_path.to_path_buf(),
            game_matches: Arc::new(Mutex::new(Vec::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            scan_interval: Duration::from_secs(5),
        })
    }

    pub fn init_game_paths(&mut self) -> Result<(), String> {
        println!("Initializing game process match index...");

        let conn = ProcessWatcher::open_connection(&self.db_path)?;
        let next_matches = ProcessWatcher::load_game_process_matches(&conn)?;
        let mut game_matches = self.game_matches.lock().unwrap();
        *game_matches = next_matches;

        println!(
            "Game process match index initialized with {} rules",
            game_matches.len()
        );
        Ok(())
    }

    fn load_game_process_matches(conn: &Connection) -> Result<Vec<GameProcessMatch>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, install_path, executable_path
                 FROM games
                 WHERE is_installed = 1 AND is_hidden = 0",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let id: i32 = row.get(0)?;
                let install_path: Option<String> = row.get(1)?;
                let executable_path: Option<String> = row.get(2)?;
                Ok((id, install_path, executable_path))
            })
            .map_err(|e| e.to_string())?;

        let mut matches = Vec::new();
        for row in rows {
            let (game_id, install_path, executable_path) = row.map_err(|e| e.to_string())?;

            if let Some(executable_path) = executable_path.filter(|value| !value.trim().is_empty())
            {
                matches.push(GameProcessMatch {
                    game_id,
                    path: PathBuf::from(executable_path),
                    exact: true,
                });
                continue;
            }

            if let Some(install_path) = install_path.filter(|value| !value.trim().is_empty()) {
                matches.push(GameProcessMatch {
                    game_id,
                    path: PathBuf::from(install_path),
                    exact: false,
                });
            }
        }

        Ok(matches)
    }

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

    pub fn start_monitoring(&mut self) {
        println!("Starting process monitor...");

        let db_path = self.db_path.clone();
        let game_matches = self.game_matches.clone();
        let active_sessions = self.active_sessions.clone();
        let scan_interval = self.scan_interval;

        thread::spawn(move || {
            let mut system = System::new_all();

            loop {
                ProcessWatcher::check_processes(
                    &db_path,
                    &game_matches,
                    &active_sessions,
                    &mut system,
                );

                thread::sleep(scan_interval);
            }
        });
    }

    fn check_processes(
        db_path: &Path,
        game_matches: &Arc<Mutex<Vec<GameProcessMatch>>>,
        active_sessions: &Arc<Mutex<HashMap<i32, GameSession>>>,
        system: &mut System,
    ) {
        system.refresh_all();

        let conn = match ProcessWatcher::open_connection(db_path) {
            Ok(conn) => conn,
            Err(error) => {
                println!("[Process Watcher] Failed to open database: {}", error);
                return;
            }
        };

        if let Ok(next_matches) = ProcessWatcher::load_game_process_matches(&conn) {
            let mut game_matches = game_matches.lock().unwrap();
            *game_matches = next_matches;
        }

        let indexed_matches = {
            let game_matches = game_matches.lock().unwrap();
            game_matches.clone()
        };

        let mut running_games = HashSet::new();
        for process in system.processes().values() {
            if let Some(exe_path) = process.exe() {
                for rule in &indexed_matches {
                    if ProcessWatcher::path_matches(exe_path, rule) {
                        running_games.insert(rule.game_id);
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

        for game_id in new_games {
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

    fn path_matches(exe_path: &Path, rule: &GameProcessMatch) -> bool {
        let exe = ProcessWatcher::normalize_path_for_match(exe_path);
        let target = ProcessWatcher::normalize_path_for_match(&rule.path);

        if rule.exact {
            exe == target
        } else {
            exe.starts_with(&target)
        }
    }

    fn normalize_path_for_match(path: &Path) -> PathBuf {
        let normalized = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .replace('/', "\\");

        if cfg!(target_os = "windows") {
            PathBuf::from(normalized.to_lowercase())
        } else {
            PathBuf::from(normalized)
        }
    }

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

        Ok(conn.last_insert_rowid() as i32)
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

    fn end_session(conn: &Connection, session: GameSession) {
        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;

        let duration_seconds = (end_time - session.start_time).max(0) as i32;

        if let Some(session_id) = session.session_id {
            let _ = conn.execute(
                "UPDATE game_sessions SET end_time = ?1, duration_seconds = ?2 WHERE id = ?3",
                (end_time, duration_seconds, session_id),
            );
        }

        ProcessWatcher::update_game_stats(conn, session.game_id, duration_seconds, end_time);
    }

    fn update_game_stats(
        conn: &Connection,
        game_id: i32,
        duration_seconds: i32,
        last_played_at: i64,
    ) {
        let mut stmt = match conn
            .prepare("SELECT total_playtime_seconds FROM game_stats WHERE game_id = ?1")
        {
            Ok(stmt) => stmt,
            Err(error) => {
                println!(
                    "[Process Watcher] Failed to prepare game stats query: {}",
                    error
                );
                return;
            }
        };
        let result = stmt.query_row((game_id,), |row| row.get::<_, i32>(0)).ok();

        match result {
            Some(current_playtime) => {
                let new_playtime = current_playtime + duration_seconds;
                let _ = conn.execute(
                    "UPDATE game_stats SET total_playtime_seconds = ?1, last_played_at = ?2 WHERE game_id = ?3",
                    (new_playtime, last_played_at, game_id),
                );
            }
            None => {
                let _ = conn.execute(
                    "INSERT INTO game_stats (game_id, total_playtime_seconds, last_played_at) VALUES (?1, ?2, ?3)",
                    (game_id, duration_seconds, last_played_at),
                );
            }
        }
    }
}

#[tauri::command]
pub fn start_process_monitor() -> Result<(), String> {
    let db_path = get_db_path()?;

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
