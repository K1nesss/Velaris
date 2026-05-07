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
  <a href="#安装">安装</a>
  ·
  <a href="#首次配置">首次配置</a>
  ·
  <a href="#使用教程">教程</a>
  ·
  <a href="#开发">开发</a>
  ·
  <a href="#贡献">贡献</a>
</p>

## 功能概览

- 自动扫描 Steam 已安装游戏，并同步游戏图标、库封面和详情页背景图。
- 支持手动添加游戏，可选择 exe、自动识别名称和图标，也可自定义封面与背景图。
- 自动监听游戏进程，记录开始时间、结束时间和本次游玩时长。
- Timeline 展示每条游戏记录，游戏头像优先使用已缓存的 Steam icon。
- 游戏库支持封面展示、详情页、隐藏、删除和恢复隐藏游戏。
- 游戏详情页支持查看游戏信息、背景图、游玩记录和记录删除。
- Dashboard 展示总时长、最近游玩、当前正在运行的游戏和趋势图。
- Analytics 展示不同时间区间下的游玩统计。
- 设置页支持主题、动效、刷新间隔、默认统计区间、更新检查等配置。
- 支持应用内更新提醒和一键更新。
- 支持本地数据导入导出，方便备份和迁移。

## 安装

最新安装包在 GitHub Releases：

```text
https://github.com/K1nesss/Velaris/releases/latest
```

Windows 用户优先下载：

```text
Velaris_0.1.0_x64-setup.exe
```

安装后正常启动 Velaris。首次启动时，如果系统提示安全确认，请确认来源是本仓库 Release 中下载的安装包。

## 首次配置

Velaris 可以不配置 Steam API Key 直接使用基础记录功能；但如果你想同步 Steam 游戏库图标、封面、详情页背景图等媒体资源，需要在设置页填写 Steam API Key 和 Steam64 ID。

### 获取 Steam API Key

1. 打开 Steam Web API Key 页面：

```text
https://steamcommunity.com/dev/apikey
```

2. 登录你的 Steam 账号。
3. 根据页面提示申请或查看 API Key。
4. 如果页面要求填写 Domain Name，个人本地使用可以填写你自己的域名；没有公开域名时可按页面要求填写本地用途标识，例如 `localhost`。
5. 复制生成的 API Key。

注意：

- Steam API Key 等同于访问你 Steam Web API 数据的凭证，不要公开分享。
- 不要把 API Key 提交到 GitHub、论坛、截图或日志里。
- 如果怀疑泄露，回到 Steam Web API Key 页面重新生成或撤销。

Steam 官方文档说明，使用 Steam Web API 需要 API Key，并且需要同意 Steam API Terms of Use。

### 获取 Steam64 ID

Steam64 ID 是一串 17 位左右的数字，常见格式类似：

```text
76561198000000000
```

推荐方法：

1. 打开 Steam 客户端或浏览器中的 Steam 个人资料页。
2. 复制你的个人资料链接。
3. 如果链接类似下面这样，末尾数字就是 Steam64 ID：

```text
https://steamcommunity.com/profiles/76561198000000000
```

4. 如果链接类似下面这样，说明你使用的是自定义 URL：

```text
https://steamcommunity.com/id/yourname
```

这种情况下可以用 Steam ID 查询工具把自定义 URL 转换成 Steam64 ID，例如：

```text
https://steamid.io/lookup/
```

也可以在拿到 Steam API Key 后，通过 Steam 的 ResolveVanityURL 接口解析自定义 URL。

### 在 Velaris 中填写

1. 打开 Velaris。
2. 进入设置页。
3. 找到 Steam 相关配置。
4. 填入 Steam API Key。
5. 填入 Steam64 ID。
6. 返回游戏库或 Timeline，应用会在需要时同步 Steam 游戏信息和媒体资源。

如果同步失败，优先检查：

- API Key 是否复制完整。
- Steam64 ID 是否是纯数字格式。
- Steam 个人资料和游戏详情是否允许公开访问。
- 当前网络是否能访问 Steam API 和 Steam 图片 CDN。

## 使用教程

### Dashboard

Dashboard 用来快速查看当前整体状态：

- 总游玩时长。
- 最近一次游玩记录。
- 今日或近期的游玩趋势。
- 当前正在运行的游戏。
- 最近游戏列表。

如果你正在运行某个已识别游戏，Velaris 会在 Dashboard 中显示当前游玩状态。

### 游戏库

游戏库用于集中管理所有游戏：

- Steam 扫描到的游戏会自动出现在列表中。
- 手动添加的游戏也会出现在列表中。
- 有 Steam 封面时优先展示封面。
- 没有封面时使用本地兜底显示。
- 点击游戏卡片可以进入详情页。

游戏详情页中可以查看：

- 游戏名称和基础信息。
- Steam library hero 背景图。
- 总游玩时长。
- 最近游玩记录。
- 单条记录删除操作。

删除记录前会出现确认弹窗，避免误删。如果记录仍在进行中，需要先停止游戏，不能删除活跃记录。

### 手动添加游戏

如果某个游戏不是 Steam 游戏，或者暂时无法被自动扫描，可以手动添加：

1. 进入游戏库页面。
2. 点击添加游戏。
3. 选择游戏 exe 文件。
4. Velaris 会尝试根据 exe 自动填写游戏名称。
5. 图标会优先从 exe 中提取。
6. 你可以手动修改名称。
7. 封面和背景图可以按需选择。
8. 保存后，Velaris 会通过进程匹配记录该游戏的游玩时长。

建议选择真正启动游戏的 exe，而不是启动器、更新器或卸载程序。这样进程识别会更准确。

### Timeline

Timeline 展示每一次游玩会话：

- 每条记录包含游戏、开始时间、结束时间和本次时长。
- Steam 游戏会优先使用缓存的游戏 icon。
- 可以按条件筛选记录。
- 支持加载更多历史记录。
- 单条记录可以删除，删除前会出现确认弹窗。

Timeline 适合回看“我什么时候玩了什么游戏”。

### Analytics

Analytics 用于查看统计分析：

- 按默认时间区间查看统计。
- 对比不同游戏的游玩占比。
- 查看不同日期或时间段的游玩趋势。
- 用于发现长期游玩习惯。

默认统计区间可以在设置页调整。

### 隐藏和删除游戏

游戏详情页中可以隐藏或删除游戏：

- 隐藏：从游戏库界面中隐藏，但保留数据，适合不想展示但以后可能恢复的游戏。
- 删除：用于移除不想继续保留的游戏条目，操作前会确认。

被隐藏的游戏可以在设置页的已隐藏游戏列表中恢复。恢复后，Velaris 会重新扫描并把游戏重新加入游戏库。

如果某个 Steam 工具或 Dedicated Server 不是你想记录的游戏，建议使用隐藏。这样后续扫描时不会频繁把它展示出来，也能减少无意义图片请求。

### 设置

设置页包含常用配置：

- 主题模式。
- 页面动效。
- Dashboard 刷新间隔。
- Timeline 每次加载条数。
- Analytics 默认区间。
- Steam API Key。
- Steam64 ID。
- 已隐藏游戏管理。
- 检查更新。

大部分设置会立即生效，不需要额外点击保存。

### 数据和缓存

Velaris 会在本地保存：

- SQLite 数据库。
- Steam 游戏 icon 缓存。
- Steam library 封面缓存。
- Steam hero 背景图缓存。
- 手动添加游戏的图标、封面和背景图。

这些数据只保存在本机。Steam API Key 和 Steam64 ID 也用于本机请求 Steam API，不应公开分享。

## 常见问题

### 为什么游戏库没有显示 Steam 游戏？

可能原因：

- Steam 没有安装在默认位置。
- Steam library 路径暂时没有被扫描到。
- 游戏没有安装。
- 游戏被隐藏。
- Steam 扫描还没有重新执行。

可以尝试重启 Velaris 或在恢复隐藏游戏后重新扫描。

### 为什么 Steam 图标或封面没有显示？

可能原因：

- 没有填写 Steam API Key 或 Steam64 ID。
- API Key 无效。
- Steam64 ID 填错。
- Steam 个人资料或游戏详情不是公开状态。
- 网络无法访问 Steam API 或图片 CDN。
- 某个条目不是正式游戏，而是工具、服务器或运行库，Steam 图片资源可能不存在。

Velaris 会缓存失败结果，避免对不存在的图片频繁请求。

### 为什么某些游戏记录不到时长？

可能原因：

- 手动添加时选择的不是实际运行中的 exe。
- 游戏通过启动器拉起了另一个真正的游戏进程。
- 游戏进程启动和退出太快。
- 路径或进程名发生变化。

手动添加游戏时，尽量选择最终游戏主程序 exe。

### API Key 会上传到服务器吗？

不会。Velaris 是本地桌面应用，API Key 用于本机向 Steam API 请求你的游戏库信息。不要把 API Key 发给其他人，也不要提交到公开仓库。

## 技术栈

- 前端：SvelteKit、TypeScript、Tailwind CSS、ECharts
- 桌面端：Tauri v2
- 后端：Rust
- 数据库：SQLite、rusqlite
- 更新：Tauri updater + GitHub Releases

## 开发

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

## 贡献

欢迎通过 Pull Request 参与改进。

建议流程：

1. Fork 本仓库。
2. 基于自己的 fork 创建功能分支。
3. 完成改动并在本地运行必要检查。
4. 提交 Pull Request 到本仓库。
5. 由维护者审核，通过后合并到主分支。

版本发布、安装包签名和 GitHub Releases 由维护者统一处理。贡献者不需要接触任何发布私钥或更新签名配置。

## 分支建议

- `main`：稳定主分支，用于合并已审核改动。
- `feature/*`：功能开发分支。
- `fix/*`：问题修复分支。

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

## 参考链接

- [Steam Web API Documentation](https://steamcommunity.com/dev)
- [Steam Web API Key](https://steamcommunity.com/dev/apikey)
- [SteamID I/O Lookup](https://steamid.io/lookup/)

## License

当前仓库用于个人开发与测试。正式公开发布前建议补充 License，并检查第三方依赖、Steam 资源缓存和图标资源的版权合规。
