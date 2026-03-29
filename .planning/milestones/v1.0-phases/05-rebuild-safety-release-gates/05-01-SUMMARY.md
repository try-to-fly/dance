---
phase: 05-rebuild-safety-release-gates
plan: 01
subsystem: rebuild-and-release
tags: [rust, tauri, rebuild, ci, github-actions, release]
requires:
  - phase: 02-analysis-contracts-versioned-detection
    provides: existing analysis rebuild command and stale-entry selection
  - phase: 04-search-quality-retrieval
    provides: search document rebuild capability and authoritative retrieval index
provides:
  - one rebuild path that refreshes both analysis and search documents
  - rebuild result feedback for search reindex counts in Preferences
  - pre-package validation gates in test-build and release workflows
affects: [preferences, retrieval, release, test-build]
tech-stack:
  added: []
  patterns:
    [one rebuild entry point, non-destructive history maintenance, pre-package validation gate]
key-files:
  created: []
  modified:
    [
      src-tauri/src/analysis/rebuild.rs,
      src/components/Preferences/PreferencesModal.tsx,
      src/components/Preferences/PreferencesModal.test.tsx,
      .github/workflows/test-build.yml,
      .github/workflows/release.yml,
    ]
key-decisions:
  - '不新开独立 search rebuild command，而是把 search reindex 串进现有 `rebuild_entry_analysis`。'
  - 'rebuild 继续保持 non-destructive，只刷新 companion analysis/search 数据，不触碰 raw history。'
  - 'CI gate 直接加到现有打包 workflow 中，不另起平行验证流程。'
patterns-established:
  - 'Maintenance path pattern: 用户只触发一个 rebuild 行为，backend 负责 analysis 与 search 的一致性刷新。'
  - 'Release safety pattern: 所有打包 job 先过 `pnpm test`、`pnpm build`、`cargo test`，再进入 Tauri build。'
requirements-completed: [RELY-01, RELY-02]
duration: session
completed: 2026-03-29
---

# Phase 05 Plan 01: Rebuild Safety & Release Gates Summary

**已有历史现在可以通过同一个 rebuild 入口同时刷新 analysis 与检索索引，正式打包路径也增加了统一的自动化门禁**

## Performance

- **Duration:** session
- **Started:** 2026-03-29
- **Completed:** 2026-03-29
- **Tasks:** 2
- **Files modified:** 5 core files

## Accomplishments

- 在 `src-tauri/src/analysis/rebuild.rs` 中把 `rebuild_search_documents(pool)` 串进既有 analysis rebuild 主链路，并把结果扩展为 `search_reindexed` 与 `search_failed`。
- `PreferencesModal` 及其测试现在会展示 rebuild 同步刷新的 search 统计，让用户明确知道检索索引是否被重建。
- `.github/workflows/test-build.yml` 和 `.github/workflows/release.yml` 都在任何 Tauri build 前增加了 `pnpm test`、`pnpm build`、`cargo test` gate。
- 保持 Phase 05 改动范围聚焦在维护与发布安全，没有新增平行命令、独立运维页面或额外发布流程。

## Task Commits

None recorded in this workspace pass. 这些改动在现有脏工作树上直接整合，避免为了生成文档化提交而打断用户当前上下文。

## Files Created/Modified

- `src-tauri/src/analysis/rebuild.rs` - 把 search rebuild 串进 analysis rebuild 结果。
- `src/components/Preferences/PreferencesModal.tsx` - 展示 search reindex 统计。
- `src/components/Preferences/PreferencesModal.test.tsx` - 锁定新的 rebuild 反馈字段。
- `.github/workflows/test-build.yml` - 在 Windows/macOS test build 前增加 frontend/backend validation gate。
- `.github/workflows/release.yml` - 在正式 release matrix job 的打包前增加同样 gate。

## Decisions Made

- 复用现有 rebuild command，比新增 search rebuild command 更安全，也能避免 UI 入口分裂。
- rebuild 结果要显式返回 search 统计，否则用户无法判断 retrieval stale 是否已被清理。
- workflow gate 直接进现有打包 job，比新建单独 workflow 更不容易出现“正式发版没跑验证”的偏差。

## Deviations from Plan

None. Phase 05 只补闭环，不扩张到 release 审批、canary、导出导入或远端升级服务。

## Issues Encountered

- rebuild 结果对象新增字段后，需要同步更新 Preferences UI 与测试，否则前后端 contract 会立即分叉。
- GitHub Actions 有多条平台分支，gate 必须在每条分支的 Tauri build 前都补齐，不能只改一处示意路径。

## User Setup Required

None beyond the existing CI secrets already required for signed builds.

## Next Phase Readiness

- 当前 milestone 的代码层面已经闭合，后续只剩 milestone audit / complete / cleanup 的规划生命周期动作。

## Self-Check

PASSED
