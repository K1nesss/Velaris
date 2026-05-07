# Velaris

Velaris 是一个基于 Tauri + SvelteKit + Rust 的桌面游戏时长追踪工具。它会扫描本机游戏库、监听游戏进程、记录每次游玩会话，并在 Dashboard、Timeline、Analytics 和游戏库中展示统计结果。

当前项目以 Windows 桌面端为主要目标。

## 功能概览

- 自动扫描 Steam 已安装游戏，并同步游戏图标、库封面和详情页背景图。
- 支持手动添加游戏，可选择 exe、自动识别名称和图标，也可自定义封面与背景图。
- 自动监听游戏进程，记录开始时间、结束时间和本次游玩时长。
- Timeline 展示每条游戏记录，游戏头像优先使用已缓存的 Steam icon。
- 游戏库支持封面展示、详情页、隐藏、删除和恢复隐藏游戏。
- 设置页支持主题、动效、刷新间隔、默认统计区间、更新检查等配置。
- 支持 GitHub Release 更新提醒和 Tauri updater 一键更新。
- 支持本地数据导入导出，方便备份和迁移。

## 技术栈

- 前端：SvelteKit、TypeScript、Tailwind CSS、ECharts
- 桌面端：Tauri v2
- 后端：Rust
- 数据库：SQLite、rusqlite
- 更新：Tauri updater + GitHub Releases

## 开发环境

请先安装：

- Node.js LTS，建议 20+
- pnpm
- Rust stable
- Cargo Tauri 相关构建依赖

安装依赖：

```bash
pnpm install
```

如果 Windows PowerShell 执行策略影响 `pnpm`，可以使用：

```bash
pnpm.cmd install
```

## 常用命令

启动前端开发服务：

```bash
pnpm dev
```

启动 Tauri 桌面开发模式：

```bash
cargo tauri dev
```

前端类型检查：

```bash
pnpm check
```

Rust 检查：

```bash
cargo check
```

构建发布包：

```bash
cargo tauri build
```

Windows 常见构建产物目录：

- `src-tauri/target/release/bundle/nsis/`
- `src-tauri/target/release/bundle/msi/`

## 发布与更新

项目使用 GitHub Releases 作为安装包和更新清单的发布位置。当前 updater endpoint：

```text
https://github.com/K1nesss/Velaris/releases/latest/download/latest.json
```

发布新版本时需要：

1. 更新 `src-tauri/tauri.conf.json` 里的 `version`。
2. 使用 updater 私钥打包并生成签名更新产物。
3. 创建 GitHub Release，例如 `v0.1.0`。
4. 上传安装包 `.exe`、更新包和 `latest.json`。
5. 客户端检查更新时会读取 latest release 中的 `latest.json`。

本机 updater 私钥路径：

```text
C:\Users\swp\.tauri\velaris-updater.key
```

本机 updater 私钥密码路径：

```text
C:\Users\swp\.tauri\velaris-updater-password.txt
```

打包时可设置：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY=(Get-Content -Raw "C:\Users\swp\.tauri\velaris-updater.key")
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD=(Get-Content -Raw "C:\Users\swp\.tauri\velaris-updater-password.txt")
cargo tauri build
```

私钥和密码只能保存在本机或 CI Secret 中，不能提交到仓库。

更完整的发布说明见 [UPDATE_RELEASE_GUIDE.md](./UPDATE_RELEASE_GUIDE.md)。

## 分支建议

- `main`：稳定发布分支，用于 GitHub Release。
- `dev`：日常集成分支。
- `feature/*`：功能开发分支。
- `fix/*`：问题修复分支。

当前如果是个人开发，也可以先直接在 `main` 上发布稳定版本，等功能多人协作或发布节奏变复杂后再恢复 `dev -> main` 的合并流程。

## 数据与资源

Velaris 会在本地用户目录中保存数据库、缓存图标、Steam 封面、详情页背景图和手动添加游戏的资源。构建产物、缓存、数据库和密钥不应提交到 Git。

请勿提交：

- `build/`
- `src-tauri/target/`
- 本地数据库文件
- Steam API Key
- updater 私钥
- 其他用户隐私或本机路径敏感数据

## 未来规划

后续规划记录在 [FUTURE_ROADMAP.md](./FUTURE_ROADMAP.md)，包括多平台游戏库扫描、成就与媒体增强、目标系统、云同步、备份恢复、游戏洞察和更完整的更新体验。

## License

当前仓库用于个人开发与测试。正式公开发布前建议补充 License，并检查第三方依赖、Steam 资源缓存和图标资源的版权合规。
