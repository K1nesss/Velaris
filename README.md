<p align="center">
  <img src="./icon_full.png" width="180" alt="Velaris" />
</p>

<h1 align="center">Velaris</h1>

<p align="center">
  一个面向 Windows 桌面端的游戏时长追踪工具。扫描游戏库、监听游戏进程、记录游玩会话，并用清晰的仪表盘、时间线和统计视图展示你的游戏习惯。
</p>

<p align="center">
  <a href="https://github.com/K1nesss/Velaris/releases/latest">
    <img alt="Latest release" src="https://img.shields.io/github/v/release/K1nesss/Velaris?style=flat-square&label=release" />
  </a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-2563eb?style=flat-square" />
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-v2-24c8db?style=flat-square" />
  <img alt="SvelteKit" src="https://img.shields.io/badge/SvelteKit-TypeScript-ff3e00?style=flat-square" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-backend-b7410e?style=flat-square" />
</p>

<p align="center">
  <a href="#功能概览">功能</a>
  ·
  <a href="#安装体验">安装</a>
  ·
  <a href="#开发环境">开发</a>
  ·
  <a href="#发布与更新">发布</a>
  ·
  <a href="./FUTURE_ROADMAP.md">未来规划</a>
</p>

## 功能概览

- 自动扫描 Steam 已安装游戏，并同步游戏图标、库封面和详情页背景图。
- 支持手动添加游戏，可选择 exe、自动识别名称和图标，也可自定义封面与背景图。
- 自动监听游戏进程，记录开始时间、结束时间和本次游玩时长。
- Timeline 展示每条游戏记录，游戏头像优先使用已缓存的 Steam icon。
- 游戏库支持封面展示、详情页、隐藏、删除和恢复隐藏游戏。
- 设置页支持主题、动效、刷新间隔、默认统计区间、更新检查等配置。
- 支持 GitHub Release 更新提醒和 Tauri updater 一键更新。
- 支持本地数据导入导出，方便备份和迁移。

## 安装体验

最新安装包在 GitHub Releases：

```text
https://github.com/K1nesss/Velaris/releases/latest
```

Windows 用户优先下载：

```text
Velaris_0.1.0_x64-setup.exe
```

应用内更新会读取 latest release 中的 `latest.json`，后续发布新版本时只需要上传新的安装包、签名文件和更新清单。

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
3. 创建 GitHub Release，例如 `v0.2.0`。
4. 上传安装包 `.exe`、签名文件和 `latest.json`。
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
