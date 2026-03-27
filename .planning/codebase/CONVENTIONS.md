# Coding Conventions

**Analysis Date:** 2026-03-27

## Naming Patterns

**Files:**

- React 业务组件文件优先使用 PascalCase，放在同名目录下：`src/components/DetailView/DetailView.tsx`、`src/components/SearchBar/SearchBar.tsx`、`src/components/Layout/MainLayout.tsx`。
- 基础 UI 与 shadcn 风格组件使用小写或 kebab-case：`src/components/ui/button.tsx`、`src/components/theme-provider.tsx`、`src/components/settings-button.tsx`。
- Store、service、helper 文件使用 camelCase：`src/stores/clipboardStore.ts`、`src/stores/configStore.ts`、`src/lib/preview/entryPresentation.ts`、`src/services/analytics.ts`。
- Rust 模块使用 snake_case 与 `mod.rs` 组织：`src-tauri/src/database/mod.rs`、`src-tauri/src/clipboard/content_detector.rs`、`src-tauri/src/utils/app_icon_extractor.rs`。
- `website/` 子项目延续 Next/shadcn 默认命名，文件名多为小写或 kebab-case：`website/components/ui/button.tsx`、`website/components/header.tsx`。

**Functions:**

- React 组件与 Provider 使用 PascalCase 函数名：`App`、`AppContent`、`ThemeProvider`、`DetailView`，见 `src/App.tsx`、`src/components/theme-provider.tsx`、`src/components/DetailView/DetailView.tsx`。
- Hook 统一以 `use` 开头：`useClipboardStore`、`useConfigStore`、`useResolvedTheme`，见 `src/stores/clipboardStore.ts`、`src/stores/configStore.ts`、`src/hooks/useResolvedTheme.ts`。
- 组件内部事件处理与辅助函数使用 `handle*` / `get*` / `normalize*` / `build*` / `parse*`：`handleCopy`、`normalizeEntryUrl`、`buildPreviewDescriptor`、`parseContentMetadata`。
- Rust 对外命令函数使用 snake_case，并在 Tauri 边界标注 `#[tauri::command]`：`start_monitoring`、`get_clipboard_history`、`clear_history`，见 `src-tauri/src/commands.rs`。

**Variables:**

- 布尔变量与状态常用 `is*` / `has*` / `show*` / `selected*` 前缀：`isMonitoring`、`hasMore`、`showStatistics`、`selectedEntry`，见 `src/stores/clipboardStore.ts`、`src/App.tsx`。
- 常量使用 UPPER_SNAKE_CASE：`ANALYTICS_EVENTS`、`DEFAULT_PREVIEW_CACHE_TTL_MS`、`DEGRADED_PREVIEW_CACHE_TTL_MS`，见 `src/services/analytics.ts`、`src/stores/clipboardStore.ts`。
- 临时工厂和夹具函数常用 `create*` 命名：`createStoreState`、`createDescriptor`、`createDeferred`，见 `src/components/DetailView/DetailView.test.tsx`、`src/components/DetailView/DetailPreviewContract.test.tsx`、`src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx`。

**Types:**

- TypeScript 类型、接口、枚举式联合均使用 PascalCase：`ClipboardEntry`、`ResolvedPreviewData`、`PreviewKind`、`AnalyticsEvent`，见 `src/types/clipboard.ts`、`src/services/analytics.ts`。
- 类型导入通常显式标注 `type`，避免把值导入与类型导入混用：`src/lib/utils.ts`、`src/App.tsx`、`src/components/ui/button.tsx`。
- Rust 结构体与枚举使用 PascalCase，序列化命名规则通过 `#[serde(rename_all = "snake_case")]` 控制：`PreviewKind`、`DecodedKind`，见 `src-tauri/src/commands.rs`。

## Code Style

**Formatting:**

- 根前端使用 Prettier，配置文件是 `.prettierrc`。
- 关键设置来自 `.prettierrc`：单引号、分号、`trailingComma: "es5"`、`printWidth: 100`、`tabWidth: 2`、LF 换行。
- 根仓库格式化脚本只覆盖 `src/**/*.{js,jsx,ts,tsx,json,css,md}`，见 `package.json`。`website/` 未接入这套脚本，因此存在与主应用不同的输出风格。
- Rust 文件通过 `.husky/pre-commit` 触发 `cargo fmt`，提交前自动回写格式，见 `.husky/pre-commit`。

**Linting:**

- 根前端使用 ESLint 9 + `typescript-eslint`，入口为 `eslint.config.js`。
- 当前规则重点约束：
  - `@typescript-eslint/no-unused-vars` 允许以下划线开头的未使用参数或变量。
  - `@typescript-eslint/no-explicit-any` 仅告警，不阻断提交。
  - `react-refresh/only-export-components` 为警告级别。
  - `prettier/prettier` 为错误级别，代码风格问题会被当作 lint 失败处理。
- `eslint.config.js` 显式忽略 `src-tauri`、`website`、`dist`、配置脚本，因此根 ESLint 只约束主前端 `src/`。
- `website/package.json` 依赖 `eslint-config-next`，但仓库内未检测到 `website/eslint.config.*` 或 `website/.eslintrc*`，说明站点依赖 Next 默认 lint 入口。
- staged JS/TS 文件通过 `.lintstagedrc.json` 执行 `eslint --fix` 和 `prettier --write`；JSON/MD/HTML/CSS 只跑 Prettier。

## Import Organization

**Order:**

1. React 与运行时入口先导入：见 `src/main.tsx`、`src/App.tsx`、`src/components/theme-provider.tsx`。
2. 第三方库和平台 SDK 紧随其后：如 `@tanstack/react-query`、`@tauri-apps/api/*`、`lucide-react`，见 `src/App.tsx`、`src/stores/clipboardStore.ts`。
3. 仓库内部模块最后导入，通常使用相对路径：如 `./components/...`、`../types/clipboard`、`../../lib/utils`。
4. 类型导入常放在最后或与值导入分开，并尽量写成 `import type`：见 `src/App.tsx`、`src/lib/utils.ts`、`src/components/ui/button.tsx`。

**Path Aliases:**

- 主应用 `src/` 未配置 TS path alias，根 `tsconfig.json` 也没有 `paths`，因此沿用相对路径导入。
- `website/` 单独配置了 `@/*` 别名，见 `website/tsconfig.json`；站点代码应继续使用 `@/components/*`、`@/lib/utils` 形式，见 `website/app/page.tsx`、`website/components/ui/button.tsx`。

## Error Handling

**Patterns:**

- 前端异步调用普遍使用 `try/catch` 包住 Tauri `invoke`、文件操作或解析逻辑，失败后记录日志并回退默认值或状态，见 `src/stores/configStore.ts`、`src/components/DetailView/DetailView.tsx`、`src/services/analytics.ts`。
- 读取型操作常吞掉异常并返回安全默认值：例如 `parseContentMetadata` 返回 `null`，`normalizeUrlString` 返回空字符串，`getAutoStartupStatus` 返回 `false`，见 `src/lib/preview/entryPresentation.ts`、`src/stores/clipboardStore.ts`、`src/stores/configStore.ts`。
- 写入型操作若调用方需要感知失败，则 catch 后重新抛出：`registerGlobalShortcut`、`setAutoStartup`、`cleanupExpiredEntries`，见 `src/stores/configStore.ts`。
- 组件层常在 effect 内设置“过期保护”或 source key，避免异步结果串台，见 `src/components/DetailView/DetailView.tsx`、`src/components/DetailView/scene/AlternateViews.tsx`。
- Rust 内部实现使用 `anyhow::Result`，在 Tauri 命令边界统一 `map_err(|e| e.to_string())` 暴露给前端，见 `src-tauri/src/database/mod.rs`、`src-tauri/src/commands.rs`。

## Logging

**Framework:** `console`（前端）与 `log` / `tauri-plugin-log`（Rust）。

**Patterns:**

- 前端日志主要使用 `console.error`、`console.warn`、`console.log`，并倾向带模块前缀：`[DetailView]`、`[resolveUrlPreview]`、`[ConfigStore]`，见 `src/components/DetailView/DetailView.tsx`、`src/stores/clipboardStore.ts`、`src/stores/configStore.ts`。
- 记录内容通常围绕失败原因、fallback 分支与调试状态；没有统一 logger 封装，新增日志时保持简短且可定位模块。
- Rust 侧使用 `log::info!`、`log::warn!`、`log::error!`，并在 `src-tauri/src/lib.rs` 中通过 `tauri_plugin_log::Builder` 输出到 stdout、日志目录和 webview。

## Comments

**When to Comment:**

- 注释偏少，保留给非直观流程、平台差异、兼容性 fallback 和测试步骤说明。
- 常见位置包括：
  - 兼容性/降级说明：`src/components/ClipboardMenuHandler.tsx`。
  - 平台或目录说明：`src-tauri/src/lib.rs`、`src-tauri/Cargo.toml`。
  - 测试步骤标记：`src-tauri/src/integration_tests.rs`、`src-tauri/src/performance_tests.rs`。
- 新代码如果语义明显，不额外补注释；只有在 fallback、协议映射、异步竞态保护等逻辑不直观时才写短注释。

**JSDoc/TSDoc:**

- 未检测到系统性 JSDoc/TSDoc 约定。类型语义主要通过 TypeScript 接口与明确命名表达，见 `src/types/clipboard.ts`、`src/stores/configStore.ts`。

## Function Design

**Size:** 大多数 helper 函数较短，先在模块顶部定义，再由组件或 store 复用；复杂特性会在单文件中累积较多局部 helper，例如 `src/stores/clipboardStore.ts`、`src/lib/preview/previewDescriptor.ts`、`src/components/DetailView/DetailView.tsx`。

**Parameters:** 以显式对象形参和强类型参数为主：

- 纯工具函数偏向位置参数：`normalizeContentPreview(value, maxLength)`，见 `src/lib/preview/entryPresentation.ts`。
- 较复杂构建器偏向对象参数，便于扩展：`buildPreviewDescriptor({ entry, resolvedData, labels })`，见 `src/lib/preview/previewDescriptor.ts`。
- Store action 和 Tauri command 明确写出参数类型与返回 Promise/Result，见 `src/stores/configStore.ts`、`src-tauri/src/commands.rs`。

**Return Values:**

- 工具函数常返回简单可空值作为 guard：`''`、`null`、`undefined`，见 `src/lib/preview/entryPresentation.ts`、`src/stores/clipboardStore.ts`。
- React 组件在无内容时直接 `return null` 或返回空状态组件，见 `src/components/DetailView/scene/AlternateViews.tsx`、`src/components/DetailView/DetailView.tsx`。
- Store action 使用 `Promise<void>` 为主；需要调用方分支判断时返回具体布尔值或数据对象，见 `src/stores/configStore.ts`、`src/stores/clipboardStore.ts`。

## Module Design

**Exports:** 主应用以命名导出为主，默认导出只出现在入口页面或应用壳层：

- 默认导出：`src/App.tsx`、`website/app/page.tsx`、`website/app/layout.tsx`。
- 命名导出：`src/components/DetailView/DetailView.tsx`、`src/stores/configStore.ts`、`src/services/analytics.ts`。

**Barrel Files:** 使用很少，仅在需要聚合同类 renderer 时使用 barrel：`src/components/DetailView/ContentRenderers/index.ts`。新增模块默认直接从源文件导出，只有同类组件集合才新增 barrel。

**Cross-project Note:**

- `src/`、`src-tauri/`、`website/` 当前并不共享统一代码风格工具链。
- 在 `src/` 中继续遵守根 `.prettierrc` 与 `eslint.config.js`。
- 在 `src-tauri/` 中继续遵守 `cargo fmt` / `cargo clippy`。
- 在 `website/` 中保持现有 Next/shadcn 风格，不要强行套用根前端的单引号加分号输出。

---

_Convention analysis: 2026-03-27_
