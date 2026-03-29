# Phase 5: Rebuild Safety & Release Gates - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 5 负责把已有 history、analysis 和 retrieval 升级链路收口为“可重建但不丢数据”的本地维护能力，并在打包前增加自动化门禁，降低桌面发布回归风险。

本 phase 聚焦两件事：

1. 让 analysis rebuild 同时覆盖 search documents/index，使现有历史在版本升级后仍可用。
2. 在 release/test-build 工作流里把 `pnpm test`、`pnpm build`、`cargo test` 作为打包前 gate。

不在本 phase 扩展云备份、跨设备迁移、远端数据库修复、在线回滚或自动发布审批流。

</domain>

<decisions>
## Implementation Decisions

### Rebuild Authority

- **D-01:** 不新开独立的 search rebuild command，而是把 search document rebuild 串到既有 `rebuild_entry_analysis` 内。
- **D-02:** rebuild 必须是 non-destructive、可重跑、可在已有历史上反复执行的维护动作，绝不能清空 `clipboard_entries`。
- **D-03:** rebuild 结果需要同时反馈 analysis 与 search 两部分计数，便于 UI 向用户解释做了什么。

### Release Gates

- **D-04:** package workflow 在真正打包前必须先过 `pnpm test`、`pnpm build`、`cargo test`，否则不允许直接进入 Tauri build。
- **D-05:** 同样的 gate 要落到 test-build 与 release 两条工作流上，避免“手动验证过、正式发布没跑”的偏差。
- **D-06:** 当前 release gate 只覆盖本地桌面关键路径，不把 website 或额外平台流程混入本 phase。

### UI Surface

- **D-07:** rebuild 的用户入口继续复用 Preferences 里的现有 rebuild affordance，不新增独立运维面板。
- **D-08:** UI 文案要显式呈现 `search_reindexed` / `search_failed`，让用户知道检索索引是否已同步刷新。

### the agent's Discretion

- rebuild 结果对象的具体字段名和 UI 展示排布。
- CI step 的具体命名、排列顺序和跨平台复用方式，只要保证 gate 在打包前执行。

</decisions>

<specifics>
## Specific Ideas

- “analysis 或 search 升级后，已有历史也要能安全重建，不要让我清数据。”
- “发版前至少先跑测试和构建，不要直接打包才发现回归。”
- “保持本地客户端边界，不做远端升级服务。”

</specifics>

<canonical_refs>

## Canonical References

### Project Scope And Requirements

- `.planning/ROADMAP.md` — Phase 5 的目标与 `RELY-01`、`RELY-02`。
- `.planning/REQUIREMENTS.md` — reliability & maintenance requirement 定义。
- `.planning/STATE.md` — 记录当前 phase 已完成到 retrieval，下一步是 rebuild safety 与 release gates。

### Upstream Phase Contracts

- `.planning/phases/02-analysis-contracts-versioned-detection/02-VERIFICATION.md` — analysis rebuild 已经是现有能力基础。
- `.planning/phases/04-search-quality-retrieval/04-RESEARCH.md` — search document 设计与 retrieval authority 已确定。
- `.planning/phases/04-search-quality-retrieval/04-01-SUMMARY.md` — 当前 retrieval/search doc 实现已经落地，Phase 5 只补生命周期保障。

</canonical_refs>

<code_context>

## Existing Code Insights

- `src-tauri/src/analysis/rebuild.rs`: 当前已有 authoritative analysis rebuild 入口，是串联 search rebuild 的最佳位置。
- `src-tauri/src/retrieval/mod.rs`: 已持有 search document upsert 与 rebuild 能力，可直接被 rebuild phase 复用。
- `src/components/Preferences/PreferencesModal.tsx`: 已有 rebuild 按钮和结果摘要，适合作为 search rebuild 反馈的既有 UI 壳。
- `.github/workflows/test-build.yml` 与 `.github/workflows/release.yml`: 当前都执行 Tauri 打包，是加入 gate 的唯一必要位置。

</code_context>

<deferred>
## Deferred Ideas

- 远端备份或导出导入向导
- 自动 release approval / canary pipeline
- 多版本数据库 migration dashboard

</deferred>

---

_Phase: 05-rebuild-safety-release-gates_  
_Context gathered: 2026-03-29_
