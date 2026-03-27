---
phase: 1
slug: capture-reliability-storage-cohesion
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 1 — Validation Strategy

> Per-phase executable validation contract. Execute the task-level command after each task, then execute the wave gate before moving to the next wave.

---

## Test Infrastructure

| Property               | Value                                                                                                                                                               |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Framework**          | Rust `cargo test` for scaffolding, app paths, runtime lifecycle, dedupe migration, and capture policy; Vitest `4.1.2` only when `clipboardStore.ts` routing changes |
| **Config file**        | `vitest.config.ts`; Rust has no separate config file                                                                                                                |
| **Quick run command**  | `cd src-tauri && cargo test --no-run`                                                                                                                               |
| **Full suite command** | `pnpm test` and `cd src-tauri && cargo test`                                                                                                                        |
| **Estimated runtime**  | ~20-45 seconds for task-level commands; ~2-3 minutes for full phase gate                                                                                            |

---

## Sampling Rate

- **After every task:** run that task’s exact `<verify><automated>` command from the matrix below.
- **After every wave:** run the wave gate command for that wave before starting the next wave.
- **Before `$gsd-verify-work`:** full frontend and Rust suites must be green.
- **Max feedback latency:** 60 seconds for task-level commands.

---

## Task Verification Matrix

| Task ID    | Plan    | Wave | Requirement(s)                       | Purpose                                                                 | Automated Command                                                                                                                                                                            | Status     |
| ---------- | ------- | ---- | ------------------------------------ | ----------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| `01-01-01` | `01-01` | 1    | `CAPT-01, CAPT-02, CAPT-03, CAPT-04` | 建立 `test_support.rs` 与测试模块注册                                   | `cd src-tauri && cargo test --no-run`                                                                                                                                                        | ⬜ planned |
| `01-01-02` | `01-01` | 1    | `CAPT-01, CAPT-02, CAPT-03, CAPT-04` | 生成 Phase 1 三类测试脚手架与固定测试名                                 | `cd src-tauri && cargo test test_app_paths_temp_roots_are_isolated -- --nocapture`                                                                                                           | ⬜ planned |
| `01-02-01` | `01-02` | 2    | `CAPT-04`                            | 建立 `AppPaths` 权威层、注入式构造与核心模块 temp-root 实例化           | `cd src-tauri && cargo test test_app_paths_injected_roots_drive_core_modules -- --nocapture`                                                                                                 | ⬜ planned |
| `01-02-02` | `01-02` | 2    | `CAPT-04`                            | 完成 legacy migration 与路径调用替换                                    | `cd src-tauri && cargo test test_app_paths_migrate_legacy_roots -- --nocapture`                                                                                                              | ⬜ planned |
| `01-03-01` | `01-03` | 3    | `CAPT-01, CAPT-02`                   | 引入 `CaptureRuntime` 并接管 start/stop 生命周期                        | `cd src-tauri && cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture && cargo test test_capture_runtime_restart_is_single_owner -- --nocapture`                                | ⬜ planned |
| `01-03-02` | `01-03` | 3    | `CAPT-02`                            | 做 brownfield dedupe migration、唯一索引与 backend suppression contract | `cd src-tauri && cargo test test_capture_runtime_single_worker_and_suppression -- --nocapture && cargo test test_capture_runtime_dedupe_migration_merges_existing_duplicates -- --nocapture` | ⬜ planned |
| `01-04-01` | `01-04` | 4    | `CAPT-02`                            | 收敛共享 backend copy helper 与 store/menu 合同                         | `pnpm test -- src/stores/clipboardStore.test.ts`                                                                                                                                             | ⬜ planned |
| `01-04-02` | `01-04` | 4    | `CAPT-02`                            | sweep renderer 与 log viewer 的剩余 copy 入口并补 contract tests        | `pnpm test -- src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx`                                      | ⬜ planned |
| `01-05-01` | `01-05` | 5    | `CAPT-03`                            | 定义 marker-first policy 与策略矩阵                                     | `cd src-tauri && cargo test test_capture_policy_marker_matrix -- --nocapture`                                                                                                                | ⬜ planned |
| `01-05-02` | `01-05` | 5    | `CAPT-03`                            | 接入 macOS marker adapter 并把 policy 放到 monitor 前置阶段             | `cd src-tauri && cargo test test_capture_policy_current_only_is_non_persistent_in_v1 -- --nocapture`                                                                                         | ⬜ planned |

_Status: ⬜ planned · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave Gates

| Wave | Plans   | Gate Command                                                                                                                                                                                                                                                                                                                                | Purpose                                                               |
| ---- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| 1    | `01-01` | `cd src-tauri && cargo test --no-run && cargo test test_app_paths_temp_roots_are_isolated -- --nocapture`                                                                                                                                                                                                                                   | 确认 Phase 1 scaffolding 与 temp-root 隔离已可执行                    |
| 2    | `01-02` | `cd src-tauri && cargo test test_app_paths_injected_roots_drive_core_modules -- --nocapture && cargo test test_app_paths_migrate_legacy_roots -- --nocapture`                                                                                                                                                                               | 确认 `AppPaths` 注入和 migration 合同已经稳定                         |
| 3    | `01-03` | `cd src-tauri && cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture && cargo test test_capture_runtime_restart_is_single_owner -- --nocapture && cargo test test_capture_runtime_single_worker_and_suppression -- --nocapture && cargo test test_capture_runtime_dedupe_migration_merges_existing_duplicates -- --nocapture` | 确认 runtime 生命周期、去重迁移与后端 suppression contract 已全部通过 |
| 4    | `01-04` | `pnpm test -- src/stores/clipboardStore.test.ts src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx`                                                                                                                                                   | 确认所有当前前端 copy-routing 合同全部通过                            |
| 5    | `01-05` | `cd src-tauri && cargo test test_capture_policy_marker_matrix -- --nocapture && cargo test test_capture_policy_current_only_is_non_persistent_in_v1 -- --nocapture`                                                                                                                                                                         | 确认 marker-first policy 已接入 monitor                               |

## Manual-Only Verifications

| Behavior                                                                                                                                            | Requirement | Why Manual                                                                                                       | Test Instructions                                                                                                                                                                                                                      |
| --------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS pasteboard markers for transient, concealed, auto-generated, and remote clipboard entries are classified into skip/persist behavior correctly | CAPT-03     | marker metadata depends on real desktop integration and cannot be fully simulated with current automated harness | 1. Run dev app on macOS. 2. Trigger representative clipboard writes from normal apps, password-like/concealed flows if available, and self-copy paths. 3. Confirm skipped events do not create rows while normal events still persist. |
| Existing local installs using mixed `dance` and `clipboard-app` roots keep history, images, and cache access after migration                        | CAPT-04     | requires real filesystem state representative of past installs                                                   | 1. Seed both legacy roots with representative config, DB, and image files. 2. Launch upgraded app. 3. Confirm history loads, image previews resolve, and no duplicate or missing roots remain.                                         |

---

## Validation Sign-Off

- [ ] All 10 planned tasks have a task-level automated command
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave gate commands match the actual plan order `01-01 -> 01-02 -> 01-03 -> 01-04 -> 01-05`
- [ ] `01-03` dedupe migration is covered before unique index creation
- [ ] `CurrentOnly` and runtime suppression share the same `content_hash` key contract
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
