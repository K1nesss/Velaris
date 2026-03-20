# Velaris

> A desktop playtime tracker built with Tauri + SvelteKit.

---

## 中文介绍

Velaris 是一个面向 Steam 玩家设计的桌面应用，用于自动追踪游戏时长、统计会话数据，并以 Dashboard / Timeline / Analytics 的方式展示你的游玩习惯。

### 主要功能

- 自动检测 Steam 库与已安装游戏
- 自动记录游戏开始/结束会话
- Dashboard 实时展示当前游玩与核心统计
- Timeline 按时间线浏览会话记录
- Analytics 多维统计（日/周/时段/游戏占比）
- Games 游戏列表与详情页
- 中英文界面切换
- 托盘运行与窗口隐藏恢复
- 数据导入/导出

### 技术栈

- Frontend: SvelteKit + TypeScript + TailwindCSS + ECharts
- Desktop: Tauri v2
- Backend: Rust
- Storage: SQLite (rusqlite)

### 本地开发

1. 安装依赖

```bash
pnpm install
```

2. 启动开发模式

```bash
cargo tauri dev
```

### 打包发布

```bash
cargo tauri build
```

Windows 安装包通常输出在：

- `src-tauri/target/release/bundle/nsis/` (setup.exe)
- `src-tauri/target/release/bundle/msi/` (msi)

### 数据库位置说明

- 开发模式（debug）：项目本地数据库
- 发布模式（release）：用户目录数据库

这样可确保安装给其他用户后，数据目录稳定且可写。

### 测试版分发建议

- 将测试分支发布为 GitHub Pre-release
- 上传 `nsis` 目录下的 setup 安装器
- 不建议把可执行安装包直接提交到仓库代码历史

---

## English

Velaris is a desktop app for Steam players to automatically track play sessions, calculate playtime statistics, and visualize habits through Dashboard / Timeline / Analytics views.

### Highlights

- Auto-detect Steam libraries and installed games
- Auto-track session start/end
- Real-time Dashboard metrics and current playing card
- Timeline view for session history
- Analytics across day/week/hour/game share
- Games list and game detail pages
- Bilingual UI (Chinese / English)
- Tray support (hide/restore window)
- Database import/export

### Stack

- Frontend: SvelteKit + TypeScript + TailwindCSS + ECharts
- Desktop: Tauri v2
- Backend: Rust
- Storage: SQLite (rusqlite)

### Development

1. Install dependencies

```bash
pnpm install
```

2. Run app in development mode

```bash
cargo tauri dev
```

### Build

```bash
cargo tauri build
```

Windows installer outputs are typically:

- `src-tauri/target/release/bundle/nsis/` (setup.exe)
- `src-tauri/target/release/bundle/msi/` (msi)

### Database Path Behavior

- Debug builds: project-local database
- Release builds: user data directory database

This keeps development convenient while making installed builds reliable for end users.

---

## License

This project is currently for personal and testing use. Add a formal license before public/open-source distribution.
