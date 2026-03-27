---
phase: 1
slug: capture-reliability-storage-cohesion
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property               | Value                                                                                                                                            |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Framework**          | Rust `cargo test` for capture lifecycle, policy, migration, and persistence logic; Vitest `4.1.2` for any frontend listener contract regressions |
| **Config file**        | `vitest.config.ts`; Rust has no separate config file                                                                                             |
| **Quick run command**  | `cd src-tauri && cargo test capture_ -- --nocapture`                                                                                             |
| **Full suite command** | `pnpm test` and `cd src-tauri && cargo test`                                                                                                     |
| **Estimated runtime**  | ~90 seconds after Wave 0 test scaffolding exists                                                                                                 |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test capture_ -- --nocapture`
- **After every plan wave:** Run `pnpm test` and `cd src-tauri && cargo test`
- **Before `$gsd-verify-work`:** Full frontend and Rust suites must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type                   | Automated Command                                                                              | File Exists | Status     |
| ------- | ---- | ---- | ----------- | --------------------------- | ---------------------------------------------------------------------------------------------- | ----------- | ---------- |
| 1-01-01 | 01   | 1    | CAPT-01     | Rust integration            | `cd src-tauri && cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture`            | ❌ W0       | ⬜ pending |
| 1-02-01 | 02   | 1    | CAPT-02     | Rust integration            | `cd src-tauri && cargo test test_capture_runtime_single_worker_and_suppression -- --nocapture` | ❌ W0       | ⬜ pending |
| 1-03-01 | 03   | 2    | CAPT-03     | Rust unit + manual smoke    | `cd src-tauri && cargo test test_capture_policy_marker_matrix -- --nocapture`                  | ❌ W0       | ⬜ pending |
| 1-04-01 | 04   | 2    | CAPT-04     | Rust filesystem integration | `cd src-tauri && cargo test test_app_paths_migrate_legacy_roots -- --nocapture`                | ❌ W0       | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave 0 Requirements

- [ ] `src-tauri/src/test_support.rs` — temp-root injection helpers for `AppPaths`, `Database`, `ConfigManager`, `ContentProcessor`, and `AppState`
- [ ] `src-tauri/src/capture_runtime_tests.rs` — lifecycle tests for start/stop/restart and duplicate worker prevention
- [ ] `src-tauri/src/capture_policy_tests.rs` — marker-aware skip matrix and self-write suppression tests
- [ ] `src-tauri/src/app_paths_tests.rs` — canonical path authority and legacy-root migration tests
- [ ] `src/stores/clipboardStore.test.ts` — extend only if frontend event listener setup/cleanup or copy-routing contracts change

_If none: Existing infrastructure covers all phase requirements._

---

## Manual-Only Verifications

| Behavior                                                                                                                                            | Requirement | Why Manual                                                                                                       | Test Instructions                                                                                                                                                                                                                      |
| --------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS pasteboard markers for transient, concealed, auto-generated, and remote clipboard entries are classified into skip/persist behavior correctly | CAPT-03     | marker metadata depends on real desktop integration and cannot be fully simulated with current automated harness | 1. Run dev app on macOS. 2. Trigger representative clipboard writes from normal apps, password-like/concealed flows if available, and self-copy paths. 3. Confirm skipped events do not create rows while normal events still persist. |
| Existing local installs using mixed `dance` and `clipboard-app` roots keep history, images, and cache access after migration                        | CAPT-04     | requires real filesystem state representative of past installs                                                   | 1. Seed both legacy roots with representative config, DB, and image files. 2. Launch upgraded app. 3. Confirm history loads, image previews resolve, and no duplicate or missing roots remain.                                         |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
