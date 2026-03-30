# Velaris

面向 Steam 玩家的一体化桌面游玩时长追踪工具，基于 Tauri + SvelteKit + Rust。

本 README 为中文版开发者协作指南，目标是让新协作者在最短时间内完成环境搭建、参与开发并按统一流程合并代码。

---

## 1. 项目简介

Velaris 提供以下核心能力：

- 自动检测 Steam 库与已安装游戏
- 自动记录游戏开始/结束会话
- Dashboard 实时统计与当前游玩状态
- Timeline 会话时间线浏览
- Analytics 多维统计（日/周/时段/游戏占比）
- Games 游戏列表与详情
- 中英文切换、托盘运行、数据导入导出

---

## 2. 技术栈

- 前端：SvelteKit + TypeScript + TailwindCSS + ECharts
- 桌面层：Tauri v2
- 后端：Rust
- 存储：SQLite（rusqlite）

---

## 3. 开发环境要求

请确保本机已安装：

- Node.js LTS（建议 20+）
- pnpm
- Rust（stable）
- Cargo / Tauri 相关构建依赖

Windows 机器如果 PowerShell 执行策略阻止 pnpm，可使用：

```bash
pnpm.cmd install
```

---

## 4. 本地启动与常用命令

安装依赖：

```bash
pnpm install
```

前端开发（仅 Web）：

```bash
pnpm dev
```

桌面联调（Tauri + 前端）：

```bash
cargo tauri dev
```

代码检查：

```bash
pnpm check
pnpm lint
```

格式化：

```bash
pnpm format
```

构建发布包：

```bash
cargo tauri build
```

Windows 常见产物目录：

- src-tauri/target/release/bundle/nsis/
- src-tauri/target/release/bundle/msi/

---

## 5. Git 协作模型（推荐）

### 分支职责

- main：稳定/发布分支
- dev：团队集成分支（日常合并目标）
- feature/\*：功能开发分支
- fix/\*：问题修复分支
- test/\*：实验或验证分支（验证通过后再合并到 dev）

### 协作流程

1. 从 dev 拉取最新代码。
2. 基于 dev 创建个人分支开发。
3. 本地自测通过后推送个人分支。
4. 发起 PR 到 dev。
5. 由维护者审查并合并。
6. dev 稳定后再合并到 main。

示例命令：

```bash
git switch dev
git pull origin dev
git switch -c feature/xxx

# 开发并提交
git add .
git commit -m "feat(module): 描述本次改动"
git push -u origin feature/xxx
```

将测试分支合并到 dev：

```bash
git switch dev
git pull origin dev
git merge test/steam-path-fix
git push origin dev
```

---

## 6. 提交与 PR 规范

建议使用统一提交前缀：

- feat: 新功能
- fix: 修复问题
- refactor: 重构（无功能变化）
- perf: 性能优化
- docs: 文档变更
- chore: 工程维护

PR 描述建议包含：

- 改动目的
- 主要改动点
- 自测结果
- 风险与回滚方式（如有）
- UI 改动截图（如有）

---

## 7. 合并前检查清单

合并到 dev 前请确认：

- 已同步最新 dev 并解决冲突
- 本地可正常启动（cargo tauri dev）
- pnpm check 与 pnpm lint 通过
- 不包含无关文件、构建产物和敏感信息
- PR 描述完整，便于审查

---

## 8. 数据与目录说明

- 调试模式（debug）：使用项目本地数据库，便于开发验证。
- 发布模式（release）：使用用户目录数据库，保证安装版可写和稳定。

请勿提交以下内容：

- build 产物
- src-tauri/target 产物
- 本地缓存与临时文件
- 任何密钥、令牌、账号配置

---

## 9. 开源与许可证

当前仓库用于协作开发与测试。对外公开前请补充正式 License，并确认第三方依赖与资源版权合规。
