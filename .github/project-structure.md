# 项目结构目录说明（基于当前仓库）

> 目标：让你知道每个文件夹负责什么，改哪里不会踩雷。

## 根目录

- `src/`：前端应用（SvelteKit）
- `static/`：静态资源（会被直接拷贝到构建产物）
- `src-tauri/`：Tauri + Rust 后端
- `.svelte-kit/`：SvelteKit 构建缓存（生成物）
- `build/`：静态构建输出（生成物；通常不手改）
- `node_modules/`：依赖（生成物）
- `.vscode/`：VS Code 配置与 Copilot 导出等

## 前端（src）

- `src/app.d.ts`：SvelteKit 类型声明
- `src/app.html`：SvelteKit HTML 模板入口
- `src/lib/`：可复用的前端模块（例如 API 封装、组件、资源）
- `src/routes/`：路由页面（SvelteKit 文件路由）
  - `+layout.svelte`：全局布局
  - `+page.svelte`：首页
  - `layout.css`：布局样式

## 后端（src-tauri）

- `src-tauri/src/main.rs`：Tauri 入口（注册 commands、初始化状态）
- `src-tauri/src/lib.rs`：库入口（可放模块声明、共享逻辑）
- `src-tauri/Cargo.toml`：Rust 依赖
- `src-tauri/tauri.conf.json`：Tauri 配置（devUrl、frontendDist、权限/能力等）
- `src-tauri/capabilities/`：Tauri v2 capabilities 权限声明
- `src-tauri/icons/`：应用图标
- `src-tauri/target/`：Rust 编译产物（生成物）

## 建议新增（实现开始后你可以按这个落地）

- `src-tauri/src/commands/`：IPC commands
- `src-tauri/src/services/`：业务服务（monitor/stats/alerts…）
- `src-tauri/src/repo/`：SQL 与数据访问
- `src-tauri/src/infra/`：win32、tray、autostart 等
- `src-tauri/src/domain/`：实体与业务模型
