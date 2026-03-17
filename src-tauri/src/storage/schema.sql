CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    appid INTEGER UNIQUE,          -- Steam AppID
    name TEXT NOT NULL,
    install_path TEXT,             -- 安装路径，卸载后置为NULL
    cover_path TEXT,               -- 本地封面缓存路径
    is_installed INTEGER NOT NULL DEFAULT 1,  -- 1=已安装，0=已卸载
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,                 -- NULL 表示正在运行
    duration_seconds INTEGER,         -- 冗余字段，提高查询性能
    FOREIGN KEY (game_id) REFERENCES games(id)
);
CREATE TABLE game_stats (
    game_id INTEGER PRIMARY KEY,
    total_playtime_seconds INTEGER NOT NULL DEFAULT 0,
    last_played_at INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(id)
);