use reqwest::Client;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use crate::storage::db::get_db_connection;

const MEDIA_FAILURE_RETRY_SECONDS: i64 = 30 * 24 * 60 * 60;

#[derive(Serialize)]
pub struct SteamIconSyncSummary {
    pub matched_games: usize,
    pub downloaded_icons: usize,
    pub downloaded_covers: usize,
    pub downloaded_heroes: usize,
    pub updated_games: usize,
}

#[derive(Deserialize)]
struct OwnedGamesPayload {
    response: OwnedGamesResponse,
}

#[derive(Deserialize)]
struct OwnedGamesResponse {
    games: Option<Vec<OwnedGame>>,
}

#[derive(Deserialize)]
struct OwnedGame {
    appid: i32,
    img_icon_url: Option<String>,
}

#[derive(Clone)]
struct LocalGameMediaTarget {
    id: i32,
    appid: i32,
    needs_icon: bool,
    needs_cover: bool,
    needs_hero: bool,
}

#[tauri::command]
pub async fn sync_steam_owned_game_icons(
    app: AppHandle,
    api_key: String,
    steam64_id: String,
) -> Result<SteamIconSyncSummary, String> {
    let api_key = api_key.trim();
    let steam64_id = steam64_id.trim();

    if api_key.is_empty() || steam64_id.is_empty() {
        return Ok(SteamIconSyncSummary {
            matched_games: 0,
            downloaded_icons: 0,
            downloaded_covers: 0,
            downloaded_heroes: 0,
            updated_games: 0,
        });
    }

    let app_cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to resolve app cache directory: {}", e))?;
    let icon_cache_dir = app_cache_dir.join("steam-icons");
    let library_cache_dir = app_cache_dir.join("steam-library");
    fs::create_dir_all(&icon_cache_dir)
        .map_err(|e| format!("Failed to create Steam icon cache directory: {}", e))?;
    fs::create_dir_all(&library_cache_dir)
        .map_err(|e| format!("Failed to create Steam library cache directory: {}", e))?;

    let targets = load_games_missing_media()?;
    if targets.is_empty() {
        return Ok(SteamIconSyncSummary {
            matched_games: 0,
            downloaded_icons: 0,
            downloaded_covers: 0,
            downloaded_heroes: 0,
            updated_games: 0,
        });
    }

    let target_appids = targets
        .iter()
        .map(|target| target.appid)
        .collect::<HashSet<_>>();
    let owned_games = fetch_owned_games(api_key, steam64_id).await?;
    let icon_hashes = owned_games
        .into_iter()
        .filter(|game| target_appids.contains(&game.appid))
        .filter_map(|game| {
            let icon_hash = game.img_icon_url?.trim().to_string();
            if icon_hash.is_empty() {
                None
            } else {
                Some((game.appid, icon_hash))
            }
        })
        .collect::<HashMap<_, _>>();

    let client = Client::new();
    let mut downloaded_icons = 0;
    let mut downloaded_covers = 0;
    let mut downloaded_heroes = 0;
    let mut updates = Vec::new();

    for target in targets {
        let mut icon_path = None;
        let mut cover_path = None;
        let mut hero_path = None;

        if target.needs_icon {
            if let Some(icon_hash) = icon_hashes.get(&target.appid) {
                let next_icon_path =
                    icon_cache_dir.join(format!("{}_{}.jpg", target.appid, icon_hash));
                if !next_icon_path.exists() && !should_skip_media_download(target.appid, "icon")? {
                    let url = format!(
                        "https://media.steampowered.com/steamcommunity/public/images/apps/{}/{}.jpg",
                        target.appid, icon_hash
                    );
                    if let Err(error) = download_media(&client, &url, &next_icon_path).await {
                        eprintln!(
                            "Failed to cache Steam icon for appid {}: {}",
                            target.appid, error
                        );
                        record_media_failure(target.appid, "icon", &error)?;
                    } else {
                        clear_media_failure(target.appid, "icon")?;
                        downloaded_icons += 1;
                    }
                }

                if next_icon_path.exists() {
                    icon_path = Some(next_icon_path.to_string_lossy().to_string());
                }
            }
        }

        if target.needs_cover {
            let next_cover_path =
                library_cache_dir.join(format!("{}_library_600x900.jpg", target.appid));
            if !next_cover_path.exists() && !should_skip_media_download(target.appid, "cover")? {
                let url = format!(
                    "https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900.jpg",
                    target.appid
                );
                if let Err(error) = download_media(&client, &url, &next_cover_path).await {
                    eprintln!(
                        "Failed to cache Steam library image for appid {}: {}",
                        target.appid, error
                    );
                    record_media_failure(target.appid, "cover", &error)?;
                } else {
                    clear_media_failure(target.appid, "cover")?;
                    downloaded_covers += 1;
                }
            }

            if next_cover_path.exists() {
                cover_path = Some(next_cover_path.to_string_lossy().to_string());
            }
        }

        if target.needs_hero {
            let next_hero_path =
                library_cache_dir.join(format!("{}_library_hero.jpg", target.appid));
            if !next_hero_path.exists() && !should_skip_media_download(target.appid, "hero")? {
                let url = format!(
                    "https://cdn.akamai.steamstatic.com/steam/apps/{}/library_hero.jpg",
                    target.appid
                );
                if let Err(error) = download_media(&client, &url, &next_hero_path).await {
                    eprintln!(
                        "Failed to cache Steam hero image for appid {}: {}",
                        target.appid, error
                    );
                    record_media_failure(target.appid, "hero", &error)?;
                } else {
                    clear_media_failure(target.appid, "hero")?;
                    downloaded_heroes += 1;
                }
            }

            if next_hero_path.exists() {
                hero_path = Some(next_hero_path.to_string_lossy().to_string());
            }
        }

        if icon_path.is_some() || cover_path.is_some() || hero_path.is_some() {
            updates.push(GameMediaUpdate {
                id: target.id,
                icon_path,
                cover_path,
                hero_path,
            });
        }
    }

    let updated_games = update_game_media_paths(&updates)?;

    Ok(SteamIconSyncSummary {
        matched_games: icon_hashes.len(),
        downloaded_icons,
        downloaded_covers,
        downloaded_heroes,
        updated_games,
    })
}

fn load_games_missing_media() -> Result<Vec<LocalGameMediaTarget>, String> {
    let conn = get_db_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, appid, icon_path, cover_path, hero_path
             FROM games
             WHERE appid IS NOT NULL
               AND is_hidden = 0
               AND NOT EXISTS (
                   SELECT 1 FROM ignored_games ignored WHERE ignored.appid = games.appid
               )",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut targets = Vec::new();
    for row in rows {
        let (id, appid, icon_path, cover_path, hero_path) = row.map_err(|e| e.to_string())?;
        let has_icon = icon_path
            .as_deref()
            .map(|path| Path::new(path).exists())
            .unwrap_or(false);
        let has_cover = cover_path
            .as_deref()
            .map(|path| Path::new(path).exists())
            .unwrap_or(false);
        let has_hero = hero_path
            .as_deref()
            .map(|path| Path::new(path).exists())
            .unwrap_or(false);

        if !has_icon || !has_cover || !has_hero {
            targets.push(LocalGameMediaTarget {
                id,
                appid,
                needs_icon: !has_icon,
                needs_cover: !has_cover,
                needs_hero: !has_hero,
            });
        }
    }

    Ok(targets)
}

async fn fetch_owned_games(api_key: &str, steam64_id: &str) -> Result<Vec<OwnedGame>, String> {
    let payload = Client::new()
        .get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/")
        .query(&[
            ("key", api_key),
            ("steamid", steam64_id),
            ("include_appinfo", "true"),
            ("include_played_free_games", "true"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to request Steam owned games: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Steam owned games request failed: {}", e))?
        .json::<OwnedGamesPayload>()
        .await
        .map_err(|e| format!("Failed to parse Steam owned games response: {}", e))?;

    Ok(payload.response.games.unwrap_or_default())
}

async fn download_media(client: &Client, url: &str, path: &Path) -> Result<(), String> {
    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download Steam media: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Steam media download failed: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read Steam media bytes: {}", e))?;

    fs::write(path, bytes).map_err(|e| format!("Failed to write Steam media cache: {}", e))
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn should_skip_media_download(appid: i32, asset_type: &str) -> Result<bool, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    let retry_after = conn
        .query_row(
            "SELECT retry_after FROM steam_media_failures WHERE appid = ?1 AND asset_type = ?2",
            params![appid, asset_type],
            |row| row.get::<_, i64>(0),
        )
        .ok();

    Ok(retry_after.map(|value| value > now).unwrap_or(false))
}

fn record_media_failure(appid: i32, asset_type: &str, error: &str) -> Result<(), String> {
    let conn = get_db_connection()?;
    let failed_at = current_unix_seconds();
    let retry_after = failed_at + MEDIA_FAILURE_RETRY_SECONDS;

    conn.execute(
        "INSERT INTO steam_media_failures (appid, asset_type, failed_at, retry_after, last_error)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(appid, asset_type) DO UPDATE SET
         failed_at = excluded.failed_at,
         retry_after = excluded.retry_after,
         last_error = excluded.last_error",
        params![appid, asset_type, failed_at, retry_after, error],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn clear_media_failure(appid: i32, asset_type: &str) -> Result<(), String> {
    let conn = get_db_connection()?;
    conn.execute(
        "DELETE FROM steam_media_failures WHERE appid = ?1 AND asset_type = ?2",
        params![appid, asset_type],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

struct GameMediaUpdate {
    id: i32,
    icon_path: Option<String>,
    cover_path: Option<String>,
    hero_path: Option<String>,
}

fn update_game_media_paths(updates: &[GameMediaUpdate]) -> Result<usize, String> {
    if updates.is_empty() {
        return Ok(0);
    }

    let mut conn = get_db_connection()?;
    let transaction = conn.transaction().map_err(|e| e.to_string())?;
    let mut updated = 0;

    for update in updates {
        if let Some(icon_path) = &update.icon_path {
            updated += transaction
                .execute(
                    "UPDATE games SET icon_path = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                    params![icon_path, update.id],
                )
                .map_err(|e| e.to_string())?;
        }

        if let Some(cover_path) = &update.cover_path {
            updated += transaction
                .execute(
                    "UPDATE games SET cover_path = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                    params![cover_path, update.id],
                )
                .map_err(|e| e.to_string())?;
        }

        if let Some(hero_path) = &update.hero_path {
            updated += transaction
                .execute(
                    "UPDATE games SET hero_path = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                    params![hero_path, update.id],
                )
                .map_err(|e| e.to_string())?;
        }
    }

    transaction.commit().map_err(|e| e.to_string())?;
    Ok(updated)
}
