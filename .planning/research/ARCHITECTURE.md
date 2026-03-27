# Architecture Research: Developer-Focused Local Clipboard Manager

**Domain:** 面向开发者的本地桌面剪贴板管理器
**Scope:** brownfield Tauri + React + Rust + SQLite
**Researched:** 2026-03-27
**Overall confidence:** HIGH

## Executive Summary

这类产品要长期保持“识别准、预览稳、搜索快”，核心不是继续往 React store 或 `commands.rs` 里堆逻辑，而是把系统明确拆成四条责任链：**采集**、**分析**、**预览**、**检索**。当前代码库已经有这些能力的雏形，但它们混在 `clipboardStore.ts`、`state.rs`、`commands.rs` 和 `content_detector.rs` 中，导致前后端重复推断、缓存分散、搜索与筛选耦合、问题难以定位。

推荐方向是：**Rust 成为语义权威层，React 只负责查询、选择、渲染和轻量 UI 缓存**。也就是说，剪贴板采集、类型检测、元数据提取、预览解析、搜索索引都放在 Rust/SQLite 一侧；React 不再自行“猜” URL/媒体/JSON 预览，只消费一个稳定的、版本化的结果结构。这样才能保证同一条记录在列表、详情、搜索结果、重建索引后看到的是同一套语义。

在 brownfield 条件下，不建议重写现有 `clipboard_entries` 表或一次性拆散全部前端 store。更稳妥的做法是：**保留现有主表和 UI 外观，新增导出式边界和伴生表**。先从代码边界开始，把监控、检测、预览、搜索提成独立服务；再引入 `entry_analysis` 和 FTS 索引；最后让前端改为只调用新的查询/预览接口。这样每一步都可验证、可回滚，不会打断现有功能。

对 roadmap 的直接含义是：第一阶段不应直接做“更多预览类型”或“更强模糊搜索”，而应先完成**生命周期收口、路径统一、领域 contract 抽离**。否则后续每新增一个检测器、预览器或搜索规则，都会继续放大现有耦合。

## Recommended Architecture

```text
OS Clipboard
  -> Capture Service
  -> Analysis Pipeline
  -> Entry Repository
  -> Analysis Repository
  -> Search Indexer
  -> UI update event

React Search / Filters
  -> Query API
  -> Search Service
  -> SQLite (entry rows + analysis + FTS)
  -> Result DTOs

React Detail Selection
  -> Preview API
  -> Preview Resolver
  -> Preview Cache / Artifact Cache
  -> Typed Preview DTO
  -> Renderer

Maintenance / Debug Tools
  -> Re-analyze
  -> Rebuild index
  -> Invalidate preview cache
  -> Replay fixture
```

## Component Boundaries

| Component            | Responsibility                                                          | Communicates With                                      | Direction                                           | Brownfield Mapping                                                                                                                                                                                                                             |
| -------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------ | --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CaptureService`     | 只负责读取系统剪贴板、抓取来源应用、做原始去重和生命周期控制            | `AnalysisPipeline`                                     | `OS -> Capture -> Analysis`                         | 从 [`src-tauri/src/clipboard/monitor.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/monitor.rs) 抽离；停止把检测和持久化混进去                                                                                   |
| `AnalysisPipeline`   | 把原始内容变成 `subtype + metadata + normalized_text + analysis_status` | `EntryRepository`, `AnalysisRepository`                | `Capture -> Analysis -> Persist`                    | 以 [`src-tauri/src/clipboard/content_detector.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/content_detector.rs) 为起点，拆成 detector registry                                                                 |
| `EntryRepository`    | 管理原始条目写入和读取，维护 `clipboard_entries` 作为事实表             | `AnalysisPipeline`, `SearchService`, `PreviewResolver` | `Analysis -> EntryRepository -> SQLite`             | 从 [`src-tauri/src/state.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/state.rs) 和 [`src-tauri/src/database/mod.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/database/mod.rs) 提取                 |
| `AnalysisRepository` | 存储可重算的派生结果、版本号、错误信息、调试痕迹                        | `AnalysisPipeline`, `SearchIndexer`, `PreviewResolver` | `Analysis -> AnalysisRepository`                    | 新增 Rust 模块和伴生表；初期可与现有 `content_subtype` / `metadata` 双写                                                                                                                                                                       |
| `SearchIndexer`      | 把分析结果变成可检索文档，维护 FTS 和过滤字段                           | `AnalysisRepository`, `SearchService`                  | `AnalysisRepository -> SearchIndexer -> SQLite FTS` | 新增 `src-tauri/src/search/`，不要再让 React 和 SQL 各自重复搜索逻辑                                                                                                                                                                           |
| `SearchService`      | 负责 query 解析、候选召回、排序、snippet/highlight、结果分页            | `EntryRepository`, `AnalysisRepository`                | `UI -> SearchService -> SQLite -> UI`               | 从 [`src-tauri/src/state.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/state.rs) 的 `%LIKE%` 查询升级                                                                                                                     |
| `PreviewResolver`    | 根据 entry + analysis 生成唯一权威预览结果；必要时做远程/媒体补充       | `AnalysisRepository`, `PreviewCache`                   | `UI -> PreviewResolver -> UI`                       | 从 [`src-tauri/src/commands.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/commands.rs) 和 [`src/stores/clipboardStore.ts`](/Users/smile/Documents/try-to-fly/clipboard-app/src/stores/clipboardStore.ts) 的双边逻辑中抽离 |
| `PreviewCache`       | 缓存重计算成本高的预览结果或衍生资产，支持版本失效                      | `PreviewResolver`                                      | `PreviewResolver <-> Cache`                         | 先做 Rust 内存缓存；文件/磁盘缓存只留给大图、远程元数据、缩略图                                                                                                                                                                                |
| `UI Query Layer`     | 管理选择态、分页、debounce、虚拟列表、视图状态                          | `SearchService`, `PreviewResolver`                     | `User -> UI Query Layer -> Backend`                 | React 侧继续用 store，但 store 不再自带语义推断                                                                                                                                                                                                |
| `Renderer Layer`     | 只消费稳定的 `PreviewDescriptor` / `ResolvedPreviewData` 渲染           | `UI Query Layer`                                       | `Typed Preview -> Renderer`                         | 保留 [`src/lib/preview/previewDescriptor.ts`](/Users/smile/Documents/try-to-fly/clipboard-app/src/lib/preview/previewDescriptor.ts) 作为前端渲染适配层                                                                                         |
| `DiagnosticsTooling` | 支持重分析、重建索引、导出调试记录、fixture replay                      | 全部服务                                               | `Operator -> Diagnostics -> Services`               | 新增维护命令，不再靠人工看日志猜状态                                                                                                                                                                                                           |

## Data Flow

### 1. 采集与持久化

**明确方向：** `OS Clipboard -> CaptureService -> AnalysisPipeline -> Repository -> SQLite -> UI event`

1. `CaptureService` 只负责拿到一份 `RawClipboardPayload`，内容包括原始文本/图片引用、来源应用、原始 hash、采集时间。
2. `AnalysisPipeline` 在 Rust 内完成类型检测、子类型归类、元数据提取、标准化文本生成。
3. `EntryRepository` 写入 `clipboard_entries`。
4. `AnalysisRepository` 写入对应分析结果和分析版本。
5. `SearchIndexer` 在同一事务或紧邻事务内更新检索文档。
6. 后端只向前端发一个“条目已 upsert”事件，不把半成品状态散落到多个事件里。

**架构要点：**

- 原始采集数据和派生分析结果要分开看待。
- `clipboard_entries` 应尽量不可变，只允许 `copy_count`、`favorite` 这类业务字段变更。
- 检测失败也要持久化 `analysis_status=failed` 和错误原因，而不是直接吞掉。

### 2. 搜索与结果列表

**明确方向：** `SearchBar -> SearchService -> SQLite FTS / indexed facets -> Result DTO -> React list`

1. React 只发送搜索词、筛选项、分页参数。
2. `SearchService` 负责 query normalization、tokenization、排序和 snippet。
3. SQLite 返回候选文档，再由 repository 取完整条目和分析信息。
4. React 列表只渲染结果，不再做第二遍 `content_data.includes(...)` 过滤。

**推荐实现：**

- 主搜索使用 SQLite FTS5。
- 排序使用 `bm25()` 之类的 FTS 排名能力。
- 过滤项优先落在普通索引列或 generated columns 上，不要把所有条件都塞进 `MATCH`。
- 真正的 typo-tolerant fuzzy 排序，放在 Rust 对 top N 候选做轻量 rerank，而不是一开始就引入新的搜索引擎。

### 3. 详情预览

**明确方向：** `Selected entry -> PreviewResolver -> PreviewCache -> Typed preview -> Renderer`

1. 用户选中条目后，React 只发 `entry_id + content_hash + requested_mode`。
2. `PreviewResolver` 先从分析结果构造同步可得的预览。
3. 对于 URL、媒体、Base64 等需要补充解析的情况，后端统一返回稳定结构。
4. 远程增强预览应作为可选 enrichment，而不是“选中就立刻发网络请求”的默认副作用。

**架构要点：**

- React 不应再维护 URL/媒体/Base64 的语义 fallback。
- 前端允许有渲染 fallback，但不允许有“另一套检测/解析逻辑”。
- 预览结果要带 `resolver_version` 和 `cache_status`，方便排错和失效。

### 4. 重分析与重建索引

**明确方向：** `Maintenance command -> Re-analysis job -> AnalysisRepository / SearchIndexer -> UI refresh`

1. 当检测规则、预览规则、搜索文档规则变化时，不应要求清库。
2. 通过显式 job 重跑分析和索引即可。
3. 前端可看到某条记录是由哪个版本的 detector / resolver / indexer 生成的。

## How To Isolate Parser / Preview / Search In A Brownfield Codebase

### Parser / Detector

推荐把现有的 [`src-tauri/src/clipboard/content_detector.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/content_detector.rs) 改造成“注册表 + 规则对象”结构，而不是继续放一个持续膨胀的超大文件。

**建议边界：**

- `RawClipboardPayload`: 纯采集模型，不带语义结论。
- `DetectionResult`: `subtype`, `metadata_json`, `confidence`, `normalized_text`, `analysis_warnings`。
- `Detector`: 单一职责规则，例如 URL、JSON、Color、Code、Base64。
- `DetectorRegistry`: 统一优先级和冲突处理。

**不要做：**

- 不要让 detector 直接读写数据库。
- 不要让 detector 直接生成 React 预览组件需要的结构。
- 不要在前端再补一套 detector。

### Preview

推荐把 preview 看成“读取模型”，不是“检测副产物”。

**建议边界：**

- `PreviewResolver`: 输入 entry + analysis，输出 `ResolvedPreviewData`。
- `PreviewArtifactFetcher`: 负责远程内容、媒体探测、缩略图等昂贵操作。
- `PreviewCache`: 按 `entry_id + content_hash + resolver_version` 缓存。
- `PreviewDescriptorAdapter` 继续留在前端，只做展示层适配。

**关键原则：**

- “URL 是什么”属于 detector。
- “URL 该展示成卡片、文本、图片、视频还是 JSON”属于 preview resolver。
- “怎么渲染这个预览”属于 React renderer。

### Search

搜索不要继续嵌在 `get_clipboard_history(search)` 里做 `%LIKE%`。

**建议边界：**

- `SearchDocumentBuilder`: 负责把 entry + analysis 变成可索引文本。
- `SearchIndexer`: 负责索引 upsert / delete / rebuild。
- `SearchService`: 负责查询解释、排序、分页和高亮片段。
- `FacetProjection`: 负责把 JSON metadata 暴露成可过滤字段。

**关键原则：**

- 搜索文档是派生物，不是原始数据。
- 搜索和列表筛选必须共用一套后端语义。
- React 侧不再做二次搜索或隐藏过滤。

## Persistence And Indexing Model

### Recommended Storage Layout

| Storage                                  | Purpose        | Why                                                         |
| ---------------------------------------- | -------------- | ----------------------------------------------------------- |
| `clipboard_entries`                      | 原始条目事实表 | 兼容现有 schema，保留稳定主记录                             |
| `entry_analysis`                         | 派生分析表     | 隔离 subtype / metadata / normalized text / version / error |
| `entry_search_fts`                       | FTS5 检索表    | 本地检索继续复用 SQLite，不引入新引擎                       |
| `preview_artifacts` or memory/file cache | 昂贵预览产物   | 只缓存高成本结果，不缓存所有 UI 状态                        |

### Practical Brownfield Recommendation

- **短期**：保留 `clipboard_entries.content_subtype` 和 `metadata`，同时新增 `entry_analysis`，做双写。
- **中期**：所有搜索与预览改读 `entry_analysis`。
- **长期**：把 `clipboard_entries` 收敛为事实表，把 derived 字段逐步从主表迁出。

### Search Schema Recommendation

推荐使用 SQLite FTS5，而不是现在就引入 Tantivy、Meilisearch 或单独的本地搜索进程。原因很直接：当前产品是单机、单进程、SQLite 已存在，FTS5 能用最小架构成本覆盖主要需求。

**关键实现建议：**

- 新建 `entry_analysis`，使用 `analysis_id INTEGER PRIMARY KEY` 作为稳定 rowid。
- FTS 表以 `entry_analysis` 为内容源，避免直接拿 `TEXT PRIMARY KEY` 的 `entry_id` 去适配 FTS rowid。
- 检索文本应包含：主文本、source app、URL host/path token、文件名、检测到的语言、可读标题。
- 过滤字段使用普通索引列或 generated columns，不要全部堆在 JSON 反序列化路径里。

### Metadata Projection Recommendation

当前 `metadata` 已经是 JSON 字符串，这是可利用的，不必因为“想筛选”就把所有字段平铺成硬编码列。更好的做法是：

1. 保留 `metadata_json` 作为完整派生结构。
2. 为高频筛选字段增加 generated columns，例如 `url_host`、`detected_language`、`mime`、`preview_kind`。
3. 只给高频字段建索引。

这样既保留演进空间，又不会把 schema 变成一堆一次性字段。

## Suggested Build Order

### Phase 1: Stabilize Runtime Boundaries

先修监控生命周期、取消机制、统一路径助手，再抽服务边界。没有这一步，后面加分析和搜索只会叠加不可控后台任务。

### Phase 2: Introduce Domain Contracts

新增 `RawClipboardPayload`、`DetectionResult`、`PreviewResult`、`SearchDocument` 这些内部 contract，并让 `commands.rs` 和 `state.rs` 只做装配，不再承载大量规则。

### Phase 3: Add `entry_analysis` And Double-Write

先不动 UI，后端开始把检测结果写入伴生表，并记录版本号、错误、耗时。这样后续 preview/search 才有稳定输入。

### Phase 4: Make Rust The Only Preview Authority

把 [`src/stores/clipboardStore.ts`](/Users/smile/Documents/try-to-fly/clipboard-app/src/stores/clipboardStore.ts) 里的 URL/Base64/媒体 fallback 移出，React 只消费统一 preview 结果。

### Phase 5: Replace `%LIKE%` With Search Service + FTS5

完成搜索索引、分页、排序和 snippet，高频筛选改走 indexed facets。此时 React 才能删掉本地二次过滤。

### Phase 6: Add Rebuild / Reanalyze / Diagnostics

最后补管理命令、fixture replay、索引重建、缓存失效工具。没有这些，系统虽然能跑，但不可持续演进。

## Suggested Code Moves Without A Rewrite

| Current File                                                                                                                                 | Recommended Role | Action                                                  |
| -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | ------------------------------------------------------- |
| [`src-tauri/src/clipboard/monitor.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/monitor.rs)                   | 采集服务         | 保留文件，但删掉检测和持久化职责                        |
| [`src-tauri/src/clipboard/content_detector.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/content_detector.rs) | 检测注册表入口   | 按 subtype 拆子模块                                     |
| [`src-tauri/src/commands.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/commands.rs)                                     | Tauri 边界层     | 变薄，只做 DTO 映射和授权                               |
| [`src-tauri/src/state.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/state.rs)                                           | 运行时装配层     | 移出 SQL、搜索、预览细节                                |
| [`src-tauri/src/database/mod.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/database/mod.rs)                             | 连接与迁移       | 新增 repository/search migration，不再堆业务 SQL        |
| [`src/stores/clipboardStore.ts`](/Users/smile/Documents/try-to-fly/clipboard-app/src/stores/clipboardStore.ts)                               | UI 状态层        | 保留列表/选中/分页，移出语义解析和远程 preview fallback |
| [`src/lib/preview/previewDescriptor.ts`](/Users/smile/Documents/try-to-fly/clipboard-app/src/lib/preview/previewDescriptor.ts)               | 展示适配层       | 保留，但其输入改为单一权威 preview DTO                  |

## Anti-Patterns To Avoid

### Anti-Pattern 1: Frontend And Backend Both Infer Semantics

**Why bad:** 同一条记录可能在列表里被判成 URL，在详情里又因为 fallback 被当成文本或媒体，难以复现和测试。

**Instead:** Rust 产出唯一权威语义，React 只渲染。

### Anti-Pattern 2: Raw Data And Derived Data Mutate Together Without Versioning

**Why bad:** 一旦检测规则升级，旧数据无法区分“旧规则结果”还是“数据本身问题”。

**Instead:** 把分析结果独立存储，并记录 detector/indexer/resolver version。

### Anti-Pattern 3: Search Hidden Inside Generic History Query

**Why bad:** 搜索、分页、排序、筛选都会互相污染，最后只能继续加 if/else。

**Instead:** 单独的 `SearchService` 和查询 DTO。

### Anti-Pattern 4: Selecting A Row Triggers Unbounded Background Work

**Why bad:** URL 预览、媒体探测、远程请求会在 UI 切换时堆积，且难以取消。

**Instead:** 预览解析必须可取消、可缓存、可观察；远程增强要显式受策略控制。

## Confidence Assessment

| Area                                   | Confidence | Notes                                      |
| -------------------------------------- | ---------- | ------------------------------------------ |
| 组件边界建议                           | HIGH       | 主要来自现有代码结构和已暴露痛点，结论稳定 |
| 数据流与 build order                   | HIGH       | 直接由当前耦合点和 brownfield 风险推导     |
| SQLite FTS5 作为主搜索方案             | HIGH       | 官方能力足够，且最适合现有 SQLite 单机架构 |
| generated columns + JSON metadata 投影 | HIGH       | 对 brownfield 增量演进友好                 |
| typo-tolerant fuzzy rerank             | MEDIUM     | 实现细节仍要结合真实 query 样本调优        |
| 远程 preview enrichment 策略           | MEDIUM     | 需要结合产品偏好和隐私策略落地             |

## Sources

### Codebase-Derived

- [`/Users/smile/Documents/try-to-fly/clipboard-app/.planning/PROJECT.md`](/Users/smile/Documents/try-to-fly/clipboard-app/.planning/PROJECT.md)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/ARCHITECTURE.md`](/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/ARCHITECTURE.md)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/CONCERNS.md`](/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/CONCERNS.md)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/STRUCTURE.md`](/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/STRUCTURE.md)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/monitor.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/monitor.rs)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/content_detector.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/content_detector.rs)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/state.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/state.rs)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/commands.rs`](/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/commands.rs)
- [`/Users/smile/Documents/try-to-fly/clipboard-app/src/stores/clipboardStore.ts`](/Users/smile/Documents/try-to-fly/clipboard-app/src/stores/clipboardStore.ts)

### Official / Primary References

- Tauri Brownfield Pattern: https://v2.tauri.app/start/#brownfield-pattern
- Tauri Calling Rust From The Frontend: https://tauri.app/develop/calling-rust/
- Tauri State Management: https://v2.tauri.app/develop/state-management/
- SQLite FTS5: https://sqlite.org/fts5.html
- SQLite Generated Columns: https://sqlite.org/gencol.html
- SQLite JSON Functions: https://sqlite.org/json1.html
- `tokio_util::sync::CancellationToken` docs: https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html
