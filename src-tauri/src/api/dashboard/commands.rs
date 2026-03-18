use rusqlite::{params, Connection};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::db::get_db_connection;

const LOCAL_TIME_OFFSET_SECONDS: i64 = 8 * 3600;

#[derive(Serialize)]
pub struct DurationSummary {
    pub seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct DailyChartItem {
    pub day: String,
    pub seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct CurrentPlayingGame {
    pub session_id: i32,
    pub game_id: i32,
    pub game_name: String,
    pub start_time: i64,
    pub current_duration_seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct DonutChartItem {
    pub game_id: i32,
    pub name: String,
    pub seconds: i64,
    pub formatted: String,
    pub percentage: f64,
}

#[derive(Serialize)]
pub struct RecentSessionItem {
    pub session_id: i32,
    pub game_id: i32,
    pub game_name: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct DashboardSnapshot {
    pub today: DurationSummary,
    pub week: DurationSummary,
    pub current_playing: Option<CurrentPlayingGame>,
    pub recent_sessions: Vec<RecentSessionItem>,
    pub daily_chart: Vec<DailyChartItem>,
    pub donut: Vec<DonutChartItem>,
}

#[tauri::command]
pub fn dashboard_snapshot(
    days: Option<i64>,
    donut_limit: Option<i64>,
    recent_limit: Option<i64>,
) -> Result<DashboardSnapshot, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();

    Ok(DashboardSnapshot {
        today: query_today_total(&conn, now)?,
        week: query_week_total(&conn, now)?,
        current_playing: query_current_playing(&conn, now)?,
        recent_sessions: query_recent_sessions(&conn, now, recent_limit.unwrap_or(8))?,
        daily_chart: query_daily_chart(&conn, now, days.unwrap_or(7))?,
        donut: query_donut_data(&conn, donut_limit.unwrap_or(10))?,
    })
}

#[tauri::command]
pub fn dashboard_today_total() -> Result<DurationSummary, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    query_today_total(&conn, now)
}

#[tauri::command]
pub fn dashboard_week_total() -> Result<DurationSummary, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    query_week_total(&conn, now)
}

#[tauri::command]
pub fn dashboard_daily_chart(days: Option<i64>) -> Result<Vec<DailyChartItem>, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    query_daily_chart(&conn, now, days.unwrap_or(7))
}

#[tauri::command]
pub fn dashboard_current_playing() -> Result<Option<CurrentPlayingGame>, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    query_current_playing(&conn, now)
}

fn query_current_playing(conn: &Connection, now: i64) -> Result<Option<CurrentPlayingGame>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.game_id, g.name, s.start_time
             FROM game_sessions s
             JOIN games g ON g.id = s.game_id
             WHERE s.end_time IS NULL
             ORDER BY s.start_time DESC
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row([], |row| {
        let start_time: i64 = row.get(3)?;
        let current_duration_seconds = (now - start_time).max(0);
        Ok(CurrentPlayingGame {
            session_id: row.get(0)?,
            game_id: row.get(1)?,
            game_name: row.get(2)?,
            start_time,
            current_duration_seconds,
            formatted: format_duration(current_duration_seconds),
        })
    });

    match result {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
pub fn dashboard_donut_data(limit: Option<i64>) -> Result<Vec<DonutChartItem>, String> {
    let conn = get_db_connection()?;
    query_donut_data(&conn, limit.unwrap_or(5))
}

fn query_donut_data(conn: &Connection, limit: i64) -> Result<Vec<DonutChartItem>, String> {
    let limit = limit.clamp(1, 20);

    let total_seconds: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_playtime_seconds), 0) FROM game_stats",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT gs.game_id, g.name, gs.total_playtime_seconds
             FROM game_stats gs
             JOIN games g ON g.id = gs.game_id
             WHERE gs.total_playtime_seconds > 0
             ORDER BY gs.total_playtime_seconds DESC, g.name ASC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![limit], |row| {
            let seconds: i64 = row.get(2)?;
            let percentage = if total_seconds > 0 {
                ((seconds as f64 / total_seconds as f64) * 10000.0).round() / 100.0
            } else {
                0.0
            };

            Ok(DonutChartItem {
                game_id: row.get(0)?,
                name: row.get(1)?,
                seconds,
                formatted: format_duration(seconds),
                percentage,
            })
        })
        .map_err(|e| e.to_string())?;

    let items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(items)
}

#[tauri::command]
pub fn dashboard_recent_sessions(limit: Option<i64>) -> Result<Vec<RecentSessionItem>, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    query_recent_sessions(&conn, now, limit.unwrap_or(10))
}

fn query_recent_sessions(
    conn: &Connection,
    now: i64,
    limit: i64,
) -> Result<Vec<RecentSessionItem>, String> {
    let limit = limit.clamp(1, 50);
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.game_id, g.name, s.start_time, s.end_time,
                    COALESCE(s.duration_seconds, COALESCE(s.end_time, ?1) - s.start_time) AS actual_duration
             FROM game_sessions s
             JOIN games g ON g.id = s.game_id
             ORDER BY s.start_time DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![now, limit], |row| {
            let duration_seconds: i64 = row.get(5)?;
            Ok(RecentSessionItem {
                session_id: row.get(0)?,
                game_id: row.get(1)?,
                game_name: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                duration_seconds,
                formatted: format_duration(duration_seconds),
            })
        })
        .map_err(|e| e.to_string())?;

    let items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(items)
}

fn query_today_total(conn: &Connection, now: i64) -> Result<DurationSummary, String> {
    let (day_start, day_end) = current_local_day_bounds(now);
    let seconds = query_overlap_duration(conn, day_start, day_end, now)?;

    Ok(DurationSummary {
        seconds,
        formatted: format_duration(seconds),
    })
}

fn query_week_total(conn: &Connection, now: i64) -> Result<DurationSummary, String> {
    let (week_start, week_end) = current_local_week_bounds(now);
    let seconds = query_overlap_duration(conn, week_start, week_end, now)?;

    Ok(DurationSummary {
        seconds,
        formatted: format_duration(seconds),
    })
}

fn query_daily_chart(conn: &Connection, now: i64, days: i64) -> Result<Vec<DailyChartItem>, String> {
    let days = days.clamp(1, 30);
    let (today_start, _) = current_local_day_bounds(now);

    let mut items = Vec::new();
    for offset in (0..days).rev() {
        let range_start = today_start - (offset * 86_400);
        let range_end = range_start + 86_400;
        let seconds = query_overlap_duration(conn, range_start, range_end, now)?;
        items.push(DailyChartItem {
            day: format_local_day(range_start),
            seconds,
            formatted: format_duration(seconds),
        });
    }

    Ok(items)
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn current_local_day_bounds(now_utc: i64) -> (i64, i64) {
    let local_now = now_utc + LOCAL_TIME_OFFSET_SECONDS;
    let day_start_local = local_now - local_now.rem_euclid(86_400);
    let day_start_utc = day_start_local - LOCAL_TIME_OFFSET_SECONDS;
    (day_start_utc, day_start_utc + 86_400)
}

fn current_local_week_bounds(now_utc: i64) -> (i64, i64) {
    let local_now = now_utc + LOCAL_TIME_OFFSET_SECONDS;
    let day_index = local_now.div_euclid(86_400);
    let weekday_from_monday = (day_index + 3).rem_euclid(7);
    let week_start_day_index = day_index - weekday_from_monday;
    let week_start_local = week_start_day_index * 86_400;
    let week_start_utc = week_start_local - LOCAL_TIME_OFFSET_SECONDS;
    (week_start_utc, week_start_utc + 7 * 86_400)
}

fn format_local_day(day_start_utc: i64) -> String {
    let local = day_start_utc + LOCAL_TIME_OFFSET_SECONDS;
    let day_index = local.div_euclid(86_400);
    let (year, month, day) = civil_from_days(day_index);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m, d)
}

fn query_overlap_duration(
    conn: &Connection,
    range_start: i64,
    range_end: i64,
    now_utc: i64,
) -> Result<i64, String> {
    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(SUM(
                CASE
                    WHEN COALESCE(end_time, ?3) <= ?1 OR start_time >= ?2 THEN 0
                    ELSE MIN(COALESCE(end_time, ?3), ?2) - MAX(start_time, ?1)
                END
            ), 0)
            FROM game_sessions",
        )
        .map_err(|e| e.to_string())?;

    let seconds = stmt
        .query_row(params![range_start, range_end, now_utc], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;

    Ok(seconds)
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
