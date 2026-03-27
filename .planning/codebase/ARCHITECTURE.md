# Architecture

**Analysis Date:** 2026-03-27

## Pattern Overview

**Overall:** 混合式桌面应用架构。主运行链路由 `src/` 中的 React/Vite 前端和 `src-tauri/` 中的 Tauri/Rust 原生层组成，二者通过 Tauri `invoke` 命令和事件总线双向通信。

**Key Characteristics:**

- 根应用是单窗口桌面端，前端入口在 `src/main.tsx`，原生入口在 `src-tauri/src/main.rs` 和 `src-tauri/src/lib.rs`。
- 前端把业务状态、异步调用和前后端桥接集中在 `src/stores/clipboardStore.ts` 与 `src/stores/configStore.ts`，组件层主要消费 store。
- 原生层把长期存活的资源聚合到 `src-tauri/src/state.rs` 的 `AppState`，再由 `src-tauri/src/commands.rs` 暴露成 Tauri 命令。
- 详情预览采用“语义描述 + 渲染器”分层，`src/lib/preview/previewDescriptor.ts` 负责把条目转换成 `PreviewDescriptor`，`src/components/DetailView/scene/PrimaryPreviewRenderer.tsx` 负责具体渲染。
- 仓库中存在并列应用目录 `website/`、`dance-sync-server/`、`dance-sync-server-node/`，但主桌面应用的运行链路不依赖它们。

## Layers

**前端入口与应用壳层:**

- Purpose: 初始化 React 树、全局 Provider、国际化与桌面壳布局。
- Location: `src/main.tsx`, `src/App.tsx`, `src/components/Layout/MainLayout.tsx`, `src/components/theme-provider.tsx`
- Contains: `ReactDOM.createRoot`、`QueryClientProvider`、主题 Provider、懒加载模态框和主两栏布局。
- Depends on: `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`, `src/i18n/config.ts`, `src/components/*`
- Used by: Tauri WebView 加载的前端应用。

**前端展示层:**

- Purpose: 渲染列表、详情、筛选、搜索、偏好设置、统计和更新提示。
- Location: `src/components/ClipboardList/`, `src/components/DetailView/`, `src/components/Preferences/`, `src/components/Statistics/`, `src/components/SearchBar/`, `src/components/TypeFilter/`, `src/components/UpdateChecker/`
- Contains: 业务组件、详情视图场景组件、内容渲染器、Radix UI 组合组件。
- Depends on: Zustand store、`src/types/clipboard.ts`、`src/lib/preview/*`、`src/lib/utils.ts`
- Used by: `src/App.tsx`

**前端状态与桥接层:**

- Purpose: 持有应用状态，封装对 Tauri 命令、系统剪贴板插件和前端缓存的访问。
- Location: `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`, `src/components/MenuEventHandler/MenuEventHandler.tsx`, `src/components/ClipboardMenuHandler.tsx`
- Contains: 条目列表状态、分页、搜索筛选、预览缓存、配置状态、菜单事件监听、全局快捷键回调。
- Depends on: `@tauri-apps/api/core`, `@tauri-apps/api/event`, `@tauri-apps/plugin-clipboard-manager`, `src/types/clipboard.ts`
- Used by: 所有业务组件，尤其是 `src/components/ClipboardList/ClipboardList.tsx`、`src/components/DetailView/DetailView.tsx`、`src/components/Preferences/PreferencesModal.tsx`

**前端语义与预览层:**

- Purpose: 把 `ClipboardEntry` 转成可展示的语义结构，减少视图直接解析原始字段。
- Location: `src/lib/preview/entryPresentation.ts`, `src/lib/preview/previewDescriptor.ts`, `src/lib/clipboardFilters.ts`, `src/types/clipboard.ts`
- Contains: 子类型归一化、元数据解析、标题生成、详情视图 descriptor、筛选项定义。
- Depends on: `ClipboardEntry`、`ContentMetadata`、`ResolvedPreviewData`
- Used by: `src/components/DetailView/DetailView.tsx`, `src/components/TypeFilter/TypeFilter.tsx`, `src/components/DetailView/scene/*`

**原生命令边界层:**

- Purpose: 把前端请求映射到 Rust 侧能力，并作为唯一的 Tauri 命令出口。
- Location: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- Contains: `#[tauri::command]` 函数、`invoke_handler` 注册、应用生命周期与菜单事件绑定。
- Depends on: `src-tauri/src/state.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/updater/mod.rs`, `src-tauri/src/utils/*`
- Used by: `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`, `src/components/UpdateChecker/UpdateChecker.tsx`, `src/App.tsx`

**原生状态与领域服务层:**

- Purpose: 管理桌面端长期状态、剪贴板监控、配置、数据库和系统托盘。
- Location: `src-tauri/src/state.rs`, `src-tauri/src/clipboard/monitor.rs`, `src-tauri/src/clipboard/content_detector.rs`, `src-tauri/src/clipboard/processor.rs`, `src-tauri/src/tray.rs`, `src-tauri/src/updater/mod.rs`
- Contains: `AppState`、剪贴板监听循环、内容识别、图片落盘、托盘图标、更新检查。
- Depends on: `src-tauri/src/database/mod.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/utils/*`
- Used by: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`

**原生持久化与系统集成层:**

- Purpose: 负责数据落盘、配置文件、系统应用扫描、图标提取和外部媒体检查。
- Location: `src-tauri/src/database/mod.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/utils/app_detector.rs`, `src-tauri/src/utils/app_icon_extractor.rs`, `src-tauri/src/utils/app_list.rs`
- Contains: SQLite 初始化与迁移、JSON 配置读写、系统活跃应用探测、已安装应用列表、应用图标提取。
- Depends on: `sqlx`, `tokio::fs`, 平台 API、`dirs`
- Used by: `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/clipboard/monitor.rs`

## Data Flow

**剪贴板采集与列表刷新:**

1. `src/App.tsx` 在启动时调用 `useClipboardStore().setupEventListener()` 和 `startMonitoring()`。
2. `src/stores/clipboardStore.ts` 通过 `invoke('start_monitoring')` 调用 `src-tauri/src/commands.rs`，后者转发到 `src-tauri/src/state.rs` 的 `AppState::start_monitoring()`。
3. `src-tauri/src/clipboard/monitor.rs` 持续检查剪贴板，调用 `src-tauri/src/clipboard/content_detector.rs` 识别文本子类型，必要时调用 `src-tauri/src/clipboard/processor.rs` 保存图片。
4. `src-tauri/src/state.rs` 中的后台保存任务把条目写入 `src-tauri/src/database/mod.rs` 创建的 SQLite，并向前端发出 `clipboard-update` 事件。
5. `src/stores/clipboardStore.ts` 监听 `clipboard-update`，更新 `entries` 和 `selectedEntry`，`src/components/ClipboardList/ClipboardList.tsx` 与 `src/components/DetailView/DetailView.tsx` 自动重渲染。

**详情预览解析:**

1. `src/components/DetailView/DetailView.tsx` 观察 `selectedEntry`，调用 `useClipboardStore().resolveEntryPreview()`。
2. `src/stores/clipboardStore.ts` 先基于 `content_type` 和 `content_subtype` 决定本地图片、URL 还是 Base64 分支，并缓存结果到 `previewResolutionCache`。
3. URL 预览会走 `invoke('resolve_url_preview')` 到 `src-tauri/src/commands.rs`；Base64 预览会走 `invoke('decode_base64_preview')`；图片则通过 `invoke('get_image_url')` 生成可访问 URL。
4. 若原生 URL/Base64 解析失败，`src/stores/clipboardStore.ts` 内部会使用前端 fallback 逻辑补齐文本、JSON 或媒体信息。
5. `src/lib/preview/previewDescriptor.ts` 把条目和解析结果组合成 `PreviewDescriptor`，再由 `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`、`AlternateViews.tsx`、`InspectorPanel.tsx` 完成渲染。

**配置、快捷键与更新链路:**

1. `src/App.tsx` 调用 `useConfigStore().loadConfig()`，并按配置切换语言、更新窗口标题。
2. `src/stores/configStore.ts` 使用 `invoke('get_config')`、`invoke('update_config')`、`invoke('register_global_shortcut')` 与 `src-tauri/src/config/mod.rs` 和 `src-tauri/src/state.rs` 对接。
3. `src/components/MenuEventHandler/MenuEventHandler.tsx` 和 `src/components/ClipboardMenuHandler.tsx` 监听来自 `src-tauri/src/lib.rs` 的菜单事件与快捷键事件，驱动偏好设置弹窗和前端复制粘贴行为。
4. `src/components/UpdateChecker/UpdateChecker.tsx` 在启动时调用 `invoke('should_check_for_updates')` 与 `invoke('check_for_update')`，后端由 `src-tauri/src/updater/mod.rs` 执行实际检查，并通过 `update-download-progress` 事件回推下载进度。

**State Management:**

- 业务状态以 Zustand 为主，集中在 `src/stores/clipboardStore.ts` 和 `src/stores/configStore.ts`。
- 界面瞬时状态仍保留在组件内部，例如 `src/App.tsx` 的统计弹窗开关、`src/components/UpdateChecker/UpdateChecker.tsx` 的下载进度。
- `src/App.tsx` 已挂载 `QueryClientProvider`，但在当前已读代码中未见基于 React Query 的业务查询 hook，实际数据流仍以 store + Tauri 命令为主。

## Key Abstractions

**ClipboardEntry:**

- Purpose: 表示一条剪贴板历史记录，是前后端共享的核心数据形状。
- Examples: `src/types/clipboard.ts`, `src-tauri/src/models/mod.rs`
- Pattern: 前端和后端分别定义同名结构，通过 Tauri 序列化传输，字段围绕 `content_type`、`content_subtype`、`metadata`、`file_path` 展开。

**AppState:**

- Purpose: 聚合原生侧数据库、监控器、配置、广播通道、快捷键和应用句柄。
- Examples: `src-tauri/src/state.rs`
- Pattern: 单实例状态容器，由 `src-tauri/src/lib.rs` 在启动时创建并注入 `app.manage(state)`。

**PreviewDescriptor:**

- Purpose: 作为详情预览的中间表示，屏蔽原始条目与渲染组件之间的耦合。
- Examples: `src/types/clipboard.ts`, `src/lib/preview/previewDescriptor.ts`
- Pattern: 先构建 descriptor，再由 `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx` 按 `primaryKind` 分发到对应渲染器。

**AppConfig:**

- Purpose: 表示应用配置，包括文本/图片保留策略、排除应用、快捷键、自动启动、自动更新和语言。
- Examples: `src/stores/configStore.ts`, `src-tauri/src/config/mod.rs`
- Pattern: Rust 侧落盘到配置文件，前端通过 store 拉取和更新。

**InstalledApp / AppListManager:**

- Purpose: 作为“排除应用”与图标提取的系统抽象。
- Examples: `src-tauri/src/utils/app_list.rs`, `src-tauri/src/commands.rs`
- Pattern: 由命令层暴露成可选配置数据源，供 `src/components/Preferences/PreferencesModal.tsx` 使用。

## Entry Points

**前端启动入口:**

- Location: `src/main.tsx`
- Triggers: Vite/Tauri 加载根页面。
- Responsibilities: 导入全局样式、国际化配置并挂载 `App`。

**前端应用入口:**

- Location: `src/App.tsx`
- Triggers: `src/main.tsx`
- Responsibilities: 装配 Provider、启动监控、同步配置、挂载菜单/更新监听、组合列表与详情主界面。

**原生二进制入口:**

- Location: `src-tauri/src/main.rs`
- Triggers: `cargo run` / `tauri dev` / 打包后的桌面应用启动。
- Responsibilities: 调用 `dance_lib::run()`。

**Tauri 应用构建入口:**

- Location: `src-tauri/src/lib.rs`
- Triggers: `src-tauri/src/main.rs`
- Responsibilities: 注册插件、创建 `AppState`、创建托盘、绑定菜单/窗口事件、注册全部 Tauri 命令。

**独立官网入口:**

- Location: `website/app/page.tsx`
- Triggers: `website/` 自己的 Next.js 运行命令。
- Responsibilities: 渲染营销官网首页；不属于桌面应用运行链路。

## Error Handling

**Strategy:** 前端以 `try/catch + store.error/console` 处理调用失败，原生层以 `Result<_, String>` 暴露命令错误，复杂预览能力采用“后端优先，前端 fallback 兜底”的退化策略。

**Patterns:**

- `src/stores/clipboardStore.ts` 和 `src/stores/configStore.ts` 在 `invoke(...)` 周围捕获异常，并在必要时更新 `error` 字段。
- `src-tauri/src/commands.rs` 普遍使用 `.map_err(|e| e.to_string())` 把领域错误转换为命令层可序列化字符串。
- `src/stores/clipboardStore.ts` 的 `resolveUrlPreview` 与 `decodeBase64Preview` 在失败时回退到前端解析逻辑，而不是直接中断详情视图。
- `src-tauri/src/lib.rs`、`src-tauri/src/state.rs`、`src-tauri/src/clipboard/*.rs` 使用 `log::info!/warn!/error!` 记录运行期信息。

## Cross-Cutting Concerns

**Logging:** 原生日志主入口在 `src-tauri/src/lib.rs`，通过 `tauri-plugin-log` 写入 stdout、日志目录和 webview；前端普遍使用 `console.log/error`，日志查看命令也由 `src-tauri/src/commands.rs` 暴露。

**Validation:** 文本内容识别与子类型判断在 `src-tauri/src/clipboard/content_detector.rs`；URL 预览命令在 `src-tauri/src/commands.rs` 强制校验绝对 HTTP(S) URL；快捷键校验同样由 `src-tauri/src/commands.rs` 暴露。

**Authentication:** Not applicable。当前已读主桌面应用没有用户账户、会话或权限认证链路。

---

_Architecture analysis: 2026-03-27_
