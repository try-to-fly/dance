# Project Research Summary

**Project:** Dance
**Domain:** 面向开发者的本地桌面剪贴板管理器
**Researched:** 2026-03-27
**Confidence:** HIGH

## Executive Summary

这是一个已经具备可用基础的 brownfield 桌面产品，不需要重写，也不该把精力放到云同步、协作或 AI 检索上。最优方向是继续沿用现有 `Tauri 2 + React 18 + Rust + SQLite`，但明确收口职责边界：Rust 成为唯一的采集、检测、预览解析和搜索权威层，React 只负责查询、选择和渲染。路线优先级也很明确，先把监听生命周期、检测版本化、预览契约和 FTS 搜索底座做稳，再扩展 JSON、URL、代码、颜色、命令、日志等开发者视图能力。

需求层面必须把“历史可信、识别准确、预览清楚、搜索找得到”当成主线。对开发者用户来说，表面上是剪贴板历史，实质上是对高频复制内容的快速理解与回用工作台。最大风险不在 UI，而在底层链路漂移：监听停不干净、内容被过早压平成字符串、前后端重复推断语义、搜索继续依赖 `%LIKE%`、测试没有覆盖真实桌面链路。路线上的每个阶段都应围绕“让同一条记录在采集、列表、详情、搜索里保持同一语义”来设计。

## Key Findings

### Recommended Stack

推荐继续使用现有桌面栈，不做框架迁移。核心增量是把本地能力补齐，而不是换技术名词：SQLite 用 FTS5 做主检索，Rust 负责 parser-first 检测、索引构建和 fuzzy rerank，React 保留 Monaco 和现有预览体系但改为纯渲染消费端。

**Core technologies:**

- `Tauri 2.x`：桌面壳层与系统集成继续保留，当前瓶颈不在壳层。
- `React 18.3.1`：保留现有 UI 运行时，只收缩 store 职责，不做前端重构。
- `Rust stable`：统一承载采集、检测、预览解析、搜索服务，是本轮核心强化点。
- `SQLite + sqlx`：继续做事实存储，增量加入 `entry_analysis` 与 FTS5 即可获得可维护搜索。
- `Monaco`：继续作为代码/原文主视图，无需改投 CodeMirror。
- `tree-sitter`：用于代码与非代码、主流语言的稳健判定，优先覆盖高频 grammar。
- `react-markdown + remark-gfm + rehype-sanitize`：补齐 Markdown 预览，但保持严格安全边界。
- `nucleo-matcher`：只作为 FTS5 召回后的 Rust 侧二次排序，不替代主索引。

### Expected Features

**Must have (table stakes):**

- 本地历史记录、键盘优先召回、收藏/删除/清空、稳定启动/停止监听。
- 即时全文搜索、内容类型筛选、来源应用筛选、最近时间与固定项过滤。
- 本地隐私控制，包括忽略指定应用、控制保留策略、默认本地存储。
- 基础开发类型识别，至少把文本、图片、文件、URL、颜色、JSON、代码、命令分开。
- 最低限度结构化预览，确保用户在粘贴前能判断内容是否正确。

**Should have now (competitive differentiators):**

- 高置信度 subtype 检测和明确 fallback，降低 JSON/URL/代码/命令误判。
- JSON、URL、代码、颜色的双视图预览，支持 raw 与 semantic 切换。
- 类型感知搜索，能搜索 URL host/path、JSON key、代码语言、命令名等规范化 token。
- FTS5 + bm25 + snippet/highlight + fuzzy rerank 的搜索体验，而不是继续全表 `%LIKE%`。
- 列表摘要与判别信息，让用户在结果列表阶段就区分相似条目。

**Defer (v2+):**

- JWT、XML、TOML、CSV/TSV 的更深层开发者视图，可在主链路稳定后逐步加入。
- 日志与命令的重度语义视图，仅在真实样本验证价值后再深化。
- 联网 URL 抓取、远程元数据增强、复杂媒体探测，必须是显式操作而非默认行为。

### Anti-Features / Non-Goals

- 云同步、多设备同步、账号体系。
- 团队协作、共享剪贴板、公共工作区。
- AI 语义搜索作为主检索路径。
- 默认联网抓取所有 URL 预览。
- 把产品做成重型 snippet manager、标签系统或长期知识库。

### Architecture Approach

架构上应把系统固定成四条责任链：`CaptureService -> AnalysisPipeline -> Repository/Indexer -> Preview/Search API`。原始条目继续保存在 `clipboard_entries`，新增 `entry_analysis` 记录 subtype、metadata、normalized text、版本号和错误，搜索通过 FTS5 伴生表维护，预览由 Rust 统一生成稳定 DTO。React 侧保留现有列表、选择态、虚拟滚动和 renderer，但移除 URL/Base64/媒体等语义 fallback，避免同一条记录在列表、详情和搜索中被解释成不同东西。

**Major components:**

1. `CaptureService`：处理系统监听、去重、来源应用和可取消生命周期。
2. `AnalysisPipeline`：负责 parser-first 检测、metadata 提取、版本化结果与失败诊断。
3. `SearchService`：负责 FTS5 查询、facet 过滤、bm25 排序、fuzzy rerank 与 snippet 返回。
4. `PreviewResolver`：根据 entry + analysis 生成唯一权威预览结果，并控制缓存与远程增强策略。
5. `UI Query/Renderer Layer`：只管理选择态、分页、视图切换和渲染，不再重复理解内容。

### Critical Pitfalls

1. **监听做成 fire-and-forget 后台任务**：会导致 start/stop 失真、重复订阅和重复入库；必须做成可取消服务并补启停集成测试。
2. **捕获阶段过早压平内容**：会让历史数据无法重判；必须分离原始载荷、规范化文本和检测结果，并保存版本号。
3. **忽略 transient / concealed / remote marker**：会带来敏感内容入库和远端噪音；必须先做 capture policy 再做 subtype 检测。
4. **预览层同时做解析、抓取、回写**：会造成 stale response、语义漂移和卡顿；必须建立单一 authoritative preview pipeline。
5. **继续用 `%LIKE%` 假装做开发者搜索**：会导致召回差、排序怪、扩容后卡顿；必须尽早落 FTS5 与评估集。

## Implications for Roadmap

基于研究，路线应按“先让输入可信，再让语义稳定，最后放大用户可见能力”来组织，而不是直接堆新预览类型。

### Phase 1: Runtime Reliability And Capture Policy

**Rationale:** 如果监听启停、来源识别和忽略策略不可信，后续所有检测和搜索都建立在脏输入上。  
**Delivers:** 可取消监听服务、start/stop/restart 测试、忽略 transient/concealed/remote 规则、统一 path helper。  
**Addresses:** 历史记录可靠性、本地隐私控制、基础筛选可信度。  
**Avoids:** 监听任务泄漏、重复入库、敏感内容误记、brownfield 路径漂移。

### Phase 2: Analysis Contracts And Versioned Detection

**Rationale:** 先把 capture、analysis、preview、search 之间的内部 contract 固定下来，后面才能稳定演进。  
**Delivers:** `RawClipboardPayload`、`DetectionResult`、`entry_analysis` 双写、detector registry、失败诊断与 re-detect 能力。  
**Implements:** Rust 成为唯一语义权威层。  
**Avoids:** subtype 漂移、历史数据不可重判、前后端重复检测。

### Phase 3: Preview Unification For Developer Types

**Rationale:** 预览应建立在稳定 analysis 上，否则越加 renderer 越乱。  
**Delivers:** 统一 `PreviewResolver`、JSON/URL/代码/颜色的 raw + semantic 双视图、React 侧去语义 fallback。  
**Addresses:** 结构化预览、类型判别、列表与详情一致性。  
**Avoids:** stale preview、异步覆盖、详情与列表语义不一致。

### Phase 4: Search Index And Retrieval Quality

**Rationale:** 开发者体验提升最大的单点来自搜索，从 `%LIKE%` 升级到结构化检索应尽早完成。  
**Delivers:** FTS5 external-content 表、facet 投影、bm25、snippet/highlight、Top N fuzzy rerank、评估集。  
**Addresses:** 即时搜索、类型感知搜索、模糊匹配排序。  
**Avoids:** 全表扫描、不可解释排序、历史增长后的性能塌陷。

### Phase 5: Diagnostics, Rebuild, And Release Gates

**Rationale:** 没有重建索引、重分析和 packaged smoke test，前四阶段会持续回归。  
**Delivers:** re-index / re-analyze 命令、fixture replay、桌面 smoke 流程、CI 质量门禁。  
**Addresses:** 稳定性、迁移可维护性、打包后行为一致性。  
**Avoids:** 假绿测试、发布后桌面特有故障、索引和缓存长期漂移。

### Phase Ordering Rationale

- 先修采集可靠性，再做类型能力，否则只是把错误数据更漂亮地展示出来。
- 先建立 analysis contract，再做 preview 和 search，避免两个消费面各自发明语义。
- 搜索应在核心 preview 稳定后快速落地，因为它是最直接的用户价值放大器。
- 诊断与重建能力必须进入主路线，而不是收尾补丁，否则 brownfield 改造不可持续。

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 1:** 桌面平台监听差异、remote/transient marker 覆盖范围、打包后行为差异需要结合目标平台继续核实。
- **Phase 4:** fuzzy rerank 的权重、Top N 截断、tokenizer 细节需要结合真实 query 样本和历史库调优。
- **Phase 5:** packaged smoke、权限诊断、跨平台自动化深度要按 CI 资源和发版策略校准。

Phases with standard patterns (skip research-phase):

- **Phase 2:** detector registry、versioned analysis、伴生表双写属于当前代码库可直接落地的标准内部改造。
- **Phase 3:** Monaco + 结构化 renderer + raw/semantic 双视图路径已经有明确方向，主要是工程收口问题。

## Confidence Assessment

| Area         | Confidence | Notes                                                    |
| ------------ | ---------- | -------------------------------------------------------- |
| Stack        | HIGH       | 主要基于官方能力和现有代码栈，方向稳定且迁移成本最低。   |
| Features     | HIGH       | 来自主流剪贴板产品共识与项目定位，需求边界清晰。         |
| Architecture | HIGH       | 直接由当前 brownfield 痛点反推，组件职责与演进顺序清楚。 |
| Pitfalls     | HIGH       | 同时有代码库内部证据和官方/成熟项目经验支持。            |

**Overall confidence:** HIGH

### Gaps to Address

- `nucleo-matcher` 的具体排序权重与阈值需要真实查询集验证，不能只按研究结论直接拍板。
- JWT、日志、命令深度视图的用户价值仍需在 requirements 或后续 phase 中做取舍，不应挤占主链路。
- URL 远程增强预览的默认策略需要产品明确隐私立场，目前只能确定“不默认联网”。

## Sources

### Primary (HIGH confidence)

- `.planning/PROJECT.md` — 产品边界、已验证能力、当前活跃目标。
- `.planning/research/STACK.md` — 技术选型、版本建议、FTS5 与 parser-first 路线。
- `.planning/research/FEATURES.md` — table stakes、差异化方向、非目标边界。
- `.planning/research/ARCHITECTURE.md` — 组件边界、数据流、brownfield 改造顺序。
- `.planning/research/PITFALLS.md` — phase bucket、关键失败模式与预防策略。
- SQLite FTS5 official docs — 搜索索引与排序基础能力。
- Tauri state/calling Rust 官方文档 — brownfield 下的职责收口边界。

### Secondary (MEDIUM confidence)

- Monaco、react-markdown、tree-sitter、nucleo-matcher、roxmltree、csv、toml 等官方仓库或 docs.rs 文档。
- Raycast、Paste、Alfred、Maccy、CopyQ 的公开文档与能力边界。
- Apple NSPasteboard / Microsoft Clipboard API 文档与社区 marker 约定。

---

_Research completed: 2026-03-27_
_Ready for roadmap: yes_
