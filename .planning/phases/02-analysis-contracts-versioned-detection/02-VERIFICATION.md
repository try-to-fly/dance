---
phase: 02-analysis-contracts-versioned-detection
verified: 2026-03-28T05:34:26Z
status: human_needed
score: 4/4 must-haves verified
human_verification:
  - test: '运行桌面应用并复制 URL、JSON、畸形 JSON、Base64 文本'
    expected: '新历史项显示稳定 subtype；畸形输入降级为 plain_text 且保留 diagnostics；raw 内容仍可查看'
    why_human: '真实剪贴板监听依赖 OS pasteboard、Tauri 事件与桌面壳，静态检查和单元测试不能完全替代'
  - test: '在 Preferences 触发 analysis rebuild 后打开 URL 与长 JSON 详情'
    expected: '重建结果摘要出现并刷新历史；URL 仍以 url_card 为主视图；JSON Raw 入口可见，长内容列与 Monaco 代码视图可滚动'
    why_human: '最终交互、滚动手感与桌面布局只能在真实渲染环境中确认'
---

# Phase 2: Analysis Contracts & Versioned Detection Verification Report

**Phase Goal:** Users get stable developer-content analysis that can be re-applied to history and still falls back cleanly when parsing fails.
**Verified:** 2026-03-28T05:34:26Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

本次验证以 Phase 02 各执行计划 frontmatter 里的 `must_haves` 为实现合同，以 `ROADMAP.md` 的 4 条 success criteria 作为 phase 级 observable truths。代码、关键链路、数据流和定向测试都已核对；未发现阻塞性 gap。

### Observable Truths

| #   | Truth                                                                                                                                                          | Status     | Evidence                                                                                                                                                                                                                                                                                                                                          |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | 用户复制受支持的 developer content 时，能得到稳定 subtype：URL、JSON、code、command、color、markdown、email、IP、timestamp、base64，以及 plain-text fallback。 | ✓ VERIFIED | `src-tauri/src/analysis/contract.rs` 定义稳定 subtype/version/status；`src-tauri/src/analysis/service.rs` 统一分析入口并映射现有 detector；`src-tauri/src/clipboard/monitor.rs` 在 `Persist` gating 后只走 `TextAnalysisService`；`src-tauri/src/analysis_contract_tests.rs` 的 `test_text_analysis_contract_preserves_supported_subtypes` 通过。 |
| 2   | 用户能看到 subtype-specific metadata，包括 URL parts、color formats、detected language、timestamp formats 及其他结构化提示。                                   | ✓ VERIFIED | `AnalysisMetadata` 为各 subtype 提供稳定 variant；`TextAnalysisService` 构造 URL/IP/email/color/code/command/timestamp/json/markdown/base64 metadata；`entry_analysis` 持久化 `metadata_json`；`src/lib/preview/entryPresentation.ts` 和 `src/lib/preview/previewDescriptor.ts` 以 analysis-first 消费 metadata。                                 |
| 3   | 用户可以对既有历史重跑升级后的 detection，而不需要重新复制内容。                                                                                               | ✓ VERIFIED | `src-tauri/src/analysis/repository.rs` 提供 `list_stale_entry_ids(...)`；`src-tauri/src/analysis/rebuild.rs` 批量重建 stale/missing analysis；`src-tauri/src/state.rs`、`src-tauri/src/commands.rs`、`src/components/Preferences/PreferencesModal.tsx` 形成前后端触发闭环；`test_rebuild_text_analysis_updates_existing_history` 通过。           |
| 4   | 当解析失败时，用户仍可查看 raw content，并保留 diagnostics 以便后续修复。                                                                                      | ✓ VERIFIED | `AnalysisSnapshot::fallback_plain_text(...)` 明确保存 `status=fallback`、plain-text metadata 与 diagnostics；`src-tauri/src/capture/runtime.rs` 同事务写 raw row 与 companion analysis；`src/components/DetailView/DetailView.tsx` 和 `src/lib/preview/previewDescriptor.ts` 暴露 fallback badge/inspector/raw view；后端与前端回归测试均通过。   |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact                                                      | Expected                                                      | Status     | Details                                                                                                                                                |
| ------------------------------------------------------------- | ------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `src-tauri/src/analysis/mod.rs`                               | 汇总 analysis 模块公开入口                                    | ✓ VERIFIED | 导出 contract、service、repository、rebuild，以及核心类型和 helper。                                                                                   |
| `src-tauri/src/analysis/contract.rs`                          | Rust authoritative analysis contract                          | ✓ VERIFIED | 定义 `ANALYSIS_CONTRACT_VERSION`、`TEXT_ANALYSIS_VERSION`、`AnalysisSubtype`、`AnalysisStatus`、typed metadata、diagnostics、`AnalysisSnapshot`。      |
| `src-tauri/src/analysis/service.rs`                           | 单一分析服务入口                                              | ✓ VERIFIED | `TextAnalysisService::analyze()` 统一调用 detector、构造 typed metadata，并在 malformed JSON/URL/Base64 时走显式 fallback。                            |
| `src-tauri/src/analysis_contract_tests.rs`                    | subtype/precedence/fallback contract tests                    | ✓ VERIFIED | 覆盖 11 个 subtype、歧义 precedence、fallback serialization diagnostics。                                                                              |
| `src-tauri/src/database/mod.rs`                               | `entry_analysis` schema 与迁移                                | ✓ VERIFIED | 建表含 `contract_version`、`analysis_version`、`metadata_json`、`diagnostics_json`、外键和索引；数据库测试覆盖列与 upsert。                            |
| `src-tauri/src/analysis/repository.rs`                        | analysis persistence/join/stale selector                      | ✓ VERIFIED | 提供 `upsert_entry_analysis(...)`、`load_entry_analysis_for_history(...)`、`list_stale_entry_ids(...)`，并把 joined row 解析回 `AnalysisSnapshot`。    |
| `src-tauri/src/models/mod.rs`                                 | `ClipboardEntry.analysis` DTO 与兼容字段回写                  | ✓ VERIFIED | `ClipboardEntry` 新增 `analysis`，`attach_analysis()` 同步回写 `content_subtype`/`metadata` 兼容字段。                                                 |
| `src/types/clipboard.ts`                                      | 前端 `analysis` 类型合同                                      | ✓ VERIFIED | 定义 `EntryAnalysisSnapshot`、`EntryAnalysisMetadata`、diagnostics/status/subtype 类型。                                                               |
| `src-tauri/src/state.rs`                                      | authoritative history read model 与 rebuild state entry point | ✓ VERIFIED | `get_clipboard_history()` 改走 `load_entry_analysis_for_history()`；`rebuild_entry_analysis()` 透传到 rebuilder。                                      |
| `src-tauri/src/clipboard/monitor.rs`                          | Persist gating 后 authoritative analysis 接入                 | ✓ VERIFIED | `process_text_capture_with_analysis(...)` 在 gating 后创建 `AnalysisSnapshot` 并附到 entry。                                                           |
| `src-tauri/src/capture/runtime.rs`                            | raw row + companion analysis 同步持久化                       | ✓ VERIFIED | `persist_entry(...)` 在同一事务里 upsert `clipboard_entries`，再 upsert `entry_analysis`，最后把 joined-compatible entry 发回前端。                    |
| `src-tauri/src/integration_tests.rs`                          | capture/fallback/rebuild 集成测试                             | ✓ VERIFIED | 覆盖 capture 写入 analysis、fallback diagnostics、history rebuild、不改 raw row 等关键路径。                                                           |
| `src/lib/preview/entryPresentation.ts`                        | analysis-first helper                                         | ✓ VERIFIED | `getEntryAnalysisSubtype()`、`getEntryPresentationMetadata()` 等 helper 优先消费 `entry.analysis`，仅在 companion row 缺失时 fallback 到 legacy 字段。 |
| `src/lib/preview/previewDescriptor.ts`                        | analysis-first detail descriptor                              | ✓ VERIFIED | 以 analysis-first 决定 `primaryKind`、badges、inspector、alternate views；URL 条目固定 `url_card` 主视图。                                             |
| `src/stores/clipboardStore.ts`                                | 预览解析只基于 authoritative subtype 做渲染降级               | ✓ VERIFIED | `resolveEntryPreview()` 使用 `getEntryAnalysisSubtype(entry)`，URL 解析只产出 resolved payload，不反向决定语义 subtype。                               |
| `src/components/DetailView/DetailView.tsx`                    | detail UI 对 fallback/status/diagnostics 的可见消费           | ✓ VERIFIED | 用 analysis-first helper 构造 metadata pills、descriptor 和 fallback 提示。                                                                            |
| `src/components/DetailView/DetailPreviewContract.test.tsx`    | detail contract regressions                                   | ✓ VERIFIED | 覆盖 analysis-first override、fallback diagnostics、URL-first preview、resolved alternate views。                                                      |
| `src-tauri/src/analysis/rebuild.rs`                           | batched history reanalysis                                    | ✓ VERIFIED | 使用 stale selector + `TextAnalysisService` + `upsert_entry_analysis()` 批量更新历史 analysis。                                                        |
| `src-tauri/src/commands.rs`                                   | rebuild Tauri command 暴露                                    | ✓ VERIFIED | 暴露 `rebuild_entry_analysis` 命令并返回 `RebuildEntryAnalysisResult`。                                                                                |
| `src/components/Preferences/PreferencesModal.tsx`             | 用户可触发 rebuild 的最小入口                                 | ✓ VERIFIED | 包含 rebuild 按钮、结果摘要、错误反馈，并在成功后刷新 preview cache、history 与 cache statistics。                                                     |
| `src/components/DetailView/scene/AlternateViews.tsx`          | raw-only 与 resolved media/json/text alternate 渲染           | ✓ VERIFIED | 单一 raw 视图仍渲染；resolved image/audio/video/json/text 都有明确渲染路径。                                                                           |
| `src/components/DetailView/scene/DetailScene.tsx`             | raw-only alternate 可见性与滚动容器合同                       | ✓ VERIFIED | 非 immersive 场景保留 raw-only alternate view；主列使用 `overflow-y-auto` 保证长内容可滚动。                                                           |
| `src/components/DetailView/ContentRenderers/JsonRenderer.tsx` | 显式高度 JSON tree/code renderer                              | ✓ VERIFIED | 统一 `contentHeight` 容器；树视图与 Monaco 代码视图都绑定显式高度。                                                                                    |
| `src/components/DetailView/DetailView.test.tsx`               | JSON raw-only/fallback detail regressions                     | ✓ VERIFIED | 覆盖 JSON raw-only alternate 可见、fallback raw 主视图与分析提示等场景。                                                                               |

### Key Link Verification

| From                                                          | To                                                                 | Via                                                                        | Status  | Details                                                                                                            |
| ------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------ |
| `src-tauri/src/analysis/service.rs`                           | `src-tauri/src/clipboard/content_detector.rs`                      | contract adapter over existing detector heuristics                         | ✓ WIRED | `TextAnalysisService::analyze()` 调用 `ContentDetector::detect(...)`，同时把 tuple 输出提升为 `AnalysisSnapshot`。 |
| `src-tauri/src/lib.rs`                                        | `src-tauri/src/analysis_contract_tests.rs`                         | `cfg(test)` module registration                                            | ✓ WIRED | `src-tauri/src/lib.rs` 注册了 `mod analysis_contract_tests;`。                                                     |
| `src-tauri/src/database/mod.rs`                               | `src-tauri/src/analysis/repository.rs`                             | `entry_analysis` migration and SQL helper usage                            | ✓ WIRED | schema 建立 `entry_analysis`；数据库测试直接调用 repository helper 验证 upsert/join。                              |
| `src-tauri/src/state.rs`                                      | `src-tauri/src/models/mod.rs`                                      | history join row mapped into `ClipboardEntry` DTO                          | ✓ WIRED | `get_clipboard_history()` 走 joined loader；`map_history_row()` + `ClipboardEntry::attach_analysis()` 组装 DTO。   |
| `src/types/clipboard.ts`                                      | `src-tauri/src/models/mod.rs`                                      | Tauri serialization contract                                               | ✓ WIRED | 前后端共享 `analysis`/diagnostics/version 字段，命令返回 `ClipboardEntry` 时可序列化到 TS 类型。                   |
| `src-tauri/src/clipboard/monitor.rs`                          | `src-tauri/src/analysis/service.rs`                                | text capture analysis                                                      | ✓ WIRED | `ClipboardMonitor` 持有 `TextAnalysisService`，仅在 `Persist` 文本路径创建 snapshot。                              |
| `src-tauri/src/capture/runtime.rs`                            | `src-tauri/src/analysis/repository.rs`                             | same save flow persists companion row                                      | ✓ WIRED | `persist_entry()` 内在事务中调用 `upsert_entry_analysis(...)`。                                                    |
| `src-tauri/src/integration_tests.rs`                          | `src-tauri/src/capture/runtime.rs`                                 | persisted history assertions                                               | ✓ WIRED | 集成测试验证 capture 后 raw row、analysis row 和 history read model 都一致。                                       |
| `src/lib/preview/entryPresentation.ts`                        | `src/types/clipboard.ts`                                           | analysis snapshot helpers                                                  | ✓ WIRED | helper 直接读取 `EntryAnalysisSnapshot`、diagnostics 和 metadata unions。                                          |
| `src/stores/clipboardStore.ts`                                | `src/components/DetailView/DetailView.tsx`                         | `resolveEntryPreview` keeps rendering data separate from subtype authority | ✓ WIRED | `DetailView` 从 store 读取 `resolveEntryPreview()`，descriptor 仍以 analysis-first subtype 决定主视图。            |
| `src/lib/preview/previewDescriptor.ts`                        | `src/components/DetailView/DetailPreviewContract.test.tsx`         | contract tests                                                             | ✓ WIRED | `buildPreviewDescriptor(...)` 的 analysis-first、URL-first、fallback 行为被前端 contract tests 锁定。              |
| `src-tauri/src/analysis/rebuild.rs`                           | `src-tauri/src/analysis/repository.rs`                             | select stale rows and upsert fresh snapshots                               | ✓ WIRED | rebuilder 调用 `list_stale_entry_ids(...)` 加载候选项，并对每项调用 `upsert_entry_analysis(...)`。                 |
| `src-tauri/src/commands.rs`                                   | `src-tauri/src/state.rs`                                           | `rebuild_entry_analysis` command                                           | ✓ WIRED | 命令层直接调用 `state.rebuild_entry_analysis(batch_size)`。                                                        |
| `src/components/Preferences/PreferencesModal.tsx`             | `src-tauri/src/commands.rs`                                        | `invoke('rebuild_entry_analysis')`                                         | ✓ WIRED | rebuild 按钮调用 Tauri command，成功后刷新 history、preview cache 和 cache stats。                                 |
| `src/stores/clipboardStore.ts`                                | `src/lib/preview/previewDescriptor.ts`                             | resolveEntryPreview only supplies actual resolved URL payload              | ✓ WIRED | URL 分支只把真实 resolved payload 注入 descriptor 所需的 `resolvedData`，不会预填 raw URL 误导主视图。             |
| `src/components/DetailView/scene/AlternateViews.tsx`          | `src/components/DetailView/scene/AlternateViews.test.tsx`          | `renderView`                                                               | ✓ WIRED | resolved-json/image/audio/video 与 raw-only 路径都有组件测试覆盖。                                                 |
| `src/components/DetailView/scene/DetailScene.tsx`             | `src/components/DetailView/scene/AlternateViews.tsx`               | `showAlternateViews`                                                       | ✓ WIRED | 仅 immersive + raw-only 时隐藏冗余备用视图；普通 JSON/code/markdown/plain_text 场景继续显示。                      |
| `src/components/DetailView/ContentRenderers/JsonRenderer.tsx` | `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx` | explicit height contract                                                   | ✓ WIRED | `contentHeight` 同时驱动 shell 与 Monaco height，并被测试断言。                                                    |
| `src/components/DetailView/DetailView.test.tsx`               | `src/components/DetailView/scene/DetailScene.tsx`                  | JSON raw-only detail regression                                            | ✓ WIRED | detail tests 断言 JSON raw-only alternate 可见、主列可滚动、fallback 诊断可见。                                    |

### Data-Flow Trace (Level 4)

| Artifact                                          | Data Variable                                                   | Source                                            | Produces Real Data                                                                                               | Status    |
| ------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | --------- |
| `src-tauri/src/clipboard/monitor.rs`              | `snapshot`                                                      | `TextAnalysisService::analyze(trimmed_text)`      | 是。由 authoritative analyzer 生成 subtype、metadata、diagnostics，并附到 `ClipboardEntry`。                     | ✓ FLOWING |
| `src-tauri/src/capture/runtime.rs`                | `snapshot` / `stored_entry.analysis`                            | monitor 传入的 `entry.analysis`                   | 是。事务中写入 `clipboard_entries` 与 `entry_analysis`，随后把兼容字段和 `analysis` 一起发回前端。               | ✓ FLOWING |
| `src-tauri/src/analysis/repository.rs`            | `analysis_*` joined columns                                     | `LEFT JOIN entry_analysis a ON a.entry_id = e.id` | 是。真实查询 `entry_analysis` 并解析为 `AnalysisSnapshot`；缺失 row 时保留 legacy fallback。                     | ✓ FLOWING |
| `src-tauri/src/state.rs`                          | history `Vec<ClipboardEntry>`                                   | `load_entry_analysis_for_history(...)`            | 是。历史读取优先得到 authoritative `analysis`，并通过 tests 验证 legacy row 仍可读。                             | ✓ FLOWING |
| `src/lib/preview/previewDescriptor.ts`            | `subType` / `analysisStatus` / `diagnostics` / `alternateViews` | `entry.analysis` + `resolvedData`                 | 是。descriptor 直接消费 joined entry 与 store resolved payload，生成 URL-first/fallback-aware preview contract。 | ✓ FLOWING |
| `src/components/Preferences/PreferencesModal.tsx` | `rebuildResult`                                                 | `invoke('rebuild_entry_analysis')`                | 是。真实调用后端命令，成功时刷新 preview cache、history、cache statistics。                                      | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior                                                    | Command                                                                                                                                                                                                                                    | Result                     | Status |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------- | ------ |
| 稳定 subtype corpus                                         | `cargo test test_text_analysis_contract_preserves_supported_subtypes -- --nocapture`                                                                                                                                                       | `1 passed`                 | ✓ PASS |
| authoritative analysis 与 legacy fallback joined read model | `cargo test test_database_merges_analysis_fields -- --nocapture`                                                                                                                                                                           | `1 passed`                 | ✓ PASS |
| 历史 reanalysis 更新 companion rows 而不改写 raw row        | `cargo test test_rebuild_text_analysis_updates_existing_history -- --nocapture`                                                                                                                                                            | `1 passed`                 | ✓ PASS |
| 前端 analysis-first/fallback/UI regressions                 | `pnpm test --run src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx src/components/Preferences/PreferencesModal.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx` | `4 files, 26 tests passed` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan                                                   | Description                                                         | Status      | Evidence                                                                                                                |
| ----------- | ------------------------------------------------------------- | ------------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------- |
| `DETE-01`   | `02-01`, `02-03`, `02-04`                                     | Stable subtype classification for supported developer content       | ✓ SATISFIED | authoritative contract + `TextAnalysisService` + capture integration + analysis-first frontend contract tests。         |
| `DETE-02`   | `02-01`, `02-02`, `02-03`, `02-04`, `02-06`                   | Subtype-specific structured metadata for supported content          | ✓ SATISFIED | typed `AnalysisMetadata` variants、`entry_analysis.metadata_json`、history join、detail descriptor/inspector 展示。     |
| `DETE-03`   | `02-02`, `02-05`                                              | Existing history can benefit from upgraded detection without recopy | ✓ SATISFIED | stale selector + batched rebuilder + Tauri command + Preferences rebuild entry point + integration test。               |
| `DETE-04`   | `02-01`, `02-02`, `02-03`, `02-04`, `02-05`, `02-06`, `02-07` | Graceful fallback to raw content with preserved diagnostics         | ✓ SATISFIED | fallback snapshot、diagnostics persistence、detail fallback badge/inspector/raw tab、JSON raw-only scroll regressions。 |

Phase 02 在 `PLAN` frontmatter 中声明的 requirement IDs 为 `DETE-01`、`DETE-02`、`DETE-03`、`DETE-04`。这些 ID 在 `REQUIREMENTS.md` 中全部可追踪；未发现额外映射到 Phase 02 但未被任何 plan 声明的 orphaned requirement。  
补充说明：`REQUIREMENTS.md` traceability 表仍把 `DETE-01` 与 `DETE-03` 标为 `Pending`，但当前代码证据和定向测试已经满足这两项 requirement，文档状态落后于实现。

### Anti-Patterns Found

| File                                              | Line | Pattern            | Severity | Impact                                                                                              |
| ------------------------------------------------- | ---- | ------------------ | -------- | --------------------------------------------------------------------------------------------------- |
| `src-tauri/src/commands.rs`                       | 1715 | `placeholder` 注释 | ℹ️ Info  | 这是日志级别运行时切换的旧占位说明，与 Phase 02 的 analysis/rebuild 链路无关，不阻塞本 phase goal。 |
| `src/components/Preferences/PreferencesModal.tsx` | 125  | `console.log`      | ℹ️ Info  | Preferences 仍保留调试日志，会产生噪声，但不影响 rebuild 行为闭环。                                 |

未发现会阻断 Phase 02 目标达成的 stub、orphaned artifact 或未接线关键链路。

### Human Verification Required

### 1. Live Clipboard Capture

**Test:** 运行桌面应用，依次复制 HTTPS URL、合法 JSON、畸形 JSON、Base64 文本。  
**Expected:** 新历史项显示稳定 subtype；合法内容显示对应 metadata；畸形输入降级为 `plain_text`，但 raw 内容仍可查看且 diagnostics 可见。  
**Why human:** 真实剪贴板监听依赖 OS pasteboard、Tauri runtime 和桌面事件循环，代码审查与单元/集成测试不能完全替代。

### 2. Desktop Rebuild + Detail UI Sanity

**Test:** 打开 Preferences 的 system/cache 区域，点击 analysis rebuild；随后打开一个 URL 条目和一个长 JSON 条目的详情。  
**Expected:** rebuild 结果摘要出现并刷新历史；URL 仍以 `url_card` 为主视图；JSON Raw 入口可达；长内容列和 Monaco 代码视图可以滚动。  
**Why human:** 最终交互、布局裁切、滚动手感和真实桌面渲染效果只能在运行中的应用里确认。

### Gaps Summary

未发现阻塞性 gap。Phase 02 的 must-haves 在代码、关键链路、数据流和定向测试层面都已闭合：Rust authoritative analysis contract 已存在并接入 capture/runtime，`entry_analysis` companion table 成为持久化权威，history 可重建并从 Preferences 触发，前端 detail/store 也已 analysis-first 且在 fallback 场景保留 raw content 与 diagnostics。剩余事项只是在真实桌面环境中做最终 sanity check。

---

_Verified: 2026-03-28T05:34:26Z_  
_Verifier: Claude (gsd-verifier)_
