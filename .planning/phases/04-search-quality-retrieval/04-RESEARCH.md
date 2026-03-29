# Phase 04: Search Quality & Retrieval - Research

**Researched:** 2026-03-29
**Domain:** 本地桌面剪贴板检索管线（Tauri + Rust + SQLite + React）
**Confidence:** MEDIUM

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Claude's Discretion

- backend 查询表结构、token 持久化形式和索引策略。
- fuzzy 算法细节，只要保持本地可解释、稳定且可测试。
- filter UI 的具体控件布局与交互节奏，只要不破坏当前桌面端的紧凑工作流。
- result highlight/snippet 的具体样式和截断阈值。

### Deferred Ideas (OUT OF SCOPE)

- 语义 embedding / 向量检索
- 远端同步搜索
- 团队共享与跨设备检索
- 自然语言问答式检索
  </user_constraints>

<phase_requirements>

## Phase Requirements

| ID      | Description                                                                                                                                            | Research Support                                                                                                    |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| RETR-01 | User can retrieve history through indexed interactive search that remains responsive on large local datasets                                           | 推荐以 Rust authoritative query + companion search documents + SQLite FTS5 为主链路，并补齐 backfill/perf 门禁      |
| RETR-02 | User can narrow results by content type or subtype, source app, favorites, and recency-oriented filters                                                | 推荐显式 query contract，把 type/subtype/source/favorites/recency 全部下沉到 Rust 统一过滤                          |
| RETR-03 | User can find entries with fuzzy fragments, abbreviations, or partial developer tokens when exact text is unknown                                      | 推荐 prefix FTS + trigram substring recall + alias tokens 三段式召回，不再只靠单列 prefix FTS                       |
| RETR-04 | User can search normalized structured tokens where available, such as URL host or path fragments, JSON keys, command names, and alternate color values | 推荐以 authoritative analysis 为 structured token 主源，补 JSON key path 权威化，并让 companion doc 存储结构化 term |
| RETR-05 | User sees ranked results with enough snippet, highlight, or summary context to distinguish similar matches quickly                                     | 推荐保留 Phase 3 retrieval summary，并新增 backend `retrieval` match metadata/snippet/highlight contract            |

</phase_requirements>

## Summary

当前 workspace 已经存在一套 Phase 04 的半成品 backend 检索实现，而不是完全空白。未提交的 `src-tauri/src/retrieval/mod.rs` 已经引入 `ClipboardHistoryQuery`、`entry_search_documents`、`entry_search_fts`、`ClipboardRetrievalMatch` 和基础 ranking，`capture/runtime` 也会在持久化时同步写 search document。但是前端主链路仍然停留在 split authority 状态：`SearchBar` 只驱动 `searchTerm`，`TypeFilter` 和 `getFilteredEntries()` 仍在 React/Zustand 里二次过滤，`entry.retrieval` 也没有进入 TypeScript/UI 渲染层。更现实的问题是，这套 retrieval WIP 目前在本地 `cargo check` 下就是红的。

因此 Phase 04 不应该“从零造新的搜索系统”，也不应该把当前 retrieval WIP 原样视为可交付。正确方向是把现有 retrieval seed 收口成一个版本化、可重建、可测试的 Rust authoritative retrieval subsystem：一个显式 query contract，一个 companion search document 层，一个面向 recall 的索引策略，一个 deterministic ranking/match-context contract，以及一个与 Phase 5 rebuild 直接衔接的 stale detection/backfill 机制。

最关键的设计取舍有三个。第一，前端只保留 query 输入和结果渲染，不再对结果集做二次 includes/filter。第二，当前单列 `search_text` FTS 只能覆盖 token-prefix 命中，不足以闭合 path/hex/camelCase 片段和开发者常见缩写；推荐补成 “prefix FTS + trigram substring recall + alias token” 的三段式召回。第三，当前 JSON key 检索是 retrieval 层直接重解析 raw content，和 D-02 的 authoritative-analysis 方向不完全一致；Phase 04 应把 JSON key path 正式纳入 analysis metadata，并让 retrieval-time raw parse 仅作为迁移期 fallback。

**Primary recommendation:** 以当前未提交的 `retrieval` 模块为 Phase 04 的实现种子，但必须把它升级成“Rust authoritative query + versioned search documents + prefix/trigram/alias recall + rebuild-safe indexing”的完整闭环，前端彻底退回输入/渲染角色。

## Project Constraints (from CLAUDE.md)

- 前端命令必须在仓库根目录执行；Rust `cargo` 命令必须在 `src-tauri/` 目录执行。
- 执行命令前要明确当前工作目录，不能混用 root 和 `src-tauri/` 上下文。
- 新增 Tauri command 时，必须同时更新 `src-tauri/src/commands.rs`、`src-tauri/src/lib.rs` 的 `invoke_handler`、对应前端 store action，以及共享 TypeScript 类型。
- 修改数据库 schema 时，必须更新 `src-tauri/src/database/mod.rs`，并保持迁移向后兼容。
- Rust async 路径应继续使用 `tokio::sync` 原语，不要退回 `std::sync::Mutex`。
- Rust 继续是 analysis / preview semantics / retrieval semantics 的权威层；React 消费合同，不重新推断 subtype 语义。

## Current State Audit

- 本地 worktree 存在未提交 retrieval WIP：`src-tauri/src/retrieval/mod.rs` 是未跟踪文件，`src-tauri/src/state.rs` 和 `src-tauri/src/lib.rs` 有未提交改动并已接入它。
- 该 WIP 已经把 `ClipboardHistoryQuery` 接到 Tauri command `search_clipboard_history`，并让 `get_clipboard_history(search)` 在有 text 时转调 backend retrieval。
- companion table/FTS seed 已存在于本地 workspace：`entry_search_documents`、`entry_search_fts`、删除 trigger、capture 持久化同步写入、favorite toggle 同步刷新都已经落地。
- 当前前端没有消费这套 authoritative retrieval contract。`clipboardStore.ts` 仍然只把 `searchTerm` 传给 legacy `get_clipboard_history`，`selectedType` 和文本 includes 仍由 `getFilteredEntries()` 在前端二次执行。
- Rust model 已有 `ClipboardRetrievalMatch`，但 TypeScript `ClipboardEntry` 还没有 `retrieval` 字段，列表 UI 也没有展示命中理由/snippet。
- `cargo check` 于 2026-03-29 在本地失败，报错点位于 `src-tauri/src/retrieval/mod.rs` 中 `.execute(&mut **tx)` 的事务执行写法。这意味着 Phase 04 的 Wave 0 必须包含 retrieval baseline build-fix。
- 当前 search doc 没有 `search_version` 或 `analysis_version_at_index`，也没有完整 backfill/rebuild API。新捕获条目能入索引，但老历史和 analysis 重建后的 search stale row 无法可靠闭环。
- 当前 structured token 覆盖 URL host/path/query、command name、color alternate values、source app，以及通过 raw JSON 重解析得到的 JSON key path。JSON key 还没有进入 authoritative analysis metadata。

## Standard Stack

### Core

| Library                    | Version                                    | Purpose                                                           | Why Standard                                        |
| -------------------------- | ------------------------------------------ | ----------------------------------------------------------------- | --------------------------------------------------- |
| SQLite FTS5                | `3.51.0` local CLI, `ENABLE_FTS5` verified | 本地全文检索、`bm25`、`snippet/highlight`、tokenizer 组合         | 已在本机可用、零额外服务、天然适合本地桌面历史检索  |
| SQLx                       | `0.7.4` (`Cargo.lock`)                     | Rust 侧 schema migration、query builder、事务和 joined read model | 项目现有 DB 层已经统一在 SQLx 上，继续沿用成本最低  |
| Rust + Tauri command layer | Rust `1.91.0`, Tauri `2.x`                 | authoritative retrieval query、ranking、IPC 边界                  | 大历史检索和 ranking 不应落在 React 主线程          |
| React + Zustand            | React `18.3.1`, Zustand `4.5.0`            | query 输入状态、结果列表选择态、复用 Phase 3 retrieval summary    | 已有 UI 主栈；Phase 04 只需要把它收缩成 thin client |

### Supporting

| Library                                                     | Version                    | Purpose                                                  | When to Use                                                     |
| ----------------------------------------------------------- | -------------------------- | -------------------------------------------------------- | --------------------------------------------------------------- |
| Existing `entry_analysis` contract                          | contract `1`, analysis `1` | URL/command/color/timestamp 等 structured token 权威来源 | 生成 search document 时优先消费 authoritative analysis          |
| Existing `buildPreviewSummary(entry, 'retrieval')` contract | current local Phase 3 code | 结果摘要 headline/secondary summary                      | 所有 retrieval list row 都继续用它，不再新造 subtype 卡片       |
| SQLite FTS5 `unicode61` + prefix indexes                    | built-in                   | token/prefix recall                                      | host、command、source app、JSON key path 前缀匹配               |
| SQLite FTS5 `trigram` tokenizer                             | built-in                   | 子串 recall                                              | path/color/camelCase/嵌入式片段命中，例如 `hub` / `f55` / `sto` |

### Alternatives Considered

| Instead of                                             | Could Use                                | Tradeoff                                                                                                                                      |
| ------------------------------------------------------ | ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| 多列 search document + prefix/trigram/alias recall     | 继续沿用当前单列 `search_text` FTS       | 当前实现最省改动，但 recall 权重、snippet 质量、camelCase/hex/path 片段命中和 rebuild 能力都不够                                              |
| 手动同步 `entry_search_documents` + `entry_search_fts` | FTS5 external-content table              | external-content 更贴近官方 rebuild 模式，但要重做 rowid/content mapping；当前 workspace 已有 docs table 手动同步 seed，更适合先闭合 Phase 04 |
| Rust authoritative query                               | 前端拿分页结果后继续 `includes`/`filter` | 会继续制造分页漂移、实时更新错乱和“backend 命中但 frontend 裁掉”的 split-brain                                                                |

**Installation:**

```bash
# 推荐方案不需要新增 npm 包或 cargo crate
# 继续使用现有 Rust + SQLx + SQLite FTS5 + React/Zustand 即可
```

**Version verification:** 2026-03-29 已在本地验证 `sqlite3 --version` 为 `3.51.0`，`sqlite3 ':memory:' 'pragma compile_options;'` 含 `ENABLE_FTS5`；`cargo --version` 为 `1.91.0`，`pnpm --version` 为 `10.0.0`；`sqlx 0.7.4` 来自 `src-tauri/Cargo.lock`；`react 18.3.1` 与 `zustand 4.5.0` 来自 `package.json`。

## Architecture Patterns

### Recommended Project Structure

```text
src-tauri/src/
├── retrieval/
│   ├── contract.rs       # query/page/match DTOs
│   ├── documents.rs      # companion search document builder + alias generation
│   ├── repository.rs     # SQL query, FTS recall, stale detection
│   ├── rank.rs           # deterministic ranking + match context selection
│   └── rebuild.rs        # backfill/rebuild by search_version + analysis_version
├── analysis/
│   └── ...               # authoritative subtype/metadata source
└── database/
    └── mod.rs            # schema migration for search docs + FTS tables

src/
├── lib/retrieval/
│   └── queryAdapter.ts   # UI selectedType -> backend query adapter
├── stores/
│   └── clipboardStore.ts # thin orchestration only
└── components/
    ├── SearchBar/
    ├── TypeFilter/
    └── ClipboardList/    # render retrieval summary + match context
```

### Pattern 1: Rust Authoritative Query Boundary

**What:** 搜索、过滤、排序、分页只存在一个 backend query path。React 只持有输入态和选中态，不再对结果集做二次过滤。

**When to use:** 所有列表加载、搜索输入变化、type/subtype 变化、source/favorites/recency 变化、实时更新后的刷新都走同一条 Rust query。

**Example:**

```rust
// Source: local retrieval seed + Phase 04 recommendation
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClipboardHistoryQuery {
    pub text: Option<String>,
    pub content_type: Option<String>,     // text | image | file
    pub subtype: Option<AnalysisSubtype>, // url | json | command | ...
    pub source_app: Option<String>,
    pub favorites_only: Option<bool>,
    pub recency_days: Option<i64>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardHistoryPage {
    pub items: Vec<ClipboardEntry>,
    pub has_more: bool,
    pub next_offset: Option<i32>,
}
```

### Pattern 2: Companion Search Document With Dual Recall Paths

**What:** 保留 companion `entry_search_documents`，但把当前单列 `search_text` 演进成更明确的文档层，至少拆成 `body_text`、`structured_text`、`alias_text`、`source_app_text`；再使用两种 FTS recall 路径：

- prefix FTS: 命中 token 起始前缀、source app、command、URL host/path 前缀
- trigram FTS: 命中路径片段、颜色片段、camelCase 中间片段、嵌入式 substring

**When to use:** interactive search、结构化 token 搜索、片段 recall。只浏览 recent history 时不需要触发 text recall。

**Example:**

```sql
-- Source: https://sqlite.org/fts5.html
CREATE TABLE entry_search_documents (
  entry_id TEXT PRIMARY KEY,
  content_hash TEXT NOT NULL,
  content_type TEXT NOT NULL,
  semantic_type TEXT NOT NULL,
  source_app_text TEXT NOT NULL,
  body_text TEXT NOT NULL,
  structured_text TEXT NOT NULL,
  alias_text TEXT NOT NULL,
  search_version INTEGER NOT NULL,
  analysis_version_at_index INTEGER NOT NULL,
  indexed_at INTEGER NOT NULL
);

CREATE VIRTUAL TABLE entry_search_fts USING fts5(
  entry_id UNINDEXED,
  source_app_text,
  structured_text,
  alias_text,
  body_text,
  tokenize = "unicode61 remove_diacritics 2 tokenchars '._-/:#@'",
  prefix = '2 3 4'
);

CREATE VIRTUAL TABLE entry_search_trigram USING fts5(
  entry_id UNINDEXED,
  body_text,
  structured_text,
  tokenize = "trigram",
  detail = "none"
);
```

### Pattern 3: Deterministic Ranking And Match Context

**What:** ranking 采用明确的层次，不做黑盒打分。推荐顺序：

- exact structured match
- exact/prefix alias match
- exact/prefix content match
- trigram substring match
- favorite bonus
- recency tie-break

同时返回 `retrieval` 元信息，至少包含 `match_kind`、`label`、`snippet`、`matched_terms`，供 UI 解释“为什么命中”。

**When to use:** 所有 text query 结果。空 query 只按 recent order 返回，不附带 search-specific score。

**Example:**

```rust
// Source: local retrieval seed + Phase 04 recommendation
let mut score = 0.0;

if structured_exact {
    score += 120.0;
    match_kind = ClipboardRetrievalMatchKind::JsonKey;
} else if alias_prefix {
    score += 95.0;
    match_kind = ClipboardRetrievalMatchKind::Fuzzy;
} else if body_prefix {
    score += 72.0;
    match_kind = ClipboardRetrievalMatchKind::Content;
} else if trigram_substring {
    score += 54.0;
    match_kind = ClipboardRetrievalMatchKind::Fuzzy;
}

score += favorite_bonus(is_favorite);
score += recency_bonus(created_at);
```

### Pattern 4: Search Versioning And Phase 5 Rebuild Hook

**What:** search document 需要自己的版本轴，不要把 search stale 检测完全绑定到 `entry_analysis.analysis_version`。推荐新增：

- `SEARCH_DOCUMENT_VERSION`
- `analysis_version_at_index`
- `indexed_at`

并提供 `list_stale_search_entry_ids()`、`rebuild_search_documents()`、`rebuild_all_search_documents()`。

**When to use:** startup migration、manual rebuild、analysis rebuild 之后、Phase 5 release validation。

**Example:**

```sql
-- Source: Phase 04 recommendation
SELECT e.id
FROM clipboard_entries e
LEFT JOIN entry_search_documents d ON d.entry_id = e.id
LEFT JOIN entry_analysis a ON a.entry_id = e.id
WHERE d.entry_id IS NULL
   OR d.content_hash <> e.content_hash
   OR d.search_version < ?
   OR d.analysis_version_at_index < COALESCE(a.analysis_version, 0)
ORDER BY e.created_at DESC
LIMIT ?;
```

### Anti-Patterns to Avoid

- **Frontend second-pass filtering:** `getFilteredEntries()` 再次裁剪 backend 结果会破坏 RETR-01/02/05。
- **Single-column search text forever:** 继续只靠 `search_text` 一列 FTS，会让 snippet、权重和 recall 无法调优。
- **Retrieval-time JSON reparse as final design:** 当前 `collect_json_terms()` 直接 parse raw JSON 只能做迁移期 fallback，不该成为权威方案。
- **Optimistic query-result mutation under active filters:** `clipboard-update` / `toggleFavorite` 不能继续无脑改当前数组，必须走 refetch/reconcile。
- **No versioned search rebuild:** 没有 `search_version` / stale detection，Phase 5 不可能稳定闭环。

## Don't Hand-Roll

| Problem           | Don't Build                     | Use Instead                                                | Why                                                                         |
| ----------------- | ------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------- |
| 本地全文检索引擎  | 自定义 Rust/TS 倒排索引         | SQLite FTS5                                                | 已内建 `bm25`、prefix、trigram、snippet/highlight，且与 SQLite 事务天然一致 |
| 结果上下文生成    | 全量手写 snippet/highlight 规则 | FTS5 `snippet()`/`highlight()` + `ClipboardRetrievalMatch` | 文本边界、片段截断、权重解释比手切字符串更稳                                |
| 多处 filter 逻辑  | 前端一套、后端一套              | 单一 Rust query contract                                   | 否则分页、实时更新、favorites/source filter 必然漂移                        |
| Search stale 检测 | 一次性 ad-hoc migration 脚本    | versioned search documents + batch rebuild service         | Phase 5 要求历史可重建、升级不丢检索能力                                    |
| 模糊能力兜底      | 每次输入扫描整页前端数组        | alias tokens + trigram recall + deterministic rerank       | 大历史下前端扫描不可扩展，且解释性差                                        |

**Key insight:** Phase 04 不需要外接新的搜索服务；它需要的是把当前已存在的本地 search seed 变成“单一权威 query + 可解释 ranking + 可重建索引”的稳定子系统。

## Common Pitfalls

### Pitfall 1: Search/Filter Authority Still Split

**What goes wrong:** backend 搜到结果，frontend `selectedType` / `includes` 又裁掉；或者 favorites/source/recency 只在 backend 生效，UI 仍然显示旧选择态。

**Why it happens:** 现有 `clipboardStore.ts` 仍保留 `getFilteredEntries()` 和 `selectedType` 本地过滤逻辑。

**How to avoid:** 所有 query 输入统一转成 `ClipboardHistoryQuery`，结果列表直接消费 backend page。

**Warning signs:** `fetchHistory()` 之后还要调用 `getFilteredEntries()`；切换 type filter 时没有重新请求 backend。

### Pitfall 2: Old History Never Gets Fully Indexed

**What goes wrong:** 新捕获条目能搜到，老条目只有最近一小段能搜到，更老历史在 text query 下“像丢了一样”。

**Why it happens:** 当前 search doc 只在 capture 和 favorite toggle 时同步，缺少完整 backfill/rebuild。

**How to avoid:** Phase 04 必须引入 search stale detection 和 batch rebuild；Phase 5 再把它纳入 release-safe rebuild 流程。

**Warning signs:** 搜索只稳定命中新条目；`entry_search_documents` 行数明显小于 `clipboard_entries`。

### Pitfall 3: Prefix FTS Cannot Cover Substring Or Abbreviation Recall

**What goes wrong:** `clip*` 能命中 `clipboardStore`，但 `store`、`ff55`、`gh`、`gc` 这类开发者常见查询会漏掉。

**Why it happens:** 当前 `unicode61 tokenchars '._-/:#@'` 更像 token-prefix recall，不是 substring/abbreviation recall。

**How to avoid:** 使用 trigram 处理 substring，用 alias tokens 处理 `gc`/`gh` 这类缩写；prefix FTS 继续处理正常 token。

**Warning signs:** URL/命令名前缀查询表现好，但 path 中段、hex 片段、camelCase 中段和缩写命中率明显差。

### Pitfall 4: Analysis Rebuild And Search Index Drift Apart

**What goes wrong:** analysis 已升级，`entry_analysis` 是新的，但 search document 还是旧 structured terms，结果出现“详情是新 subtype，检索仍按旧 token 命中/漏掉”。

**Why it happens:** 当前 `rebuild_entry_analysis()` 不会顺带刷新 search document。

**How to avoid:** Phase 04 至少记录 `analysis_version_at_index`；Phase 5 rebuild 时串联 analysis rebuild 和 search rebuild。

**Warning signs:** reanalysis 后 structured token 搜索结果不变；search docs 的 `updated_at` 早于 `entry_analysis.analyzed_at`。

### Pitfall 5: Retrieval Result Metadata Never Reaches The UI

**What goes wrong:** backend 已经有 `ClipboardRetrievalMatch`，但列表仍然只显示通用 summary，用户看不出为什么命中。

**Why it happens:** TS `ClipboardEntry` 没有 `retrieval` 字段，列表组件没有渲染 retrieval context。

**How to avoid:** 给 TS type 加 `retrieval`，结果 row 在 Phase 3 summary 下方增加命中标签/snippet 区域。

**Warning signs:** 搜索结果在 UI 上与普通 list row 看起来完全一样。

### Pitfall 6: Planning Assumes Retrieval Baseline Already Builds

**What goes wrong:** 直接计划功能扩展，却忽略当前 retrieval WIP 在 Rust 侧本地编译已红。

**Why it happens:** 当前 worktree 是 partially integrated WIP，不是稳定主线。

**How to avoid:** 把 retrieval baseline compile-fix 和 smoke-check 放进 Wave 0。

**Warning signs:** `cargo check` 在 implementation 前就失败；`mod retrieval_tests;` 已声明但对应测试文件缺失。

## Code Examples

Verified patterns from official sources and current workspace:

### Backend Query Invocation From Frontend

```ts
// Source: local current workspace + Phase 04 recommendation
const page = await invoke<ClipboardHistoryPage>('search_clipboard_history', {
  query: {
    text,
    content_type: filter.contentType,
    subtype: filter.subtype,
    source_app: filter.sourceApp,
    favorites_only: filter.favoritesOnly,
    recency_days: filter.recencyDays,
    limit,
    offset,
  },
});
```

### Weighted Snippet Query In SQLite FTS5

```sql
-- Source: https://sqlite.org/fts5.html
SELECT
  entry_id,
  bm25(entry_search_fts, 1.0, 6.0, 5.0, 2.0) AS rank,
  highlight(entry_search_fts, 1, '<mark>', '</mark>') AS structured_hit,
  snippet(entry_search_fts, 3, '<mark>', '</mark>', '…', 18) AS body_snippet
FROM entry_search_fts
WHERE entry_search_fts MATCH ?
ORDER BY rank
LIMIT ?;
```

### Alias Tokens For Abbreviation Recall

```rust
// Source: Phase 04 recommendation backed by local sqlite probes
fn build_alias_tokens(command_name: &str, host: Option<&str>, identifier: Option<&str>) -> Vec<String> {
    let mut aliases = Vec::new();

    let initials = command_name
        .split(|ch: char| ch == ' ' || ch == '-' || ch == '_')
        .filter(|part| !part.is_empty())
        .map(|part| part.chars().next().unwrap())
        .collect::<String>()
        .to_lowercase();
    if initials.len() >= 2 {
        aliases.push(initials);
    }

    if let Some(host) = host {
        // github.com -> gh
        let normalized = host.split('.').next().unwrap_or_default();
        let chars = normalized.chars().take(2).collect::<String>();
        if chars.len() == 2 {
            aliases.push(chars);
        }
    }

    if let Some(identifier) = identifier {
        // clipboardStore -> cs
        let capitals = identifier
            .chars()
            .filter(|ch| ch.is_ascii_uppercase())
            .collect::<String>()
            .to_lowercase();
        if capitals.len() >= 1 {
            aliases.push(format!("{}{}", identifier.chars().next().unwrap().to_ascii_lowercase(), capitals));
        }
    }

    aliases
}
```

## State of the Art

| Old Approach                                                           | Current Approach                                                               | When Changed                                               | Impact                                               |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------------- | ---------------------------------------------------- |
| `content_data/source_app LIKE '%term%'` + frontend `includes` 二次过滤 | companion search documents + SQLite FTS5 recall + deterministic rerank         | Phase 04 target；current workspace 已经出现 retrieval seed | 支撑大历史响应式检索，避免 split authority           |
| 单列 `search_text` FTS                                                 | 多列 `body/structured/alias/source` + prefix/trigram 组合                      | Phase 04 recommended evolution                             | 排序更可解释，snippet/highlight 和结构化 recall 更稳 |
| retrieval-time raw JSON parse 作为唯一 JSON key 来源                   | analysis metadata 提供 `json_key_paths`，retrieval raw parse 只作过渡 fallback | Phase 04 recommended                                       | 与 D-02 保持一致，Phase 5 rebuild 更简单             |
| capture-only 索引更新                                                  | `search_version` + stale detection + batch rebuild                             | Phase 04/05 handoff                                        | 历史升级后仍可检索，不靠重新复制                     |
| 列表只显示通用 preview summary                                         | preview summary + retrieval match context 并列显示                             | Phase 04 target                                            | RETR-05 可解释性真正落地                             |

**Deprecated/outdated:**

- 直接在 `clipboard_entries` 上做 `LIKE` 搜索并让前端再 `includes`
- 把 `selectedType` 的 UI 值直接当作 backend 唯一 canonical filter model
- 没有 version/backfill 的 search document 持久化

## Open Questions

1. **当前未提交 retrieval WIP 是要“收口”还是“丢弃”？**
   - What we know: 本地 worktree 存在未提交 retrieval 代码，且 `cargo check` 于 2026-03-29 失败。
   - What's unclear: 这是用户正在推进的分支种子，还是一次未完成 spike。
   - Recommendation: 计划阶段先假设“收口现有 WIP”，除非用户明确要求回退到提交基线。

2. **JSON key 搜索要不要在 Phase 04 就升级 analysis contract？**
   - What we know: 当前 retrieval WIP 通过 `collect_json_terms()` 直接 parse raw JSON 来拿 key path。
   - What's unclear: 这是否被接受为 D-02 下的临时实现，还是必须先把 JSON key path 放进 authoritative analysis。
   - Recommendation: Phase 04 直接升级 analysis metadata；retrieval raw parse 只保留迁移期 fallback。

3. **Phase 04 是否立即迁移到 external-content FTS？**
   - What we know: 官方 FTS5 external-content table 有更直接的 rebuild 语义；当前 workspace 已经有 docs table + manual sync seed。
   - What's unclear: rowid/content mapping 迁移成本是否值得当前 phase 吸收。
   - Recommendation: Phase 04 先保留 manual sync，但补齐 version/backfill API；等 Phase 5 若 rebuild 复杂度仍高，再评估 external-content 迁移。

## Environment Availability

| Dependency  | Required By                                      | Available | Version    | Fallback                                                     |
| ----------- | ------------------------------------------------ | --------- | ---------- | ------------------------------------------------------------ |
| Node.js     | 前端 Vitest、Tauri frontend build                | ✓         | `v24.13.0` | —                                                            |
| pnpm        | 前端测试与构建                                   | ✓         | `10.0.0`   | npm 可跑部分命令，但不推荐切换                               |
| cargo       | Rust build/test                                  | ✓         | `1.91.0`   | —                                                            |
| rustc       | Rust compile toolchain                           | ✓         | `1.91.0`   | —                                                            |
| sqlite3 CLI | 本地 capability probe、manual index verification | ✓         | `3.51.0`   | App runtime 仍可依赖 SQLx 内置 SQLite 驱动，但手工诊断会变差 |

**Missing dependencies with no fallback:**

- None.

**Missing dependencies with fallback:**

- None.

## Validation Architecture

### Test Framework

| Property           | Value                                                                                                                        |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| Framework          | Vitest `4.1.2` + Rust `cargo test`                                                                                           |
| Config file        | `vitest.config.ts`；Rust 无单独 config 文件                                                                                  |
| Quick run command  | `pnpm test -- src/lib/preview/previewSummary.test.ts && (cd src-tauri && cargo test test_search_integration -- --nocapture)` |
| Full suite command | `pnpm test && (cd src-tauri && cargo test)`                                                                                  |

### Phase Requirements → Test Map

| Req ID  | Behavior                                                        | Test Type                   | Automated Command                                                                   | File Exists?                              |
| ------- | --------------------------------------------------------------- | --------------------------- | ----------------------------------------------------------------------------------- | ----------------------------------------- |
| RETR-01 | 大历史下 indexed interactive search 保持响应                    | integration + performance   | `cd src-tauri && cargo test test_query_performance_with_large_dataset -- --ignored` | ✅ `[src-tauri/src/performance_tests.rs]` |
| RETR-02 | type/subtype/source/favorites/recency 统一走 backend query      | integration                 | `cd src-tauri && cargo test test_search_query_filters -- --nocapture`               | ❌ Wave 0                                 |
| RETR-03 | fragments/abbreviations/partial developer tokens 命中           | unit + integration          | `cd src-tauri && cargo test test_search_alias_and_trigram_ranking -- --nocapture`   | ❌ Wave 0                                 |
| RETR-04 | URL/JSON key/command/color structured tokens 可检索             | integration                 | `cd src-tauri && cargo test test_search_structured_terms -- --nocapture`            | ❌ Wave 0                                 |
| RETR-05 | ranked results 带 snippet/highlight/match context，并在 UI 可见 | Rust unit + React component | `pnpm test -- src/components/ClipboardList/ClipboardList.retrieval.test.tsx`        | ❌ Wave 0                                 |

### Sampling Rate

- **Per task commit:** `pnpm test -- src/lib/preview/previewSummary.test.ts` and `cd src-tauri && cargo test test_search_integration -- --nocapture`
- **Per wave merge:** `pnpm test && (cd src-tauri && cargo test)`
- **Phase gate:** full frontend suite green，Rust retrieval integration + perf smoke green，且 `cargo check` 先恢复为绿

### Wave 0 Gaps

- [ ] `src-tauri/src/retrieval/mod.rs` 当前本地 `cargo check` 失败，需要先修复事务执行写法并恢复 baseline 编译
- [ ] `src-tauri/src/retrieval_tests.rs` 缺失，但 `src-tauri/src/lib.rs` 已声明 `mod retrieval_tests;`
- [ ] backend query contract filter tests：type/subtype/source/favorites/recency 组合覆盖还不存在
- [ ] backend fuzzy/alias/trigram ranking tests 还不存在
- [ ] backend search rebuild/backfill tests 还不存在
- [ ] frontend retrieval context render tests 还不存在

## Sources

### Primary (HIGH confidence)

- Local workspace code:
  - `src/stores/clipboardStore.ts`
  - `src/components/SearchBar/SearchBar.tsx`
  - `src/components/TypeFilter/TypeFilter.tsx`
  - `src/lib/preview/previewSummary.ts`
  - `src/lib/preview/entryPresentation.ts`
  - `src-tauri/src/retrieval/mod.rs`
  - `src-tauri/src/database/mod.rs`
  - `src-tauri/src/state.rs`
  - `src-tauri/src/capture/runtime.rs`
  - `src-tauri/src/models/mod.rs`
- Official SQLite FTS5 docs: https://sqlite.org/fts5.html
- Official SQLite PRAGMA docs: https://sqlite.org/pragma.html#pragma_optimize
- Local runtime verification on 2026-03-29:
  - `sqlite3 --version` → `3.51.0`
  - `sqlite3 ':memory:' 'pragma compile_options;'` → includes `ENABLE_FTS5`
  - local FTS probes verified prefix vs trigram vs alias behavior
  - `cargo check` in `src-tauri/` currently fails on retrieval WIP
  - `pnpm test -- --runInBand` passes (`14` files / `81` tests)

### Secondary (MEDIUM confidence)

- Project planning artifacts:
  - `.planning/PROJECT.md`
  - `.planning/ROADMAP.md`
  - `.planning/REQUIREMENTS.md`
  - `.planning/STATE.md`
  - `.planning/phases/04-search-quality-retrieval/04-CONTEXT.md`

### Tertiary (LOW confidence)

- None.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - 主要建立在现有仓库、已验证的本地 SQLite FTS5 能力和官方 SQLite 文档之上
- Architecture: MEDIUM - 方向清晰，但当前 workspace 是未提交 WIP，且 Rust baseline 目前编译失败
- Pitfalls: HIGH - 大多直接来自本地代码审计、运行探测和当前 split authority 现状

**Research date:** 2026-03-29
**Valid until:** 2026-04-05
