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

| Property               | Value                                                                                                                                     |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Framework**          | Rust `cargo test` for contract, persistence, capture, rebuild, and regression coverage; Vitest for frontend analysis-contract consumption |
| **Config file**        | `vitest.config.ts`; Rust has no separate config file                                                                                      |
| **Quick run command**  | `cd src-tauri && cargo test --no-run`                                                                                                     |
| **Full suite command** | `pnpm test && cd src-tauri && cargo test`                                                                                                 |
| **Estimated runtime**  | ~45-90 seconds for task-level commands; ~4-6 minutes for the full phase gate including gap-closure waves                                  |

---

## Sampling Rate

- **After every task:** run that task's exact `<verify><automated>` command from the matrix below.
- **After every wave:** run the wave gate command for that wave before starting the next wave.
- **Before `$gsd-verify-work`:** full frontend and Rust suites must be green.
- **Max feedback latency:** 60 seconds for task-level commands.

---

## Task Verification Matrix

| Task ID    | Plan    | Wave | Requirement(s)              | Purpose                                                                                         | Automated Command                                                                                                                                                                                     | Status     |
| ---------- | ------- | ---- | --------------------------- | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| `02-01-01` | `02-01` | 1    | `DETE-01, DETE-02, DETE-04` | 建立 Rust `AnalysisSnapshot` contract 与 fallback shape                                         | `cd src-tauri && cargo test --no-run`                                                                                                                                                                 | ✅ green   |
| `02-01-02` | `02-01` | 1    | `DETE-01, DETE-02, DETE-04` | 锁定 stable subtype corpus、precedence 与 fallback serialization                                | `cd src-tauri && cargo test test_text_analysis_contract_preserves_supported_subtypes -- --nocapture && cargo test test_text_analysis_contract_failure_fallback_serializes_diagnostics -- --nocapture` | ✅ green   |
| `02-02-01` | `02-02` | 2    | `DETE-02, DETE-03, DETE-04` | 建立 `entry_analysis` companion table、repository helper 和 version-aware SQL 约束              | `cd src-tauri && cargo test test_database_reads_analysis_columns_after_migration -- --nocapture && cargo test test_database_upserts_entry_analysis_rows -- --nocapture`                               | ✅ green   |
| `02-02-02` | `02-02` | 2    | `DETE-02, DETE-03, DETE-04` | 扩展共享 `ClipboardEntry`/TypeScript 类型，并把 history 读模型切到 authoritative analysis first | `cd src-tauri && cargo test test_clipboard_entry_analysis_fields_default_to_none -- --nocapture && cargo test test_database_merges_analysis_fields -- --nocapture`                                    | ✅ green   |
| `02-03-01` | `02-03` | 3    | `DETE-01, DETE-02, DETE-04` | 扩展 authoritative analyzer，输出 subtype-specific metadata 与 explicit fallback diagnostics    | `cd src-tauri && cargo test test_content_detector_supported_subtypes_are_stable -- --nocapture && cargo test test_text_analysis_service_fallback_preserves_diagnostics -- --nocapture`                | ✅ green   |
| `02-03-02` | `02-03` | 3    | `DETE-01, DETE-02, DETE-04` | 接入 monitor/runtime，让 raw entry 与 companion analysis 在同一保存链路内落地                   | `cd src-tauri && cargo test test_capture_runtime_failure_analysis_falls_back_to_plain_text -- --nocapture && cargo test test_integration_text_entries_store_analysis_contract -- --nocapture`         | ✅ green   |
| `02-04-01` | `02-04` | 4    | `DETE-02, DETE-04`          | helper/descriptor 改成 analysis-first，并直接测试 `entryPresentation` 语义                      | `pnpm test -- src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx`                                                                                     | ✅ green   |
| `02-04-02` | `02-04` | 4    | `DETE-01, DETE-04`          | 收紧 store / DetailView，停止前端 subtype inference，只保留 rendering fallback                  | `pnpm test -- src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/DetailView.test.tsx`                                                                                 | ✅ green   |
| `02-05-01` | `02-05` | 4    | `DETE-03, DETE-04`          | 建立 version-aware rebuild command 与 stale-row selection                                       | `cd src-tauri && cargo test test_rebuild_text_analysis_updates_existing_history -- --nocapture && cargo test test_rebuild_text_analysis_skips_non_text_entries -- --nocapture`                        | ✅ green   |
| `02-05-02` | `02-05` | 4    | `DETE-03`                   | 接入 Preferences rebuild trigger、history refresh 与 cache invalidation                         | `pnpm test -- src/components/Preferences/PreferencesModal.test.tsx`                                                                                                                                   | ✅ green   |
| `02-06-01` | `02-06` | 5    | `DETE-02, DETE-04`          | 把 URL 条目的主视图合同收口回 `url_card`，并移除 raw URL 到 JSON fallback 的错误桥接            | `pnpm test -- src/components/DetailView/DetailPreviewContract.test.tsx`                                                                                                                               | ⬜ planned |
| `02-06-02` | `02-06` | 5    | `DETE-02, DETE-04`          | 让 URL resolved 备用视图真正可渲染，并补组件级媒体/文本回归测试                                 | `pnpm test -- src/components/DetailView/scene/AlternateViews.test.tsx`                                                                                                                                | ⬜ planned |
| `02-07-01` | `02-07` | 6    | `DETE-04`                   | 恢复 JSON raw-only 备用视图入口，并给 detail 左列建立稳定滚动容器                               | `pnpm test -- src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/DetailView.test.tsx`                                                                                  | ⬜ planned |
| `02-07-02` | `02-07` | 6    | `DETE-04`                   | 给 `JsonRenderer` 的代码视图和树视图建立显式高度合同                                            | `pnpm test -- src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`                                                                                                                       | ⬜ planned |

_Status: ⬜ planned · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave Gates

| Wave | Plans            | Gate Command                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Purpose                                                                           |
| ---- | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| 1    | `02-01`          | `cd src-tauri && cargo test --no-run && cargo test test_text_analysis_contract_preserves_supported_subtypes -- --nocapture && cargo test test_text_analysis_contract_failure_fallback_serializes_diagnostics -- --nocapture`                                                                                                                                                                                                                                                                  | 确认 `AnalysisSnapshot` contract、subtype corpus 与 fallback serialization 已固定 |
| 2    | `02-02`          | `cd src-tauri && cargo test test_database_reads_analysis_columns_after_migration -- --nocapture && cargo test test_database_upserts_entry_analysis_rows -- --nocapture && cargo test test_clipboard_entry_analysis_fields_default_to_none -- --nocapture && cargo test test_database_merges_analysis_fields -- --nocapture`                                                                                                                                                                   | 确认 `entry_analysis` schema、repository 和 joined read model 已闭合              |
| 3    | `02-03`          | `cd src-tauri && cargo test test_content_detector_supported_subtypes_are_stable -- --nocapture && cargo test test_text_analysis_service_fallback_preserves_diagnostics -- --nocapture && cargo test test_capture_runtime_failure_analysis_falls_back_to_plain_text -- --nocapture && cargo test test_integration_text_entries_store_analysis_contract -- --nocapture`                                                                                                                         | 确认 capture-time analysis service 已成为唯一 authority，并能持久化 fallback      |
| 4    | `02-04`, `02-05` | `pnpm test -- src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx && pnpm test -- src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/DetailView.test.tsx && pnpm test -- src/components/Preferences/PreferencesModal.test.tsx && cd src-tauri && cargo test test_rebuild_text_analysis_updates_existing_history -- --nocapture && cargo test test_rebuild_text_analysis_skips_non_text_entries -- --nocapture` | 确认前端 analysis-first 消费与历史 rebuild 闭环都通过                             |
| 5    | `02-06`          | `pnpm test -- src/components/DetailView/DetailPreviewContract.test.tsx && pnpm test -- src/components/DetailView/scene/AlternateViews.test.tsx`                                                                                                                                                                                                                                                                                                                                               | 确认 URL-first detail contract 与 resolved alternate rendering 已修复             |
| 6    | `02-07`          | `pnpm test -- src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/DetailView.test.tsx && pnpm test -- src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`                                                                                                                                                                                                                                                                                         | 确认 JSON raw-only 可达性、detail 滚动与代码视图高度合同已修复                    |

---

## Manual-Only Verifications

| Behavior                                                                                             | Requirement | Why Manual                                                                           | Test Instructions                                                                                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 复制损坏 JSON、无效 base64 或冲突型文本时，历史仍保留 raw content 并显示 diagnostics                 | `DETE-04`   | 真实 clipboard 复制链路、joined read model 和 detail view 组合仍需桌面端 smoke       | 1. 启动桌面应用并开始监控。 2. 依次复制损坏 JSON、无效 base64、普通文本。 3. 确认条目均进入历史。 4. 对 degraded 条目打开详情，确认 raw 内容仍可查看，且能看到 fallback diagnostics 或状态提示。                                                                        |
| 从已有历史触发 rebuild 后，旧条目的 joined analysis 会更新而不需要重新复制                           | `DETE-03`   | 需要真实 seeded history、按钮触发、joined read refresh 和旧 legacy fallback 一起验证 | 1. 预填若干只有 `clipboard_entries` 原始内容和旧 subtype/metadata 的历史数据。 2. 启动应用并打开 Preferences 的 system/cache 区域。 3. 点击 rebuild action。 4. 刷新列表并确认 `entry_analysis` 已补齐，subtype/metadata 以新分析为准，原始内容与 copy_count 保持不变。 |
| 复制一个 HTTPS URL 后，详情主区域仍显示 URL 卡片，协议/host/path 可见，且不再出现 JSON/树形视图误抢  | `DETE-02`   | 需要真实 URL 条目、detail 主视图、备用视图与动作按钮一起验证                         | 1. 复制一个 HTTPS URL。 2. 在列表中选中对应条目并打开详情。 3. 确认主区域显示 URL 卡片而不是 JSON/树形视图。 4. 确认右侧仍能看到协议、host、path 等结构信息，并且可执行打开链接操作。 5. 如有远端 resolved 内容，只应作为备用视图出现。                                 |
| 复制合法 JSON 后，detail 默认进入 JSON 结构化预览，但 Raw 入口可达，代码视图可见且超长内容可滚动查看 | `DETE-04`   | 需要真实布局、Monaco 高度、左列滚动容器和 Raw 切换入口一起验证                       | 1. 复制一段合法 JSON，内容至少包含足够长的对象或数组。 2. 打开详情，确认默认进入 JSON 结构化预览。 3. 确认能看到 Raw 备用视图入口并切换查看原文。 4. 切换到代码视图，确认代码内容不为空。 5. 验证长内容和下方备用视图都能通过滚动完整访问。                             |

---

## Validation Sign-Off

- [ ] All 14 planned tasks have a task-level automated command
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave gates match actual execution order `02-01 -> 02-02 -> 02-03 -> 02-04/02-05 -> 02-06 -> 02-07`
- [ ] `entryPresentation` helper semantics are validated directly, not only through upper-layer contract tests
- [ ] `entry_analysis` joined read model is validated separately from raw `clipboard_entries`
- [ ] Rebuild behavior is validated separately from capture-time analysis
- [ ] URL-first detail regression has a dedicated automated gate before JSON/layout fixes
- [ ] JSON raw-only/detail scroll regression has a dedicated automated gate before manual UAT closure
- [ ] Failure fallback distinguishes true plain text from analyzer degradation
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
