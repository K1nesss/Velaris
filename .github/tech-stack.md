# 技术栈与工程约定

## 技术栈

- 桌面：Tauri（IPC commands + Bundler）
- 后端：Rust
- Windows 能力：Win32 API（通过 windows-rs）
- 数据库：SQLite（rusqlite）
- 前端：SvelteKit + Tailwind CSS
- 图表：ECharts（必要时可补 uPlot）
- 包管理：pnpm

## 测试/生产（dev/release）区分约定

> 目标：避免“开发测试污染正式用户数据”。

- Rust：用 `cfg!(debug_assertions)` 区分 debug/release
- 前端：用 `import.meta.env.DEV` / `import.meta.env.PROD`
- 数据库路径：
  - debug 默认使用 `*.dev.sqlite3`
  - release 默认使用 `*.sqlite3`
  - 允许环境变量覆盖（例如 `PLAYTIME_DB_PATH`）便于测试

## Git 约定（最简单够用）

- `main`：永远可发布（能 build、能跑迁移）
- `feature/*`：按功能拆分开发
- 提交粒度：一次提交只做一个可描述变化（例如“新增 identities 表 + repo + command”）

## 代码组织约定（减少后期痛苦）

- Commands 只负责：参数校验、调用 service、返回错误
- SQL 集中在 repo/service：避免散落在 commands 里
- 统计/提醒只依赖 sessions：不维护第二份“累计时长事实表”

## 构建/产物目录提示

- `node_modules/`、`src-tauri/target/`、`build/`、`.svelte-kit/` 属于依赖/构建产物，一般不进 Git。
