---
phase: 03-unified-developer-previews
plan: 01
subsystem: frontend-preview
tags: [frontend, preview, semantic-core, retrieval, vitest]
requires:
  - phase: 02-04
    provides: analysis-first frontend preview consumption from authoritative Rust analysis
  - phase: 02-06
    provides: URL-first detail preview contract without default remote dependency
  - phase: 02-07
    provides: raw-accessible detail preview contract and explicit preview height rules
provides:
  - analysis-first `SemanticPreviewModel` contract for all recognized preview types
  - shared `buildSemanticPreviewModel()` semantic entry point for list/detail/retrieval consumers
  - density-aware `buildPreviewSummary(entry, 'list' | 'retrieval')` adapter for Phase 3 and Phase 4 reuse
affects: [03-02, phase-04, clipboard-list, detail-preview, retrieval]
tech-stack:
  added: []
  patterns:
    [
      analysis-first semantic preview core,
      explicit per-type preview intent mapping,
      list/retrieval density projection from one semantic summary,
    ]
key-files:
  created: [src/lib/preview/previewSummary.ts, src/lib/preview/previewSummary.test.ts]
  modified:
    [
      src/types/clipboard.ts,
      src/lib/preview/entryPresentation.ts,
      src/lib/preview/entryPresentation.test.ts,
    ]
key-decisions:
  - 'Semantic preview contract 保持同步、analysis-first，只依赖 `entry.analysis`、本地 metadata 和 raw content。'
  - 'list/retrieval 的差异落在 density adapter，而不是重新解析 subtype 或引入新的 preview 语义入口。'
patterns-established:
  - 'Semantic core pattern: 所有已识别类型都先产出 `semanticType`、`previewIntent`、headline 和 secondary summary，再供不同 surface 投影。'
  - 'Density adapter pattern: list 取 compact secondary segment，retrieval 取 full semantic summary，保持相同 semanticType/previewIntent。'
requirements-completed: [PREV-05]
duration: 11min
completed: 2026-03-28
---

# Phase 03 Plan 01: Semantic Preview Core Summary

**Analysis-first semantic preview core with explicit per-type contracts and shared list/retrieval density adapters**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-28T16:00:40Z
- **Completed:** 2026-03-28T16:11:59Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- 在 `src/types/clipboard.ts` 建立 Phase 03 的 `PreviewIntent`、`SemanticPreviewModel`、`PreviewSummaryDensity` 与 `PreviewSummaryDescriptor` 合同。
- 在 `src/lib/preview/entryPresentation.ts` 建立 analysis-first `buildSemanticPreviewModel()`，并为 JSON、URL、颜色、代码、命令以及 remaining types 提供显式 semantic mapping 与 fallback。
- 新增 `src/lib/preview/previewSummary.ts`，让 list 与 retrieval 通过同一 semantic core 派生不同密度摘要，而不重新解析 subtype 或依赖异步 preview resolution。

## Task Commits

Each TDD task was committed atomically:

1. **Task 1 RED: semantic preview core contract tests** - `4928a8d` (`test`)
2. **Task 1 GREEN: analysis-first semantic preview core** - `fb2b1e0` (`feat`)
3. **Task 2 RED: preview summary density contract tests** - `3567ab8` (`test`)
4. **Task 2 GREEN: list/retrieval preview summary adapter** - `9952b54` (`feat`)

## Files Created/Modified

- `src/types/clipboard.ts` - 新增 Phase 03 共享 preview contract 类型。
- `src/lib/preview/entryPresentation.ts` - 把 preview 语义入口提升为 `buildSemanticPreviewModel()`，并让旧 `buildEntrySemanticSummary()` 从新 model 派生。
- `src/lib/preview/entryPresentation.test.ts` - 锁定高频类型与 remaining types 的 semantic core contract。
- `src/lib/preview/previewSummary.ts` - 新增 list/retrieval 共用的 density adapter。
- `src/lib/preview/previewSummary.test.ts` - 锁定所有已识别类型的 list/retrieval summary contract。

## Decisions Made

- 保持 `buildSemanticPreviewModel()` 纯同步，不接入 `resolveEntryPreview()` 或任何远端 URL enrichment 数据源。
- 顶层 `image` / `file` 继续按 `content_type` 优先判定，以避免被 text subtype fallback 抢走预览语义。
- retrieval contract 只复用并展开 semantic core 的 `secondarySummary`，不提前实现搜索匹配、高亮或 fuzzy 逻辑。

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `ClipboardItem.tsx` 可以在 03-02 直接消费 `buildPreviewSummary(entry, 'list')`，替换现有 subtype-specific list 分支。
- detail preview 后续可以逐步改为消费 `SemanticPreviewModel`，避免继续扩张平行 headline/summary 逻辑。
- Phase 4 检索结果已经有可复用的 `retrieval` density contract，不需要重新发明 preview summary 语义。

## Self-Check

PASSED
