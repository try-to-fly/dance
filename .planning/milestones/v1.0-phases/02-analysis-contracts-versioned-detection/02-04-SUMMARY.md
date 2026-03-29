---
phase: 02-analysis-contracts-versioned-detection
plan: 04
subsystem: frontend-preview
tags: [frontend, detail-view, preview, analysis, diagnostics]
requires:
  - phase: 02-02
    provides: authoritative joined history model with `entry.analysis`
  - phase: 02-03
    provides: capture-time authoritative `AnalysisSnapshot` with fallback diagnostics
provides:
  - analysis-first entry presentation helpers for subtype, metadata, status, and diagnostics
  - detail preview descriptor consuming authoritative analysis before legacy fallback
  - store/detail regression coverage for degraded entries and legacy-only rows
affects: [02-05, phase-03, detail-preview, preview-store]
tech-stack:
  added: []
  patterns:
    [
      analysis-first frontend consumption,
      descriptor-level degraded badges and diagnostics inspector,
      rendering fallback without subtype inference,
    ]
key-files:
  created: [src/lib/preview/entryPresentation.test.ts, src/stores/configStore.test.ts]
  modified:
    [
      src/types/clipboard.ts,
      src/lib/preview/entryPresentation.ts,
      src/lib/preview/previewDescriptor.ts,
      src/stores/clipboardStore.ts,
      src/components/DetailView/DetailView.tsx,
      src/components/DetailView/scene/DetailScene.tsx,
      src/components/DetailView/scene/PrimaryPreviewRenderer.tsx,
      src/components/DetailView/DetailPreviewContract.test.tsx,
      src/components/DetailView/DetailView.test.tsx,
    ]
key-decisions:
  - '`entry.analysis` 一旦存在，就不再回退 legacy `content_subtype` / `metadata` 参与 detail 语义决策。'
  - 'URL/Base64 的前端 fallback 继续只负责 previewKind 和渲染数据解析，不再反向定义 entry subtype。'
  - 'fallback analysis 通过 warning badge、metadata pill 和 diagnostics inspector 同时向用户暴露，但 raw content 仍然是主视图。'
patterns-established:
  - 'Presentation helpers: `getEntryAnalysisSubtype` / `getEntryPresentationMetadata` / `getEntryAnalysisDiagnostics` 统一封装 authoritative-first 读取规则。'
  - 'Descriptor contract: degraded entries 在 descriptor 层暴露 badge 与 inspector，而不是把解析失败细节散落在组件里。'
requirements-completed: [DETE-02, DETE-04]
duration: continued session
completed: 2026-03-28
---

# Phase 02 Plan 04: Frontend Analysis Consumption Summary

**前端 detail/store 已停止继续发明 subtype 语义，当前 detail 流程会优先消费 Rust authoritative analysis，并在 degraded 场景下稳定展示 raw content 与 diagnostics**

## Performance

- **Duration:** continued session
- **Completed:** 2026-03-28
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- 新增 analysis-first presentation helper，把 subtype、metadata、status 和 diagnostics 的 authoritative-first 读取规则收敛到一个入口。
- 让 `buildPreviewDescriptor()` 改为优先消费 `entry.analysis`，并在 `fallback` 时输出 warning badge 与 diagnostics inspector。
- 收紧 `clipboardStore.resolveEntryPreview()`，删除 URL 推断 subtype 的旧逻辑，只保留 preview resolution 级别的降级能力。
- 让 `DetailView` 和 `DetailScene` 把 degraded 状态直接暴露给用户，同时保留 raw-first 主视图和 legacy-only 兼容路径。

## Files Created/Modified

- `src/types/clipboard.ts` - 对齐 Rust analysis metadata shape，修复 command / markdown / url analysis 类型契约。
- `src/lib/preview/entryPresentation.ts` - 增加 analysis-first helper 和 analysis-to-legacy presentation metadata 映射。
- `src/lib/preview/previewDescriptor.ts` - descriptor 改为 authoritative analysis first，并增加 fallback badge / diagnostics inspector。
- `src/stores/clipboardStore.ts` - 去掉 subtype inference，改成只根据 authoritative subtype 决定 preview resolution。
- `src/components/DetailView/DetailView.tsx` - detail 头部和 metadata 改为消费 analysis status / diagnostics。
- `src/components/DetailView/scene/DetailScene.tsx` - 渲染 descriptor badge，保证 degraded 状态可见。
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx` - renderer 优先使用 descriptor 归一化后的 metadata，而不是依赖旧 `entry.metadata`。
- `src/components/DetailView/DetailPreviewContract.test.tsx` - 覆盖 analysis-first override legacy 与 fallback diagnostics contract。
- `src/components/DetailView/DetailView.test.tsx` - 覆盖 degraded raw-first 行为与 diagnostics 可见性。
- `src/lib/preview/entryPresentation.test.ts` - 锁定 helper 的 authoritative-first 读取语义和 legacy fallback。

## Decisions Made

- analysis metadata 一旦存在，即便它映射不到 legacy `ContentMetadata` 结构，也不能再回退旧 metadata，以免被陈旧字段重新污染 UI。
- degraded 可见性不放在单一组件里兜底，而是让 descriptor 输出 badge 与 inspector，使后续 Phase 3 复用同一 contract。
- `resolveUrlPreview()` 仍可根据 URL 后缀猜测 `previewKind`，但这种猜测只服务渲染，不再影响 entry subtype authority。

## Deviations from Plan

None - plan executed within scope, but顺手补上了 `configStore.test.ts`，让 Wave 4 的前端 gate 能以真实文件名闭合。

## Issues Encountered

- `PreferencesModal` 现有测试基础为空，补新测试时需要额外 mock dialog/tabs 以避免 UI 容器细节阻塞 Phase 2 的功能回归。

## Next Phase Readiness

- `02-05` 现在可以安全触发 rebuild 并刷新 detail，因为前端已经能正确消费 `entry.analysis` companion row。
- Phase 3 可以直接复用这批 helper 与 descriptor contract，继续统一 list/detail/search 的 preview 语义。

## Self-Check

PASSED
