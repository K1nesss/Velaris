use rusqlite::params;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::db::get_db_connection;

#[derive(Serialize)]
pub struct GameListItem {
    pub id: i32,
    pub appid: Option<i32>,
    pub name: String,
    pub cover_path: Option<String>,
    pub is_installed: bool,
    pub total_playtime_seconds: i64,
    pub total_playtime_formatted: String,
    pub last_played_at: Option<i64>,
    pub session_count: i64,
}

#[derive(Serialize)]
pub struct GameDetailSession {
    pub session_id: i32,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct GameDetail {
    pub id: i32,
    pub appid: Option<i32>,
    pub name: String,
    pub install_path: Option<String>,
    pub cover_path: Option<String>,
    pub is_installed: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub total_playtime_seconds: i64,
    pub total_playtime_formatted: String,
    pub last_played_at: Option<i64>,
    pub session_count: i64,
    pub average_session_seconds: i64,
    pub average_session_formatted: String,
    pub recent_sessions: Vec<GameDetailSession>,
}

#[tauri::command]
pub fn games_list(
    search: Option<String>,
    installed_only: Option<bool>,
    limit: Option<i64>,
) -> Result<Vec<GameListItem>, String> {
    let conn = get_db_connection()?;
    let limit = limit.unwrap_or(200).clamp(1, 1000);
    let installed_only = installed_only.map(|value| if value { 1_i64 } else { 0_i64 });

    let mut stmt = conn
        .prepare(
            "SELECT g.id,
                    g.appid,
                    g.name,
                    g.cover_path,
                    g.is_installed,
                    COALESCE(gs.total_playtime_seconds, 0) AS total_playtime_seconds,
                    gs.last_played_at,
                    COALESCE((SELECT COUNT(1) FROM game_sessions s WHERE s.game_id = g.id), 0) AS session_count
             FROM games g
             LEFT JOIN game_stats gs ON gs.game_id = g.id
             WHERE (?1 IS NULL OR g.name LIKE '%' || ?1 || '%')
               AND (?2 IS NULL OR g.is_installed = ?2)
             ORDER BY g.is_installed DESC, total_playtime_seconds DESC, g.name COLLATE NOCASE ASC
             LIMIT ?3",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![search, installed_only, limit], |row| {
            let total_playtime_seconds: i64 = row.get(5)?;
            Ok(GameListItem {
                id: row.get(0)?,
                appid: row.get(1)?,
                name: row.get(2)?,
                cover_path: row.get(3)?,
                is_installed: row.get::<_, i64>(4)? == 1,
                total_playtime_seconds,
                total_playtime_formatted: format_duration(total_playtime_seconds),
                last_played_at: row.get(6)?,
                session_count: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn game_detail(id: i64, recent_limit: Option<i64>) -> Result<GameDetail, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    let recent_limit = recent_limit.unwrap_or(20).clamp(1, 100);

    let mut game_stmt = conn
        .prepare(
            "SELECT g.id,
                    g.appid,
                    g.name,
                    g.install_path,
                    g.cover_path,
                    g.is_installed,
                    g.created_at,
                    g.updated_at,
                    COALESCE(gs.total_playtime_seconds, 0) AS total_playtime_seconds,
                    gs.last_played_at,
                    COALESCE((SELECT COUNT(1) FROM game_sessions s WHERE s.game_id = g.id), 0) AS session_count
             FROM games g
             LEFT JOIN game_stats gs ON gs.game_id = g.id
             WHERE g.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let (
        id,
        appid,
        name,
        install_path,
        cover_path,
        is_installed,
        created_at,
        updated_at,
        total_playtime_seconds,
        last_played_at,
        session_count,
    ): (
        i32,
        Option<i32>,
        String,
        Option<String>,
        Option<String>,
        i64,
        i64,
        i64,
        i64,
        Option<i64>,
        i64,
    ) = game_stmt
        .query_row([id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let average_session_seconds = if session_count > 0 {
        total_playtime_seconds / session_count
    } else {
        0
    };

    let mut sessions_stmt = conn
        .prepare(
            "SELECT id,
                    start_time,
                    end_time,
                    COALESCE(duration_seconds, COALESCE(end_time, ?2) - start_time) AS actual_duration
             FROM game_sessions
             WHERE game_id = ?1
             ORDER BY start_time DESC
             LIMIT ?3",
        )
        .map_err(|e| e.to_string())?;

    let sessions = sessions_stmt
        .query_map(params![id, now, recent_limit], |row| {
            let duration_seconds: i64 = row.get(3)?;
            Ok(GameDetailSession {
                session_id: row.get(0)?,
                start_time: row.get(1)?,
                end_time: row.get(2)?,
                duration_seconds,
                formatted: format_duration(duration_seconds),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(GameDetail {
        id,
        appid,
        name,
        install_path,
        cover_path,
        is_installed: is_installed == 1,
        created_at,
        updated_at,
        total_playtime_seconds,
        total_playtime_formatted: format_duration(total_playtime_seconds),
        last_played_at,
        session_count,
        average_session_seconds,
        average_session_formatted: format_duration(average_session_seconds),
        recent_sessions: sessions,
    })
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn format_duration(total_seconds: i64) -> String {
    let seconds = total_seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let remaining_seconds = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, remaining_seconds)
    } else {
        format!("{}s", remaining_seconds)
    }
}
