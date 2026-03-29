---
phase: 02-analysis-contracts-versioned-detection
plan: 06
subsystem: frontend-preview
tags: [frontend, detail-view, preview, url, vitest]
requires:
  - phase: 02-04
    provides: analysis-first detail descriptor and store preview resolution contract
provides:
  - URL-first detail preview contract that keeps URL entries on `url_card`
  - resolved JSON/text/media alternate views for URL detail scenes
  - regression coverage for URL descriptor, store preview resolution, and alternate view rendering
affects: [02-07, phase-03, detail-preview, preview-store]
tech-stack:
  added: []
  patterns:
    [
      URL-first primary preview semantics,
      resolved alternate media rendering with native elements,
      store-side resolved payloads without raw URL prefill,
    ]
key-files:
  created: []
  modified:
    [
      src/lib/preview/previewDescriptor.ts,
      src/stores/clipboardStore.ts,
      src/stores/clipboardStore.test.ts,
      src/components/DetailView/DetailPreviewContract.test.tsx,
      src/components/DetailView/DetailView.test.tsx,
      src/components/DetailView/scene/AlternateViews.tsx,
      src/components/DetailView/scene/AlternateViews.test.tsx,
    ]
key-decisions:
  - 'URL 条目无论远端 resolved `previewKind` 是 JSON、文本还是媒体，主视图都保持 `url_card`。'
  - 'URL 条目的 `ResolvedPreviewData` 只能承载真实远端 payload，不能再把原始 URL 字符串预填成 `textContent`。'
  - '备用视图里的 image/audio/video 使用原生媒体元素渲染，不复用统一文本 fallback。'
patterns-established:
  - 'Descriptor contract: URL resolved JSON/text/media 一律作为 `resolved-*` alternate views 暴露，URL 结构卡片继续保留。'
  - 'Store contract: 仅普通文本或 JSON 条目预填 raw text；URL 条目等待 `resolveUrlPreview()` 的真实结果。'
requirements-completed: [DETE-02, DETE-04]
duration: 10min
completed: 2026-03-28
---

# Phase 02 Plan 06: URL-First Detail Preview Summary

**URL 条目的 detail 预览已恢复为 URL-first 合同，远端 JSON/文本/媒体只作为备用视图出现，并且这些备用视图现在可以真正渲染**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-28T05:00:00Z
- **Completed:** 2026-03-28T05:10:23Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- 把 `buildPreviewDescriptor()` 的 URL 主视图语义收口回 `url_card`，不再被 `previewKind=json`、文本或媒体结果抢占。
- 调整 `resolveEntryPreview()`，让 URL 条目只携带真实 resolved payload，彻底移除 raw URL 到 `textContent` 的错误桥接。
- 为 URL resolved alternate views 补上图片、音频和视频的原生渲染，保证 descriptor 暴露出来的 `resolved-*` key 能在界面上真实使用。
- 补齐 descriptor、store、detail scene 和 AlternateViews 的 URL 回归测试，锁住 UAT Test 2 的回归面。

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: URL-first descriptor/store contract** - `6e22f91` (test)
2. **Task 1 GREEN: URL-first descriptor/store contract** - `186c518` (fix)
3. **Task 2 RED: AlternateViews resolved media rendering** - `419a89a` (test)
4. **Task 2 GREEN: AlternateViews resolved media rendering** - `01f20c4` (feat)

## Files Created/Modified

- `src/lib/preview/previewDescriptor.ts` - 固定 URL 主视图为 `url_card`，并为远端 resolved 内容生成 `resolved-*` alternate keys。
- `src/stores/clipboardStore.ts` - 移除 URL 条目的 raw text 预填，只保留远端解析返回的真实 preview payload。
- `src/stores/clipboardStore.test.ts` - 新增 URL resolved data 不预填 raw URL 的 store 回归测试。
- `src/components/DetailView/DetailPreviewContract.test.tsx` - 把 URL 契约测试改成 URL-first，并补远端文本和媒体 alternate views 回归。
- `src/components/DetailView/DetailView.test.tsx` - 同步更新 detail scene 的竞态测试，匹配新的 URL-first 主视图行为。
- `src/components/DetailView/scene/AlternateViews.tsx` - 为 image/audio/video alternate views 增加原生媒体渲染。
- `src/components/DetailView/scene/AlternateViews.test.tsx` - 增加 JSON、image、audio、video alternate view 的组件级回归测试。

## Decisions Made

- URL 条目的语义权威来自“条目本身是 URL”，不是远端资源 MIME 或 `previewKind` 推断结果。
- resolved JSON/text/media 在当前阶段只进入备用视图，不改写主视图，也不重建 PrimaryPreviewRenderer 的主预览分发逻辑。
- `AlternateViews` 只做最小原生媒体渲染，避免在 Phase 02 提前扩张到更复杂的媒体状态管理。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Regression] 限定 `resolved-*` 媒体备用视图只作用于 URL 条目**

- **Found during:** Task 1 GREEN
- **Issue:** 初版实现把 `resolved-image` 之类的 alternate keys 应用于所有 descriptor，导致 Base64 图片契约被意外污染。
- **Fix:** 把 image/audio/video alternate views 收回到 `subType === 'url'` 分支，并同步更新 `DetailView.test.tsx` 中仍锁旧 URL 图片主视图的断言。
- **Files modified:** `src/lib/preview/previewDescriptor.ts`, `src/components/DetailView/DetailView.test.tsx`
- **Verification:** `pnpm exec vitest run src/components/DetailView/DetailPreviewContract.test.tsx src/stores/clipboardStore.test.ts src/components/DetailView/DetailView.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx`
- **Committed in:** `186c518` (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 regression)
**Impact on plan:** Auto-fix 直接消除了本次实现引入的 descriptor 回归，没有扩展范围。

## Issues Encountered

- `pnpm test -- ...` 在当前脚本下会把 `--` 作为额外模式传给 Vitest，容易把无关测试也卷进来；执行期改用 `pnpm exec vitest run ...` 做精确定向验证。
- 并行 `git add` 会触发 `.git/index.lock` 竞争，后续 staging 改为顺序处理。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- URL 条目现在可以稳定先展示 URL 卡片，再通过备用视图查看 resolved JSON、文本或媒体内容。
- Phase 03 可以直接复用这份 URL-first contract，把主预览和备用视图体系继续统一到更完整的 preview 重构里。

## Self-Check

PASSED

---

_Phase: 02-analysis-contracts-versioned-detection_
_Completed: 2026-03-28_
