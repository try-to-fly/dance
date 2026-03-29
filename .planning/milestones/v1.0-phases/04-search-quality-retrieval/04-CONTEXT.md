# Phase 4: Search Quality & Retrieval - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 4 交付本地检索能力，让用户能在较大历史里快速找到目标内容，并通过类型筛选、模糊匹配、结构化 token 和稳定排序提升命中率与可扫描性。

本 phase 聚焦本地搜索 contract、查询路径、索引或归一化 token、筛选模型和结果呈现，不扩展到云同步、多设备、移动端、团队协作、分享或语义向量检索。

</domain>

<decisions>
## Implementation Decisions

### Retrieval Authority

- **D-01:** 检索结果继续以 Rust + SQLite 为权威源，前端不再承担“完整结果集 + 本地 includes 过滤”的主搜索职责。
- **D-02:** 搜索 contract 必须直接消费 Phase 2 的 authoritative analysis 与 Phase 3 的 preview summary contract，而不是重新解析原始文本。
- **D-03:** 检索链路必须保持本地优先与可重建，不依赖默认远端 URL 抓取或任何云端服务。

### Query Semantics

- **D-04:** 精确匹配、子串匹配、模糊匹配和结构化 token 匹配需要统一进入一个明确的 ranked retrieval contract，而不是零散叠加多个前端 filter。
- **D-05:** 结构化 token 至少覆盖当前已识别类型里最有价值的字段，例如 URL host/path、JSON keys、command name、color alternate formats、source app。
- **D-06:** 模糊匹配的目标是帮助用户在“不记得完整文本”时仍能找到条目，优先服务开发者常见的 host、路径片段、命令名、代码 token 和颜色值片段。
- **D-07:** 搜索结果排序必须稳定、可解释，优先综合文本命中、结构化命中、模糊分数、时间和收藏等本地信号，而不是黑盒排序。

### Filter Model

- **D-08:** 搜索界面要支持类型或 subtype、收藏、来源 app 和近期时间窗口等缩小范围的筛选，但这些筛选应进入统一 query model，而不是前后端各做一半。
- **D-09:** 当前已有的 `selectedType` 是可复用入口，但需要演进成 retrieval filter contract 的一部分，而不是继续停留在纯前端列表过滤。
- **D-10:** 筛选与搜索必须能组合工作，不能出现“搜到了但 filter 端又二次裁掉”或“filter 生效但 backend 搜索不知道”的分裂状态。

### Result Presentation

- **D-11:** 检索结果继续复用 Phase 3 的 `buildPreviewSummary(entry, 'retrieval')`，保持与列表、详情一致的 semantic type 与 preview intent。
- **D-12:** 结果列表需要明确展示“为什么命中”的上下文，例如更完整的 secondary summary、可扫描 snippet 或 highlight contract，但不退回 subtype-specific 卡片分裂。
- **D-13:** 大量历史下必须保持交互响应，不允许为了 fuzzy 或 structured match 退回到每次输入都全量前端扫描。

### the agent's Discretion

- backend 查询表结构、token 持久化形式和索引策略。
- fuzzy 算法细节，只要保持本地可解释、稳定且可测试。
- filter UI 的具体控件布局与交互节奏，只要不破坏当前桌面端的紧凑工作流。
- result highlight/snippet 的具体样式和截断阈值。

</decisions>

<specifics>
## Specific Ideas

- “可以准确的预览对应类型的内容，并且支持可靠的检索能力。”
- “方便我快速筛选，或者模糊匹配。”
- “仅考虑客户端。”
- “关键指标就是类型识别准确率、搜索筛选、模糊匹配。”
- 检索高频场景会围绕 URL、JSON、颜色、命令、代码和普通文本展开。

</specifics>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Scope And Requirements

- `.planning/ROADMAP.md` — 定义 Phase 4 的目标、依赖和 5 条 retrieval success criteria。
- `.planning/PROJECT.md` — 定义本地客户端边界、开发者优先、无云同步/多端协作约束。
- `.planning/REQUIREMENTS.md` — 定义 `RETR-01` 到 `RETR-05` 的目标范围。
- `.planning/STATE.md` — 记录当前阶段状态与前置 phase 决策。

### Upstream Phase Contracts

- `.planning/phases/02-analysis-contracts-versioned-detection/02-VERIFICATION.md` — authoritative analysis、reanalysis、fallback diagnostics 已是稳定前提。
- `.planning/phases/03-unified-developer-previews/03-CONTEXT.md` — retrieval summary 需要继承 Phase 3 的 unified semantic preview contract。
- `.planning/phases/03-unified-developer-previews/03-VERIFICATION.md` — 确认 list/detail/retrieval 三个场景必须共享 semantic type 与 preview intent。
- `.planning/phases/03-unified-developer-previews/03-HUMAN-UAT.md` — 记录用户已确认的 JSON、URL、颜色预览真实期望。

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `src/stores/clipboardStore.ts`: 当前持有 `searchTerm`、`selectedType`、`setSearchTerm()` 和 `getFilteredEntries()`，但仍混合 backend fetch 与 frontend 二次过滤。
- `src/components/SearchBar/SearchBar.tsx`: 已有 200ms debounce、`useDeferredValue` 和 analytics 埋点，可继续复用为 retrieval query 输入层。
- `src/components/TypeFilter/TypeFilter.tsx`: 已有主类型和 text subtype 入口，适合作为统一 filter model 的现有 UI 壳。
- `src/lib/preview/previewSummary.ts`: 已有 `buildPreviewSummary(entry, 'retrieval')`，可直接作为搜索结果的中密度摘要 contract。
- `src/lib/preview/entryPresentation.ts`: 已有 analysis-first semantic model，可为 structured token 和 result snippet 提供稳定语义源。
- `src-tauri/src/analysis/repository.rs`: `load_entry_analysis_for_history()` 已负责 joined history read model，是后续 retrieval query 扩展的自然入口。

### Established Patterns

- 当前 `fetchHistory()` 已把 `searchTerm` 传给 backend 的 `get_clipboard_history(search)`，但 frontend `getFilteredEntries()` 又重复做一次 `content_data/source_app includes`，说明查询职责仍然分裂。
- backend 当前搜索仅是 `content_data LIKE ? OR source_app LIKE ?`，没有 fuzzy、structured token、ranking、favorites 或 recency query model。
- type 或 subtype filter 目前只存在于前端 store 的列表过滤阶段，没有进入 backend query，因此无法对大数据量和分页保持真正一致。
- Phase 3 已把 list/detail/retrieval summary contract 统一，说明 Phase 4 的结果呈现不该再新造 subtype-specific 结果卡。

### Integration Points

- retrieval query model 应从 `SearchBar` / `TypeFilter` 输入，穿过 `clipboardStore`，最终落到 Rust repository，而不是停留在 UI 层。
- backend query 需要 joined `clipboard_entries + entry_analysis`，这样 URL、JSON、command、color 等结构化 token 才能成为一等检索字段。
- 结果渲染应优先基于 `buildPreviewSummary(entry, 'retrieval')`，而不是复制列表项或详情卡片逻辑。
- 后续 rebuild safety 会依赖本 phase 的 token 或索引设计，因此持久化策略必须能被重建而不丢历史。

</code_context>

<deferred>
## Deferred Ideas

- 语义 embedding / 向量检索
- 远端同步搜索
- 团队共享与跨设备检索
- 自然语言问答式检索

</deferred>

---

_Phase: 04-search-quality-retrieval_
_Context gathered: 2026-03-29_
