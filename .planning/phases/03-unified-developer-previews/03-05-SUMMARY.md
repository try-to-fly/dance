---
phase: 03-unified-developer-previews
plan: 05
subsystem: detail-preview
tags: [frontend, detail, workbench, code, command, vitest]
requires:
  - phase: 03-03
    provides: stable local-first detail path and semantic descriptor contract
provides:
  - session-scoped local workbench for code and command detail previews
  - detail-level buffer ownership and copy delegation to the current edited content
  - reset semantics on entry switch and detail close without mutating stored history
affects: [detail-preview, code-workbench, command-workbench]
tech-stack:
  added: []
  patterns:
    [
      session-keyed local editor reset,
      detail-owned temporary workbench buffer,
      copy-current-buffer interaction contract,
    ]
key-files:
  created: []
  modified:
    [
      src/components/DetailView/ContentRenderers/UnifiedTextRenderer.tsx,
      src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx,
      src/components/DetailView/DetailView.tsx,
      src/components/DetailView/DetailView.test.tsx,
      src/components/DetailView/scene/PrimaryPreviewRenderer.tsx,
    ]
key-decisions:
  - '以 `03-CONTEXT.md` 的 D-14..D-17 为准，覆盖 PREV-04 原先的 read-only wording。'
  - 'workbench buffer 只存在于 detail 会话内，由 `DetailView` 持有，不回写 store 历史内容。'
  - '代码与命令才接入 sessionKey/onContentChange，markdown 与 plain_text 不扩散这套本地编辑状态。'
patterns-established:
  - 'Workbench ownership pattern: renderer 负责上报 buffer，DetailView 负责 session reset、copy delegation 与会话生命周期。'
  - 'Session reset pattern: `entry.id + content_hash` 组成 sessionKey，即使原文相同也能在切换条目时强制重置编辑缓冲。'
requirements-completed: [PREV-04, PREV-05]
duration: 1min
completed: 2026-03-29
---

# Phase 03 Plan 05: Local Detail Workbench Summary

**Code and command detail previews now behave as temporary local workbenches with copy-current-buffer semantics**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-29T00:42:57+08:00
- **Completed:** 2026-03-29T00:43:08+08:00
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- 为 `UnifiedTextRenderer` 增加 `sessionKey` 与 `onContentChange` contract，并在 session 切换时重置本地 `editedContent`，同时持续向上游上报当前 buffer。
- 把 `DetailView` 改为持有 detail-local `workbenchBuffer`，在 code / command 场景下默认复制当前编辑缓冲，而不是历史原文。
- 通过 `PrimaryPreviewRenderer` 仅向 code / command renderer 传递 workbench props，避免把这套本地编辑语义扩散到 markdown / plain_text。
- 补齐 `UnifiedTextRenderer.test.tsx` 与 `DetailView.test.tsx` 的回归覆盖，锁定 D-15、D-16、D-17 的 reset / report / copy contract。

## Task Commits

Each TDD task was committed atomically:

1. **Task 1 RED: local workbench contract coverage** - `1343a55` (`test`)
2. **Task 2 GREEN: detail workbench buffer ownership** - `e8548ea` (`feat`)

## Files Created/Modified

- `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.tsx` - 新增 session reset 与 buffer 上报能力。
- `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` - 锁定 sessionKey / onContentChange contract。
- `src/components/DetailView/DetailView.tsx` - 持有并消费 detail-local workbenchBuffer。
- `src/components/DetailView/DetailView.test.tsx` - 锁定 copy-current-buffer 与 close/reset 行为。
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx` - 仅为 code / command 传递 workbench props。

## Decisions Made

- code / command detail 的编辑只在当前 detail 会话内存在，不会回写 `selectedEntry.content_data`。
- 默认 header copy 动作在 workbench 场景下复制当前编辑值；关闭 detail 或切换条目后恢复到新条目的原始内容。
- session reset 以 `entry.id:content_hash` 为主键，而不是只比较内容字符串，避免相同原文的不同记录复用旧缓冲。

## Deviations from Plan

None - plan executed as specified.

## Issues Encountered

DetailView 新增测试最初因为 mock store 每次 render 返回新对象而触发 effect 无限重跑；修正为稳定引用 mock 后，验证结果恢复正常，业务实现无额外偏差。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- unified developer previews 的 list/detail/workbench 三条主线已经闭合，Phase 4 可以直接基于这些稳定 contract 推进检索与筛选体验。

## Self-Check

PASSED
