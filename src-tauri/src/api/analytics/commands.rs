use rusqlite::{params, Connection};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::db::get_db_connection;

const LOCAL_TIME_OFFSET_SECONDS: i64 = 8 * 3600;

#[derive(Serialize)]
pub struct AnalyticsSummary {
    pub total_seconds: i64,
    pub total_formatted: String,
    pub session_count: i64,
    pub average_session_seconds: i64,
    pub average_session_formatted: String,
    pub active_days: i64,
    pub longest_session_seconds: i64,
    pub longest_session_formatted: String,
}

#[derive(Serialize)]
pub struct AnalyticsDailyItem {
    pub day: String,
    pub seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct AnalyticsHourlyItem {
    pub hour: i32,
    pub label: String,
    pub seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct AnalyticsTopGameItem {
    pub game_id: i32,
    pub name: String,
    pub seconds: i64,
    pub formatted: String,
    pub session_count: i64,
    pub percentage: f64,
    pub last_played_at: Option<i64>,
}

#[derive(Serialize)]
pub struct AnalyticsWeekdayItem {
    pub weekday: i32,
    pub label: String,
    pub seconds: i64,
    pub formatted: String,
    pub sessions: i64,
}

#[derive(Serialize)]
pub struct AnalyticsSnapshot {
    pub summary: AnalyticsSummary,
    pub daily: Vec<AnalyticsDailyItem>,
    pub hourly: Vec<AnalyticsHourlyItem>,
    pub top_games: Vec<AnalyticsTopGameItem>,
    pub weekday_breakdown: Vec<AnalyticsWeekdayItem>,
}

#[tauri::command]
pub fn analytics_snapshot(
    days: Option<i64>,
    top_limit: Option<i64>,
) -> Result<AnalyticsSnapshot, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    let days = days.unwrap_or(30).clamp(1, 365);
    let top_limit = top_limit.unwrap_or(8).clamp(3, 20);

    let (today_start, _) = current_local_day_bounds(now);
    let range_start = today_start - ((days - 1) * 86_400);
    let range_end = today_start + 86_400;

    let daily = query_daily(&conn, now, days)?;
    let total_seconds = daily.iter().map(|item| item.seconds).sum::<i64>();

    Ok(AnalyticsSnapshot {
        summary: query_summary(&conn, now, range_start, range_end, total_seconds)?,
        daily,
        hourly: query_hourly_today(&conn, now)?,
        top_games: query_top_games(&conn, now, range_start, range_end, top_limit, total_seconds)?,
        weekday_breakdown: query_weekday_breakdown(&conn, now, days, range_start, range_end)?,
    })
}

fn query_summary(
    conn: &Connection,
    now: i64,
    range_start: i64,
    range_end: i64,
    total_seconds: i64,
) -> Result<AnalyticsSummary, String> {
    let (session_count, longest_session_seconds): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(1),
                    COALESCE(MAX(COALESCE(duration_seconds, COALESCE(end_time, ?3) - start_time)), 0)
             FROM game_sessions
             WHERE start_time < ?2 AND COALESCE(end_time, ?3) > ?1",
            params![range_start, range_end, now],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let active_days: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT ((start_time + ?1) / 86400))
             FROM game_sessions
             WHERE start_time >= ?2 AND start_time < ?3",
            params![LOCAL_TIME_OFFSET_SECONDS, range_start, range_end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let average_session_seconds = if session_count > 0 {
        total_seconds / session_count
    } else {
        0
    };

    Ok(AnalyticsSummary {
        total_seconds,
        total_formatted: format_duration(total_seconds),
        session_count,
        average_session_seconds,
        average_session_formatted: format_duration(average_session_seconds),
        active_days,
        longest_session_seconds,
        longest_session_formatted: format_duration(longest_session_seconds),
    })
}

fn query_daily(conn: &Connection, now: i64, days: i64) -> Result<Vec<AnalyticsDailyItem>, String> {
    let (today_start, _) = current_local_day_bounds(now);
    let mut items = Vec::new();

    for offset in (0..days).rev() {
        let day_start = today_start - (offset * 86_400);
        let day_end = day_start + 86_400;
        let seconds = query_overlap_duration(conn, day_start, day_end, now)?;
        items.push(AnalyticsDailyItem {
            day: format_local_day(day_start),
            seconds,
            formatted: format_duration(seconds),
        });
    }

    Ok(items)
}

fn query_hourly_today(conn: &Connection, now: i64) -> Result<Vec<AnalyticsHourlyItem>, String> {
    let (today_start, _) = current_local_day_bounds(now);
    let mut items = Vec::with_capacity(24);

    for hour in 0..24_i64 {
        let range_start = today_start + hour * 3_600;
        let range_end = range_start + 3_600;
        let seconds = query_overlap_duration(conn, range_start, range_end, now)?;
        items.push(AnalyticsHourlyItem {
            hour: hour as i32,
            label: format!("{:02}:00", hour),
            seconds,
            formatted: format_duration(seconds),
        });
    }

    Ok(items)
}

fn query_top_games(
    conn: &Connection,
    now: i64,
    range_start: i64,
    range_end: i64,
    top_limit: i64,
    total_seconds: i64,
) -> Result<Vec<AnalyticsTopGameItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT g.id,
                    g.name,
                    COALESCE(SUM(
                        CASE
                            WHEN COALESCE(s.end_time, ?3) <= ?1 OR s.start_time >= ?2 THEN 0
                            ELSE MIN(COALESCE(s.end_time, ?3), ?2) - MAX(s.start_time, ?1)
                        END
                    ), 0) AS playtime,
                    COUNT(s.id) AS session_count,
                    gs.last_played_at
             FROM games g
             JOIN game_sessions s ON s.game_id = g.id
             LEFT JOIN game_stats gs ON gs.game_id = g.id
             WHERE s.start_time < ?2 AND COALESCE(s.end_time, ?3) > ?1
             GROUP BY g.id, g.name, gs.last_played_at
             HAVING playtime > 0
             ORDER BY playtime DESC, g.name COLLATE NOCASE ASC
             LIMIT ?4",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![range_start, range_end, now, top_limit], |row| {
            let seconds: i64 = row.get(2)?;
            let percentage = if total_seconds > 0 {
                ((seconds as f64 / total_seconds as f64) * 10000.0).round() / 100.0
            } else {
                0.0
            };

            Ok(AnalyticsTopGameItem {
                game_id: row.get(0)?,
                name: row.get(1)?,
                seconds,
                formatted: format_duration(seconds),
                session_count: row.get(3)?,
                percentage,
                last_played_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn query_weekday_breakdown(
    conn: &Connection,
    now: i64,
    days: i64,
    range_start: i64,
    range_end: i64,
) -> Result<Vec<AnalyticsWeekdayItem>, String> {
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let mut seconds_by_weekday = [0_i64; 7];

    let (today_start, _) = current_local_day_bounds(now);
    for offset in (0..days).rev() {
        let day_start = today_start - (offset * 86_400);
        let day_end = day_start + 86_400;
        let local_day_index = (day_start + LOCAL_TIME_OFFSET_SECONDS).div_euclid(86_400);
        let weekday = ((local_day_index + 3).rem_euclid(7)) as usize;
        let seconds = query_overlap_duration(conn, day_start, day_end, now)?;
        seconds_by_weekday[weekday] += seconds;
    }

    let mut sessions_by_weekday = [0_i64; 7];
    let mut stmt = conn
        .prepare(
            "SELECT ((start_time + ?1) / 86400 + 3) % 7 AS weekday, COUNT(1)
             FROM game_sessions
             WHERE start_time >= ?2 AND start_time < ?3
             GROUP BY weekday",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![LOCAL_TIME_OFFSET_SECONDS, range_start, range_end], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (weekday, count) = row.map_err(|e| e.to_string())?;
        if (0..=6).contains(&weekday) {
            sessions_by_weekday[weekday as usize] = count;
        }
    }

    let mut output = Vec::with_capacity(7);
    for index in 0..7 {
        let seconds = seconds_by_weekday[index];
        output.push(AnalyticsWeekdayItem {
            weekday: index as i32,
            label: labels[index].to_string(),
            seconds,
            formatted: format_duration(seconds),
            sessions: sessions_by_weekday[index],
        });
    }

    Ok(output)
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

    stmt.query_row(params![range_start, range_end, now_utc], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())
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