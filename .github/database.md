# 数据库设计（SQLite）

## 设计原则

- `sessions` 是唯一事实来源：统计、导出、提醒都从 sessions 推导。
- 识别稳定性靠 `identities` 显式规则，而不是“猜”。
- 提醒的稳定性靠 `alert_events` 的唯一约束，而不是内存标记。

## 表结构（v1）

### 1) games

用途：游戏条目（v1 以 Steam 为主，但为未来留扩展）。

- `id` INTEGER PK
- `source` TEXT NOT NULL（v1 固定 `steam`）
- `steam_appid` INTEGER NULL
- `name` TEXT NOT NULL
- `created_at` INTEGER NOT NULL
- `updated_at` INTEGER NOT NULL
  约束：`UNIQUE(source, steam_appid)`

### 2) identities（关键）

用途：把观察到的 exe 映射为 game。

- `id` INTEGER PK
- `game_id` INTEGER NOT NULL FK → games(id)
- `match_type` TEXT NOT NULL（v1：`exe_path_exact`、`exe_name`）
- `match_value` TEXT NOT NULL
- `priority` INTEGER NOT NULL DEFAULT 0
- `enabled` INTEGER NOT NULL DEFAULT 1
- `created_at` INTEGER NOT NULL
  索引：`(match_type, match_value)`、`(game_id)`

### 3) sessions（事实源）

用途：一次连续游玩区间。

- `id` INTEGER PK
- `game_id` INTEGER NOT NULL FK → games(id)
- `start_ts` INTEGER NOT NULL
- `end_ts` INTEGER NULL（进行中为 NULL）
- `source` TEXT NOT NULL（monitor/manual/import）
- `created_at` INTEGER NOT NULL
  约束：`end_ts IS NULL OR end_ts >= start_ts`
  索引：`(game_id, start_ts)`、`(start_ts)`

### 4) tags / game_tags

- `tags(id PK, name TEXT UNIQUE, created_at INTEGER)`
- `game_tags(game_id FK, tag_id FK, PRIMARY KEY(game_id, tag_id))`

### 5) settings

用途：配置与开关，value 存 JSON 字符串。

- `key` TEXT PRIMARY KEY
- `value` TEXT NOT NULL
- `updated_at` INTEGER NOT NULL

建议 key：

- `monitor.pollMs`
- `monitor.switchGraceSec`
- `autostart.enabled`
- `alerts.globalDailySeconds`

### 6) alert_rules / alert_events（全局当天累计提醒）

`alert_rules`

- `id` INTEGER PK
- `scope` TEXT NOT NULL（v1 固定 `global`）
- `scope_id` INTEGER NULL
- `daily_seconds` INTEGER NOT NULL
- `enabled` INTEGER NOT NULL
- `created_at` INTEGER NOT NULL
- `updated_at` INTEGER NOT NULL

`alert_events`

- `id` INTEGER PK
- `rule_id` INTEGER NOT NULL FK → alert_rules(id)
- `day` TEXT NOT NULL（本地日历日：YYYY-MM-DD）
- `triggered_at` INTEGER NOT NULL
- `payload` TEXT NULL（JSON：可存触发时累计秒数等）
  约束：`UNIQUE(rule_id, day)`（保证同一天只触发一次）

## “今天累计秒数”的计算（用于提醒与展示）

设：

- `day_start` = 今天 00:00:00（本地）对应 unix 秒
- `day_end` = 明天 00:00:00（本地）对应 unix 秒
  对每条 session：
- `effective_start = max(start_ts, day_start)`
- `effective_end = min(coalesce(end_ts, now_ts), day_end)`
- 若 `effective_end > effective_start`，贡献 `effective_end - effective_start`

## 迁移建议

- 使用 `schema_migrations` 或类似机制记录版本号。
- 每次改表都写新 migration，保证 release 用户升级不丢数据。
