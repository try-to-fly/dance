---
phase: 1
reviewers: [claude]
reviewed_at: 2026-03-27T12:37:49Z
plans_reviewed:
  - 01-01-PLAN.md
  - 01-02-PLAN.md
  - 01-03-PLAN.md
  - 01-04-PLAN.md
  - 01-05-PLAN.md
---

# Cross-AI Plan Review — Phase 1

## the agent Review

# Phase 1 Plan Review: Capture Reliability & Storage Cohesion

## Overall Phase Assessment

This is a well-structured trust-repair phase for a brownfield desktop app. The 5-plan, 5-wave sequential design correctly prioritizes test scaffolding -> storage authority -> runtime lifecycle -> frontend routing -> capture policy. The research document is thorough and the plans are grounded in real codebase evidence. The phase avoids scope creep into detection/preview work and stays focused on the four CAPT requirements.

---

## Plan 01-01: Test Scaffolding & Module Registration

### Summary

A clean "wave 0" plan that establishes test infrastructure before any production code changes. It creates `test_support.rs` with temp-root helpers, registers four test modules in `lib.rs`, and seeds named test stubs with `#[ignore]` annotations for later plans to fill in. One real test (`test_app_paths_temp_roots_are_isolated`) runs immediately.

### Strengths

- **Test-first discipline**: No production code changes, just verification scaffolding - exactly right for wave 1
- **Named test contracts**: Fixed test names with `#[ignore = "implemented in 01-0X"]` create a clear handoff protocol between plans
- **Minimal risk**: Only adds `#[cfg(test)]` modules, zero chance of breaking production builds
- **Good acceptance criteria**: Uses `rg` patterns that are machine-verifiable

### Concerns

- **LOW**: `TestAppRoots` exposes raw `PathBuf` fields but the plan doesn't mention ensuring directories are actually created on construction. If `create_temp_app_roots()` only builds paths without `create_dir_all`, downstream tests in later plans may fail with confusing "directory not found" errors
- **LOW**: No mention of cleanup semantics - `TempDir` auto-deletes on drop, but if tests panic before `TempDir` is dropped, orphan directories could accumulate. This is standard `tempfile` behavior and not a real concern, but worth noting

### Suggestions

- Ensure `create_temp_app_roots()` calls `create_dir_all` for all four subdirectories, not just returns path objects
- Consider adding a `seed_legacy_roots()` helper in `test_support.rs` now (even if empty), since plans 01-02 and 01-03 will both need to seed `dance/` and `clipboard-app/` directories for migration tests

### Risk Assessment: **LOW**

Pure test infrastructure with no production impact.

---

## Plan 01-02: AppPaths Authority & Legacy Migration

### Summary

Introduces `AppPaths` as the single storage authority, refactors `Database`, `ConfigManager`, `ContentProcessor`, and `AppIconExtractor` to accept injected paths, and implements explicit migration from the `dance` and `clipboard-app` legacy roots. Two TDD tasks with clear behavior specifications.

### Strengths

- **Correct architectural seam**: `AppPaths::from_app()` for production, `AppPaths::from_roots()` for tests - clean injection boundary
- **Idempotent migration**: Migration marker file (`capt04-storage-roots.json`) prevents re-migration
- **Thorough sweep**: Acceptance criteria explicitly check that `dirs::config_dir()` calls are eliminated from all target files
- **Well-defined merge rules**: Migration specifies which legacy root wins for each file type

### Concerns

- **HIGH**: The plan says `from_app(app: &AppHandle)` uses Tauri's `PathResolver`, but `AppState::new()` is currently called inside `tauri::async_runtime::block_on` during `setup()` where `AppHandle` is available. However, the existing `Database::new()` and `ConfigManager::new()` are called _before_ Tauri setup completes in the current code. The plan needs to be explicit about the initialization order: `AppPaths` must be constructed from `AppHandle` during `setup()`, and then passed to `AppState::new(paths)`. If the executor doesn't get this ordering right, the app will panic at startup
- **MEDIUM**: The migration copies `dance/clipboard.db` -> `data/clipboard.db`, but the current `Database::new()` uses `dirs::config_dir().join("dance").join("clipboard.db")` while `AppState::get_db_path()` uses `dirs::config_dir().join("clipboard-app").join("clipboard.db")`. There are potentially **two** database files already. The plan's migration rules need to specify which DB wins when both exist, or whether they should be merged. This is mentioned in passing ("new root takes priority") but the DB merge case deserves explicit attention since it could mean silent data loss
- **MEDIUM**: `resolve_relative_asset_path(relative: &str)` is listed but its semantics aren't fully specified. Existing code has `if file_path.starts_with("imgs/")` scattered across `commands.rs`, `state.rs`, and `monitor.rs`. The plan should clarify whether this method handles the `imgs/` prefix stripping or expects callers to change
- **LOW**: `AppIconExtractor::new()` currently hard-codes `clipboard-app/icons`. After migration, icons should go to `app_cache_dir/icons`. But the plan doesn't mention whether existing cached icons need migration or can just be re-extracted on demand

### Suggestions

- Add an explicit note about initialization order in the execution context: `AppPaths` must be constructed from `AppHandle` in `setup()`, then passed into `AppState::new()`
- Add a migration rule for when both `dance/clipboard.db` AND `clipboard-app/clipboard.db` exist - recommend keeping the one with more rows or more recent `created_at`
- Consider making `resolve_relative_asset_path` handle `imgs/` prefix transparently to reduce caller-side changes
- Icons in `clipboard-app/icons` can be treated as cache (re-extractable), so skip migration for those and document the decision

### Risk Assessment: **MEDIUM**

The dual-database edge case and initialization ordering are real risks that could cause data loss or startup failures if not handled carefully during execution.

---

## Plan 01-03: CaptureRuntime & Dedupe Migration

### Summary

The core reliability plan. Introduces `CaptureRuntime` with `CancellationToken` + `JoinHandle` for proper lifecycle control, adds a brownfield dedupe migration for existing duplicate `content_hash` rows, creates a `UNIQUE` index, switches persistence to `UPSERT`, and establishes a backend suppression registry. Two tasks with explicit execution boundary.

### Strengths

- **Correct ownership model**: `CaptureRuntime` encapsulates cancel + join, fixing the core "stopped but still running" bug
- **Brownfield-aware dedupe**: Migrating existing duplicates before creating the unique index is exactly right - skipping this step would cause `CREATE UNIQUE INDEX` to fail on real databases
- **Unified key contract**: SHA256 hex `content_hash` is used consistently across `last_observed_hash`, `SuppressionEntry`, and UPSERT - this prevents the "different hash format in different places" bug
- **Smart UPSERT with COALESCE**: The `ON CONFLICT DO UPDATE` uses `COALESCE(excluded.*, clipboard_entries.*)` to avoid overwriting richer metadata with nulls - good defensive design
- **Explicit execution boundary**: Task 1 must pass before Task 2 begins

### Concerns

- **HIGH**: The dedupe migration (`merge_existing_content_hash_duplicates`) is described at a high level but the actual SQL merge logic is complex. The plan says "保留 `created_at` 最大的那一行" and "合并后的 `copy_count = SUM(copy_count)`". This requires a multi-step transaction: (1) identify groups with duplicate `content_hash`, (2) for each group, compute the survivor row and aggregate values, (3) update the survivor, (4) delete losers. If this transaction fails partway (e.g., disk full, power loss), the database could be left in an inconsistent state with some groups partially merged. The plan should specify that the migration is wrapped in a single transaction with rollback on failure
- **MEDIUM**: The plan removes `start_database_save_task()` and moves the save loop into `CaptureRuntime`. But the existing save task in `state.rs` does more than just persist - it also emits `clipboard-update` events to the frontend via `app_handle.emit()`. The plan doesn't explicitly mention preserving this frontend notification. If the executor forgets it, the UI will stop updating in real-time
- **MEDIUM**: `SuppressionEntry` uses `expires_at_ms: i64` with a 1500ms TTL. But the monitor polls every 500ms. If the system is under load and a poll is delayed, a 1500ms TTL might expire before the suppressed content is observed. Consider making the TTL configurable or at least 3x the poll interval (1500ms is exactly 3x, which is tight)
- **LOW**: The plan creates `src-tauri/src/capture/mod.rs` and `runtime.rs` but the existing `src-tauri/src/clipboard/monitor.rs` stays in its current location. This creates a split: `capture/runtime.rs` for lifecycle and `clipboard/monitor.rs` for polling. The research recommends eventually moving monitor into `capture/`, but this plan doesn't do it. This is fine for now but should be noted as technical debt

### Suggestions

- Explicitly state that `merge_existing_content_hash_duplicates` must run inside a single SQLite transaction with automatic rollback on any error
- Add a note in Task 1 that the save loop must preserve `app_handle.emit("clipboard-update", &updated_entry)` behavior from the current `start_database_save_task()`
- Consider bumping the default suppression TTL to 2000ms to give more headroom against poll jitter
- Add a brief note about the `clipboard/monitor.rs` -> `capture/monitor.rs` path not being in scope for this plan

### Risk Assessment: **MEDIUM**

The dedupe migration is the riskiest part of the entire phase. If it corrupts existing databases, users lose clipboard history. The runtime refactor is well-designed but the frontend notification preservation needs explicit attention.

---

## Plan 01-04: Frontend Copy-Routing Unification

### Summary

Migrates all frontend copy paths from direct `writeText` / `navigator.clipboard.writeText` to the backend `copy_to_clipboard` command, completing the CAPT-02 UI coverage. Two tasks: first store/menu level, then renderer/log viewer sweep.

### Strengths

- **Systematic sweep**: The plan identifies all current copy entry points (store, menu handler, JSON renderer, text renderer, log viewer) and migrates each one
- **Contract tests**: Each migrated entry point gets an explicit test asserting the backend route is used
- **Acceptance criteria are precise**: `rg` patterns check for absence of `writeText` and presence of `invoke('copy_to_clipboard')`

### Concerns

- **MEDIUM**: `ClipboardMenuHandler.tsx` currently handles `menu_copy` by reading `window.getSelection()` and writing it with `writeText`. After migration, the handler would call `invoke('copy_to_clipboard', { content: selectedText })`. But `invoke` is async and the menu event handler doesn't currently await properly in all paths. The plan should note that the menu copy flow must await the invoke call to ensure the suppression is registered before the clipboard write happens
- **MEDIUM**: `LogViewer.tsx` currently uses `navigator.clipboard.writeText(logText)` for a "copy all logs" feature. This is a user-initiated copy of app-generated content (log text), not a history recall. Should this really go through the suppression pipeline? Copying logs isn't a "clipboard history recall" - it's new content the user explicitly wants to copy. The plan should consider whether log copying needs suppression at all, or if it should be an exception
- **LOW**: The plan removes `writeText` from `@tauri-apps/plugin-clipboard-manager` imports, but the `ClipboardMenuHandler` also imports `readText` from the same module. The `readText` import should be preserved for the paste handler
- **LOW**: The acceptance criteria check `rg -n "writeText|navigator\\.clipboard\\.writeText"` but there might be other clipboard write paths (e.g., `document.execCommand('copy')` in edge cases). A broader sweep would be safer

### Suggestions

- Clarify whether `LogViewer.tsx` log copying should go through suppression or be treated as a non-history copy (my recommendation: keep it as backend copy for consistency, but with a note that it's intentional new content, not recall)
- Ensure `ClipboardMenuHandler` properly awaits the `invoke` call in the menu_copy handler
- Add `document.execCommand('copy')` to the sweep check pattern, though it's less likely to be present
- Consider adding a `// CAPT-02: routed through backend suppression` comment at each migrated call site for future maintainability

### Risk Assessment: **LOW**

Frontend-only changes with good test coverage. The main risk is subtle behavioral differences (async timing, log copy semantics) rather than correctness failures.

---

## Plan 01-05: Marker-First Capture Policy & macOS Adapter

### Summary

Implements the marker-first capture policy with a pure policy function, a macOS-specific pasteboard marker adapter using `cocoa`/`objc`, and a no-op adapter for other platforms. Wires the policy into the monitor before content detection.

### Strengths

- **Pure function design**: `decide_capture(markers, self_generated, excluded_app, text_size_valid)` is testable without any system dependencies
- **Platform abstraction**: macOS gets real marker reading, other platforms get a safe no-op - this is exactly the right abstraction boundary
- **Correct ordering enforcement**: "read markers -> decide policy -> only then detect content" is explicitly stated and enforced in acceptance criteria
- **Unified key contract**: Plan explicitly references Plan 03's `remember_observed_hash()` for `CurrentOnly` events, maintaining the single dedupe key contract

### Concerns

- **HIGH**: Reading macOS pasteboard markers via `cocoa`/`objc` is inherently unsafe Rust code. The plan doesn't mention error handling for the case where `NSPasteboard.generalPasteboard` returns nil, or where a marker type string isn't present (which is different from being false). The existing `app_icon_extractor.rs` wraps its unsafe code in `std::panic::catch_unwind`, and the marker adapter should do the same
- **MEDIUM**: The plan says `CurrentOnly` in Phase 1 means "不入库" (don't persist) and only updates `remember_observed_hash()`. But `CurrentOnly` is semantically different from `Skip` - it means "show as current clipboard but don't save to history." If Phase 2 or later needs a "current clipboard" display, collapsing `CurrentOnly` to effectively `Skip` now might make the later change harder. The plan should add a comment in the code explaining this deliberate simplification
- **MEDIUM**: The marker adapter reads specific NSPasteboard type strings. These strings are not part of Apple's stable public API - they come from `nspasteboard.org` community documentation. If Apple changes these strings in a future macOS version, the markers silently stop working. The plan should mention version-checking or graceful degradation
- **LOW**: The `text_size_valid: bool` parameter in `decide_capture` mixes a content property with marker metadata. The policy function is described as "marker-first" but text size is a content property. Consider separating this into a pre-filter or making the naming more explicit (e.g., `content_exceeds_limit`)

### Suggestions

- Wrap all `unsafe` NSPasteboard access in `std::panic::catch_unwind` and return `PasteboardMarkers::default()` on any failure
- Add a code comment at the `CurrentOnly -> Skip` collapse point explaining it's a Phase 1 simplification
- Consider logging when markers are successfully read vs. when the adapter falls back to defaults, to aid debugging on future macOS versions
- Rename `text_size_valid` to `within_size_limit` or split it out of the marker-focused function signature

### Risk Assessment: **MEDIUM**

The unsafe macOS interop is the main risk. If not properly guarded, a nil pointer or unexpected pasteboard state could crash the app. The rest of the plan is well-designed.

---

## Cross-Plan Assessment

### Dependency Chain

```text
01-01 (scaffolding) --> 01-02 (AppPaths) --> 01-03 (Runtime) --> 01-04 (Frontend)
        |                                            |
        +--------------------------------------------+--> 01-05 (Policy)
```

The dependency ordering is correct. 01-05 depends on 01-01 and 01-03 (needs test scaffolding and runtime's `remember_observed_hash`), and can run in parallel with 01-04 since they touch different layers.

### Coverage Analysis

| Requirement | Primary Plan        | Supporting Plans | Gap?                |
| ----------- | ------------------- | ---------------- | ------------------- |
| CAPT-01     | 01-03 Task 1        | -                | No                  |
| CAPT-02     | 01-03 Task 2, 01-04 | -                | No                  |
| CAPT-03     | 01-05               | -                | No                  |
| CAPT-04     | 01-02               | -                | See dual-DB concern |

### Top 3 Risks Across All Plans

1. **Brownfield dedupe migration (01-03)**: If the SQL merge logic has a bug, existing users lose history data. This is the highest-impact failure mode in the entire phase. **Mitigation**: The migration should create a backup of the database before starting, and the test should seed realistic duplicate patterns.
2. **Dual database discovery (01-02)**: Both `dance/clipboard.db` and `clipboard-app/clipboard.db` may exist with different data. The plan's "new root takes priority" rule could discard valid history. **Mitigation**: Add explicit handling for the two-DB case, preferring the one with more recent data.
3. **macOS unsafe interop (01-05)**: Reading pasteboard markers via raw Objective-C message sends can crash on unexpected nil returns. **Mitigation**: Wrap all marker reads in `catch_unwind` with default fallback.

### Overall Phase Risk: **MEDIUM**

The plans are well-structured and the research is thorough. The main risks are in data migration (01-02, 01-03) and platform interop (01-05), not in architectural design. With the suggested mitigations, this phase should execute successfully.

---

## Consensus Summary

本次 review 只有 1 个独立 reviewer，因此这里是单 reviewer 归纳，不是多模型交叉共识。

### Agreed Strengths

- Phase 1 的 5-plan / 5-wave 顺序合理，先测试脚手架，再路径权威、runtime 可靠性、前端 copy 路由、最后 marker policy。
- 文档整体能守住范围，没有把 detection / preview / search 工作混进 Phase 1。
- `content_hash` 作为统一 dedupe key 的合同已经比较清晰，`01-03` 到 `01-05` 的文本口径一致。
- `01-03` 和 `01-04` 拆开后，runtime/backend 与 frontend sweep 的边界明显比之前更稳。

### Agreed Concerns

- `01-02` 还需要更明确地处理 dual-database 场景，避免 `dance/clipboard.db` 和 `clipboard-app/clipboard.db` 并存时静默丢数据。
- `01-03` 的 brownfield dedupe migration 应明确强调单事务回滚语义，并提醒执行时保留现有前端 `clipboard-update` 事件发射。
- `01-05` 的 macOS pasteboard marker 读取属于 unsafe interop，建议在执行时显式加入 `catch_unwind` / 默认回退保护。
- `01-04` 的 UI copy 路径已经闭环，但执行时仍要留意 `ClipboardMenuHandler` 的 async timing 与 `LogViewer` copy 语义。

### Divergent Views

- 无。只有 1 个 reviewer，暂时不存在多 reviewer 间的分歧项。
