# 架构与模块分层

## 总体架构（Tauri IPC）

本项目是典型的 **桌面壳 + 本地后端**：

- 前端：SvelteKit（渲染 UI、图表、表单）
- 后端：Rust（监控前台窗口、识别游戏、写入 SQLite、聚合统计、触发提醒）
- 通信：Tauri Commands（IPC `invoke`），不是 HTTP API

### 数据流（最重要）

1. `system.getForegroundIdentity` 读取当前前台窗口/进程信息（pid、exePath、exeName、windowTitle 等）。
2. `identities.match` 用规则把“当前 exe”映射到某个 `game_id`。
3. `monitor` 状态机根据前台变化创建/结束 `sessions`。
4. 统计/提醒/导出只从 `sessions` 计算（不引入第二套“时长事实表”）。

## Rust 后端建议分层（稳定、可扩展）

> 你可以先按文件夹拆，后面再细化。

- `domain/`（纯业务模型）
  - `Game`, `IdentityRule`, `Session`, `Tag`, `AlertRule`
  - 业务约束：例如 session 必须满足 `end >= start` 等

- `services/`（业务用例）
  - `MonitorService`：状态机、去抖/宽限（grace）逻辑、崩溃恢复（补齐 end_ts）
  - `StatsService`：日聚合、Top games、今日累计等
  - `AlertsService`：全局当天累计提醒（同一天只触发一次）
  - `SteamImportService`：读取本地 Steam library 并 upsert games

- `repo/`（数据访问层，SQL 集中管理）
  - `GamesRepo`, `IdentitiesRepo`, `SessionsRepo`, `TagsRepo`, `AlertsRepo`, `SettingsRepo`
  - 原则：Commands 不直接拼复杂 SQL，放到 repo/service，方便测试与迁移

- `infra/`（系统集成）
  - `win32_foreground`：获取前台窗口/进程路径
  - `autostart`：开机自启
  - `tray`：托盘菜单/事件
  - `clock`：统一时间来源（便于测试）

- `commands/`（Tauri commands 暴露层）
  - 只做参数解析/调用 service/返回结构化错误

## SQLite 并发与稳定性建议

- 推荐 **单写入者**：监控线程只发事件；DB 写入由一个专门的 worker 串行处理。
- 读操作（统计）也建议在 service 内集中，避免 UI 高频调用导致锁竞争。

## 可维护性边界（v1 必须守住）

- 不做“自动智能 AppID 识别”来当真源；v1 用 identities 规则保证可控。
- 不建第二套“累计时长事实表”，避免一致性问题；一切从 sessions 推导。
- 每个功能必须能映射到：一个 command + 一个 service +（可选）一个 repo 方法。
