# Codebase Structure

**Analysis Date:** 2026-03-27

## Directory Layout

```text
clipboard-app/
├── .planning/codebase/     # 代码库映射文档输出目录
├── src/                    # 根桌面应用的 React + TypeScript 前端
├── src-tauri/              # 根桌面应用的 Tauri + Rust 原生层
├── public/                 # Vite 静态资源
├── scripts/                # 版本与辅助脚本
├── docs/                   # 设计与实验文档
├── website/                # 独立 Next.js 官网
├── dance-sync-server/      # 并列工作区，当前未接入根桌面应用构建
├── dance-sync-server-node/ # 并列 Node 工作区，当前未接入根桌面应用构建
├── dist/                   # 根前端构建产物
├── package.json            # 根应用脚本与依赖
├── vite.config.ts          # 根前端构建配置
└── vitest.config.ts        # 根前端测试配置
```

`node_modules/`、`src-tauri/target/`、`website/.next/`、`website/out/`、`dance-sync-server-node/dist/` 是生成产物目录，不是新增业务代码的放置位置。

## Directory Purposes

**`src/`:**

- Purpose: 主桌面应用的前端代码。
- Contains: 入口、业务组件、store、类型、预览辅助、国际化和测试支持。
- Key files: `src/main.tsx`, `src/App.tsx`, `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`

**`src/components/`:**

- Purpose: 前端展示层和交互层。
- Contains: 功能组件目录 `ClipboardList/`, `DetailView/`, `Preferences/`, `Statistics/`, `SearchBar/`, `TypeFilter/`，以及通用 `ui/` 原子组件。
- Key files: `src/components/ClipboardList/ClipboardList.tsx`, `src/components/DetailView/DetailView.tsx`, `src/components/Preferences/PreferencesModal.tsx`, `src/components/ui/button.tsx`

**`src/stores/`:**

- Purpose: 根应用的业务状态和前后端桥接层。
- Contains: `clipboardStore`、`configStore` 及其单元测试。
- Key files: `src/stores/clipboardStore.ts`, `src/stores/configStore.ts`, `src/stores/clipboardStore.test.ts`

**`src/lib/`:**

- Purpose: 前端共享语义与工具逻辑。
- Contains: 预览 descriptor、条目语义提取、筛选配置、类名辅助。
- Key files: `src/lib/preview/previewDescriptor.ts`, `src/lib/preview/entryPresentation.ts`, `src/lib/clipboardFilters.ts`, `src/lib/utils.ts`

**`src/services/`:**

- Purpose: 前端横切服务。
- Contains: 当前主要是分析埋点服务。
- Key files: `src/services/analytics.ts`

**`src/i18n/` 与 `src/locales/`:**

- Purpose: 国际化初始化和多语言文案。
- Contains: `i18next` 配置与各语言 JSON 资源。
- Key files: `src/i18n/config.ts`, `src/locales/zh/common.json`, `src/locales/en/clipboard.json`

**`src/types/`:**

- Purpose: 前端共享类型定义。
- Contains: 剪贴板条目、预览数据、详情视图 descriptor 类型。
- Key files: `src/types/clipboard.ts`

**`src/test/`:**

- Purpose: 前端测试基础设施。
- Contains: 测试初始化和第三方 mock。
- Key files: `src/test/setup.ts`, `src/test/mocks/monaco-editor.ts`

**`src-tauri/src/`:**

- Purpose: 主桌面应用的 Rust 原生代码。
- Contains: 应用入口、Tauri 命令、状态容器、剪贴板监控、数据库、配置、托盘、更新、系统工具。
- Key files: `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/state.rs`, `src-tauri/src/clipboard/monitor.rs`, `src-tauri/src/database/mod.rs`

**`website/`:**

- Purpose: 独立官网站点，不参与根桌面应用的运行链路。
- Contains: Next.js App Router 页面、官网组件、官网自己的依赖和构建配置。
- Key files: `website/package.json`, `website/app/page.tsx`, `website/app/layout.tsx`, `website/next.config.js`

**`docs/`:**

- Purpose: 补充文档和实验记录。
- Contains: 当前可见内容以说明文档为主。
- Key files: `docs/content-detection/`

**`dance-sync-server/` 与 `dance-sync-server-node/`:**

- Purpose: 并列工作区。
- Contains: 当前根仓库未见由 `package.json`、`vite.config.ts` 或 `src-tauri/tauri.conf.json` 直接引用的运行入口；`dance-sync-server-node/` 主要可见 `dist/`、`logs/` 和 `src/` 目录。
- Key files: `dance-sync-server-node/dist/app.js`

## Key File Locations

**Entry Points:**

- `src/main.tsx`: 根桌面前端入口。
- `src/App.tsx`: 根桌面应用壳与主布局装配点。
- `src-tauri/src/main.rs`: Rust 二进制入口。
- `src-tauri/src/lib.rs`: Tauri builder、插件注册、命令注册和应用生命周期入口。
- `website/app/page.tsx`: 独立官网首页入口。

**Configuration:**

- `package.json`: 根桌面应用脚本、依赖和 Tauri/Vite 开发命令。
- `vite.config.ts`: 根前端构建和 chunk 切分配置。
- `tsconfig.json`: 根 TypeScript 配置。
- `vitest.config.ts`: 根前端测试配置。
- `src-tauri/Cargo.toml`: Rust 依赖与 crate 定义。
- `src-tauri/tauri.conf.json`: 桌面窗口、打包、更新端点配置。
- `website/package.json`: 官网自己的运行和构建脚本。
- `website/next.config.js`: 官网 Next.js 配置。

**Core Logic:**

- `src/stores/clipboardStore.ts`: 剪贴板历史、筛选、分页、预览解析和事件监听。
- `src/stores/configStore.ts`: 配置、缓存统计、快捷键和自启动状态。
- `src/lib/preview/previewDescriptor.ts`: 详情预览中间层。
- `src/components/DetailView/DetailView.tsx`: 详情面板入口。
- `src-tauri/src/state.rs`: 原生核心状态与数据库保存任务。
- `src-tauri/src/commands.rs`: 所有 Tauri 命令边界。
- `src-tauri/src/clipboard/monitor.rs`: 剪贴板采集主流程。
- `src-tauri/src/clipboard/content_detector.rs`: 文本内容识别。
- `src-tauri/src/database/mod.rs`: SQLite 初始化与迁移。
- `src-tauri/src/config/mod.rs`: 配置读写与迁移。

**Testing:**

- `src/stores/clipboardStore.test.ts`: 前端 store 单元测试。
- `src/components/DetailView/DetailView.test.tsx`: 详情视图测试。
- `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`: 内容渲染器测试。
- `src/test/setup.ts`: 前端测试初始化。
- `src-tauri/src/state_tests.rs`: Rust 状态测试。
- `src-tauri/src/integration_tests.rs`: Rust 集成测试。
- `src-tauri/src/performance_tests.rs`: Rust 性能相关测试。

## Naming Conventions

**Files:**

- 业务 React 组件遵循“功能目录 + 同名组件文件”模式，例如 `src/components/ClipboardList/ClipboardList.tsx`、`src/components/DetailView/DetailView.tsx`、`src/components/Preferences/PreferencesModal.tsx`。
- 详情子场景继续放在子目录内，例如 `src/components/DetailView/scene/DetailScene.tsx`、`src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`。
- 通用 UI 包装组件使用小写或 kebab-case 文件名，例如 `src/components/ui/button.tsx`、`src/components/theme-provider.tsx`、`src/components/settings-button.tsx`。
- store、service、lib 使用 lowerCamel 文件名，例如 `src/stores/clipboardStore.ts`、`src/stores/configStore.ts`、`src/services/analytics.ts`、`src/lib/clipboardFilters.ts`。
- Rust 模块使用 snake_case 文件名，例如 `src-tauri/src/state.rs`、`src-tauri/src/utils/app_list.rs`、`src-tauri/src/clipboard/content_detector.rs`。

**Directories:**

- 前端功能目录偏向 PascalCase，例如 `src/components/ClipboardList/`, `src/components/DetailView/`, `src/components/Preferences/`。
- 前端基础设施目录偏向小写，例如 `src/stores/`, `src/services/`, `src/lib/`, `src/i18n/`, `src/test/`。
- Rust 侧按职责拆目录，采用小写和模块化目录，例如 `src-tauri/src/clipboard/`, `src-tauri/src/utils/`, `src-tauri/src/database/`。

## Where to Add New Code

**New Feature:**

- Primary code: 桌面端新功能优先落在 `src/components/<Feature>/` 和 `src/stores/`；如果需要系统能力或持久化，同步扩展 `src-tauri/src/commands.rs`，并把真正逻辑放到 `src-tauri/src/state.rs`、`src-tauri/src/clipboard/`、`src-tauri/src/config/`、`src-tauri/src/database/` 或 `src-tauri/src/utils/` 的对应模块。
- Tests: 前端测试继续与实现靠近放置，使用 `src/**/*.test.ts` 或 `src/**/*.test.tsx`；Rust 测试继续放在 `src-tauri/src/*_tests.rs` 或相关模块的 `#[cfg(test)]` 中。

**New Component/Module:**

- Implementation: 新业务组件放在 `src/components/<Feature>/<Feature>.tsx`；详情预览新形态放在 `src/components/DetailView/ContentRenderers/` 或 `src/components/DetailView/scene/`，并同步扩展 `src/lib/preview/previewDescriptor.ts` 与 `src/types/clipboard.ts`；新的前端语义辅助放在 `src/lib/`。

**Utilities:**

- Shared helpers: 前端共享工具放在 `src/lib/`，原生系统工具放在 `src-tauri/src/utils/`。

新增桌面端功能时，不要把代码放到 `dist/`、`src-tauri/target/`、`website/.next/`、`website/out/`、`dance-sync-server-node/dist/`。如果功能面向根桌面应用，也不要放到 `website/`。

## Special Directories

**`.planning/codebase/`:**

- Purpose: 代码库映射和规划文档。
- Generated: Yes
- Committed: Yes

**`dist/`:**

- Purpose: 根前端构建产物，供 `src-tauri/tauri.conf.json` 的 `frontendDist` 使用。
- Generated: Yes
- Committed: No

**`src-tauri/target/`:**

- Purpose: Rust 构建产物。
- Generated: Yes
- Committed: No

**`website/.next/`:**

- Purpose: 官网开发和构建缓存。
- Generated: Yes
- Committed: No

**`website/out/`:**

- Purpose: 官网静态导出产物。
- Generated: Yes
- Committed: No

**`public/`:**

- Purpose: 根前端静态资源目录。
- Generated: No
- Committed: Yes

---

_Structure analysis: 2026-03-27_
