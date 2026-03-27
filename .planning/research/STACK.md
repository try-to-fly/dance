# Technology Stack Research: Incremental Clipboard Intelligence Stack

**Project:** Dance
**Researched:** 2026-03-27
**Scope:** 仅研究现有 `Tauri + React + Rust + SQLite` 项目在 `1) 检测/解析栈 2) 预览/渲染相关库 3) 本地搜索/模糊匹配 4) 观测与测试支撑` 上的增量演进方案
**Baseline:** 保持现有 `Tauri 2 + React 18 + Rust + SQLite + sqlx + Monaco`，不做重写、不引入云产品、不引入独立搜索服务

## Executive Recommendation

最强实用路线不是换栈，而是把现有工程里的边界收紧，并把最薄弱的四条链路补齐：

1. **Rust 原生层成为唯一权威的检测与搜索层**  
   继续在 `src-tauri` 做类型检测、元数据提取、搜索索引维护和结果排序。前端不再自己兜底做第二套 URL / Base64 / 文本推断逻辑，避免当前 `clipboardStore.ts` 那种前后端重复判断。

2. **React 层只负责结构化渲染，不负责重新理解内容**  
   继续保留 Monaco 作为开发者文本/代码主视图，再补上 Markdown、JSON tree、CSV 表格等专用视图。预览质量应该来自“后端给了正确 subtype + metadata”，而不是前端猜。

3. **SQLite FTS5 做主检索，Rust fuzzy matcher 做二次重排**  
   不要再用 `%LIKE%` 扫全表。主方案是 `FTS5 external-content table + bm25 + snippet/highlight`，再对 Top N 候选用 Rust fuzzy scorer 做 typo tolerance 和缩写匹配。这样保留 SQLite/`sqlx`，但把搜索体验从“能搜”提升到“开发者可用”。

4. **观测与测试围绕“检测正确性”和“搜索相关性”建立**  
   当前项目最需要的不是更重的监控平台，而是本地可复现的 fixture、golden tests、性能基线和统一日志字段。否则类型识别、结构化预览和搜索排序都会在后续迭代中持续回退。

## Recommended Stack

### Core Stack Decisions

| Area                      | Technology                                          | Version / Status                    | Use                        | Why                                                                | Confidence  |
| ------------------------- | --------------------------------------------------- | ----------------------------------- | -------------------------- | ------------------------------------------------------------------ | ----------- |
| Desktop shell             | Tauri                                               | 2.x existing                        | 保持                       | 当前问题不在桌面壳层；重写没有收益                                 | HIGH        |
| Frontend runtime          | React                                               | 18.3.1 existing                     | 保持                       | 当前问题是数据理解与检索，不是 UI runtime 能力不足                 | HIGH        |
| Native app logic          | Rust                                                | stable / edition 2021 existing      | 强化                       | 检测、索引、排序都更适合放在本地原生层做权威实现                   | HIGH        |
| Persistence               | SQLite + sqlx                                       | existing                            | 保持                       | Brownfield 最低风险；FTS5 可直接在现有数据库上增量接入             | HIGH        |
| Code/text preview         | `@monaco-editor/react` + `monaco-editor`            | `4.7.0` existing + upgrade `0.55.1` | 保持并升级 `monaco-editor` | 现有已集成，支持多模型、DiffEditor、语言高亮、标记与只读预览       | HIGH        |
| JSON tree preview         | `react-json-view-lite`                              | `2.5.0` existing                    | 保持                       | 轻量、无依赖、适合大 JSON 展开/折叠，不必再引重型 JSON inspector   | HIGH        |
| Markdown preview          | `react-markdown` + `remark-gfm` + `rehype-sanitize` | `10.1.0` / `4.0.1` / `6.0.0`        | 新增                       | 适合开发者复制的 README、issue 片段、表格、task list，且默认更安全 | HIGH        |
| Structured text detection | `tree-sitter`                                       | `0.26.7`                            | 新增                       | 适合代码/命令片段的稳健语法探测，比纯 regex 更抗误判               | HIGH        |
| Fuzzy rerank              | `nucleo-matcher`                                    | `0.3.1`                             | 新增                       | Rust 侧高性能 fuzzy 排序，适合对 FTS 召回结果做二次重排            | MEDIUM-HIGH |
| TOML detection            | `toml`                                              | `1.0.6+spec-1.1.0`                  | 新增                       | 开发者高频复制配置片段；解析稳定、维护活跃                         | HIGH        |
| CSV/TSV detection         | `csv`                                               | `1.4.0`                             | 新增                       | 对日志、表格片段、导出数据做结构化表格预览很实用                   | HIGH        |
| XML detection             | `roxmltree`                                         | `0.21.1`                            | 新增                       | 只读树解析足够适合预览，不必引入可写 DOM 或更重解析栈              | HIGH        |
| Color parsing             | `csscolorparser`                                    | `0.8.3`                             | 新增                       | 比当前手写 regex 覆盖更广，支持 CSS Color 4 语法                   | HIGH        |
| JWT inspection            | `jsonwebtoken`                                      | `10.3.0`                            | 可选新增                   | 对开发者复制 token 的场景价值很高，能直接解 header/payload         | MEDIUM      |

### Strongest Practical Architecture for This Brownfield

**结论：**
继续用现有 `Tauri + React + Rust + SQLite`，但把新增能力按下面的边界落地：

| Layer               | Recommendation                                                          | Why                                                                   |
| ------------------- | ----------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Rust detector       | 所有 subtype 判断、metadata 抽取、search document 归一化都集中到 Rust   | 避免前端 fallback 和后端命令双轨逻辑漂移                              |
| SQLite index        | `clipboard_entries` 继续做主表；新增 `FTS5 external-content` 搜索索引表 | 最小迁移成本，保留现有数据模型和 `sqlx`，不必改掉现有 `TEXT` 业务主键 |
| Rust search service | 负责 MATCH 查询、bm25 排序、fuzzy rerank、snippet/highlight 回传        | 搜索逻辑统一，前端只消费结果                                          |
| React preview layer | 根据 `content_subtype + metadata + resolved_preview` 纯渲染             | 让 preview quality 建立在正确分类之上                                 |
| Monaco preview      | 继续做代码/原文/格式化文本主视图                                        | 已在仓库内，迁移成本最低，开发者认知成本最低                          |

## Prescriptive Recommendations

### 1. Type Detection: Parser-First, Regex-Last

**推荐方案：**
把 `ContentDetector` 改成“分层判定管线”，顺序固定，避免今天这种串行 regex/heuristic 相互抢分类。

**建议顺序：**

1. 超快确定型检测：URL、Email、IP、时间戳、Data URI、二进制头、文件路径。
2. 结构化文本解析：JSON -> TOML -> CSV/TSV -> XML -> JWT -> 颜色。
3. Markdown / command 语义判断。
4. 代码语言判断：只在前面都未命中时，才进入 `tree-sitter`。
5. 兜底 `plain_text`。

**具体库建议：**

| Content Type  | Library / Approach                           | Recommendation | Why                                                           | Confidence |
| ------------- | -------------------------------------------- | -------------- | ------------------------------------------------------------- | ---------- |
| JSON          | `serde_json` existing                        | 保持           | 已有、稳定、足够快                                            | HIGH       |
| TOML          | `toml` `1.0.6+spec-1.1.0`                    | 新增           | 开发配置文件频率高，解析精确                                  | HIGH       |
| CSV / TSV     | `csv` `1.4.0`                                | 新增           | 能把“看起来像文本”的数据片段变成结构化表格预览                | HIGH       |
| XML           | `roxmltree` `0.21.1`                         | 新增           | 只读树足够；比更重的 DOM 方案更适合 preview                   | HIGH       |
| CSS colors    | `csscolorparser` `0.8.3`                     | 新增           | 覆盖 `rgb()/hsl()/oklch()/named colors`，明显优于当前手写规则 | HIGH       |
| JWT           | `jsonwebtoken` `10.3.0`                      | 可选新增       | 开发者常复制 token；可直接拆 header/payload                   | MEDIUM     |
| Code snippets | `tree-sitter` `0.26.7` + 精选 grammar crates | 新增           | 允许在语法有错的片段上仍做稳健分类                            | HIGH       |

**对 `tree-sitter` 的具体建议：**

- 只接入高频 grammar，不要一次性拉几十种语言，先覆盖：`bash`, `javascript`, `typescript`, `tsx`, `rust`, `python`, `sql`, `html`, `css`, `go`。
- 先做“能把代码和非代码区分开、并把主流语言分准”的目标，不追求 GitHub Linguist 级别的全语言识别。
- 对输入长度做上限，例如只对前 `32KB-64KB` 文本做解析判定，避免大型日志/JSON 拖慢热路径。
- 把当前每次调用都 `Regex::new(...)` 的实现改成 `std::sync::LazyLock` 预编译；这是低风险高收益的热路径修复。

**明确不要这样做：**

- 不要继续扩张纯 regex 检测器。当前误判根因之一就是规则互相抢占，而且每次检测都在热路径重新编译正则。
- 不要先做“AI/模型识别语言”。本地剪贴板场景需要的是稳定和可解释，不是概率黑盒。

### 2. Structured Preview: Keep Monaco, Add Specialized Views

**推荐方案：**
前端保留 Monaco 做开发者主视图，但不是所有 subtype 都硬塞进统一文本编辑器。最佳体验是“专用视图 + 原文视图并存”。

| Preview Type              | Recommended UI Stack                                 | Recommendation   | Why                                                                 | Confidence |
| ------------------------- | ---------------------------------------------------- | ---------------- | ------------------------------------------------------------------- | ---------- |
| Code / command / raw text | `@monaco-editor/react` + `monaco-editor`             | 保持             | 已接入、可做只读高亮、Diff、marker、multi-model                     | HIGH       |
| JSON                      | `react-json-view-lite` + Monaco 原文视图             | 保持并标准化     | Tree view 适合理解结构，Monaco 适合复制和原文查看                   | HIGH       |
| Markdown                  | `react-markdown` + `remark-gfm` + `rehype-sanitize`  | 新增             | README、issue、表格、task list、autolink 都是开发者常见内容         | HIGH       |
| CSV / TSV                 | 现有 React 组件 + `@tanstack/react-virtual` existing | 复用现有虚拟滚动 | 预览表格时没必要引 AG Grid 这类重组件                               | HIGH       |
| URL                       | Rust 侧 metadata + React 卡片/inspector              | 强化而不是重写   | URL 的结构化信息应该来自后端一次解析，而不是前端多次 fetch fallback | HIGH       |
| JWT                       | `react-json-view-lite` + Monaco                      | 可选新增         | header/payload 树 + 原始 token 非常符合开发者心智                   | MEDIUM     |

**Monaco 的具体使用建议：**

- 主详情视图改成 `readOnly: true`。当前 `UnifiedTextRenderer` 默认可编辑，对剪贴板历史预览不是最优交互。
- 为每条记录分配稳定虚拟路径，例如 `memory:///clipboard/<id>.<ext>`，利用 Monaco multi-model 能力保存 view state、语言、格式化状态。
- 对 JSON / TOML / XML / SQL 这种可格式化内容，增加“Raw / Pretty / Diff”三种视图。`DiffEditor` 已由 `@monaco-editor/react` 提供，不需要换库。
- 不要让前端继续自行抓取 URL 文本、媒体信息做 fallback。预览元数据必须由 Rust 一次性返回统一 shape。

**Markdown 的具体建议：**

- 使用 `react-markdown` 渲染。
- 开启 `remark-gfm` 支持表格、task list、自动链接等 GitHub 风格 markdown。
- 使用 `rehype-sanitize`，并把它放在最后一个不可信内容处理点之后。
- 默认禁用 raw HTML 透传，不要把“能显示更多内容”建立在扩大 XSS 面之上。

**明确不要这样做：**

- 不要因为想升级预览体验就把 Monaco 替换成 CodeMirror 或 Shiki-only 渲染。现有项目已经用 Monaco，换编辑器是高迁移成本、低业务收益。
- 不要为了 CSV 预览引入 AG Grid / Handsontable。这个产品不是电子表格应用，现有 `react-virtual` 足够。
- 不要把 `@shikijs/monaco` 作为第一步。它适合追求更接近 TextMate 的语法着色，但当前核心问题不是高亮 fidelity，而是分类和搜索。它可以作为后续增强，不该占前排优先级。

### 3. Local Search and Filtering: FTS5 First, Fuzzy Second

**推荐方案：**
把搜索体系从“列表 API + `%LIKE%`”升级为“**SQLite FTS5 召回 + Rust fuzzy rerank + SQL filters**”。

**推荐实现：**

| Search Component  | Recommendation                                       | Why                                                                  | Confidence  |
| ----------------- | ---------------------------------------------------- | -------------------------------------------------------------------- | ----------- |
| Primary index     | SQLite `FTS5 external-content` table                 | 直接挂在现有 `clipboard_entries` 上，迁移成本最低                    | HIGH        |
| Tokenizer         | `unicode61 remove_diacritics 2 tokenchars '-_./:#@'` | 更适合开发者常搜的路径、标识符、URL 片段、包名                       | HIGH        |
| Prefix indexes    | `prefix='2 3 4'`                                     | 提升短前缀检索性能，适合开发者常见短查询                             | HIGH        |
| Ranking           | `bm25()`                                             | SQLite 内建，排序够用且实现简单                                      | HIGH        |
| Highlight/snippet | `highlight()` + `snippet()`                          | 搜索结果可直接高亮片段，减少前端二次处理                             | HIGH        |
| Fuzzy rerank      | `nucleo-matcher` `0.3.1`                             | 对 FTS Top N 结果做 typo / abbreviation 重排，比纯 JS 更适合大历史库 | MEDIUM-HIGH |

**表设计建议：**

- 保留 `clipboard_entries` 为主表。
- 新增 `clipboard_search` FTS5 索引表，采用 external-content 模式，使用主表 hidden `rowid` 做关联，继续保留现有 `id TEXT PRIMARY KEY` 作为业务主键。
- 维护一个后端生成的 `search_text`，而不是让 FTS 直接吃原始 `content_data`。
- `search_text` 应至少包含：
  - 归一化正文摘要
  - `content_subtype`
  - `source_app`
  - `detected_language`
  - URL host / path tokens
  - 文件名 / 扩展名
  - 结构化字段摘要（例如 JWT claim keys、JSON top-level keys）

**查询流程建议：**

1. 先在 SQL 层应用硬过滤：收藏、类型、子类型、来源应用、时间窗口。
2. 对剩余集合执行 FTS5 `MATCH`。
3. 用 `ORDER BY bm25(clipboard_search)` 做首轮排序。
4. 取前 `100-300` 条候选给 `nucleo-matcher` 二次重排。
5. 把 `highlight/snippet` 一起返回给前端列表项。

**为什么这是最强实用方案：**

- FTS5 解决召回与规模问题。
- `nucleo-matcher` 解决缩写、跳字、轻微 typo 的开发者搜索习惯问题。
- 两者结合比单独用 `LIKE`、单独用纯 fuzzy、或者直接上独立搜索引擎都更适合这个 brownfield。

**关于 trigram 的明确立场：**

- **默认不把 trigram tokenizer 作为第一步。**
- 原因：它更适合真 substring 搜索，但索引更重，而且 `<3` 字符查询本身存在限制。
- 如果后续发现用户大量按 URL/path 中间片段搜内容，再考虑给特定列或单独字段加 trigram 索引，而不是一开始全量替换 `unicode61`。

### 4. Observability and Testing: Local, Deterministic, Regression-Focused

**推荐方案：**
围绕现有 Tauri + Rust + React 工程，补齐“检测、预览、搜索”的可观测性和回归测试，而不是接第三方云监控。

| Area                   | Technology / Approach            | Version / Status            | Recommendation                        | Why                                                       | Confidence |
| ---------------------- | -------------------------------- | --------------------------- | ------------------------------------- | --------------------------------------------------------- | ---------- |
| Native logs            | `tauri-plugin-log`               | `2.x` existing              | 保持并结构化字段                      | 已在项目里，增量成本最低；关键是统一字段而不是换库        | HIGH       |
| Frontend tests         | `vitest` + Testing Library       | `4.1.2` existing / existing | 保持并扩展到 preview contract         | 当前前端已有基础，最适合承接 descriptor/renderer 合约测试 | HIGH       |
| Rust tests             | `cargo test` + 现有单元/集成测试 | existing                    | 保持并扩展到 detector/search fixtures | 类型识别和搜索应优先在 Rust 层锁定行为                    | HIGH       |
| Performance checks     | 现有 Rust performance tests      | existing                    | 保持并加 FTS/排序基线                 | 搜索替换后最容易出现性能回退，必须留基线                  | HIGH       |
| Snapshot/golden corpus | 仓库内 fixture corpus            | new approach                | 强烈建议新增                          | 这是类型检测与搜索相关性回归的最低成本护栏                | HIGH       |

**具体落地建议：**

- 为 `ContentDetector` 建一个固定样本库，按 subtype 分目录维护：
  - `json/`, `toml/`, `csv/`, `url/`, `code/`, `markdown/`, `base64/`, `jwt/`, `plain_text/`
  - 每个样本包含：原文、期望 subtype、期望 metadata 关键字段、已知误判说明
- 为搜索建立 relevance fixtures：
  - 输入 query
  - 预置若干条 clipboard entries
  - 断言 Top 5 / Top 10 的排序和高亮片段
- 为预览建立 contract tests，而不是只测组件能不能 render：
  - 输入 `ClipboardEntry + metadata + resolved preview`
  - 断言应该进入哪个 renderer
  - 断言 raw/pretty/tree/tabular 这些视图何时出现
- 所有检测和搜索性能测试都用固定数据集跑：
  - 小库，例如 `500` 条
  - 中库，例如 `10_000` 条
  - 大库，例如 `100_000` 条
  - 这样后续替换 tokenizer、grammar 或 fuzzy 权重时，能立刻看到退化

**日志字段建议：**

- 给 Rust 侧检测与搜索日志统一字段：
  - `entry_id`
  - `content_len`
  - `detected_subtype`
  - `detector_stage`
  - `parse_time_ms`
  - `search_query`
  - `fts_hits`
  - `rerank_count`
  - `search_time_ms`
- 这些字段应写进本地日志，不要依赖远端 SaaS 才能排查。

**测试优先级建议：**

1. `ContentDetector` fixture tests
2. FTS5 migration + index maintenance tests
3. Search relevance tests
4. Preview descriptor contract tests
5. 大文本 / 大历史库性能测试

**明确不要这样做：**

- 不要先引入 Datadog、Sentry、OpenTelemetry collector 这类云观测方案。本轮问题核心是本地逻辑正确性，不是分布式链路追踪。
- 不要把搜索质量验证只放在手工点点点上。没有固定 relevance corpus，后面改 tokenizer 和 reranker 一定会回退。
- 不要只测 React 组件快照而不测 Rust 输出契约。这个项目的正确性核心在原生层。

## Recommended Package Additions

### Rust

```toml
[dependencies]
tree-sitter = "0.26.7"
toml = "1.0.6"
csv = "1.4.0"
roxmltree = "0.21.1"
csscolorparser = "0.8.3"
nucleo-matcher = "0.3.1"
jsonwebtoken = "10.3.0" # optional
```

### Frontend

```bash
pnpm add react-markdown@10.1.0 remark-gfm@4.0.1 rehype-sanitize@6.0.0
pnpm add monaco-editor@0.55.1
```

## Alternatives Rejected

| Category               | Recommended                    | Rejected                          | Why Not                                                            |
| ---------------------- | ------------------------------ | --------------------------------- | ------------------------------------------------------------------ |
| Search engine          | SQLite FTS5 + `nucleo-matcher` | Tantivy / Meilisearch / Typesense | 对单机剪贴板历史过重，索引迁移与打包复杂度不值当                   |
| Fuzzy location         | Rust backend                   | 前端 `fuzzysort` 全量搜索         | 会把历史数据和排序压力重新拉回 renderer，放大内存和 IPC 成本       |
| Code preview           | Monaco existing                | CodeMirror rewrite                | 已有 Monaco，换编辑器只会制造 UI 级重构，不解决核心问题            |
| Markdown rendering     | `react-markdown` + sanitize    | 手写 regex/HTML 拼接              | 不安全且维护成本高                                                 |
| Color parsing          | `csscolorparser`               | 继续扩张手写 regex                | 规则覆盖不足，CSS Color 4 语法很快变得不可维护                     |
| YAML parsing           | 延后或仅做 Monaco 文本预览     | `serde_yaml`                      | `serde_yaml` 已被 docs.rs 标记为 deprecated / no longer maintained |
| SQLite fuzzy extension | FTS5 + Rust rerank             | `spellfix1` 依赖                  | 不是最稳妥的跨平台默认方案，分发和构建一致性更难保证               |
| Observability          | 本地结构化日志 + fixture tests | 云观测平台优先                    | 当前是单机桌面应用，先把本地回归链路做实更重要                     |

## Incremental Adoption Order

1. **Search foundation first**  
   先把 `%LIKE%` 升级为 `FTS5 + bm25 + snippet/highlight`。这是用户体感提升最大的改动，而且与预览、检测解耦。

2. **Detector refactor second**  
   把 `ContentDetector` 改成 parser-first 管线，并引入统一 metadata schema。先解决误判与重复逻辑。

3. **Preview standardization third**  
   前端只按 subtype 渲染；把 JSON / Markdown / CSV / URL inspector 路径统一到新的 descriptor / resolved preview shape。

4. **Observability and regression harness fourth**  
   把 detector fixtures、search relevance corpus、preview contract tests 和性能基线补齐，否则前三步无法稳定演进。

5. **Optional developer types last**  
   JWT、TOML、XML、CSV 这些类型可以按价值逐步加入，而不是一次性铺满。

## Confidence and Caveats

| Area                                                            | Confidence  | Notes                                                                  |
| --------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------- |
| SQLite FTS5 as main search index                                | HIGH        | 官方能力成熟，直接适配现有 SQLite 架构                                 |
| Monaco as primary developer preview surface                     | HIGH        | 现有仓库已在用，增量成本最低                                           |
| `tree-sitter` for code-vs-non-code and major language detection | HIGH        | 非常适合鲁棒语法探测，但“精确识别所有语言”仍不应承诺过高               |
| `nucleo-matcher` as reranker                                    | MEDIUM-HIGH | 路线正确，但具体阈值、Top N 和打分权重需要结合真实数据调优             |
| Local observability via logs + fixture corpus                   | HIGH        | 与当前单机桌面场景最匹配，最容易形成长期回归护栏                       |
| YAML structured parsing                                         | LOW         | 当前 Rust YAML 生态不够适合做本轮核心依赖，建议暂不作为核心 stack 决策 |

## Sources

- SQLite FTS5 official docs: https://www.sqlite.org/fts5.html
- Tree-sitter Rust bindings (`tree-sitter 0.26.7`): https://docs.rs/crate/tree-sitter/latest
- Tree-sitter official repo: https://github.com/tree-sitter/tree-sitter
- Monaco React wrapper: https://github.com/suren-atoyan/monaco-react
- Monaco Editor API: https://microsoft.github.io/monaco-editor/typedoc/
- Monaco Editor official repo: https://github.com/microsoft/monaco-editor
- react-markdown: https://github.com/remarkjs/react-markdown
- remark-gfm: https://github.com/remarkjs/remark-gfm
- rehype-sanitize: https://github.com/rehypejs/rehype-sanitize
- react-json-view-lite: https://github.com/AnyRoad/react-json-view-lite
- `nucleo-matcher 0.3.1`: https://docs.rs/crate/nucleo-matcher/latest/source/
- `toml 1.0.6+spec-1.1.0`: https://docs.rs/crate/toml/latest
- `csv 1.4.0`: https://docs.rs/crate/csv/latest
- `roxmltree 0.21.1`: https://docs.rs/crate/roxmltree/latest
- `csscolorparser 0.8.3`: https://docs.rs/csscolorparser/latest/csscolorparser/
- `jsonwebtoken 10.3.0`: https://docs.rs/crate/jsonwebtoken/latest
- `serde_yaml 0.9.34+deprecated`: https://docs.rs/serde-yaml
- npm registry versions verified locally on 2026-03-27 for: `monaco-editor`, `react-markdown`, `remark-gfm`, `rehype-sanitize`, `react-json-view-lite`, `fuzzysort`, `@shikijs/monaco`
