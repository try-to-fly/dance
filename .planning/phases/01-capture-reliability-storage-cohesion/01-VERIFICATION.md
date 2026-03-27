---
phase: 01-capture-reliability-storage-cohesion
verified: 2026-03-27T15:37:48Z
status: human_needed
score: 4/4 must-haves verified
---

# Phase 1: Capture Reliability & Storage Cohesion Verification Report

**Phase Goal:** Users can trust clipboard capture to start and stop cleanly, avoid unwanted entries, and keep local data under one coherent storage lifecycle.
**Verified:** 2026-03-27T15:37:48Z
**Status:** human_needed
**Re-verification:** Yes - after gap closure plan `01-06`

## Goal Achievement

### Observable Truths

| #   | Truth                                                                                                                                 | Status     | Evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| --- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | User can start monitoring, stop monitoring, and confirm no new clipboard items are saved after monitoring is turned off.              | ✓ VERIFIED | `AppState::start_monitoring()` / `stop_monitoring()` 仍保持单一 runtime owner；`CaptureRuntime::stop()` 会 cancel 并等待 monitor/save 任务结束；`cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture` 与 `cargo test test_capture_runtime_restart_is_single_owner -- --nocapture` 的既有验证结果仍成立。                                                                                                                                           |
| 2   | User sees each eligible clipboard change recorded once, even after repeated start/stop cycles or app-driven copy flows.               | ✓ VERIFIED | SQLite `content_hash` UPSERT、后端 suppression contract 和前端统一 copy route 没有在 gap closure 中被回退；既有 Rust/Vitest 合同测试仍覆盖这条链路。                                                                                                                                                                                                                                                                                                             |
| 3   | User can keep ignored, transient, concealed, or other non-persistent clipboard events out of saved history.                           | ✓ VERIFIED | `decide_capture()`、macOS marker adapter 和 monitor 前置 gating 仍在位；`cargo test test_capture_policy_current_only_is_non_persistent_in_v1 -- --nocapture` 的既有验证结果仍成立。                                                                                                                                                                                                                                                                              |
| 4   | User can restart the app and still access history, cached previews, and image assets without path-related mismatches or missing data. | ✓ VERIFIED | `ClipboardMonitor::get_saved_file_size()` 已改为通过 `ContentProcessor::resolve_relative_asset_path()` 解析 `imgs/...`；`get_log_content()` / `clear_logs()` 已委托 `AppPaths::log_dir()/clipboard-app.log`；`cargo test test_app_paths_log_commands_follow_log_dir -- --nocapture`、`cargo test test_app_paths_resolve_relative_asset_path_for_nested_imgs_assets -- --nocapture` 与 `cargo test test_app_paths_migrate_legacy_roots -- --nocapture` 全部通过。 |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact                                                                  | Expected                                | Status                 | Details                                                                                                                                            |
| ------------------------------------------------------------------------- | --------------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/test_support.rs`                                           | temp-root、fixture seed 与测试隔离辅助  | ✓ VERIFIED             | 继续为 CAPT-04 temp-root regression 提供隔离根目录与 seed helpers。                                                                                |
| `src-tauri/src/app_paths_tests.rs`                                        | CAPT-04 路径与迁移测试                  | ✓ EXISTS + SUBSTANTIVE | 现含 temp-root roots、legacy migration、nested `imgs/...` 解析，以及 `clipboard-app.log` helper regression 测试。                                  |
| `src-tauri/src/clipboard/processor.rs`                                    | 图片资产路径 authority bridge           | ✓ EXISTS + SUBSTANTIVE | `ContentProcessor` 保存 `Arc<AppPaths>`，并暴露 `resolve_relative_asset_path()` 给 monitor 复用。                                                  |
| `src-tauri/src/clipboard/monitor.rs`                                      | 图片 metadata 回读接入 `AppPaths`       | ✓ EXISTS + SUBSTANTIVE | `get_saved_file_size(&self, ...)` 对 `imgs/...` 改走 `self.processor.resolve_relative_asset_path(...)`。                                           |
| `src-tauri/src/commands.rs`                                               | 日志读取/清理接入 `AppPaths::log_dir()` | ✓ EXISTS + SUBSTANTIVE | `app_log_file_path()`、`read_log_content_in()`、`clear_log_file_in()` 已形成共享路径 helper，Tauri command 改为从 `State<'_, AppState>` 委托执行。 |
| `src-tauri/src/capture_runtime_tests.rs`                                  | CAPT-01/CAPT-02 自动化验证              | ✓ VERIFIED             | stop cancel、single worker/suppression、restart single owner、brownfield dedupe migration 测试仍在位。                                             |
| `src-tauri/src/capture_policy_tests.rs`                                   | CAPT-03 自动化验证                      | ✓ VERIFIED             | marker matrix、CurrentOnly 非持久化和非 macOS no-op marker 测试仍在位。                                                                            |
| `src/stores/clipboardStore.test.ts`                                       | store copy-routing contract tests       | ✓ VERIFIED             | 共享 helper 合同未被 gap closure 回退。                                                                                                            |
| `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`        | renderer copy-routing contract tests    | ✓ VERIFIED             | JSON 复制继续走后端 contract。                                                                                                                     |
| `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` | renderer copy-routing contract tests    | ✓ VERIFIED             | 文本/命令复制继续走后端 contract。                                                                                                                 |

### Key Link Verification

| From                                   | To                                     | Via                             | Status  | Details                                                                                                                                                |
| -------------------------------------- | -------------------------------------- | ------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `src-tauri/src/app_paths.rs`           | `src-tauri/src/clipboard/processor.rs` | `image_assets_dir()`            | ✓ WIRED | `ContentProcessor::new_in(paths)` 继续以 `paths.image_assets_dir()` 构造图片目录。                                                                     |
| `src-tauri/src/clipboard/processor.rs` | `src-tauri/src/clipboard/monitor.rs`   | `resolve_relative_asset_path()` | ✓ WIRED | `ClipboardMonitor::get_saved_file_size(&self, ...)` 对 `imgs/...` 调用 `self.processor.resolve_relative_asset_path(file_path)`。                       |
| `src-tauri/src/app_paths.rs`           | `src-tauri/src/commands.rs`            | `log_dir()`                     | ✓ WIRED | `app_log_file_path(paths)` 返回 `paths.log_dir().join("clipboard-app.log")`，`get_log_content` / `clear_logs` 只委托该 helper。                        |
| `src-tauri/src/commands.rs`            | `src-tauri/src/app_paths_tests.rs`     | log helper regression           | ✓ WIRED | `test_app_paths_log_commands_follow_log_dir` 直接调用 `app_log_file_path` / `read_log_content_in` / `clear_log_file_in`，验证 temp-root 日志路径合同。 |
| `src-tauri/src/app_paths.rs`           | `src-tauri/src/app_paths_tests.rs`     | relative asset regression       | ✓ WIRED | `test_app_paths_resolve_relative_asset_path_for_nested_imgs_assets` 锁定 `imgs/nested/example.png` -> `data/imgs/nested/example.png`。                 |
| `src-tauri/src/capture/policy.rs`      | `src-tauri/src/clipboard/monitor.rs`   | `decide_capture`                | ✓ WIRED | marker-first gating 继续发生在 `ContentDetector::detect(...)` 之前。                                                                                   |
| `src-tauri/src/capture/runtime.rs`     | `src-tauri/src/database/mod.rs`        | UPSERT save loop                | ✓ WIRED | save loop 仍通过 `persist_entry()` 做 `content_hash` UPSERT。                                                                                          |
| `src/stores/clipboardStore.ts`         | renderer copy contract tests           | shared backend copy helper      | ✓ WIRED | 前端 copy contract 在本次 gap closure 中未被更改。                                                                                                     |

### Behavioral Spot-Checks

| Behavior                               | Command                                                                                                                                                                                   | Result                                                    | Status |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- | ------ |
| 停止监听后不再持久化新条目             | `cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture`                                                                                                                       | 1 test passed                                             | ✓ PASS |
| 重启监听仍只有一个 runtime owner       | `cargo test test_capture_runtime_restart_is_single_owner -- --nocapture`                                                                                                                  | 1 test passed                                             | ✓ PASS |
| eligible 条目去重且 suppression 生效   | `cargo test test_capture_runtime_single_worker_and_suppression -- --nocapture`                                                                                                            | 1 test passed                                             | ✓ PASS |
| CurrentOnly 事件不检测、不发送、不入库 | `cargo test test_capture_policy_current_only_is_non_persistent_in_v1 -- --nocapture`                                                                                                      | 1 test passed                                             | ✓ PASS |
| legacy root 迁移幂等可执行             | `cargo test test_app_paths_migrate_legacy_roots -- --nocapture`                                                                                                                           | 1 test passed                                             | ✓ PASS |
| nested `imgs/...` 相对路径解析固定     | `cargo test test_app_paths_resolve_relative_asset_path_for_nested_imgs_assets -- --nocapture`                                                                                             | 1 test passed                                             | ✓ PASS |
| 日志命令只走 `AppPaths::log_dir()`     | `cargo test test_app_paths_log_commands_follow_log_dir -- --nocapture`                                                                                                                    | 1 test passed                                             | ✓ PASS |
| gap closure 未破坏整体编译             | `cargo test --no-run`                                                                                                                                                                     | build/test binaries generated successfully                | ✓ PASS |
| 前端复制入口统一走 backend contract    | `pnpm test -- src/stores/clipboardStore.test.ts src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` | 既有验证结果为 36 tests passed，本次 gap closure 未改前端 | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan               | Description                                                                                                                           | Status      | Evidence                                                                                                           |
| ----------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------ |
| `CAPT-01`   | `01-01`, `01-03`          | User can start/stop monitoring without hidden listeners continuing after stop                                                         | ✓ SATISFIED | `AppState` 统一持有 runtime，`CaptureRuntime::stop()` 会 cancel 并 await 两个后台任务，Rust 行为测试已通过。       |
| `CAPT-02`   | `01-01`, `01-03`, `01-04` | User sees each eligible clipboard change recorded once without duplicate entries from repeated listeners or self-generated copy flows | ✓ SATISFIED | SQLite `content_hash` UPSERT、后端 suppression contract、前端统一 copy route 与合同测试均在位。                    |
| `CAPT-03`   | `01-01`, `01-05`          | User can keep ignored, transient, concealed, or otherwise non-persistent clipboard events out of saved history                        | ✓ SATISFIED | marker-first `decide_capture()`、macOS marker adapter、monitor 前置 gating、CurrentOnly 非持久化测试均在位。       |
| `CAPT-04`   | `01-01`, `01-02`, `01-06` | User can rely on one consistent local storage lifecycle for history, cache, image assets, and related metadata                        | ✓ SATISFIED | `get_saved_file_size()` 与日志命令都已改走 `AppPaths`，且新增 temp-root regression 覆盖 nested assets 与日志路径。 |

**Requirement accounting:** `CAPT-01`、`CAPT-02`、`CAPT-03`、`CAPT-04` 全部在至少一个 PLAN frontmatter 中出现，且当前代码与自动化证据都能回溯到对应 requirement。

### Anti-Patterns Found

| File                                        | Line  | Pattern                                   | Severity | Impact                                            |
| ------------------------------------------- | ----- | ----------------------------------------- | -------- | ------------------------------------------------- |
| `src-tauri/src/commands.rs`                 | 1690+ | `set_log_level()` 仍是 placeholder 级实现 | ℹ️ Info  | 非 Phase 01 主线，不影响 CAPT-01..04。            |
| `src-tauri/src/utils/app_icon_extractor.rs` | 206   | placeholder Windows icon fallback         | ℹ️ Info  | 预先存在的跨平台占位逻辑，不阻塞当前 phase goal。 |

## Human Verification Required

### 1. Real Clipboard Stop/Restart Smoke

**Test:** 在 macOS 真机上启动监听，复制一条普通文本；停止监听后再复制第二条；重启应用后检查历史列表。  
**Expected:** 第一条仍在历史中，第二条不会在 stop 后被偷偷入库。  
**Why human:** 依赖真实系统剪贴板事件、应用焦点和重启生命周期，单元测试不能完全替代。

### 2. Marker-First Capture Smoke

**Test:** 在 macOS 上分别触发普通文本复制、可识别的 auto-generated / concealed / remote clipboard 场景，观察历史列表。  
**Expected:** 普通文本进入历史；被 marker 标记为非持久化的事件不会进入历史。  
**Why human:** NSPasteboard marker 来自系统运行时，测试代码只能验证策略与接线，不能完全重放真实 OS 行为。

### 3. Legacy Install Migration Smoke

**Test:** 在带有历史 `dance/` 或 `clipboard-app/` 数据根的真实本地安装上启动应用，确认历史、图片资产和日志查看器行为。  
**Expected:** 历史仍可读取，已有图片资产可打开，日志查看器正常读取 `clipboard-app.log`，且不会因为旧根目录残留导致路径失配。  
**Why human:** 自动化只验证了 temp-root 迁移合同；真实本地安装还涉及现存用户目录、历史资产规模和现有日志文件落盘行为。

## Gaps Summary

本轮 re-verification 没有再发现自动化层面的 blocker。`CAPT-04` 的两个 residual bypass 已经闭合：monitor 图片 metadata 回读不再手写 legacy 根目录，日志读取/清理也已收敛到 `AppPaths::log_dir()/clipboard-app.log`，对应 temp-root regression tests 也已落地并通过。结合 Phase 1 之前已经成立的 runtime、suppression、marker-first 与 copy-routing 证据，4 个 must-have truths 都能在代码和测试层面被验证。

剩余工作只是真机 human smoke，因此当前状态是 `human_needed`，而不是 `gaps_found`。Phase 01 的代码缺口已经清零，但最终宣告“完全通过”仍需要用户在 macOS 真实剪贴板环境里走完两条 smoke。

## Verification Metadata

**Verification approach:** Goal-backward (derived from phase goal)  
**Must-haves source:** Phase 01 PLAN frontmatter + ROADMAP phase goal  
**Automated checks:** 9 passed, 0 failed  
**Human checks required:** 3  
**Total verification time:** 8 min

---

_Verified: 2026-03-27T15:37:48Z_  
_Verifier: Codex (local re-verification after gap closure)_
