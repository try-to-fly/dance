# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Critical Directory Navigation Rules

**IMPORTANT**: This project has two main directories:

- **Project Root**: `./` (contains package.json, CLAUDE.md, src/, src-tauri/)
- **Rust Backend**: `./src-tauri/` (contains Cargo.toml, src/, target/)

**Working Directory Context**:

- The project root contains package.json and CLAUDE.md
- Most commands assume you're already in the project root directory
- Always verify your current working directory before running commands

**Command Execution Guidelines**:

1. **Frontend commands** (pnpm, npm, node) require project root as working directory
2. **Rust commands** (cargo) require `src-tauri/` directory as working directory
3. **ALWAYS** check current directory before executing commands
4. Use `pwd` or equivalent to verify your location if unsure

**Correct Command Patterns**:

- ✅ From project root: `cd src-tauri && cargo fmt`
- ✅ From project root: `pnpm build`
- ❌ From wrong directory: `cargo fmt` (will fail if not in src-tauri)
- ❌ From src-tauri: `pnpm build` (will fail, no package.json)

## Development Commands

### Frontend (React + TypeScript)

**Working Directory**: Project root (where package.json is located)

- `pnpm install` - Install all dependencies
- `pnpm dev` - Start Vite development server (frontend only)
- `pnpm build` - Build production frontend bundle (TypeScript compilation + Vite build)
- `pnpm preview` - Preview production build

### Code Quality & Linting

**Note**: Always run from project root, then navigate to appropriate directory

- `cd src-tauri && cargo fmt` - Format Rust code
- `cd src-tauri && cargo clippy` - Run Rust linter
- TypeScript checking is included in `pnpm build` command

### Full Application (Tauri)

**Working Directory**: Project root (where package.json and start.sh are located)

- `./start.sh` - Automated startup script (installs deps + runs dev)
- `pnpm tauri dev` - Start full development environment (Rust backend + React frontend)
- `pnpm tauri build` - Build production application bundle

### Rust Backend

**Working Directory**: Project root, then navigate to src-tauri/

- `cd src-tauri && cargo check` - Verify Rust code compiles
- `cd src-tauri && cargo test` - Run Rust unit tests (if any)

## Architecture Overview

Dance is a macOS clipboard management application built with a Tauri (Rust + React) hybrid architecture.

### Backend Architecture (Rust)

The Rust backend (`src-tauri/`) follows a modular async architecture:

- **AppState**: Central application state using `Arc<T>` and `tokio::sync` primitives for thread-safe async operations
- **Clipboard Monitor**: Uses macOS NSPasteboard API directly via `cocoa` and `objc` crates for system-level clipboard monitoring (non-polling)
- **Database Layer**: SQLx with SQLite for async database operations, stored in `~/Library/Application Support/dance/`
- **Event System**: Tauri's built-in event system for real-time frontend-backend communication via `tauri::Emitter`

Key async patterns:

- `tokio::sync::Mutex` instead of `std::sync::Mutex` for async-safe locking
- `broadcast::channel` for event distribution between clipboard monitor and database workers
- `spawn_blocking` for CPU-bound operations (clipboard access, file I/O)

### Frontend Architecture (React)

The React frontend (`src/`) uses modern patterns:

- **State Management**: Zustand store (`clipboardStore.ts`) with async actions that call Tauri commands
- **UI Components**: @radix-ui for accessible primitives, custom CSS with CSS variables for theming
- **Real-time Updates**: Event listeners via `@tauri-apps/api/event` for live clipboard updates
- **Type Safety**: Full TypeScript integration with shared types (`types/clipboard.ts`)
- **Additional Dependencies**: date-fns for time formatting, lucide-react for icons, clsx for conditional classes

### Critical Integration Points

1. **Tauri Commands**: Defined in `src-tauri/src/commands.rs`, exposed in `lib.rs` invoke_handler
2. **Event Flow**: Clipboard changes → Rust monitor → Database → Frontend event → UI update
3. **Async State Handling**: Frontend Zustand actions directly invoke Tauri commands, with loading/error states
4. **Data Storage**: SQLite with automatic migration on startup, image files saved to filesystem

### macOS-Specific Implementation Details

- **NSPasteboard Integration**: Direct Objective-C bridge via `msg_send!` macros for clipboard access
- **NSWorkspace**: App source detection using macOS workspace APIs
- **File System**: Uses `dirs` crate for proper macOS config directory placement
- **Permissions**: App requires accessibility permissions for system-level clipboard monitoring

### Development Notes

- Rust async runtime is `tokio` with full features enabled
- Frontend uses Vite for development server with HMR
- Database schema auto-migrates on application startup
- Image processing via `image` crate with PNG output format, WebP support enabled
- Content deduplication using SHA256 hashing (`sha2` crate)
- File type detection using `infer` crate for content type identification
- Base64 encoding for binary data transport between frontend and backend

### Common Patterns

When adding new Tauri commands:

1. Add async function to `commands.rs` with `#[tauri::command]`
2. Add to `invoke_handler` in `lib.rs`
3. Create corresponding frontend action in `clipboardStore.ts`
4. Use TypeScript types from `types/clipboard.ts`

When modifying database schema:

- Update `database/mod.rs` init method
- Ensure migrations are backwards compatible
- Update `models/mod.rs` structs with `sqlx::FromRow` derive

### Key File Locations

- Database: `~/Library/Application Support/dance/clipboard.db`
- Images: `~/Library/Application Support/dance/imgs/`
- Main Zustand store: `src/stores/clipboardStore.ts`
- Tauri commands: `src-tauri/src/commands.rs`
- TypeScript types: `src/types/clipboard.ts`
- Clipboard monitoring: `src-tauri/src/clipboard/monitor.rs`

<!-- GSD:project-start source:PROJECT.md -->

## Project

**Dance**

Dance 是一个面向开发者的本地桌面剪贴板管理工具，当前以客户端能力为中心，负责稳定监听剪贴板、持久化历史记录、识别复制内容的类型，并为不同内容提供合适的详情预览。它不是通用型云端协作产品，而是一个帮助开发者更高效查看、理解、筛选和回用剪贴板内容的工作台。

**Core Value:** 开发者复制任意常见内容后，应用都能稳定记录、准确识别，并以最合适的结构化方式展示出来。

### Constraints

- **Platform**: 仅考虑桌面客户端能力 — 当前目标明确排除云端、同步和移动端扩展
- **Primary Audience**: 面向开发者 — 功能优先围绕开发工作流中的内容识别、预览和检索
- **Existing Stack**: 基于当前 Tauri + React + Rust + SQLite 架构演进 — 避免脱离现有代码基础重做产品
- **Reliability**: 监听、存储、预览和检索链路必须可靠 — 这是用户持续使用该产品的前提
- **Scope Control**: 不扩展到团队协作、分享和多设备体系 — 防止主线目标被平台化诉求稀释
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->

## Technology Stack

## Languages

- TypeScript `~5.6.2` - Desktop frontend lives in `src/`; website source lives in `website/app/`, `website/components/`, and `website/lib/`; build/test config also uses TS in `vite.config.ts` and `vitest.config.ts`
- Rust `edition = 2021` - Native desktop backend, updater, clipboard processing, database access, and OS integration live in `src-tauri/src/`
- JavaScript - Tooling and release automation live in `eslint.config.js`, `tailwind.config.js`, `postcss.config.js`, `website/next.config.js`, `website/tailwind.config.js`, `website/postcss.config.js`, and `scripts/update-version.js`
- SQL (SQLite dialect) - Schema creation, migrations, and queries are embedded in `src-tauri/src/database/mod.rs`, `src-tauri/src/state.rs`, and `src-tauri/src/integration_tests.rs`
- Shell/YAML - CI/CD and deployment automation live in `website/scripts/should-build-website.sh`, `.github/workflows/release.yml`, and `.github/workflows/test-build.yml`

## Runtime

- Node.js - JavaScript toolchain for the repo root and `website/`; `website/package.json` explicitly requires `>=22`, while `.github/workflows/release.yml` and `.github/workflows/test-build.yml` install Node `lts/*`
- Rust stable toolchain - Native layer is built from `src-tauri/Cargo.toml` and locked by `src-tauri/Cargo.lock`
- Tauri `2.x` desktop runtime - Desktop shell is configured in `src-tauri/tauri.conf.json` and bootstrapped in `src-tauri/src/lib.rs`
- `pnpm` - Root desktop app uses `package.json` plus `pnpm-lock.yaml`; website subproject uses `website/package.json` plus `website/pnpm-lock.yaml`
- `cargo` - Native desktop backend uses `src-tauri/Cargo.toml` plus `src-tauri/Cargo.lock`
- Lockfile: present in all active JS and Rust package layers

## Frameworks

- Tauri `2.x` - Desktop shell, IPC commands, tray, updater, dialog, clipboard, autostart, and global shortcut plugins are wired from `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, and `src-tauri/tauri.conf.json`
- React `18.3.1` - Desktop UI entry points are `src/main.tsx` and `src/App.tsx`
- Vite `6.0.3` - Desktop frontend dev server and bundle pipeline are configured in `vite.config.ts`
- Next.js `15.1.6` with React `19.0.0` - Marketing website runs from `website/app/` and is configured by `website/package.json` and `website/next.config.js`
- Tailwind CSS `3.4.0` - Styling pipelines are configured in `tailwind.config.js`, `postcss.config.js`, `website/tailwind.config.js`, and `website/postcss.config.js`
- Zustand `4.5.0` - Client state stores live in `src/stores/clipboardStore.ts` and `src/stores/configStore.ts`
- TanStack React Query `5.18.0` and React Virtual `3.13.12` - Query provider is created in `src/App.tsx`; virtualization is used in `src/components/ClipboardList/ClipboardList.tsx`
- Vitest `4.1.2` - Frontend tests are configured in `vitest.config.ts` and bootstrapped in `src/test/setup.ts`
- Testing Library (`@testing-library/react`, `@testing-library/jest-dom`, `@testing-library/user-event`) - Component tests live under `src/components/**/*.test.tsx`
- Rust `cargo test` with async/sqlite coverage - Backend tests live in `src-tauri/src/state_tests.rs`, `src-tauri/src/integration_tests.rs`, and `src-tauri/src/performance_tests.rs`
- TypeScript `~5.6.2` - Compiler settings live in `tsconfig.json`, `tsconfig.node.json`, and `website/tsconfig.json`
- ESLint `9.33.0` plus `typescript-eslint` `8.39.0` - Desktop lint rules live in `eslint.config.js`
- Prettier `3.6.2` - Formatting rules live in `.prettierrc`
- Tauri CLI `2.x` - Desktop dev/build commands are exposed through root `package.json`
- GitHub Actions - Release and test-build automation live in `.github/workflows/release.yml` and `.github/workflows/test-build.yml`
- Custom version automation - `scripts/update-version.js` updates `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.lock`

## Key Dependencies

- `@tauri-apps/api` `^2` and `tauri` `2` - Frontend/backend bridge, window events, and command invocation used in `src/App.tsx`, `src/stores/clipboardStore.ts`, and `src-tauri/src/lib.rs`
- `sqlx` `0.7` with SQLite - Local persistence, schema init, and queries live in `src-tauri/src/database/mod.rs` and `src-tauri/src/state.rs`
- `reqwest` `0.11` - Remote URL preview resolution and content fetching are implemented in `src-tauri/src/commands.rs`
- `@aptabase/tauri` `^0.4.1` and `tauri-plugin-aptabase` `1` - Product analytics live in `src/services/analytics.ts` and `src-tauri/src/lib.rs`
- `@tauri-apps/plugin-updater` `^2.9.0` and `tauri-plugin-updater` `2` - In-app update checks/install live in `src/components/UpdateChecker/UpdateChecker.tsx`, `src-tauri/src/updater/mod.rs`, and `src-tauri/tauri.conf.json`
- `@monaco-editor/react` `^4.7.0` and `monaco-editor` `^0.52.2` - Rich code/JSON preview renderers live in `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.tsx` and `src/components/DetailView/ContentRenderers/JsonRenderer.tsx`
- `@tauri-apps/plugin-clipboard-manager`, `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`, `tauri-plugin-dialog`, `tauri-plugin-log`, and `tauri-plugin-opener` - Native desktop capabilities are registered in `src-tauri/src/lib.rs`
- `arboard`, `image`, `infer`, `uuid`, `chrono`, `serde`, `serde_json`, `serde_with`, `regex`, `url`, and `dirs` - Clipboard parsing, media handling, serialization, and path management live across `src-tauri/src/clipboard/`, `src-tauri/src/models/`, `src-tauri/src/commands.rs`, and `src-tauri/src/config/mod.rs`
- `@radix-ui/*`, `class-variance-authority`, `lucide-react`, `tailwind-merge`, and `tailwindcss-animate` - UI primitives and styling helpers live in `src/components/ui/` and `website/components/ui/`
- `i18next`, `react-i18next`, and `i18next-browser-languagedetector` - Internationalization is initialized in `src/i18n/config.ts` and backed by locale files in `src/locales/`

## Configuration

- `src-tauri/.env` is present and is loaded manually in debug builds via `dotenvy::dotenv()` in `src-tauri/src/lib.rs`
- `APTABASE_APP_KEY` configures analytics in `src-tauri/src/lib.rs`; CI injects it in `.github/workflows/release.yml` and `.github/workflows/test-build.yml`
- `TAURI_DEV_HOST` customizes Vite/Tauri dev host and HMR behavior in `vite.config.ts`
- `GITHUB_TOKEN`, `TAURI_SIGNING_PRIVATE_KEY`, and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` are required by `.github/workflows/release.yml` and `.github/workflows/test-build.yml` for signed release builds
- Additional env files are present at `dance-sync-server/.env` and `dance-sync-server-node/.env`; no active package manifest was detected alongside them during this scan
- Desktop frontend/build config lives in `package.json`, `vite.config.ts`, `tsconfig.json`, `tailwind.config.js`, `postcss.config.js`, `eslint.config.js`, `.prettierrc`, and `vitest.config.ts`
- Native desktop config lives in `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/tauri.test.conf.json`, and `src-tauri/capabilities/default.json`
- Website config lives in `website/package.json`, `website/next.config.js`, `website/tsconfig.json`, `website/tailwind.config.js`, and `website/postcss.config.js`
- Deployment/build automation lives in `vercel.json`, `website/scripts/should-build-website.sh`, `.github/workflows/release.yml`, `.github/workflows/test-build.yml`, and `scripts/update-version.js`

## Platform Requirements

- `pnpm` plus a working Node.js toolchain is required for the JS layers; `website/package.json` is the only manifest that explicitly enforces `node >=22`
- Rust stable, Cargo, and Tauri system dependencies are required for `src-tauri/`
- macOS is the primary native integration target: `src-tauri/Cargo.toml` enables `macos-private-api`, `src-tauri/src/lib.rs` sets `ActivationPolicy::Accessory`, and `src-tauri/src/utils/app_list.rs` plus `src-tauri/src/utils/app_icon_extractor.rs` call macOS `NSWorkspace` APIs
- Windows code paths and build targets are present via `winapi` in `src-tauri/Cargo.toml`, Windows branches in `src-tauri/src/utils/app_list.rs` and `src-tauri/src/utils/app_icon_extractor.rs`, and Windows jobs in `.github/workflows/release.yml`
- `ffprobe` is an optional external binary for media metadata extraction; runtime checks live in `src-tauri/src/commands.rs`
- Desktop bundles are produced as signed Tauri artifacts with updater metadata via `src-tauri/tauri.conf.json` and `.github/workflows/release.yml`
- `src-tauri/tauri.conf.json` enables `createUpdaterArtifacts`, so release builds are expected to emit GitHub-hosted updater manifests
- The website is configured as a static Next.js export in `website/next.config.js`; repo-level Vercel settings are defined in `vercel.json`
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

## Naming Patterns

- React 业务组件文件优先使用 PascalCase，放在同名目录下：`src/components/DetailView/DetailView.tsx`、`src/components/SearchBar/SearchBar.tsx`、`src/components/Layout/MainLayout.tsx`。
- 基础 UI 与 shadcn 风格组件使用小写或 kebab-case：`src/components/ui/button.tsx`、`src/components/theme-provider.tsx`、`src/components/settings-button.tsx`。
- Store、service、helper 文件使用 camelCase：`src/stores/clipboardStore.ts`、`src/stores/configStore.ts`、`src/lib/preview/entryPresentation.ts`、`src/services/analytics.ts`。
- Rust 模块使用 snake_case 与 `mod.rs` 组织：`src-tauri/src/database/mod.rs`、`src-tauri/src/clipboard/content_detector.rs`、`src-tauri/src/utils/app_icon_extractor.rs`。
- `website/` 子项目延续 Next/shadcn 默认命名，文件名多为小写或 kebab-case：`website/components/ui/button.tsx`、`website/components/header.tsx`。
- React 组件与 Provider 使用 PascalCase 函数名：`App`、`AppContent`、`ThemeProvider`、`DetailView`，见 `src/App.tsx`、`src/components/theme-provider.tsx`、`src/components/DetailView/DetailView.tsx`。
- Hook 统一以 `use` 开头：`useClipboardStore`、`useConfigStore`、`useResolvedTheme`，见 `src/stores/clipboardStore.ts`、`src/stores/configStore.ts`、`src/hooks/useResolvedTheme.ts`。
- 组件内部事件处理与辅助函数使用 `handle*` / `get*` / `normalize*` / `build*` / `parse*`：`handleCopy`、`normalizeEntryUrl`、`buildPreviewDescriptor`、`parseContentMetadata`。
- Rust 对外命令函数使用 snake_case，并在 Tauri 边界标注 `#[tauri::command]`：`start_monitoring`、`get_clipboard_history`、`clear_history`，见 `src-tauri/src/commands.rs`。
- 布尔变量与状态常用 `is*` / `has*` / `show*` / `selected*` 前缀：`isMonitoring`、`hasMore`、`showStatistics`、`selectedEntry`，见 `src/stores/clipboardStore.ts`、`src/App.tsx`。
- 常量使用 UPPER_SNAKE_CASE：`ANALYTICS_EVENTS`、`DEFAULT_PREVIEW_CACHE_TTL_MS`、`DEGRADED_PREVIEW_CACHE_TTL_MS`，见 `src/services/analytics.ts`、`src/stores/clipboardStore.ts`。
- 临时工厂和夹具函数常用 `create*` 命名：`createStoreState`、`createDescriptor`、`createDeferred`，见 `src/components/DetailView/DetailView.test.tsx`、`src/components/DetailView/DetailPreviewContract.test.tsx`、`src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx`。
- TypeScript 类型、接口、枚举式联合均使用 PascalCase：`ClipboardEntry`、`ResolvedPreviewData`、`PreviewKind`、`AnalyticsEvent`，见 `src/types/clipboard.ts`、`src/services/analytics.ts`。
- 类型导入通常显式标注 `type`，避免把值导入与类型导入混用：`src/lib/utils.ts`、`src/App.tsx`、`src/components/ui/button.tsx`。
- Rust 结构体与枚举使用 PascalCase，序列化命名规则通过 `#[serde(rename_all = "snake_case")]` 控制：`PreviewKind`、`DecodedKind`，见 `src-tauri/src/commands.rs`。

## Code Style

- 根前端使用 Prettier，配置文件是 `.prettierrc`。
- 关键设置来自 `.prettierrc`：单引号、分号、`trailingComma: "es5"`、`printWidth: 100`、`tabWidth: 2`、LF 换行。
- 根仓库格式化脚本只覆盖 `src/**/*.{js,jsx,ts,tsx,json,css,md}`，见 `package.json`。`website/` 未接入这套脚本，因此存在与主应用不同的输出风格。
- Rust 文件通过 `.husky/pre-commit` 触发 `cargo fmt`，提交前自动回写格式，见 `.husky/pre-commit`。
- 根前端使用 ESLint 9 + `typescript-eslint`，入口为 `eslint.config.js`。
- 当前规则重点约束：
- `eslint.config.js` 显式忽略 `src-tauri`、`website`、`dist`、配置脚本，因此根 ESLint 只约束主前端 `src/`。
- `website/package.json` 依赖 `eslint-config-next`，但仓库内未检测到 `website/eslint.config.*` 或 `website/.eslintrc*`，说明站点依赖 Next 默认 lint 入口。
- staged JS/TS 文件通过 `.lintstagedrc.json` 执行 `eslint --fix` 和 `prettier --write`；JSON/MD/HTML/CSS 只跑 Prettier。

## Import Organization

- 主应用 `src/` 未配置 TS path alias，根 `tsconfig.json` 也没有 `paths`，因此沿用相对路径导入。
- `website/` 单独配置了 `@/*` 别名，见 `website/tsconfig.json`；站点代码应继续使用 `@/components/*`、`@/lib/utils` 形式，见 `website/app/page.tsx`、`website/components/ui/button.tsx`。

## Error Handling

- 前端异步调用普遍使用 `try/catch` 包住 Tauri `invoke`、文件操作或解析逻辑，失败后记录日志并回退默认值或状态，见 `src/stores/configStore.ts`、`src/components/DetailView/DetailView.tsx`、`src/services/analytics.ts`。
- 读取型操作常吞掉异常并返回安全默认值：例如 `parseContentMetadata` 返回 `null`，`normalizeUrlString` 返回空字符串，`getAutoStartupStatus` 返回 `false`，见 `src/lib/preview/entryPresentation.ts`、`src/stores/clipboardStore.ts`、`src/stores/configStore.ts`。
- 写入型操作若调用方需要感知失败，则 catch 后重新抛出：`registerGlobalShortcut`、`setAutoStartup`、`cleanupExpiredEntries`，见 `src/stores/configStore.ts`。
- 组件层常在 effect 内设置“过期保护”或 source key，避免异步结果串台，见 `src/components/DetailView/DetailView.tsx`、`src/components/DetailView/scene/AlternateViews.tsx`。
- Rust 内部实现使用 `anyhow::Result`，在 Tauri 命令边界统一 `map_err(|e| e.to_string())` 暴露给前端，见 `src-tauri/src/database/mod.rs`、`src-tauri/src/commands.rs`。

## Logging

- 前端日志主要使用 `console.error`、`console.warn`、`console.log`，并倾向带模块前缀：`[DetailView]`、`[resolveUrlPreview]`、`[ConfigStore]`，见 `src/components/DetailView/DetailView.tsx`、`src/stores/clipboardStore.ts`、`src/stores/configStore.ts`。
- 记录内容通常围绕失败原因、fallback 分支与调试状态；没有统一 logger 封装，新增日志时保持简短且可定位模块。
- Rust 侧使用 `log::info!`、`log::warn!`、`log::error!`，并在 `src-tauri/src/lib.rs` 中通过 `tauri_plugin_log::Builder` 输出到 stdout、日志目录和 webview。

## Comments

- 注释偏少，保留给非直观流程、平台差异、兼容性 fallback 和测试步骤说明。
- 常见位置包括：
- 新代码如果语义明显，不额外补注释；只有在 fallback、协议映射、异步竞态保护等逻辑不直观时才写短注释。
- 未检测到系统性 JSDoc/TSDoc 约定。类型语义主要通过 TypeScript 接口与明确命名表达，见 `src/types/clipboard.ts`、`src/stores/configStore.ts`。

## Function Design

- 纯工具函数偏向位置参数：`normalizeContentPreview(value, maxLength)`，见 `src/lib/preview/entryPresentation.ts`。
- 较复杂构建器偏向对象参数，便于扩展：`buildPreviewDescriptor({ entry, resolvedData, labels })`，见 `src/lib/preview/previewDescriptor.ts`。
- Store action 和 Tauri command 明确写出参数类型与返回 Promise/Result，见 `src/stores/configStore.ts`、`src-tauri/src/commands.rs`。
- 工具函数常返回简单可空值作为 guard：`''`、`null`、`undefined`，见 `src/lib/preview/entryPresentation.ts`、`src/stores/clipboardStore.ts`。
- React 组件在无内容时直接 `return null` 或返回空状态组件，见 `src/components/DetailView/scene/AlternateViews.tsx`、`src/components/DetailView/DetailView.tsx`。
- Store action 使用 `Promise<void>` 为主；需要调用方分支判断时返回具体布尔值或数据对象，见 `src/stores/configStore.ts`、`src/stores/clipboardStore.ts`。

## Module Design

- 默认导出：`src/App.tsx`、`website/app/page.tsx`、`website/app/layout.tsx`。
- 命名导出：`src/components/DetailView/DetailView.tsx`、`src/stores/configStore.ts`、`src/services/analytics.ts`。
- `src/`、`src-tauri/`、`website/` 当前并不共享统一代码风格工具链。
- 在 `src/` 中继续遵守根 `.prettierrc` 与 `eslint.config.js`。
- 在 `src-tauri/` 中继续遵守 `cargo fmt` / `cargo clippy`。
- 在 `website/` 中保持现有 Next/shadcn 风格，不要强行套用根前端的单引号加分号输出。
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

## Pattern Overview

- 根应用是单窗口桌面端，前端入口在 `src/main.tsx`，原生入口在 `src-tauri/src/main.rs` 和 `src-tauri/src/lib.rs`。
- 前端把业务状态、异步调用和前后端桥接集中在 `src/stores/clipboardStore.ts` 与 `src/stores/configStore.ts`，组件层主要消费 store。
- 原生层把长期存活的资源聚合到 `src-tauri/src/state.rs` 的 `AppState`，再由 `src-tauri/src/commands.rs` 暴露成 Tauri 命令。
- 详情预览采用“语义描述 + 渲染器”分层，`src/lib/preview/previewDescriptor.ts` 负责把条目转换成 `PreviewDescriptor`，`src/components/DetailView/scene/PrimaryPreviewRenderer.tsx` 负责具体渲染。
- 仓库中存在并列应用目录 `website/`、`dance-sync-server/`、`dance-sync-server-node/`，但主桌面应用的运行链路不依赖它们。

## Layers

- Purpose: 初始化 React 树、全局 Provider、国际化与桌面壳布局。
- Location: `src/main.tsx`, `src/App.tsx`, `src/components/Layout/MainLayout.tsx`, `src/components/theme-provider.tsx`
- Contains: `ReactDOM.createRoot`、`QueryClientProvider`、主题 Provider、懒加载模态框和主两栏布局。
- Depends on: `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`, `src/i18n/config.ts`, `src/components/*`
- Used by: Tauri WebView 加载的前端应用。
- Purpose: 渲染列表、详情、筛选、搜索、偏好设置、统计和更新提示。
- Location: `src/components/ClipboardList/`, `src/components/DetailView/`, `src/components/Preferences/`, `src/components/Statistics/`, `src/components/SearchBar/`, `src/components/TypeFilter/`, `src/components/UpdateChecker/`
- Contains: 业务组件、详情视图场景组件、内容渲染器、Radix UI 组合组件。
- Depends on: Zustand store、`src/types/clipboard.ts`、`src/lib/preview/*`、`src/lib/utils.ts`
- Used by: `src/App.tsx`
- Purpose: 持有应用状态，封装对 Tauri 命令、系统剪贴板插件和前端缓存的访问。
- Location: `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`, `src/components/MenuEventHandler/MenuEventHandler.tsx`, `src/components/ClipboardMenuHandler.tsx`
- Contains: 条目列表状态、分页、搜索筛选、预览缓存、配置状态、菜单事件监听、全局快捷键回调。
- Depends on: `@tauri-apps/api/core`, `@tauri-apps/api/event`, `@tauri-apps/plugin-clipboard-manager`, `src/types/clipboard.ts`
- Used by: 所有业务组件，尤其是 `src/components/ClipboardList/ClipboardList.tsx`、`src/components/DetailView/DetailView.tsx`、`src/components/Preferences/PreferencesModal.tsx`
- Purpose: 把 `ClipboardEntry` 转成可展示的语义结构，减少视图直接解析原始字段。
- Location: `src/lib/preview/entryPresentation.ts`, `src/lib/preview/previewDescriptor.ts`, `src/lib/clipboardFilters.ts`, `src/types/clipboard.ts`
- Contains: 子类型归一化、元数据解析、标题生成、详情视图 descriptor、筛选项定义。
- Depends on: `ClipboardEntry`、`ContentMetadata`、`ResolvedPreviewData`
- Used by: `src/components/DetailView/DetailView.tsx`, `src/components/TypeFilter/TypeFilter.tsx`, `src/components/DetailView/scene/*`
- Purpose: 把前端请求映射到 Rust 侧能力，并作为唯一的 Tauri 命令出口。
- Location: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- Contains: `#[tauri::command]` 函数、`invoke_handler` 注册、应用生命周期与菜单事件绑定。
- Depends on: `src-tauri/src/state.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/updater/mod.rs`, `src-tauri/src/utils/*`
- Used by: `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`, `src/components/UpdateChecker/UpdateChecker.tsx`, `src/App.tsx`
- Purpose: 管理桌面端长期状态、剪贴板监控、配置、数据库和系统托盘。
- Location: `src-tauri/src/state.rs`, `src-tauri/src/clipboard/monitor.rs`, `src-tauri/src/clipboard/content_detector.rs`, `src-tauri/src/clipboard/processor.rs`, `src-tauri/src/tray.rs`, `src-tauri/src/updater/mod.rs`
- Contains: `AppState`、剪贴板监听循环、内容识别、图片落盘、托盘图标、更新检查。
- Depends on: `src-tauri/src/database/mod.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/utils/*`
- Used by: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- Purpose: 负责数据落盘、配置文件、系统应用扫描、图标提取和外部媒体检查。
- Location: `src-tauri/src/database/mod.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/utils/app_detector.rs`, `src-tauri/src/utils/app_icon_extractor.rs`, `src-tauri/src/utils/app_list.rs`
- Contains: SQLite 初始化与迁移、JSON 配置读写、系统活跃应用探测、已安装应用列表、应用图标提取。
- Depends on: `sqlx`, `tokio::fs`, 平台 API、`dirs`
- Used by: `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/clipboard/monitor.rs`

## Data Flow

- 业务状态以 Zustand 为主，集中在 `src/stores/clipboardStore.ts` 和 `src/stores/configStore.ts`。
- 界面瞬时状态仍保留在组件内部，例如 `src/App.tsx` 的统计弹窗开关、`src/components/UpdateChecker/UpdateChecker.tsx` 的下载进度。
- `src/App.tsx` 已挂载 `QueryClientProvider`，但在当前已读代码中未见基于 React Query 的业务查询 hook，实际数据流仍以 store + Tauri 命令为主。

## Key Abstractions

- Purpose: 表示一条剪贴板历史记录，是前后端共享的核心数据形状。
- Examples: `src/types/clipboard.ts`, `src-tauri/src/models/mod.rs`
- Pattern: 前端和后端分别定义同名结构，通过 Tauri 序列化传输，字段围绕 `content_type`、`content_subtype`、`metadata`、`file_path` 展开。
- Purpose: 聚合原生侧数据库、监控器、配置、广播通道、快捷键和应用句柄。
- Examples: `src-tauri/src/state.rs`
- Pattern: 单实例状态容器，由 `src-tauri/src/lib.rs` 在启动时创建并注入 `app.manage(state)`。
- Purpose: 作为详情预览的中间表示，屏蔽原始条目与渲染组件之间的耦合。
- Examples: `src/types/clipboard.ts`, `src/lib/preview/previewDescriptor.ts`
- Pattern: 先构建 descriptor，再由 `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx` 按 `primaryKind` 分发到对应渲染器。
- Purpose: 表示应用配置，包括文本/图片保留策略、排除应用、快捷键、自动启动、自动更新和语言。
- Examples: `src/stores/configStore.ts`, `src-tauri/src/config/mod.rs`
- Pattern: Rust 侧落盘到配置文件，前端通过 store 拉取和更新。
- Purpose: 作为“排除应用”与图标提取的系统抽象。
- Examples: `src-tauri/src/utils/app_list.rs`, `src-tauri/src/commands.rs`
- Pattern: 由命令层暴露成可选配置数据源，供 `src/components/Preferences/PreferencesModal.tsx` 使用。

## Entry Points

- Location: `src/main.tsx`
- Triggers: Vite/Tauri 加载根页面。
- Responsibilities: 导入全局样式、国际化配置并挂载 `App`。
- Location: `src/App.tsx`
- Triggers: `src/main.tsx`
- Responsibilities: 装配 Provider、启动监控、同步配置、挂载菜单/更新监听、组合列表与详情主界面。
- Location: `src-tauri/src/main.rs`
- Triggers: `cargo run` / `tauri dev` / 打包后的桌面应用启动。
- Responsibilities: 调用 `dance_lib::run()`。
- Location: `src-tauri/src/lib.rs`
- Triggers: `src-tauri/src/main.rs`
- Responsibilities: 注册插件、创建 `AppState`、创建托盘、绑定菜单/窗口事件、注册全部 Tauri 命令。
- Location: `website/app/page.tsx`
- Triggers: `website/` 自己的 Next.js 运行命令。
- Responsibilities: 渲染营销官网首页；不属于桌面应用运行链路。

## Error Handling

- `src/stores/clipboardStore.ts` 和 `src/stores/configStore.ts` 在 `invoke(...)` 周围捕获异常，并在必要时更新 `error` 字段。
- `src-tauri/src/commands.rs` 普遍使用 `.map_err(|e| e.to_string())` 把领域错误转换为命令层可序列化字符串。
- `src/stores/clipboardStore.ts` 的 `resolveUrlPreview` 与 `decodeBase64Preview` 在失败时回退到前端解析逻辑，而不是直接中断详情视图。
- `src-tauri/src/lib.rs`、`src-tauri/src/state.rs`、`src-tauri/src/clipboard/*.rs` 使用 `log::info!/warn!/error!` 记录运行期信息。

## Cross-Cutting Concerns

<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.

<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.

<!-- GSD:profile-end -->
