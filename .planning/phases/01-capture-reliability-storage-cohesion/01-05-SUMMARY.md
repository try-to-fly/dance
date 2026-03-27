---
phase: 01-capture-reliability-storage-cohesion
plan: 05
subsystem: infra
tags: [rust, tauri, macos, clipboard, capture-policy]
requires:
  - phase: 01-01
    provides: capture policy test module registration and temp-root test scaffolding
  - phase: 01-03
    provides: CaptureRuntime suppression contract and observed-hash dedupe lifecycle
provides:
  - marker-first CapturePolicy disposition matrix for CAPT-03
  - macOS pasteboard marker adapter with non-macOS no-op fallback
  - monitor pre-detection capture gating for text and image clipboard events
  - CAPT-03 automated tests covering policy matrix and CurrentOnly non-persistence
affects: [Phase 02 analysis contracts, CAPT-03, clipboard monitor]
tech-stack:
  added: []
  patterns:
    [
      marker-first capture policy,
      macOS pasteboard marker adapter,
      shared observed-hash dedupe contract,
    ]
key-files:
  created: [src-tauri/src/capture/policy.rs, src-tauri/src/capture/macos_markers.rs]
  modified:
    [
      src-tauri/src/capture/mod.rs,
      src-tauri/src/capture/runtime.rs,
      src-tauri/src/clipboard/monitor.rs,
      src-tauri/src/capture_policy_tests.rs,
    ]
key-decisions:
  - 'CAPT-03 先读取 pasteboard marker 和 source metadata，再决定是否进入 ContentDetector::detect 或图片处理。'
  - 'CurrentOnly 在 Phase 1 只更新 observed-hash dedupe 状态，不发送到 runtime save loop。'
  - 'macOS 使用 NSPasteboard marker adapter；非 macOS 明确保持 no-op markers，不破坏现有采集。'
patterns-established:
  - 'Capture gating: monitor 统一先执行 decide_capture()，只有 Persist 才继续做文本检测、图片处理和 tx.send(entry)。'
  - 'Observed hash contract: suppression、CurrentOnly 和 Skip 共用 remember_observed_hash()，保持与 register_suppression_key() 相同的 content_hash 语义。'
requirements-completed: [CAPT-03]
duration: 20min
completed: 2026-03-27
---

# Phase 01 Plan 05: Capture Policy Summary

**marker-first CAPT-03 capture policy、macOS pasteboard marker adapter，以及 monitor 前置持久化闸门已经接到同一条 observed-hash dedupe 合同上**

## Performance

- **Duration:** 20 min
- **Started:** 2026-03-27T14:32:55Z
- **Completed:** 2026-03-27T14:52:25Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- 在 `src-tauri/src/capture/policy.rs` 中固定 CAPT-03 的 `Persist` / `CurrentOnly` / `Skip` 纯函数策略矩阵，并移除了测试里的 ignore 占位。
- 新增 `src-tauri/src/capture/macos_markers.rs`，在 macOS 读取 transient、concealed、auto-generated、source 和 remote clipboard markers，非 macOS 回退到 no-op adapter。
- 把 `src-tauri/src/clipboard/monitor.rs` 改成 marker-first 流程：先读 marker，再跑 `decide_capture()`，只有 `Persist` 才继续 `ContentDetector::detect(...)`、图片处理和 `tx.send(entry)`。

## Task Commits

Each task was committed atomically:

1. **Task 1: 定义 marker-first CapturePolicy 并落地 CAPT-03 策略矩阵** - `03bf2c4` (test), `eaf3952` (feat)
2. **Task 2: 接入 macOS marker adapter，并在 monitor 中先判策略再做检测** - `e00662f` (test), `f353775` (feat)

_Note: TDD tasks have separate red/green commits._

## Files Created/Modified

- `src-tauri/src/capture/policy.rs` - 定义 CAPT-03 的 `CaptureDisposition`、`PasteboardMarkers` 和纯函数 `decide_capture()`。
- `src-tauri/src/capture/macos_markers.rs` - 提供 macOS pasteboard marker 读取和非 macOS no-op fallback。
- `src-tauri/src/capture/runtime.rs` - 抽出共享 `remember_observed_hash()` helper，让 monitor 和 runtime 走同一 observed-hash 合同。
- `src-tauri/src/clipboard/monitor.rs` - 在文本和图片采集路径上前置 marker/policy 判定，并把 `CurrentOnly` / `Skip` 接到非持久化分支。
- `src-tauri/src/capture_policy_tests.rs` - 覆盖 CAPT-03 策略矩阵，以及 `CurrentOnly` 不检测、不发送、不入库的自动化断言。
- `src-tauri/src/capture/mod.rs` - 注册 policy 和 macOS marker 模块，并导出 monitor 需要的 capture helpers。

## Decisions Made

- 使用 marker-first 判定而不是继续在 monitor 里堆内容启发式，这样 CAPT-03 的规则来源固定，后续 Phase 2 只需消费现成的 disposition 结果。
- `CurrentOnly` 明确解释为“更新当前观察态但不持久化”，从而把 auto-generated 事件排除在历史库之外，同时仍复用 dedupe key。
- `remember_observed_hash()` 从 runtime 提炼成共享 helper，避免 monitor 为 `CurrentOnly` 重复实现一套平行 dedupe 逻辑。

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Task 1 的红灯提交第一次被 pre-commit 的 `-D warnings` 拦下，因为新 policy 类型在实现接线前还未被生产代码消费；通过最小 `allow` 标注保留红灯测试后继续执行。
- 计划里建议的 macOS 手动 smoke 未在本次执行中补跑；当前只完成了两条 Rust 自动化验证命令。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 02 现在可以假设只有 `Persist` 的事件才会进入后续分析与持久化链路，不需要再在 subtype 检测里兜 CAPT-03 的过滤职责。
- 后续若要扩展更多平台 marker 能力，只需要继续扩展 `macos_markers.rs` 风格的 adapter，而不需要重新拆 monitor 流程。

## Self-Check

PASSED
