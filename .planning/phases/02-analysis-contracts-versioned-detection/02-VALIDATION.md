---
phase: 2
slug: analysis-contracts-versioned-detection
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-28
---

# Phase 2 — Validation Strategy

> Per-phase executable validation contract. Execute the task-level command after each task, then execute the wave gate before moving to the next wave.

---

## Test Infrastructure

| Property               | Value                                                                                                                                       |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| **Framework**          | Rust `cargo test` for contract, persistence, capture, and rebuild; Vitest for frontend analysis-contract consumption and preferences wiring |
| **Config file**        | `vitest.config.ts`; Rust has no separate config file                                                                                        |
| **Quick run command**  | `cd src-tauri && cargo test --no-run`                                                                                                       |
| **Full suite command** | `pnpm test && cd src-tauri && cargo test`                                                                                                   |
| **Estimated runtime**  | ~45-90 seconds for task-level commands; ~3-4 minutes for the full phase gate                                                                |

---

## Sampling Rate

- **After every task:** run that task's exact `<verify><automated>` command from the matrix below.
- **After every wave:** run the wave gate command for that wave before starting the next wave.
- **Before `$gsd-verify-work`:** full frontend and Rust suites must be green.
- **Max feedback latency:** 60 seconds for task-level commands.

---

## Task Verification Matrix

| Task ID    | Plan    | Wave | Requirement(s)              | Purpose                                                                 | Automated Command                                                                                                                                                           | Status     |
| ---------- | ------- | ---- | --------------------------- | ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| `02-01-01` | `02-01` | 1    | `DETE-01, DETE-02, DETE-04` | 建立 Rust `AnalysisSnapshot` contract 与 fallback shape                 | `cd src-tauri && cargo test --no-run`                                                                                                                                       | ⬜ planned |
| `02-01-02` | `02-01` | 1    | `DETE-01, DETE-02, DETE-04` | 锁定 stable subtype precedence、typed metadata 与 diagnostics           | `cd src-tauri && cargo test test_analysis_supported_content_maps_to_stable_subtypes -- --nocapture && cargo test test_analysis_metadata_contract_round_trip -- --nocapture` | ⬜ planned |
| `02-02-01` | `02-02` | 2    | `DETE-02, DETE-03, DETE-04` | 建立 `entry_analysis` companion table、repository 与 UPSERT             | `cd src-tauri && cargo test test_entry_analysis_repository_upsert_and_join_read_model -- --nocapture`                                                                       | ⬜ planned |
| `02-02-02` | `02-02` | 2    | `DETE-02, DETE-04`          | 历史读取优先 joined analysis rows，并保留 legacy fallback               | `cd src-tauri && cargo test test_history_reads_analysis_rows_with_legacy_fallback -- --nocapture`                                                                           | ⬜ planned |
| `02-03-01` | `02-03` | 3    | `DETE-01, DETE-02`          | 建立 authoritative analysis service 与固定 precedence                   | `cd src-tauri && cargo test test_analysis_service_preserves_precedence_and_typed_metadata -- --nocapture`                                                                   | ⬜ planned |
| `02-03-02` | `02-03` | 3    | `DETE-01, DETE-02, DETE-04` | 将 monitor/capture path 改接 analysis service 并持久化 diagnostics      | `cd src-tauri && cargo test test_analysis_fallback_persists_diagnostics -- --nocapture && cargo test test_capture_path_writes_entry_analysis_snapshot -- --nocapture`       | ⬜ planned |
| `02-04-01` | `02-04` | 4    | `DETE-02, DETE-04`          | 前端只消费 persisted analysis contract，不再偷跑 subtype 推断           | `pnpm test -- src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx`                                                           | ✅ green   |
| `02-04-02` | `02-04` | 4    | `DETE-04`                   | degraded entries 仍展示 raw content，并暴露 diagnostics                 | `pnpm test -- src/components/DetailView/DetailView.test.tsx`                                                                                                                | ✅ green   |
| `02-05-01` | `02-05` | 4    | `DETE-03, DETE-04`          | 建立 version-aware rebuild command 与 stale-row selection               | `cd src-tauri && cargo test test_rebuild_text_analysis_updates_existing_history -- --nocapture && cargo test test_rebuild_analysis_skips_fresh_rows -- --nocapture`         | ✅ green   |
| `02-05-02` | `02-05` | 4    | `DETE-03`                   | 接入 Preferences rebuild trigger、history refresh 与 cache invalidation | `pnpm test -- src/stores/configStore.test.ts src/components/Preferences/PreferencesModal.test.tsx`                                                                          | ✅ green   |

_Status: ⬜ planned · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave Gates

| Wave | Plans            | Gate Command                                                                                                                                                                                                                                                                                                                                                                                                             | Purpose                                                                |
| ---- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| 1    | `02-01`          | `cd src-tauri && cargo test --no-run && cargo test test_analysis_supported_content_maps_to_stable_subtypes -- --nocapture && cargo test test_analysis_metadata_contract_round_trip -- --nocapture`                                                                                                                                                                                                                       | 确认 `AnalysisSnapshot` contract、precedence 和 typed metadata 已固定  |
| 2    | `02-02`          | `cd src-tauri && cargo test test_entry_analysis_repository_upsert_and_join_read_model -- --nocapture && cargo test test_history_reads_analysis_rows_with_legacy_fallback -- --nocapture`                                                                                                                                                                                                                                 | 确认 `entry_analysis` 存储、UPSERT 和 joined read model 已闭合         |
| 3    | `02-03`          | `cd src-tauri && cargo test test_analysis_service_preserves_precedence_and_typed_metadata -- --nocapture && cargo test test_analysis_fallback_persists_diagnostics -- --nocapture && cargo test test_capture_path_writes_entry_analysis_snapshot -- --nocapture`                                                                                                                                                         | 确认 capture-time analysis service 已成为唯一 authority，并能 fallback |
| 4    | `02-04`, `02-05` | `pnpm test -- src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/DetailView.test.tsx src/stores/configStore.test.ts src/components/Preferences/PreferencesModal.test.tsx && cd src-tauri && cargo test test_rebuild_text_analysis_updates_existing_history -- --nocapture && cargo test test_rebuild_analysis_skips_fresh_rows -- --nocapture` | 确认前端 contract consumption 与 history rebuild 都通过                |

---

## Manual-Only Verifications

| Behavior                                                                             | Requirement | Why Manual                                                                           | Test Instructions                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------------ | ----------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 复制损坏 JSON、无效 base64 或冲突型文本时，历史仍保留 raw content 并显示 diagnostics | `DETE-04`   | 真实 clipboard 复制链路、joined read model 和 detail view 组合仍需桌面端 smoke       | 1. 启动桌面应用并开始监控。 2. 依次复制损坏 JSON、无效 base64、普通文本。 3. 确认条目均进入历史。 4. 对 degraded 条目打开详情，确认 raw 内容仍可查看，且能看到 fallback diagnostics 或状态提示。                                                                        |
| 从已有历史触发 rebuild 后，旧条目的 joined analysis 会更新而不需要重新复制           | `DETE-03`   | 需要真实 seeded history、按钮触发、joined read refresh 和旧 legacy fallback 一起验证 | 1. 预填若干只有 `clipboard_entries` 原始内容和旧 subtype/metadata 的历史数据。 2. 启动应用并打开 Preferences 的 system/cache 区域。 3. 点击 rebuild action。 4. 刷新列表并确认 `entry_analysis` 已补齐，subtype/metadata 以新分析为准，原始内容与 copy_count 保持不变。 |

---

## Validation Sign-Off

- [ ] All 10 planned tasks have a task-level automated command
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave gates match actual execution order `02-01 -> 02-02 -> 02-03 -> 02-04/02-05`
- [ ] `entry_analysis` joined read model is validated separately from raw `clipboard_entries`
- [ ] Rebuild behavior is validated separately from capture-time analysis
- [ ] Failure fallback distinguishes true plain text from analyzer degradation
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
