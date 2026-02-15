# 小白开发流程（只学过 HTML/CSS/JS）+ 官方文档

## 你的最佳推进方式：纵向切片（先闭环再加功能）

你这个项目难点不是 UI，而是：
**前台识别 → identities 匹配 → sessions 落库 → 统计/提醒**。

建议你按下面顺序走，每一步都能“跑起来看结果”。

## 阶段 0：跑通环境（目标：1 天内）

- `pnpm i`
- `pnpm dev`（前端先能打开）
- `pnpm tauri dev`（能起桌面壳）

学到的最少知识：会运行命令、会看控制台日志。

## 阶段 1：最小闭环（目标：你会立刻看到“今天累计”）

闭环定义：**拿到前台 exe → 写入 sessions → 前端展示今天累计**

1. DB：建 `games/identities/sessions` + 迁移机制
2. system：实现 `system.getForegroundIdentity`
3. identities：实现匹配逻辑（先支持 `exe_path_exact`、`exe_name`）
4. monitor：状态机自动开关 session（加 1–3 秒防抖）
5. stats：实现 “today total seconds” 计算接口
6. UI：只做一个页面展示今天累计 + Start/Stop

## 阶段 2：让它“可用”（Steam 导入 + 绑定体验）

- 实现 `steam.importInstalledGames`
- UI 做 Games 列表
- 做“一键绑定当前前台 exe”为某游戏 identity rule

## 阶段 3：补齐 v1 功能

- tags
- analytics（折线图 + top）
- export CSV
- alerts（全局当天累计 + 一天只触发一次）
- tray / autostart

## 学习路线（最少学习换最大产出）

你不需要先把 Rust 全学完，按需求补就够。

### 必学 1：SvelteKit + Tauri IPC

- Svelte 教程（最适合入门）：https://svelte.dev/tutorial
- Svelte 文档：https://svelte.dev/docs
- SvelteKit 文档（路由、layout、load 等）：https://svelte.dev/docs/kit
- Tauri 文档（commands、配置、打包）：https://tauri.app/
- Tauri Reference（v2 配置/能力）：https://tauri.app/reference/

### 必学 2：SQLite/SQL

- SQLite 官方文档：https://www.sqlite.org/docs.html
- SQLite date/time 相关（聚合会用到）：https://www.sqlite.org/lang_datefunc.html

### 必学 3：Rust 最小集合

- The Rust Book：https://doc.rust-lang.org/book/
- Rust By Example：https://doc.rust-lang.org/rust-by-example/
- rusqlite 文档：https://docs.rs/rusqlite/

### Windows API（先会用再深究）

- windows-rs：https://github.com/microsoft/windows-rs
- Microsoft Learn Win32 API：https://learn.microsoft.com/windows/win32/api/

## 你可以照抄的“第一周目标”

- 第 1–2 天：Svelte 教程基础 + 能调用一次 `invoke`
- 第 3–4 天：SQLite 表 + 插入假 sessions + 统计今天累计
- 第 5–7 天：接入 Win32 前台识别 + identities 匹配 + 自动写 sessions
