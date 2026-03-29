# Phase 3: Unified Developer Previews - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 交付统一开发者预览体系，让当前已支持的开发者内容在列表、详情和后续检索结果中共享同一套 preview contract、语义主视图意图、Raw 可达性和摘要规则，只在信息密度上区分场景。

本 phase 聚焦 preview contract、renderer family、摘要规则和交互模型统一，不扩展到新的检测类型、默认远端 URL 富化、云能力或 Phase 4 的搜索能力本身。

</domain>

<decisions>
## Implementation Decisions

### Unified Preview Contract

- **D-01:** 列表、详情和后续检索必须共享同一语义来源，继续以 Rust analysis 结果作为 subtype 与 metadata 的权威源，前端不再重新解释语义。
- **D-02:** 统一工作要落在 preview contract 与 renderer family 层，而不只是 badge、label 或零散 UI 修补。
- **D-03:** 统一规则覆盖所有当前已识别类型，但优先把核心体验打磨在 JSON、URL、颜色、代码、命令这几类开发者高频内容上。
- **D-04:** 同一条内容在列表、详情、检索三个场景里必须保持同一 semantic type 与 preview intent，只允许信息密度不同。

### List Summary Density

- **D-05:** 列表预览保持紧凑摘要，不升级为完整语义卡片，避免破坏高密度浏览效率。
- **D-06:** 列表采用统一两层结构：第一层负责稳定 headline，第二层负责补充语义摘要。
- **D-07:** 列表项维持固定紧凑高度，长内容一律截断，不为单条内容撑高列表。
- **D-08:** 第二层信息优先展示语义摘要，而不是简单截取原始文本。
- **D-09:** 后续检索结果采用介于列表与详情之间的中等密度，而不是复用任一端的完整布局。

### Detail View Model

- **D-10:** 详情统一采用“语义主视图 + Raw 恒定可达”的模型。
- **D-11:** Raw 应作为统一视图切换条中的一个稳定入口，而不是零散按钮或只在失败时出现的特殊路径。
- **D-12:** 只暴露那些对用户有明显增益差异的备用视图；如果语义视图增益很弱，应默认自动退回 Raw。
- **D-13:** 继承 Phase 2 已锁定的 URL-first 规则：URL 条目主视图仍然是 URL 自身语义，远端 resolved 内容只允许作为备用视图，且不默认依赖远端抓取。

### Code And Command Workbench

- **D-14:** 代码和命令的主视图默认使用可编辑编辑器，作为本地临时工作台，而不是只读展示器。
- **D-15:** 该编辑能力只在本地临时生效，不回写当前历史条目的原始内容。
- **D-16:** 切换条目或关闭详情时，所有临时编辑状态自动重置。
- **D-17:** 复制动作默认复制当前编辑器里的内容，而不是强制复制原始内容。
- **D-18:** 代码与命令视图的辅助信息以语言提示、shell 提示等轻量上下文为主，不扩张成新的复杂能力。

### the agent's Discretion

- 统一视图切换条的具体视觉样式、层级和交互细节。
- headline / secondary summary 的字符阈值、截断方式和换行策略。
- 不同 subtype 的 renderer 布局细节，以及哪些弱语义类型只做 contract-level 统一、不新增重型专用视图。
- 列表、详情、检索三种密度之间的具体 spacing、字号和信息编排，只要不违背上面的语义合同。

</decisions>

<specifics>
## Specific Ideas

- “自动用最优格式展示，但 Raw 随时可看。”
- “列表看摘要，详情看完整，检索结果是中间密度。”
- 复制 JSON 时应自动格式化，并在代码编辑器里查看。
- 复制 URL 时主体验应是 URL 自身结构化信息，例如协议、host、path、query，而不是被远端返回内容抢占主视图。
- 复制颜色时应直接看到色块与其他开发友好的色值格式。
- 代码/命令的编辑只是临时工作台，用完即走，不改变历史中的原始内容。

</specifics>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Scope And Requirements

- `.planning/ROADMAP.md` — 定义 Phase 3 的目标、依赖关系和 5 条 success criteria。
- `.planning/PROJECT.md` — 定义产品边界、开发者优先、本地客户端约束，以及已继承的 Rust 权威与 URL-first / Raw 可达规则。
- `.planning/REQUIREMENTS.md` — 定义 PREV-01 到 PREV-05 的 phase scope，并明确默认远端 URL fetching 仍然 out of scope。
- `.planning/STATE.md` — 记录当前阶段位置和从前序 phase 继承下来的关键决策。

### Phase 2 Locked Preview Contracts

- `.planning/phases/02-analysis-contracts-versioned-detection/02-VERIFICATION.md` — 验证 analysis-first detail/store 消费、URL-first detail、fallback diagnostics、Raw 可达和滚动/高度合同。
- `.planning/phases/02-analysis-contracts-versioned-detection/02-UAT.md` — 记录用户对 URL detail、JSON Raw/code view、长内容滚动的真实预期与已关闭 gap。
- `.planning/phases/02-analysis-contracts-versioned-detection/02-06-SUMMARY.md` — 明确 URL 条目保持 `url_card` 主视图，resolved JSON/text/media 只作为 alternate views。
- `.planning/phases/02-analysis-contracts-versioned-detection/02-07-SUMMARY.md` — 明确非 immersive detail 必须保留 Raw 入口，并使用共享滚动列和 JsonRenderer 显式高度合同。

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `src/lib/preview/previewDescriptor.ts`: 当前 detail preview contract 的中心装配点，已经承载 primary kind、alternate views、inspector sections 与 action 生成。
- `src/lib/preview/entryPresentation.ts`: 已有 analysis-first helper、metadata 映射和 `buildEntrySemanticSummary()`，适合抽成跨列表/详情/检索共用的语义摘要入口。
- `src/components/DetailView/ContentRenderers/index.ts`: 已存在 `UnifiedTextRenderer`、`JsonRenderer`、`ColorRenderer`、`UrlRenderer` 等 renderer 家族，可作为统一 renderer 层的基础。
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`: 当前详情主视图分发入口，后续可收口成统一 renderer contract 的 detail 适配器。
- `src/components/DetailView/scene/AlternateViews.tsx`: 已有备用视图渲染路径，适合继续统一 Raw / resolved / alternate 视图的切换合同。
- `src/components/ClipboardList/ClipboardItem.tsx`: 当前列表项包含多套 subtype-specific 分支，是 Phase 3 列表预览统一的主要改造点。
- `src/stores/clipboardStore.ts`: 持有 `resolveEntryPreview()`、resolved payload 归一化与缓存，继续承担异步 preview resolution 所有权。
- `src/types/clipboard.ts`: 已定义 `PreviewDescriptor`、`PreviewKind`、`PreviewAlternateView` 与 analysis 类型，是 contract 统一的天然边界。

### Established Patterns

- analysis-first 已是既定模式：`entry.analysis` 决定 subtype 与 metadata，resolved payload 只能补充展示，不得反向改写语义。
- detail 侧已经形成 “descriptor -> scene -> renderer” 的结构，Phase 3 应统一扩展这条主链，而不是再引入平行分发逻辑。
- URL-first、Raw-only 可达、显式高度和共享滚动列是已验证过的前端合同，Phase 3 必须在此基础上统一，而不是推翻重做。
- 列表与详情目前仍有重复判断 subtype 的现象，说明 phase 重点不是新增 renderer，而是收敛 summary / intent / fallback contract。

### Integration Points

- 列表预览应从 `ClipboardItem.tsx` 的分支式实现，收口到共享 semantic summary / preview contract。
- 详情视图应继续以 `buildPreviewDescriptor()` 为统一入口，把主视图、备用视图和 Raw 入口形成稳定合同。
- 检索结果尚未实现，但应在 Phase 3 直接预留“同语义不同密度”的 summary contract，供 Phase 4 复用而不是重算。
- 临时编辑工作台最自然的挂载点在 detail renderer 层，不能写回 store 中的原始历史条目内容。

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

_Phase: 03-unified-developer-previews_
_Context gathered: 2026-03-28_
