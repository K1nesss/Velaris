# API 接口设计（Tauri Commands / IPC）

> 说明：这里的 API 是“前端通过 `invoke` 调后端”的命令集合。你可以改命名，但建议保持分组结构。

## 约定

- 时间戳：unix 秒（INTEGER）
- 错误：建议统一返回 `{ code, message, details? }`

## A) system / monitor

### system.getForegroundIdentity

- 返回：`null | { pid, exePath, exeName, windowTitle? }`

### monitor.start

- 入参：`{ pollMs?: number }`
- 行为：启动监控循环

### monitor.stop

- 行为：停止监控

### monitor.status

- 返回：`{ running, pollMs, currentGameId?, currentSessionId?, sinceTs? }`

## B) steam

### steam.importInstalledGames

- 入参：`{ libraryPaths?: string[] }`
- 返回：`Game[]`
- 行为：离线解析 Steam 库并 upsert 到 games

## C) games / tags

### games.list

- 入参：`{ search?: string, limit?: number, offset?: number }`
- 返回：`Game[]`

### games.get

- 入参：`{ gameId: number }`
- 返回：`GameDetail`（含 tags、identities 等）

### tags.create / tags.list / tags.rename / tags.delete

### games.setTags

- 入参：`{ gameId: number, tagIds: number[] }`

## D) identities（识别规则）

### identities.list

- 入参：`{ gameId?: number }`
- 返回：`IdentityRule[]`

### identities.create

- 入参：`{ gameId, matchType, matchValue, priority? }`
- matchType（v1）：`exe_path_exact` | `exe_name`

### identities.update / identities.delete

### identities.suggestFromForeground

- 入参：`{ gameId: number }`
- 返回：`null | { matchType, matchValue }`

### identities.testMatch

- 入参：`{ exePath, exeName, windowTitle? }`
- 返回：`{ matchedGameId?: number, matchedRuleId?: number }`

## E) sessions / stats

### sessions.list

- 入参：`{ fromTs, toTs, gameId?, limit?, offset? }`
- 返回：`Session[]`

### sessions.addManual / sessions.update / sessions.delete

### stats.daily

- 入参：`{ fromTs, toTs, gameId?, tagId? }`
- 返回：`{ day: string, seconds: number }[]`

### stats.topGames

- 入参：`{ fromTs, toTs, limit? }`
- 返回：`{ gameId, name, seconds }[]`

## F) export

### export.sessionsCsv

- 入参：`{ fromTs, toTs, gameId? }`
- 返回：`string`（CSV 内容）或 `{ path }`（落盘路径）

## G) alerts（全局当天累计）

### alerts.rules.getGlobalDaily

- 返回：`AlertRule`

### alerts.rules.setGlobalDaily

- 入参：`{ dailySeconds: number, enabled: boolean }`
- 返回：`AlertRule`

### alerts.dailyTotal

- 入参：`{ day?: string }`
- 返回：`{ day: string, seconds: number }`

### alerts.checkNow

- 返回：`{ triggered: boolean, day: string, seconds: number, thresholdSeconds: number }`
- 行为：计算今天累计；满足阈值且今天未触发则写入 alert_events
