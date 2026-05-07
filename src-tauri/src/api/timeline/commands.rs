use rusqlite::params;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::db::get_db_connection;

#[derive(Serialize)]
pub struct TimelineSessionItem {
    pub session_id: i32,
    pub game_id: i32,
    pub appid: Option<i32>,
    pub game_name: String,
    pub icon_path: Option<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_seconds: i64,
    pub formatted: String,
    pub is_active: bool,
}

#[tauri::command]
pub fn timeline_sessions(
    limit: Option<i64>,
    offset: Option<i64>,
    game_id: Option<i64>,
) -> Result<Vec<TimelineSessionItem>, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    let limit = limit.unwrap_or(200).clamp(1, 1000);
    let offset = offset.unwrap_or(0).max(0);

    let mut stmt = conn
        .prepare(
            "SELECT s.id,
                    s.game_id,
                    g.appid,
                    g.name,
                    g.icon_path,
                    s.start_time,
                    s.end_time,
                    COALESCE(s.duration_seconds, COALESCE(s.end_time, ?1) - s.start_time) AS actual_duration
             FROM game_sessions s
             JOIN games g ON g.id = s.game_id
             WHERE (?2 IS NULL OR s.game_id = ?2)
             ORDER BY s.start_time DESC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![now, game_id, limit, offset], |row| {
            let end_time: Option<i64> = row.get(6)?;
            let duration_seconds: i64 = row.get(7)?;
            Ok(TimelineSessionItem {
                session_id: row.get(0)?,
                game_id: row.get(1)?,
                appid: row.get(2)?,
                game_name: row.get(3)?,
                icon_path: row.get(4)?,
                start_time: row.get(5)?,
                end_time,
                duration_seconds,
                formatted: format_duration(duration_seconds),
                is_active: end_time.is_none(),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
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
