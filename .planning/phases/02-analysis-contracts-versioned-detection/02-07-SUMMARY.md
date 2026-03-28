---
phase: 02-analysis-contracts-versioned-detection
plan: 07
subsystem: frontend-preview
tags: [frontend, detail-view, json, layout, vitest]
requires:
  - phase: 02-04
    provides: analysis-first detail descriptor and authoritative subtype consumption
  - phase: 02-06
    provides: URL-first alternate view rendering paths and alternate view contracts
provides:
  - JSON raw-only alternate views remain reachable in non-immersive detail layouts
  - non-immersive detail left column scrolls primary preview and alternate views together
  - JsonRenderer tree, invalid, and Monaco code states share one explicit-height content shell
affects: [phase-03, detail-preview, json-renderer, uat-test-3]
tech-stack:
  added: []
  patterns:
    [
      raw-only alternate visibility contract,
      shared non-immersive detail scroll container,
      explicit-height Monaco shell,
    ]
key-files:
  created: []
  modified:
    [
      src/components/DetailView/scene/DetailScene.tsx,
      src/components/DetailView/scene/AlternateViews.tsx,
      src/components/DetailView/scene/AlternateViews.test.tsx,
      src/components/DetailView/ContentRenderers/JsonRenderer.tsx,
      src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx,
      src/components/DetailView/DetailView.test.tsx,
    ]
key-decisions:
  - 'raw-only alternate views 只在 image/video/audio 这类沉浸式主预览下继续隐藏，JSON 等普通 detail 布局必须保留 Raw 入口。'
  - 'JsonRenderer 的 tree/code/invalid 三个分支共享同一个 `clamp(360px, 52vh, 920px)` 高度合同，Monaco 直接吃这个显式高度值。'
patterns-established:
  - 'DetailScene pattern: 非沉浸式详情左列使用单一 `overflow-y-auto` 容器承载主预览和备用视图。'
  - 'JsonRenderer pattern: 显式高度外壳 + 分支内滚动区，避免 Monaco 和长内容依赖父级 100% 高度。'
requirements-completed: [DETE-04]
duration: 6min
completed: 2026-03-28
---

# Phase 02 Plan 07: JSON Detail Access And Height Contract Summary

**JSON detail 重新暴露 Raw 入口，并用共享滚动列与显式高度 renderer 稳定展示长内容和代码视图**

## Performance

- **Duration:** 6min
- **Started:** 2026-03-28T05:17:19Z
- **Completed:** 2026-03-28T05:23:07Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- 恢复 JSON raw-only 备用视图入口，保证 detail 默认显示结构化 JSON 的同时仍可切回 Raw 原文。
- 把非沉浸式 detail 左列改为共享滚动容器，长主预览和下方备用视图不再被共同祖先裁掉。
- 给 JsonRenderer 的树视图、无效 JSON 提示和 Monaco 代码视图建立统一显式高度合同，避免代码视图空白。

## Task Commits

Each TDD task was committed atomically:

1. **Task 1 RED: JSON raw-only detail access tests** - `0f1c121` (`test`)
2. **Task 1 GREEN: restore raw alternate access and shared left-column scroll** - `6de23fc` (`fix`)
3. **Task 2 RED: JsonRenderer height contract tests** - `4692a0e` (`test`)
4. **Task 2 GREEN: stabilize JsonRenderer explicit-height layout** - `511ba94` (`fix`)

## Files Created/Modified

- `src/components/DetailView/scene/DetailScene.tsx` - 收紧 raw-only 隐藏规则到沉浸式媒体，并让非沉浸式左列统一滚动。
- `src/components/DetailView/scene/AlternateViews.tsx` - 删除单个 raw 备用视图的直接隐藏逻辑，保留单卡片渲染路径。
- `src/components/DetailView/scene/AlternateViews.test.tsx` - 把 raw-only 行为改成可见回归测试。
- `src/components/DetailView/DetailView.test.tsx` - 锁定 JSON detail 的 Raw 入口和左列滚动容器合同。
- `src/components/DetailView/ContentRenderers/JsonRenderer.tsx` - 为 tree/code/invalid 三个分支提供统一显式高度壳，并让 Monaco 直接使用显式高度。
- `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx` - 锁定 Monaco 高度、树视图滚动区和 invalid JSON 可见性回归。

## Decisions Made

- raw-only alternate views 不是普遍噪音，而是 JSON detail 的必要回退入口；只在沉浸式媒体主视图中继续隐藏，避免重复展示。
- 代码视图空白的根因是高度合同不成立，因此修复落在 renderer 布局层，而不是去补 Monaco 特例逻辑或前端直写剪贴板分支。

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 02 的 UAT Test 3 两个剩余前端回归已闭合，JSON detail 现在同时满足 Raw 可达性与长内容可见性。
- Phase 3 可以直接复用这套 raw-only 与显式高度合同，继续统一开发者预览的 list/detail 语义。

## Self-Check

PASSED
