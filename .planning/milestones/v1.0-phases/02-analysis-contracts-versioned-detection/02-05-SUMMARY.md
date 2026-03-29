---
phase: 02-analysis-contracts-versioned-detection
plan: 05
subsystem: analysis-rebuild
tags: [rust, tauri, preferences, rebuild, history, analysis]
requires:
  - phase: 02-02
    provides: `entry_analysis` companion persistence and stale-selector primitives
  - phase: 02-03
    provides: stable `TextAnalysisService` and capture-time authoritative analysis contract
  - phase: 02-04
    provides: frontend analysis-first consumption path for rebuilt history rows
provides:
  - batch rebuild service for existing text history
  - Tauri command exposing local analysis rebuild
  - Preferences system/cache trigger with refresh side effects and tests
affects: [phase-05, preferences, history-refresh, detail-preview]
tech-stack:
  added: []
  patterns:
    [
      version-aware companion-row rebuild,
      preferences-triggered local maintenance action,
      rebuild-result summary with UI refresh side effects,
    ]
key-files:
  created:
    [
      src-tauri/src/analysis/rebuild.rs,
      src/components/Preferences/PreferencesModal.test.tsx,
    ]
  modified:
    [
      src-tauri/src/analysis/mod.rs,
      src-tauri/src/state.rs,
      src-tauri/src/commands.rs,
      src-tauri/src/lib.rs,
      src-tauri/src/integration_tests.rs,
      src/components/Preferences/PreferencesModal.tsx,
    ]
key-decisions:
  - 'rebuild 只更新 `entry_analysis`，raw `clipboard_entries` 完全保持原值不动。'
  - 'non-text 或缺少 text payload 的 stale rows 只计入 skipped，不参与 analysis 重算，也不会阻塞 text rebuild。'
  - 'Preferences 入口保持最小：一个按钮、一段结果摘要、一次历史和缓存刷新。'
patterns-established:
  - 'Reanalysis service: `EntryAnalysisRebuilder` 复用 stale selector 与 `TextAnalysisService`，批量补齐或升级 companion row。'
  - 'Frontend closure: rebuild 成功后立即 `invalidatePreview()`、`fetchHistory()`、`loadCacheStatistics()`，保证 UI 不读旧缓存。'
requirements-completed: [DETE-03, DETE-04]
duration: continued session
completed: 2026-03-28
---

# Phase 02 Plan 05: History Reanalysis Summary

**已有历史现在可以在本地批量重建 analysis，无需重新复制内容；Preferences 里也已经有最小可见的 rebuild 入口和刷新闭环**

## Performance

- **Duration:** continued session
- **Completed:** 2026-03-28
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- 新增 `EntryAnalysisRebuilder`，按 version-aware 规则选择 stale text rows，重算 snapshot 后 UPSERT 到 `entry_analysis`。
- 在 `AppState` / Tauri command / invoke handler 链路中暴露 `rebuild_entry_analysis(batch_size)`，把 rebuild 能力开放给前端。
- 在 Preferences 的 system/cache 区域加入最小 rebuild action，并在成功后刷新历史、preview cache 和缓存统计。
- 补齐 Rust integration tests 与前端 modal tests，锁住 existing history update、non-text skip、fresh-row no-op 和 UI refresh side effects。

## Files Created/Modified

- `src-tauri/src/analysis/rebuild.rs` - 封装 batch rebuild service、stale non-text skip 统计和 batch size 处理。
- `src-tauri/src/analysis/mod.rs` - 导出 rebuild service/result，接入 analysis 模块公共接口。
- `src-tauri/src/state.rs` - 新增 `rebuild_entry_analysis()` 状态入口。
- `src-tauri/src/commands.rs` - 暴露 `#[tauri::command] rebuild_entry_analysis`。
- `src-tauri/src/lib.rs` - 注册 Tauri invoke handler。
- `src-tauri/src/integration_tests.rs` - 覆盖 existing history rebuild、non-text skip、fresh-row skip 三个集成回归。
- `src/components/Preferences/PreferencesModal.tsx` - 增加 rebuild 按钮、结果摘要与刷新副作用。
- `src/components/Preferences/PreferencesModal.test.tsx` - 验证 invoke、success summary、error feedback 和 refresh side effects。

## Decisions Made

- rebuild 不去扫描和改写 raw entry 字段，所有升级都落在 companion analysis 层，以避免 favorites / copy_count / file path 被误伤。
- skipped 统计显式把 non-text stale rows 算进去，帮助用户理解为什么并非所有历史都会被重新分析。
- rebuild batch size 暂时固定为最小命令参数，不新增设置项，避免在 Phase 2 过早引入 maintenance 配置面板。

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

- 并行跑 Rust 定向测试时仍会看到 build directory 锁等待，但三个重建相关测试都通过，没有产生真实失败。
- shell 包装层仍会偶发打印 `(eval): parse error near 'end'` 的无害噪音，命令退出码和测试结果均正常。

## Next Phase Readiness

- Phase 2 的 DETE-03 已闭环，下一步可以进入 verify-work 或直接开始 Phase 3 的 preview 统一规划。
- Phase 5 后续如果要扩展 rebuild 目标，只需要在 `EntryAnalysisRebuilder` 上继续加 selector 和 result DTO，不必改 UI 流程。

## Self-Check

PASSED
