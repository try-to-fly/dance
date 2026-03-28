---
phase: 02-analysis-contracts-versioned-detection
plan: 03
subsystem: analysis
tags: [rust, tauri, clipboard, runtime, diagnostics, analysis]
requires:
  - phase: 02-02
    provides: `entry_analysis` companion table、repository helper 和 authoritative joined read model
provides:
  - capture-time authoritative `TextAnalysisService` fallback diagnostics
  - monitor/runtime persistence of raw entry + companion analysis in one save flow
  - integration tests for matched analysis、fallback diagnostics 和 CurrentOnly non-persistence
affects: [02-04, 02-05, runtime, detail-preview]
tech-stack:
  added: []
  patterns:
    [
      malformed-structured-input fallback diagnostics,
      monitor-to-runtime analysis handoff,
      raw-entry plus companion-analysis persistence,
    ]
key-files:
  created: []
  modified:
    [
      src-tauri/src/analysis/contract.rs,
      src-tauri/src/analysis/service.rs,
      src-tauri/src/clipboard/content_detector.rs,
      src-tauri/src/clipboard/monitor.rs,
      src-tauri/src/capture/runtime.rs,
      src-tauri/src/integration_tests.rs,
      src-tauri/src/capture_policy_tests.rs,
    ]
key-decisions:
  - 'malformed JSON / URL / base64 候选不再静默掉回真正 plain text，而是持久化为 fallback plain_text + diagnostics。'
  - 'monitor 只负责 marker-first gating 和 analysis handoff，runtime 在同一保存链路里同时写 raw row 与 companion analysis。'
  - 'event payload 继续保留 legacy `content_subtype` / `metadata` 兼容层，但语义权威来自 `entry.analysis`。'
patterns-established:
  - 'Fallback semantics: 真实 plain text 与 degraded fallback 通过 `analysis.status` 区分，而不是继续共用同一个无诊断的 plain_text。'
  - 'Capture persistence: text capture 进入 Persist 分支后先生成 `AnalysisSnapshot`，再让 runtime 一次性持久化 raw + companion row。'
requirements-completed: [DETE-01, DETE-02, DETE-04]
duration: 13 min
completed: 2026-03-28
---

# Phase 02 Plan 03: Runtime Analysis Wiring Summary

**authoritative text analysis 已经接入 capture/runtime 主链路，坏掉的结构化输入也会以 raw content + fallback diagnostics 的方式落库而不是被静默吞掉**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-28T03:19:59Z
- **Completed:** 2026-03-28T03:33:04Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- 扩展 `TextAnalysisService`，让它不仅输出 stable subtype 和 typed metadata，也能对 malformed JSON / URL / base64 候选显式生成 fallback diagnostics。
- 把 `ClipboardMonitor` 的 text Persist 分支切到 authoritative analysis service，并通过 `ClipboardEntry.analysis` 把 snapshot 送进 runtime。
- 在 `CaptureRuntime` 中把 raw `clipboard_entries` 和 `entry_analysis` companion row 放到同一条保存链路里，同时补齐 matched / fallback 的集成测试。

## Task Commits

这个 plan 以一笔跨文件功能提交落地，因为 analyzer/service、monitor/runtime 接线和对应回归测试共享同一组文件：

1. **Task 1 + Task 2: authoritative analyzer fallback 语义与 runtime companion persistence 接线** - `701128b` (feat)

## Files Created/Modified

- `src-tauri/src/analysis/service.rs` - 增加 malformed structured-input fallback 诊断、typed metadata 构造和 shell/markdown hints。
- `src-tauri/src/analysis/contract.rs` - 扩展 command / markdown metadata 字段，支撑新的 typed metadata contract。
- `src-tauri/src/clipboard/monitor.rs` - Persist 分支改为调用 `TextAnalysisService` 并把 `AnalysisSnapshot` 挂到 entry 上。
- `src-tauri/src/capture/runtime.rs` - 在同一次保存链路里写 raw row 和 `entry_analysis` companion row，并把 analysis 挂回 emitted entry。
- `src-tauri/src/clipboard/content_detector.rs` - 增加 stable supported-subtype corpus 测试名，锁定 detector 稳定性。
- `src-tauri/src/integration_tests.rs` - 覆盖 precedence/typed metadata、fallback diagnostics、matched persistence 和 runtime fallback 落库。
- `src-tauri/src/capture_policy_tests.rs` - 确认 `CurrentOnly` 仍然不会进入 analysis / persistence 链路。

## Decisions Made

- 对“看起来像结构化内容但解析失败”的输入，优先保留 raw content 并持久化 diagnostics，而不是继续把它们伪装成毫无上下文的普通文本。
- command metadata 明确提升为 `command_name` + `shell_family`，让后续前端 detail 不必再自己猜 shell 语义。
- runtime 在 companion row 写入成功后直接把 analysis 挂回返回条目，避免列表和事件流拿到落后的 legacy-only payload。

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- 并行跑多个 `cargo test` 时仍然会看到构建目录锁等待，但所有目标测试都在同一版代码上通过，没有出现真实的执行失败。
- 本轮 staging 过程中 `.git/index.lock` 继续偶发残留，不过每次都能在重试前自动消失，没有导致提交中断。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `02-04` 现在可以开始让前端只消费 `entry.analysis`，并把 degraded / fallback diagnostics 暴露给 detail view。
- `02-05` 已经有 capture-time companion row 和 version-aware stale selector，可直接进入 rebuild 命令和 Preferences 入口。

## Self-Check

PASSED
