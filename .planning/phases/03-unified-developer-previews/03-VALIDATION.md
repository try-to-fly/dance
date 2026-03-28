---
phase: 03
slug: unified-developer-previews
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property               | Value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Framework**          | Vitest 4.1.2 + Testing Library                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| **Config file**        | `vitest.config.ts`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| **Quick run command**  | `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts src/lib/preview/previewSummary.test.ts src/components/ClipboardList/ClipboardItem.test.tsx src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/DetailView.test.tsx src/components/DetailView/scene/PrimaryPreviewRenderer.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/ColorRenderer.test.tsx src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx src/stores/clipboardStore.test.ts` |
| **Full suite command** | `pnpm test`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| **Estimated runtime**  | ~30 seconds                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |

---

## Sampling Rate

- **After every task commit:** Run `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx`
- **After every plan wave:** Run `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts src/lib/preview/previewSummary.test.ts src/components/ClipboardList/ClipboardItem.test.tsx src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/DetailView.test.tsx src/components/DetailView/scene/PrimaryPreviewRenderer.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/ColorRenderer.test.tsx src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx src/stores/clipboardStore.test.ts`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID  | Plan | Wave | Requirement               | Test Type            | Automated Command                                                                                                                                                                                                                                                                                                                                        | File Exists | Status     |
| -------- | ---- | ---- | ------------------------- | -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ---------- |
| 03-01-01 | 01   | 1    | PREV-05                   | contract             | `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts`                                                                                                                                                                                                                                                                                         | ✅          | ⬜ pending |
| 03-01-02 | 01   | 1    | PREV-05                   | contract             | `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts src/lib/preview/previewSummary.test.ts`                                                                                                                                                                                                                                                  | ❌ W0       | ⬜ pending |
| 03-02-01 | 02   | 2    | PREV-05                   | component            | `pnpm exec vitest run src/components/ClipboardList/ClipboardItem.test.tsx`                                                                                                                                                                                                                                                                               | ❌ W0       | ⬜ pending |
| 03-02-02 | 02   | 2    | PREV-05                   | component            | `pnpm exec vitest run src/components/ClipboardList/ClipboardItem.test.tsx`                                                                                                                                                                                                                                                                               | ❌ W0       | ⬜ pending |
| 03-03-01 | 03   | 2    | PREV-01, PREV-02, PREV-05 | contract + component | `pnpm exec vitest run src/stores/clipboardStore.test.ts src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/scene/PrimaryPreviewRenderer.test.tsx src/components/DetailView/DetailView.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx` | ❌ W0       | ⬜ pending |
| 03-03-02 | 03   | 2    | PREV-01, PREV-02, PREV-05 | contract + component | `pnpm exec vitest run src/stores/clipboardStore.test.ts src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/scene/PrimaryPreviewRenderer.test.tsx src/components/DetailView/DetailView.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx` | ❌ W0       | ⬜ pending |
| 03-04-01 | 04   | 2    | PREV-03, PREV-05          | component            | `pnpm exec vitest run src/components/DetailView/ContentRenderers/ColorRenderer.test.tsx`                                                                                                                                                                                                                                                                 | ❌ W0       | ⬜ pending |
| 03-04-02 | 04   | 2    | PREV-03, PREV-05          | component            | `pnpm exec vitest run src/components/DetailView/ContentRenderers/ColorRenderer.test.tsx`                                                                                                                                                                                                                                                                 | ❌ W0       | ⬜ pending |
| 03-05-01 | 05   | 3    | PREV-04, PREV-05          | component            | `pnpm exec vitest run src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx src/components/DetailView/DetailView.test.tsx`                                                                                                                                                                                                             | ❌ W0       | ⬜ pending |
| 03-05-02 | 05   | 3    | PREV-04, PREV-05          | component            | `pnpm exec vitest run src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx src/components/DetailView/DetailView.test.tsx`                                                                                                                                                                                                             | ❌ W0       | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave 0 Requirements

- [ ] `src/lib/preview/previewSummary.test.ts` — 锁定 `density: 'list' | 'retrieval'` 的 shared summary contract，并覆盖 PREV-05
- [ ] `src/components/ClipboardList/ClipboardItem.test.tsx` — 锁定两层固定紧凑结构、长内容截断和核心类型摘要显示
- [ ] `src/components/DetailView/scene/PrimaryPreviewRenderer.test.tsx` 或 `src/components/DetailView/ContentRenderers/UrlCardRenderer.test.tsx` — 锁定 URL 结构卡与 no-default-remote-dependency contract
- [ ] `src/components/DetailView/ContentRenderers/ColorRenderer.test.tsx` — 锁定 swatch、格式互转与复制行为
- [ ] `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` — 锁定本地工作台编辑、切条目重置、复制当前编辑内容
- [ ] `src/components/DetailView/DetailView.test.tsx` / `src/components/DetailView/scene/AlternateViews.test.tsx` — 锁定 non-immersive detail 下 Raw tab、共享滚动列和显式高度合同未回退

---

## Manual-Only Verifications

| Behavior                                                                           | Requirement                                 | Why Manual                                                                   | Test Instructions                                                                                                                                                                                                      |
| ---------------------------------------------------------------------------------- | ------------------------------------------- | ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 运行桌面应用后依次复制 JSON、URL、颜色、命令文本，检查列表摘要与详情主视图语义一致 | PREV-01, PREV-02, PREV-03, PREV-04, PREV-05 | 真正的剪贴板监听、桌面布局和 Monaco/scroll 交互只能在运行中的 Tauri 应用确认 | 启动应用，复制上述样例，确认列表 headline/summary 与 detail 主视图类型一致；JSON 仍可切 Raw；URL 默认显示 protocol/host/path/query；颜色显示 swatch 与多种格式；code/command 默认进入本地 workbench 且复制当前编辑内容 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
