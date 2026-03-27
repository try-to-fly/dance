---
phase: 01-capture-reliability-storage-cohesion
plan: 04
subsystem: ui
tags: [react, tauri, vitest, clipboard, ipc]
requires:
  - phase: 01-03
    provides: backend-owned copy_to_clipboard suppression contract
provides:
  - 共享 frontend backend-copy helper
  - store、menu、renderer、log viewer 统一 copy routing
  - 前端 copy-routing contract tests
affects: [01-05 capture policy, CAPT-02, frontend copy UX]
tech-stack:
  added: []
  patterns: [shared backend copy helper, frontend copy-routing contract tests]
key-files:
  created: []
  modified:
    [
      src/stores/clipboardStore.ts,
      src/stores/clipboardStore.test.ts,
      src/components/ClipboardMenuHandler.tsx,
      src/components/DetailView/ContentRenderers/JsonRenderer.tsx,
      src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx,
      src/components/DetailView/ContentRenderers/UnifiedTextRenderer.tsx,
      src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx,
      src/components/LogViewer/LogViewer.tsx,
    ]
key-decisions:
  - "把 frontend copy helper 收敛到 src/stores/clipboardStore.ts，由它统一调用 invoke('copy_to_clipboard', { content })。"
  - '菜单、renderer 和 log viewer 都只复用共享 helper，不再各自直写系统剪贴板。'
  - 'TDD 继续用 Vitest contract tests 锁定 copy-routing，确保回退时直接在执行阶段报红。'
patterns-established:
  - "Copy routing: 前端所有现有文本复制入口统一走 copyToClipboard() -> invoke('copy_to_clipboard', { content })。"
  - 'Contract coverage: store 与 renderer 测试显式断言 backend route 被调用，且不会再调用 writeText。'
requirements-completed: [CAPT-02]
duration: 8min
completed: 2026-03-27
---

# Phase 01 Plan 04: Frontend Copy Routing Contract Summary

**共享 frontend backend-copy helper 覆盖 store、菜单、renderer 与日志复制入口，并用 Vitest contract tests 固定 copy_to_clipboard 路由**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-27T14:12:50Z
- **Completed:** 2026-03-27T14:20:58Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- 在 `src/stores/clipboardStore.ts` 中暴露共享 `copyToClipboard()` helper，并让 store 级复制动作统一调用后端 `copy_to_clipboard` 命令。
- 把 `ClipboardMenuHandler`、`JsonRenderer`、`UnifiedTextRenderer` 和 `LogViewer` 的现有文本复制入口全部切到 backend-owned contract。
- 为 store、`JsonRenderer` 和 `UnifiedTextRenderer` 增加明确的 frontend contract tests，防止重新引入前端直写系统剪贴板。

## Task Commits

Each task was committed atomically:

1. **Task 1: 收敛共享 backend copy helper 与 store/menu 合同** - `883be55` (test), `e6bf14a` (feat)
2. **Task 2: sweep renderer 与 log viewer 的剩余 copy 入口并补 contract tests** - `1a36976` (test), `da0ecd0` (feat)

## Files Created/Modified

- `src/stores/clipboardStore.ts` - 提供共享 `copyToClipboard()` helper，并让 store action 统一调用 Tauri backend command。
- `src/stores/clipboardStore.test.ts` - 增加 store copy-routing contract test，断言走 `copy_to_clipboard` 且不触发 `writeText`。
- `src/components/ClipboardMenuHandler.tsx` - 菜单 copy/cut 入口复用共享 helper，不再直接写系统剪贴板。
- `src/components/DetailView/ContentRenderers/JsonRenderer.tsx` - JSON 复制按钮改走 backend copy route。
- `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx` - 断言 JSON renderer 的复制按钮调用 `copy_to_clipboard`。
- `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.tsx` - 文本/代码/命令 renderer 的复制按钮改走 backend copy route。
- `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` - 断言文本 renderer 的复制按钮调用 `copy_to_clipboard`。
- `src/components/LogViewer/LogViewer.tsx` - 日志复制动作改走 backend contract，并补上失败处理 toast。

## Decisions Made

- 共享 helper 放在 `clipboardStore.ts`，因为这是现有前端复制动作最稳定的共用层，能在不引入新 service 层的前提下完成 01-04 的收敛目标。
- `LogViewer` 也统一接到 backend contract，而不是保留 `navigator.clipboard.writeText` 例外路径，确保 suppression 不会被特殊 UI 分支绕过。
- contract tests 直接断言 `invoke('copy_to_clipboard', { content })`，这样能把 Phase 01-03 建立的后端合同显式延续到 UI 层。

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `gsd-tools` 在执行 `state update-progress` / `roadmap update-plan-progress` 后返回了正确的 4/5 结果，但文件里的聚合进度行没有同步刷新；已在收尾阶段手工把 `STATE.md` 和 `ROADMAP.md` 校正到 01-04 完成后的实际状态。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 01-05 现在可以假设所有现有前端文本复制入口都已经先经过后端 suppression contract，再继续扩展 marker-first capture policy。
- CAPT-02 的前端闭环已经建立，后续如果有新的 copy 入口，只需要复用共享 helper 并按现有测试模式补 contract coverage。

## Self-Check

PASSED
