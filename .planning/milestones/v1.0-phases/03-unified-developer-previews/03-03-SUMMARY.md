---
phase: 03-unified-developer-previews
plan: 03
subsystem: detail-preview
tags: [frontend, detail, url, json, local-first, vitest]
requires:
  - phase: 03-01
    provides: semantic preview model as the shared detail/list semantic source
  - phase: 02-06
    provides: URL-first detail contract groundwork
  - phase: 02-07
    provides: raw-accessible detail layout and explicit JsonRenderer height contract
provides:
  - local-first URL detail card with protocol/host/path/query presentation
  - descriptor/store contract that no longer defaults to remote URL enrichment
  - preserved JSON Raw access, shared scroll column, and JsonRenderer explicit-height regression coverage
affects: [detail-preview, url-rendering, json-preview, preview-descriptor]
tech-stack:
  added: [src/components/DetailView/ContentRenderers/UrlCardRenderer.tsx]
  patterns:
    [
      local-first URL detail rendering,
      semantic-core driven preview descriptor,
      force-mounted alternate tabs for stable detail accessibility,
    ]
key-files:
  created: [src/components/DetailView/ContentRenderers/UrlCardRenderer.tsx]
  modified:
    [
      src/stores/clipboardStore.ts,
      src/stores/clipboardStore.test.ts,
      src/lib/preview/previewDescriptor.ts,
      src/components/DetailView/scene/PrimaryPreviewRenderer.tsx,
      src/components/DetailView/scene/AlternateViews.tsx,
      src/components/DetailView/DetailView.test.tsx,
    ]
key-decisions:
  - 'URL detail 默认主路径只依赖 raw URL 与 analysis-first `url_parts`，不再自动触发 `resolveUrlPreview()`。'
  - '`previewDescriptor` 改为直接消费 `buildSemanticPreviewModel()`，由 semantic core 决定 primary kind、raw 可达性与类型标签。'
  - '备用视图 tabs 采用 `forceMount`，保证普通 detail 布局下 Raw / URL structure 的内容稳定可达。'
patterns-established:
  - 'URL card pattern: 顶部简要 badge + 本地结构网格 + query 明细 + raw URL 同屏呈现。'
  - 'Semantic descriptor pattern: detail descriptor 与 list summary 共享同一 semantic source，而不是平行 subtype 推断。'
requirements-completed: [PREV-01, PREV-02, PREV-05]
duration: 9min
completed: 2026-03-29
---

# Phase 03 Plan 03: Local-First URL Detail Summary

**URL detail now defaults to a purely local structure card while preserving JSON/raw/detail layout contracts**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-29T00:22:51+08:00
- **Completed:** 2026-03-29T00:31:11+08:00
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- 新建 `UrlCardRenderer.tsx`，把 URL detail 的主视图收口成本地结构卡，稳定展示 Protocol / Host / Path / Query，并保留 Raw URL 区块。
- 更新 `PrimaryPreviewRenderer.tsx` 与 `AlternateViews.tsx`，让 `url_card` 在主视图和备用视图都走同一个本地 renderer；备用 tabs 使用 `forceMount`，保证普通 detail 布局下 Raw / URL 结构视图稳定可达。
- 重构 `previewDescriptor.ts`，改为直接消费 `buildSemanticPreviewModel()` 生成 detail contract，并继续保留 Raw、resolved media 与 `url-structure` alternate。
- 更新 `clipboardStore.ts`，停止在 `resolveEntryPreview()` 里对 URL 条目自动调用 `resolveUrlPreview()`；默认 detail 路径不再依赖远端 enrichment。
- 修正并通过 URL/local-card/JSON-layout 相关整组回归：`src/stores/clipboardStore.test.ts`、`src/components/DetailView/DetailView.test.tsx` 等 6 个测试文件共 42 个用例全部通过。

## Task Commits

Each TDD task was committed atomically:

1. **Task 1 RED: URL detail contract coverage** - `3dfb5c9` (`test`)
2. **Task 2 GREEN: local-first URL detail implementation** - `4d15ae3` (`feat`)

## Files Created/Modified

- `src/components/DetailView/ContentRenderers/UrlCardRenderer.tsx` - 新增本地 URL 结构卡 renderer。
- `src/stores/clipboardStore.ts` - 关闭 URL 条目的默认远端 preview 解析。
- `src/stores/clipboardStore.test.ts` - 对齐新的 URL store contract。
- `src/lib/preview/previewDescriptor.ts` - 用 semantic core 驱动 detail descriptor。
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx` - 主视图改接本地 URL card。
- `src/components/DetailView/scene/AlternateViews.tsx` - 备用视图支持 `url_card` 且强制挂载 tab 内容。
- `src/components/DetailView/DetailView.test.tsx` - 对齐新的 URL local card 与切换条目回归语义。

## Decisions Made

- URL 条目即使没有任何远端 resolved data，也必须能在 detail 中稳定展示协议、host、path 和 query。
- JSON structured view、Raw tab、共享滚动列和 JsonRenderer 显式高度壳被视为既有合同，URL 改造不得回退这些行为。
- 旧 `UrlRenderer` 不再参与默认 detail 主路径；Phase 03 的目标是 local-first，而不是再扩张远端 enrichment 分支。

## Deviations from Plan

执行代理在红测提交后没有返回 completion signal，并留下半完成的 implementation diff；orchestrator 接管当前工作树，补齐剩余实现与回归修正后完成计划。

## Issues Encountered

旧回归测试里仍保留了“URL 默认可返回远端 resolved data”的旧口径，以及对 URL 原文只出现一次的假设；在落地 local-first URL card 后一并修正为符合新 contract 的断言。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `03-05` 现在可以直接建立在新的 semantic descriptor 和 local-first detail 路径上推进 code/command workbench，不再受 URL 默认远端依赖干扰。
- 后续如果要做显式 URL enrichment，只需作为附加行为接入，而不是回到默认主路径。

## Self-Check

PASSED
