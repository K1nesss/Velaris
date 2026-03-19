use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};
use steamlocate::SteamDir;

// 存储每个库的文件修改时间
type LibraryFileMap = HashMap<String, u64>;

// 获取所有Steam库路径
fn get_steam_library_paths() -> Vec<String> {
    let mut paths = Vec::new();

    if let Ok(steam_dir) = SteamDir::locate() {
        if let Ok(libraries) = steam_dir.libraries() {
            for library in libraries.flatten() {
                if let Some(path_str) = library.path().to_str() {
                    paths.push(path_str.to_string());
                    println!("Found Steam library: {}", path_str);
                }
            }
        }
    }

    paths
}

// 获取文件的修改时间
fn get_file_mtime(path: &Path) -> Option<u64> {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

// 扫描所有库目录的ACF文件
fn scan_library_files(steam_path: &str) -> LibraryFileMap {
    let mut file_map = LibraryFileMap::new();
    let steamapps_path = Path::new(steam_path).join("steamapps");

    if !steamapps_path.exists() {
        return file_map;
    }

    let acf_pattern = steamapps_path.join("*.acf");

    if let Ok(patterns) = glob::glob(acf_pattern.to_str().unwrap_or("")) {
        for path in patterns.flatten() {
            if let Some(path_str) = path.to_str() {
                if let Some(mtime) = get_file_mtime(&path) {
                    file_map.insert(path_str.to_string(), mtime);
                }
            }
        }
    }

    file_map
}

// 文件监听服务
pub fn start_file_watcher(_steam_path: String) {
    println!("=== Starting File Watcher Service ===");

    let library_paths = get_steam_library_paths();

    if library_paths.is_empty() {
        println!("No Steam libraries found");
        return;
    }

    println!("Found {} Steam libraries", library_paths.len());

    let mut initial_maps: Vec<LibraryFileMap> = Vec::new();
    let mut total_files = 0;

    for path in &library_paths {
        let file_map = scan_library_files(path);
        total_files += file_map.len();
        println!("Library {}: {} ACF files", path, file_map.len());
        initial_maps.push(file_map);
    }

    println!("Total ACF files: {}", total_files);

    let library_paths_clone = library_paths.clone();
    thread::spawn(move || {
        println!("Scheduled detection started (every 30 minutes)");

        loop {
            thread::sleep(Duration::from_secs(1800)); // 30分钟

            let mut has_changes = false;

            for (i, path) in library_paths_clone.iter().enumerate() {
                let current_map = scan_library_files(path);

                if i < initial_maps.len() {
                    if current_map != initial_maps[i] {
                        has_changes = true;
                        if cfg!(debug_assertions) {
                            println!("Detected Steam library file changes: {}", path);
                        }
                    }

                    // 更新记录
                    initial_maps[i] = current_map;
                }
            }

            if has_changes {
                println!("Detected file changes, triggering Steam scan...");
                trigger_steam_scan();
            }
        }
    });
}

// 触发 Steam 扫描
fn trigger_steam_scan() {
    use crate::core::steam_scanner::steam_scan_print;

    match steam_scan_print() {
        Ok(_) => {
            println!("Steam scan completed");
        }
        Err(e) => {
            println!("Error triggering Steam scan: {}", e);
        }
    }
}
