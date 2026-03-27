---
phase: 01-capture-reliability-storage-cohesion
verified: 2026-03-27T15:08:57Z
status: gaps_found
score: 3/4 must-haves verified
gaps:
  - truth: 'User can restart the app and still access history, cached previews, and image assets without path-related mismatches or missing data.'
    status: partial
    reason: 'AppPaths 已经接管配置、数据库、图片写入与大部分图片读取链路，但仍有生产代码绕过这套权威路径层：ClipboardMonitor 的图片文件大小回读仍按 legacy `dirs::config_dir()/clipboard-app/...` 拼绝对路径，日志读取/清理命令也仍硬编码 `~/Library/Logs/com.dance.app/dance.log`。CAPT-04 的单一存储生命周期因此没有完全闭环。'
    artifacts:
      - path: 'src-tauri/src/clipboard/monitor.rs'
        issue: '`get_saved_file_size()` 对 `imgs/...` 仍走 legacy 根目录，没有复用 AppPaths。'
      - path: 'src-tauri/src/commands.rs'
        issue: '`get_log_content()` / `clear_logs()` 仍手写日志路径，没有消费 `AppPaths::log_dir()`。'
    missing:
      - '让图片文件大小回读复用 `AppPaths::resolve_relative_asset_path()`，或把 `AppPaths` 注入 ClipboardMonitor。'
      - '让日志命令通过 `AppPaths` 解析 log root，并补一条 CAPT-04 回归测试覆盖日志路径。'
---

# Phase 1: Capture Reliability & Storage Cohesion Verification Report

**Phase Goal:** Users can trust clipboard capture to start and stop cleanly, avoid unwanted entries, and keep local data under one coherent storage lifecycle.
**Verified:** 2026-03-27T15:08:57Z
**Status:** gaps_found
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| #   | Truth                                                                                                                                 | Status     | Evidence                                                                                                                                                                                                                                                                                                                             |
| --- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | User can start monitoring, stop monitoring, and confirm no new clipboard items are saved after monitoring is turned off.              | ✓ VERIFIED | `AppState::start_monitoring()` / `stop_monitoring()` 只创建一个 runtime 并在停止时 `take()` 后 await 停止；`CaptureRuntime::stop()` 会 cancel 并等待 monitor/save 任务结束；`cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture` 与 `cargo test test_capture_runtime_restart_is_single_owner -- --nocapture` 均通过。 |
| 2   | User sees each eligible clipboard change recorded once, even after repeated start/stop cycles or app-driven copy flows.               | ✓ VERIFIED | `persist_entry()` 对 `content_hash` 做 UPSERT，数据库初始化会先 merge brownfield duplicates 再创建唯一索引；后端 `copy_to_clipboard` 先注册 suppression key；前端共享 helper 统一调用该命令；Rust 与 Vitest 合同测试均通过。                                                                                                         |
| 3   | User can keep ignored, transient, concealed, or other non-persistent clipboard events out of saved history.                           | ✓ VERIFIED | `decide_capture()` 先按 marker/source/self/excluded/size 做 `Skip` / `CurrentOnly` / `Persist` 判定；`ClipboardMonitor` 在 `ContentDetector::detect(...)` 之前完成 gating；`cargo test test_capture_policy_current_only_is_non_persistent_in_v1 -- --nocapture` 通过。                                                               |
| 4   | User can restart the app and still access history, cached previews, and image assets without path-related mismatches or missing data. | ✗ FAILED   | `AppPaths` 与迁移测试、图片读取命令和状态层大体已接线，但仍有两个生产路径绕过权威路径层：`clipboard/monitor.rs` 的图片文件大小回读仍用 legacy `dirs::config_dir()/clipboard-app/...`，`commands.rs` 的日志读写仍手写 log 路径。CAPT-04 未完全闭环。                                                                                  |

**Score:** 3/4 truths verified

### Required Artifacts

| Artifact                                                                  | Expected                                        | Status     | Details                                                                                                             |
| ------------------------------------------------------------------------- | ----------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/test_support.rs`                                           | temp-root、fixture seed 与测试隔离辅助          | ✓ VERIFIED | 60 行，提供 `TestAppRoots`、`create_temp_app_roots()`、`sqlite_url()`、`seed_file()`、`create_dir()`。              |
| `src-tauri/src/app_paths_tests.rs`                                        | CAPT-04 路径与迁移测试                          | ✓ VERIFIED | 包含 temp-root 隔离、注入式 roots、legacy migration 幂等断言，且 `test_app_paths_migrate_legacy_roots` 可执行通过。 |
| `src-tauri/src/capture_runtime_tests.rs`                                  | CAPT-01/CAPT-02 自动化验证                      | ✓ VERIFIED | 包含 stop cancel、single worker/suppression、restart single owner、brownfield dedupe migration 测试。               |
| `src-tauri/src/capture_policy_tests.rs`                                   | CAPT-03 自动化验证                              | ✓ VERIFIED | 包含 marker matrix、CurrentOnly 不检测/不发送/不入库，以及非 macOS no-op marker 测试。                              |
| `src-tauri/src/app_paths.rs`                                              | 存储权威与迁移逻辑                              | ⚠️ PARTIAL | 核心路径与迁移逻辑已实体落地，但 `log_dir()` 没有被生产日志命令消费，且 monitor 的图片 metadata 回读仍绕过它。      |
| `src-tauri/src/capture/runtime.rs`                                        | CaptureRuntime 生命周期与 suppression registry  | ✓ VERIFIED | 有 cancel token、monitor/save 双任务、suppression registry、observed hash 和记忆、SQLite UPSERT 保存环。            |
| `src-tauri/src/capture/policy.rs`                                         | marker-first capture policy                     | ✓ VERIFIED | 纯函数策略矩阵完整，规则与 Phase 01 计划一致。                                                                      |
| `src-tauri/src/capture/macos_markers.rs`                                  | macOS pasteboard marker adapter                 | ✓ VERIFIED | macOS 读取 marker，非 macOS 回退 no-op adapter。                                                                    |
| `src/stores/clipboardStore.ts`                                            | 共享 backend copy helper 与 store-level routing | ✓ VERIFIED | 导出 `copyToClipboard(content)` 并在 store action 中统一复用。                                                      |
| `src/stores/clipboardStore.test.ts`                                       | store copy-routing contract tests               | ✓ VERIFIED | 明确断言调用 `invoke('copy_to_clipboard', { content })`，且不会调用 `writeText`。                                   |
| `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`        | renderer copy-routing contract tests            | ✓ VERIFIED | 明确断言 JSON 复制按钮走 backend contract。                                                                         |
| `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` | renderer copy-routing contract tests            | ✓ VERIFIED | 明确断言文本/命令复制按钮走 backend contract。                                                                      |

### Key Link Verification

| From                                     | To                                                                   | Via                                    | Status    | Details                                                                                                                                                                    |
| ---------------------------------------- | -------------------------------------------------------------------- | -------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/lib.rs`                   | `src-tauri/src/test_support.rs`                                      | `cfg(test) module registration`        | WIRED     | `lib.rs` 注册了 `mod test_support;`。                                                                                                                                      |
| `src-tauri/src/lib.rs`                   | `src-tauri/src/app_paths_tests.rs`                                   | `cfg(test) module registration`        | WIRED     | `lib.rs` 注册了 `mod app_paths_tests;`。                                                                                                                                   |
| `src-tauri/src/lib.rs`                   | `src-tauri/src/capture_runtime_tests.rs`                             | `cfg(test) module registration`        | WIRED     | `lib.rs` 注册了 `mod capture_runtime_tests;`。                                                                                                                             |
| `src-tauri/src/lib.rs`                   | `src-tauri/src/capture_policy_tests.rs`                              | `cfg(test) module registration`        | WIRED     | `lib.rs` 注册了 `mod capture_policy_tests;`。                                                                                                                              |
| `src-tauri/src/app_paths.rs`             | `src-tauri/src/database/mod.rs`                                      | `history_db_path()`                    | WIRED     | `Database::new_in()` 读取 `paths.history_db_path()`。                                                                                                                      |
| `src-tauri/src/app_paths.rs`             | `src-tauri/src/config/mod.rs`                                        | `config_file_path()`                   | WIRED     | `ConfigManager::new_in()` 读取 `paths.config_file_path()`。                                                                                                                |
| `src-tauri/src/app_paths.rs`             | `src-tauri/src/clipboard/processor.rs`                               | `image_assets_dir()`                   | WIRED     | `ContentProcessor::new_in()` 读取 `paths.image_assets_dir()`。                                                                                                             |
| `src-tauri/src/app_paths.rs`             | `src-tauri/src/commands.rs`                                          | `resolve_relative_asset_path()`        | WIRED     | `open_file_with_system()`、`get_image_url()`、图片打开相关命令都会解析 `imgs/...`。                                                                                        |
| `src-tauri/src/state.rs`                 | `src-tauri/src/capture/runtime.rs`                                   | `start_monitoring()/stop_monitoring()` | WIRED     | 状态层在 `start_monitoring()` 中创建 runtime，在 `stop_monitoring()` 中 `take()` 并 await 停止。                                                                           |
| `src-tauri/src/commands.rs`              | `src-tauri/src/capture/runtime.rs`                                   | `register_suppression_for_text()`      | WIRED     | `copy_to_clipboard` 命令先注册 suppression，再写系统剪贴板。                                                                                                               |
| `src-tauri/src/capture/runtime.rs`       | `src-tauri/src/database/mod.rs`                                      | `UPSERT save loop`                     | WIRED     | save loop 调用 `persist_entry()`，对 `content_hash` 做 UPSERT。                                                                                                            |
| `src/stores/clipboardStore.ts`           | `src/components/ClipboardMenuHandler.tsx`                            | `copyToClipboard()`                    | WIRED     | 菜单 copy/cut 事件都调用共享 helper。                                                                                                                                      |
| `src/stores/clipboardStore.ts`           | `src/components/DetailView/ContentRenderers/JsonRenderer.tsx`        | `shared backend copy helper`           | WIRED     | JSON 复制按钮调用共享 helper。                                                                                                                                             |
| `src/stores/clipboardStore.ts`           | `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.tsx` | `shared backend copy helper`           | WIRED     | 文本/命令复制按钮调用共享 helper。                                                                                                                                         |
| `src-tauri/src/capture/macos_markers.rs` | `src-tauri/src/capture/policy.rs`                                    | `PasteboardMarkers`                    | WIRED     | marker adapter 返回 `PasteboardMarkers`，策略函数直接消费。                                                                                                                |
| `src-tauri/src/capture/policy.rs`        | `src-tauri/src/clipboard/monitor.rs`                                 | `decide_capture`                       | WIRED     | monitor 文本与图片链路都在检测前调用 `decide_capture(...)`。                                                                                                               |
| `src-tauri/src/clipboard/monitor.rs`     | `src-tauri/src/capture/runtime.rs`                                   | `only Persist reaches save loop`       | WIRED     | monitor 只有 `Persist` 才 `tx.send(entry)`，save loop 再入库。                                                                                                             |
| `src-tauri/src/app_paths.rs`             | `src-tauri/src/clipboard/monitor.rs`                                 | `image metadata path resolution`       | NOT_WIRED | `ClipboardMonitor::get_saved_file_size()` 仍用 `dirs::config_dir()/clipboard-app/...` 拼路径，没有复用 `AppPaths::image_assets_dir()` 或 `resolve_relative_asset_path()`。 |
| `src-tauri/src/app_paths.rs`             | `src-tauri/src/commands.rs`                                          | `log_dir()`                            | NOT_WIRED | `get_log_content()` / `clear_logs()` 仍手写 `~/Library/Logs/com.dance.app/dance.log`，没有消费 `AppPaths::log_dir()`。                                                     |

### Data-Flow Trace (Level 4)

| Artifact                             | Data Variable                  | Source                                                                                                                             | Produces Real Data | Status      |
| ------------------------------------ | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------ | ----------- |
| `src-tauri/src/capture/runtime.rs`   | `entry` / `stored_entry`       | `ClipboardMonitor` 通过 `broadcast::Sender<ClipboardEntry>` 推送条目，save loop 调用 `persist_entry()` 写 SQLite                   | Yes                | ✓ FLOWING   |
| `src/stores/clipboardStore.ts`       | `content`                      | 组件与 store action 传入文本，helper 调用 `invoke('copy_to_clipboard', { content })`，后端命令再注册 suppression 并写系统剪贴板    | Yes                | ✓ FLOWING   |
| `src-tauri/src/clipboard/monitor.rs` | `CaptureDisposition` / `entry` | `read_pasteboard_markers()` + 配置排除规则 + 剪贴板内容 -> `decide_capture()` -> 仅 `Persist` 进入 `tx.send(entry)`                | Yes                | ✓ FLOWING   |
| `src-tauri/src/app_paths.rs`         | 路径解析结果                   | `AppHandle.path()` / temp roots 注入已驱动 DB、配置、图片目录与图片读取命令，但 monitor 图片 metadata 回读与日志命令仍绕过这套来源 | Partial            | ⚠️ BYPASSED |

### Behavioral Spot-Checks

| Behavior                               | Command                                                                                                                                                                                   | Result                                                   | Status |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | ------ |
| 停止监听后不再持久化新条目             | `cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture`                                                                                                                       | 1 test passed, finished in 0.31s                         | ✓ PASS |
| 重启监听仍只有一个 runtime owner       | `cargo test test_capture_runtime_restart_is_single_owner -- --nocapture`                                                                                                                  | 1 test passed, finished in 0.00s                         | ✓ PASS |
| eligible 条目去重且 suppression 生效   | `cargo test test_capture_runtime_single_worker_and_suppression -- --nocapture`                                                                                                            | 1 test passed, finished in 0.29s                         | ✓ PASS |
| CurrentOnly 事件不检测、不发送、不入库 | `cargo test test_capture_policy_current_only_is_non_persistent_in_v1 -- --nocapture`                                                                                                      | 1 test passed, finished in 0.15s                         | ✓ PASS |
| legacy root 迁移幂等可执行             | `cargo test test_app_paths_migrate_legacy_roots -- --nocapture`                                                                                                                           | 1 test passed, finished in 0.01s                         | ✓ PASS |
| 前端复制入口统一走 backend contract    | `pnpm test -- src/stores/clipboardStore.test.ts src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` | Vitest reported 7 files, 36 tests passed, duration 1.78s | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan               | Description                                                                                                                           | Status      | Evidence                                                                                                                           |
| ----------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `CAPT-01`   | `01-01`, `01-03`          | User can start/stop monitoring without hidden listeners continuing after stop                                                         | ✓ SATISFIED | `AppState` 统一持有 runtime，`CaptureRuntime::stop()` 会 cancel 并 await 两个后台任务，Rust 行为测试通过。                         |
| `CAPT-02`   | `01-01`, `01-03`, `01-04` | User sees each eligible clipboard change recorded once without duplicate entries from repeated listeners or self-generated copy flows | ✓ SATISFIED | SQLite `content_hash` UPSERT + unique index、后端 suppression contract、前端统一 copy route 与合同测试都已接线。                   |
| `CAPT-03`   | `01-01`, `01-05`          | User can keep ignored, transient, concealed, or otherwise non-persistent clipboard events out of saved history                        | ✓ SATISFIED | marker-first `decide_capture()`、macOS marker adapter、monitor 前置 gating、CurrentOnly 非持久化测试均在位。                       |
| `CAPT-04`   | `01-01`, `01-02`          | User can rely on one consistent local storage lifecycle for history, cache, image assets, and related metadata                        | ✗ BLOCKED   | `AppPaths` 与迁移测试已经覆盖主路径，但 `clipboard/monitor.rs` 与 `commands.rs` 仍保留硬编码路径绕过，导致统一路径权威未完全兑现。 |

**Requirement accounting:** `CAPT-01`、`CAPT-02`、`CAPT-03`、`CAPT-04` 全部出现在至少一个 PLAN frontmatter 中；`REQUIREMENTS.md` 对 Phase 1 的映射也只有这 4 个 ID，未发现 orphaned requirement。

### Anti-Patterns Found

| File                                        | Line | Pattern                                                               | Severity   | Impact                                                                         |
| ------------------------------------------- | ---- | --------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------ |
| `src-tauri/src/clipboard/monitor.rs`        | 37   | `imgs/...` 仍通过 `dirs::config_dir()/clipboard-app/...` 解析绝对路径 | 🛑 Blocker | 新 `AppPaths` 布局下，图片 metadata 回读没有复用权威路径层，存在路径失配风险。 |
| `src-tauri/src/commands.rs`                 | 1651 | 日志读取/清理命令硬编码 `~/Library/Logs/com.dance.app/dance.log`      | ⚠️ Warning | 生产日志链路没有接入 `AppPaths::log_dir()`，存储生命周期仍有分叉。             |
| `src-tauri/src/commands.rs`                 | 1703 | placeholder 级 `set_log_level()`                                      | ℹ️ Info    | 非 Phase 01 主线，但命令层仍存在未完成实现。                                   |
| `src-tauri/src/utils/app_icon_extractor.rs` | 206  | placeholder Windows icon fallback                                     | ℹ️ Info    | 预先存在的跨平台占位逻辑，不阻塞 CAPT-01..04。                                 |

### Human Verification Required

### 1. Real Clipboard Stop/Restart Smoke

**Test:** 在 macOS 真机上启动监听，复制一条普通文本；停止监听后再复制第二条；重启应用后检查历史列表。  
**Expected:** 第一条仍在历史中，第二条不会在 stop 后被偷偷入库。  
**Why human:** 依赖真实系统剪贴板事件、应用焦点和重启生命周期，单元测试不能完全替代。

### 2. Marker-First Capture Smoke

**Test:** 在 macOS 上分别触发普通文本复制、可识别的 auto-generated / concealed / remote clipboard 场景，观察历史列表。  
**Expected:** 普通文本进入历史；被 marker 标记为非持久化的事件不会进入历史。  
**Why human:** NSPasteboard marker 来自系统运行时，测试代码只能验证策略与接线，不能完全重放真实 OS 行为。

### Gaps Summary

Phase 01 的核心可靠性链路已经基本成形：受控 `CaptureRuntime`、后端 owned suppression、marker-first policy、legacy migration 与前端 copy routing 都有实体实现，而且抽样行为测试全部通过。`CAPT-01`、`CAPT-02`、`CAPT-03` 可以判定为满足。

阻塞项集中在 `CAPT-04` 的最后一段闭环。`AppPaths` 已经成为数据库、配置、图片目录和多数图片读取命令的权威来源，但并没有真正覆盖全部存储相关读写路径。`ClipboardMonitor::get_saved_file_size()` 仍按 legacy `clipboard-app/imgs` 根目录回读图片大小，`get_log_content()` / `clear_logs()` 仍写死日志路径。这两个残留绕路意味着“本地数据只有一个 coherent storage lifecycle”还不成立，因此 phase goal 不能判定为完全达成。

---

_Verified: 2026-03-27T15:08:57Z_  
_Verifier: Claude (gsd-verifier)_
