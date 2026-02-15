# Playtime Tracker — 设计文档索引

这里存放本项目（Tauri + Rust + SQLite + SvelteKit）的“可维护/可扩展”的 v1 设计说明，目的是：

- 让你随时能回答“现在在做什么、下一步做什么、为什么这样做”；
- 让实现可以按里程碑推进，减少返工；
- 让未来扩展到非 Steam 游戏时不推翻重来。

## 文档目录

- [架构/模块分层](./architecture.md)
- [功能设计（v1 范围 + 用户流程）](./features.md)
- [技术栈与工程约定](./tech-stack.md)
- [项目结构目录说明](./project-structure.md)
- [API 接口设计（Tauri Commands / IPC）](./api.md)
- [数据库设计（SQLite）](./database.md)
- [小白开发流程（只学过 HTML/CSS/JS）+ 官方文档链接](./beginner-dev-flow.md)

## v1 核心共识（你只要记住这三条）

1. **sessions 是唯一事实来源**：统计/导出/提醒全部从 sessions 推导。
2. **identities 规则是稳定识别的关键**：exe（路径/文件名）→ game 的可控映射。
3. **全局当天累计提醒要“同一天只触发一次”**：用 `alert_events` 表的唯一约束保证，避免重启后反复提醒。
