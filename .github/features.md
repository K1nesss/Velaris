# 功能设计（v1 范围 + 用户流程）

## v1 必做（闭环）

- 前台窗口口径监控：自动创建/结束会话（session）
- Steam 游戏导入：离线读取本机 Steam 库，生成 games 列表
- 识别规则（identities）：把 exe 绑定到 game（否则计时不稳定）
- 统计：按天聚合（折线图数据源）+ Top games
- 标签：tag CRUD + game 绑定 tag
- 导出 CSV：导出 sessions（推荐）
- 托盘：开始/暂停监控、打开主界面、退出
- 开机自启：开/关
- 提醒：全局当天累计达到阈值提醒，且同一天只触发一次

## v1 明确不做（避免 scope creep）

- 非 Steam 游戏的完整流程（但模型要能扩展）
- 复杂统计（分钟级热力图、长期趋势模型、多维筛选等）
- 云同步/账号/多设备
- 依赖网络的自动匹配（Web API 智能识别）

## 核心页面/操作（建议最小 UI）

### 1) Dashboard（首页）

- 展示：今天累计时长 / 阈值
- 按钮：开始监控、停止监控

### 2) Games（游戏列表）

- 列表：Steam 导入的 games
- 详情：显示 tags、绑定的 identities 规则
- “一键绑定当前前台 exe”按钮：把当前前台进程生成 identity rule（或预填表单）

### 3) Analytics（统计）

- 折线图：按天 seconds
- Top 列表：时间区间内最长的游戏

### 4) Settings（设置）

- 监控：pollMs、切换宽限（grace）
- 开机自启：enabled
- 提醒：enabled + 阈值（小时/分钟输入，内部存秒）
- 导出：选择时间范围导出 sessions CSV

## 监控状态机（v1 最小但正确）

- 状态：`Idle`（未命中游戏） / `Playing(game_id, session_id)`
- 事件：前台变化 → identity 匹配结果变化
- 规则：
  - Idle -> Playing：命中游戏就创建 session（start_ts=now）
  - Playing(A) -> Playing(B)：结束 A 的 session，创建 B 的 session
  - Playing -> Idle：结束当前 session
  - 加 1–3 秒宽限（grace）避免瞬切导致碎片化
- 崩溃恢复：启动时把 `end_ts IS NULL` 的 session 用“应用启动时间”补齐（或另一个可解释策略）

## 提醒（全局当天累计）

- 口径：本地日历日（YYYY-MM-DD）
- 计算：把 sessions 裁剪到当天 [day_start, day_end) 区间求和（进行中 session 用 now）
- 去重：`alert_events` 唯一约束（同一天只触发一次）
