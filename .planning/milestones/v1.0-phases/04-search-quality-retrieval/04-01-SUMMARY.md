---
phase: 04-search-quality-retrieval
plan: 01
subsystem: retrieval
tags: [rust, tauri, sqlite, react, retrieval, search, filters]
requires:
  - phase: 03-unified-developer-previews
    provides: shared semantic preview contract for list/detail/retrieval summaries
provides:
  - Rust authoritative clipboard retrieval query path
  - retrieval match/snippet contract shared with the frontend
  - thin-client search/filter UI for type, source app, favorites, and recency
affects: [clipboard-list, search-bar, retrieval, rebuild, release-gates]
tech-stack:
  added: []
  patterns:
    [
      Rust authoritative retrieval boundary,
      search-document + FTS retrieval ranking,
      thin-client frontend query adapter,
      retrieval-density list rows with explainable matches,
    ]
key-files:
  created:
    [
      src-tauri/src/retrieval/mod.rs,
      src-tauri/src/retrieval_tests.rs,
      src/components/ClipboardList/RetrievalFilterBar.tsx,
    ]
  modified:
    [
      src-tauri/src/state.rs,
      src-tauri/src/commands.rs,
      src-tauri/src/lib.rs,
      src/stores/clipboardStore.ts,
      src/types/clipboard.ts,
      src/components/SearchBar/SearchBar.tsx,
      src/components/ClipboardList/ClipboardList.tsx,
      src/components/ClipboardList/ClipboardItem.tsx,
      src/components/ClipboardList/EmptyState.tsx,
    ]
key-decisions:
  - 'Rust + SQLite 成为检索唯一权威层；React 只保留 query 输入与结果渲染，不再做本地二次 includes/filter。'
  - '继续沿用 `entry_search_documents + entry_search_fts`，不引入外部搜索服务或远端依赖。'
  - "retrieval 结果继续复用 `buildPreviewSummary(entry, 'retrieval')`，而不是退回 subtype-specific 结果卡。"
  - '带点 token 的 FTS 查询必须先转义，确保 `api.example`、`deploy.service` 这类真实 developer token 可搜索。'
patterns-established:
  - 'Retrieval authority pattern: backend 返回的结果集就是最终事实，前端 store 不再叠加平行过滤逻辑。'
  - 'Explainable retrieval pattern: 每条命中都附带 `match_kind`、label 与 snippet，帮助用户快速判断是否是目标条目。'
requirements-completed: [RETR-01, RETR-02, RETR-03, RETR-04, RETR-05]
duration: session
completed: 2026-03-29
---

# Phase 04 Plan 01: Search Quality & Retrieval Summary

**本地检索已经收口为 Rust authoritative retrieval path，并把类型筛选、来源应用、收藏、时间窗口、snippet 与命中原因一起闭合到同一条查询链路里**

## Performance

- **Duration:** session
- **Started:** 2026-03-29
- **Completed:** 2026-03-29
- **Tasks:** 2
- **Files modified:** 12 core files

## Accomplishments

- 新增 `src-tauri/src/retrieval/mod.rs` 与 `src-tauri/src/retrieval_tests.rs`，把检索主链路统一到 `search_clipboard_history`，并为 URL host、JSON key path、颜色值、来源应用列表和组合过滤补齐回归测试。
- 在 Rust 侧建立 `ClipboardHistoryQuery` 与 `ClipboardRetrievalMatch` contract，让 `selected_type`、`source_app`、`favorites_only`、`recency_days`、`snippet` 和 `match_kind` 都成为一等 retrieval 字段。
- 修复 FTS 对带点 token 的语法问题，确保 `api.example`、`deploy.service` 等开发者常见查询不再因为解析错误而丢失结果。
- 把 `clipboardStore` 收口为 thin client：搜索、筛选、分页和来源应用选项都走 backend，不再在前端对结果集做 includes/filter 的二次真相。
- 新增 `RetrievalFilterBar`，并在 `ClipboardItem` / `ClipboardList` / `EmptyState` 中落地 retrieval density、snippet、命中原因、no-results 和 empty-history 的区分展示。

## Task Commits

None recorded in this workspace pass. 该实现是在已有脏工作树上持续整合完成的，未额外拆成原子提交，以避免覆盖用户的并行改动。

## Files Created/Modified

- `src-tauri/src/retrieval/mod.rs` - authoritative retrieval query、search document upsert、ranking、snippet 与 match metadata。
- `src-tauri/src/retrieval_tests.rs` - 锁定 structured token、模糊片段、组合过滤和来源应用列表行为。
- `src-tauri/src/state.rs` - 暴露 retrieval query 与来源应用读取入口。
- `src-tauri/src/commands.rs` - 增加 `search_clipboard_history`、`list_clipboard_source_apps` 命令。
- `src-tauri/src/lib.rs` - 注册 retrieval command 与测试模块。
- `src/stores/clipboardStore.ts` - 查询 contract 适配、filter state、来源应用选项加载和 retrieval active 判定。
- `src/types/clipboard.ts` - 新增 retrieval match DTO。
- `src/components/SearchBar/SearchBar.tsx` - 承接 retrieval active 与搜索提示文案。
- `src/components/ClipboardList/RetrievalFilterBar.tsx` - 来源应用、收藏和时间窗口过滤条。
- `src/components/ClipboardList/ClipboardList.tsx` - retrieval row height、loading、分页和空状态调度。
- `src/components/ClipboardList/ClipboardItem.tsx` - retrieval snippet 与命中原因展示。
- `src/components/ClipboardList/EmptyState.tsx` - 区分 empty history 与 no results。

## Decisions Made

- 保持 SQLite companion search docs 作为当前 brownfield 方案，不为 Phase 04 扩张到新搜索服务。
- retrieval 结果使用 Phase 03 的 `buildPreviewSummary(entry, 'retrieval')`，把差异控制在信息密度和命中解释层，不新造 preview 语义。
- 来源应用列表由后端提供最近活跃去重集合，避免前端靠当前页数据推断 filter 候选。
- dot token 转义被视为 Phase 04 的必须项，因为 URL host 和 JSON key path 是开发者检索最常见的真实输入。

## Deviations from Plan

None. Phase 04 的实现范围控制在 authoritative retrieval、thin-client store/UI 和回归测试，没有扩张到外部检索服务或语义向量能力。

## Issues Encountered

- 现有 retrieval WIP 最初在事务执行写法上会导致 Rust 编译失败，最终通过收敛写法并补回归测试闭合。
- 前端原有 `getFilteredEntries()`、`selectedType` 和 `searchTerm` 路径存在 split-brain，需要先把 query adapter 明确化，才能让 UI 不再重复裁剪结果。

## User Setup Required

None - retrieval 仍完全运行在本地桌面客户端内，无新增服务配置。

## Next Phase Readiness

- Phase 05 可以直接复用当前 search document 设计，把 rebuild 与 release gates 接到既有链路，不需要再发明单独的检索重建命令。
- retrieval 结果 contract 已稳定，后续只需补生命周期保障和打包验证即可。

## Self-Check

PASSED
