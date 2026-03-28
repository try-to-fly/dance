# Phase 03: Unified Developer Previews - Research

**Researched:** 2026-03-28
**Domain:** Tauri + React brownfield preview contract unification
**Confidence:** HIGH

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Claude's Discretion

- 统一视图切换条的具体视觉样式、层级和交互细节。
- headline / secondary summary 的字符阈值、截断方式和换行策略。
- 不同 subtype 的 renderer 布局细节，以及哪些弱语义类型只做 contract-level 统一、不新增重型专用视图。
- 列表、详情、检索三种密度之间的具体 spacing、字号和信息编排，只要不违背上面的语义合同。

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>

## Phase Requirements

| ID      | Description                                                                                                                                             | Research Support                                                                                                    |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| PREV-01 | User can inspect JSON entries in a formatted structured view and switch back to the raw representation                                                  | 继续复用 `JsonRenderer` + Raw 稳定入口，但把 JSON headline/summary contract 提升到 list/detail/search 共用语义层    |
| PREV-02 | User can inspect URL entries in a structured preview showing at least protocol, host, path, and query details without requiring default remote fetching | Detail 主路径必须改成本地 URL 结构卡片，不复用旧 `UrlRenderer` 的远端抓取逻辑；远端 resolved 只保留为可选 alternate |
| PREV-03 | User can inspect color entries with a visual swatch and alternate color formats suitable for development work                                           | 复用现有 `ColorRenderer` 与 `color-convert`，但补齐 list/search 摘要规则和组件测试                                  |
| PREV-04 | User can inspect code and command entries in a read-only developer-oriented view with preserved formatting and language or shell hints when available   | 以 CONTEXT.md 的 D-14..D-17 为准：detail 使用可编辑本地工作台；测试要锁定编辑重置和复制当前编辑内容的合同           |
| PREV-05 | User sees the same semantic type and preview intent for an entry across the list view, detail view, and follow-up retrieval flows                       | 需要新增 shared semantic model + density-aware summary contract，让 list/detail/search 都从同一语义核心派生         |

</phase_requirements>

## Project Constraints (from CLAUDE.md)

- 前端命令必须在项目根目录运行，Rust 命令必须在 `src-tauri/` 目录运行。
- 执行命令前要确认当前工作目录，不要在错误目录直接跑 `cargo` 或 `pnpm`。
- Phase 3 应继续基于当前 Tauri + React + Rust + SQLite brownfield 架构演进，不做脱离现有代码基础的重写。
- 产品约束仍是本地桌面客户端，不能把 preview 统一工作扩展成云同步、移动端或团队协作能力。
- `src/` 继续遵守根目录 Prettier / ESLint 约束；`src-tauri/` 继续遵守 `cargo fmt` / `cargo clippy`。
- 仓库变更应继续通过 GSD 工作流执行，不要绕开 phase 文档与执行上下文。

## Summary

Phase 03 不是“选哪个预览库”的问题，而是一个 brownfield contract 收敛问题。当前代码已经有一半正确形态：Rust analysis 是权威来源，`entryPresentation.ts` 已经承担 analysis-first 辅助职责，`previewDescriptor.ts` 已经承担 detail 语义装配，`JsonRenderer` / `UnifiedTextRenderer` / `ColorRenderer` 等 renderer family 也已经存在，Phase 2 还锁定了 URL-first、Raw 恒定可达、非 immersive detail 滚动与显式高度合同。真正缺的不是新 renderer，而是一个能同时服务 list/detail/search 的同步语义核心和密度适配层。

当前最大的规划风险有两个。第一，列表项 `ClipboardItem.tsx` 仍然直接按 `entry.content_subtype` 和 legacy metadata 做分支，和 detail 的 descriptor 主链是两套体系，这正是 PREV-05 会失真的根源。第二，生产路径里 URL detail 现在虽然是 URL-first，但 `resolveEntryPreview()` 仍会默认走 `resolveUrlPreview()` 远端解析链；与此同时，旧 `UrlRenderer` 还内置远端抓取与 FFprobe 逻辑，并且有独立测试，但它已经不是主路径。这意味着 Phase 3 必须显式区分“本地同步语义合同”和“可选异步 resolved alternate”，否则很容易把已锁定的 no-default-dependency-on-remote contract 再次稀释掉。

最稳的计划方向是：先把 `entryPresentation.ts` 提升成单一 `SemanticPreviewModel` 生产者，再从它派生两个投影，一个给 detail descriptor，一个给 list/search summary contract。这样可以先解决 semantic type / preview intent 一致性，再分别把 URL、颜色、代码、命令、JSON 的展示打磨到位，最后用轻量规则扫过 markdown、email、ip、timestamp、base64、plain_text、image、file 等所有当前 recognized types，而不需要把每种类型都做成重型专用卡片。

**Primary recommendation:** 在 `entryPresentation.ts` 上方建立单一 semantic preview model，并从它派生 `PreviewSummaryDescriptor(density)` 与现有 `PreviewDescriptor`；不要把 detail descriptor 直接下沉到列表，也不要把 legacy `UrlRenderer` 拉回主生产路径。

## Standard Stack

### Core

| Library                   | Version                                      | Purpose                                                        | Why Standard                                                                             |
| ------------------------- | -------------------------------------------- | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| React                     | `18.3.1` in repo, registry latest `19.2.4`   | 预览组件树、局部状态、renderer family 组合                     | 现有桌面前端根栈；Phase 3 不应混入 React 升级工作                                        |
| Zustand                   | `4.5.0` in repo, registry latest `5.0.12`    | 预览解析缓存、selectedEntry、searchTerm、clipboard event state | 现有 store 已持有 preview resolution 和 selection 所有权，统一 contract 应继续从这里消费 |
| `@monaco-editor/react`    | `4.7.0` in repo, registry latest `4.7.0`     | 代码、命令和 JSON code/raw 视图的本地工作台                    | 已接入本地主题、`value/onChange/beforeMount/height`，最适合继续承载 D-14..D-17           |
| `monaco-editor`           | `0.52.2` in repo, registry latest `0.55.1`   | Monaco runtime 与语言能力                                      | 当前项目版本已能满足 Phase 3；不要把 preview 重构和 Monaco 升级绑定                      |
| `react-json-view-lite`    | `2.5.0` in repo, registry latest `2.5.0`     | JSON 树视图                                                    | 轻量、只读、适合和 Monaco 形成 tree/code 双视图，而不是自建 JSON tree                    |
| `@tanstack/react-virtual` | `3.13.12` in repo, registry latest `3.13.23` | 高密度列表虚拟滚动                                             | 现有列表已采用；固定紧凑高度 + 两层摘要合同天然适配它                                    |

### Supporting

| Library                | Version                                   | Purpose                          | When to Use                                                               |
| ---------------------- | ----------------------------------------- | -------------------------------- | ------------------------------------------------------------------------- |
| `@radix-ui/react-tabs` | `1.0.4` in repo, registry latest `1.1.13` | 稳定、可访问的视图切换条         | 详情 Raw / semantic / resolved 切换，以及后续检索结果的轻量密度切换       |
| `color-convert`        | `3.1.0` in repo, registry latest `3.1.3`  | 颜色格式互转                     | 颜色 renderer 和 color summary 需要开发友好的 HEX / RGB / RGBA / HSL 对照 |
| `date-fns`             | `3.3.1` in repo, registry latest `4.1.0`  | 时间戳 headline / summary 归一化 | timestamp 类型只需要轻量格式化，不需要新时间库                            |

### Alternatives Considered

| Instead of                                     | Could Use                              | Tradeoff                                                                                                                 |
| ---------------------------------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| 新建一套 list-only preview helper              | 直接把 `PreviewDescriptor` 下沉到列表  | `PreviewDescriptor` 天生 detail-oriented，包含 actions / alternate views / payload，直接复用会把列表变成 detail 的劣化版 |
| 继续保留 `ClipboardItem.tsx` 里的 subtype 分支 | 共享 semantic model + density adapters | 继续分支会让 list/detail/search 漂移，尤其会重复解释 URL / color / timestamp 语义                                        |
| 复活旧 `UrlRenderer` 作为 URL 主路径           | 新建或改造纯本地 `UrlCardRenderer`     | 旧 `UrlRenderer` 默认抓远端文本和媒体元数据，和 D-13、PREV-02 的合同冲突                                                 |
| 自建 JSON tree / code viewer                   | `react-json-view-lite` + Monaco        | 自建 tree、copy、folding、布局处理都容易复刻一遍既有复杂度                                                               |
| 自写 tabs / toggle strip                       | Radix Tabs                             | 无障碍、焦点管理和键盘交互会白白重造一遍                                                                                 |

**Installation:**

```bash
# No new packages recommended for Phase 3.
# Reuse the repo-pinned dependencies already present in package.json.
```

**Version verification:** 2026-03-28 本地执行 `npm view`（使用 `npm_config_cache=/tmp/.npm-cache` 绕过当前机器 `~/.npm` 权限污染）核对结果：

- `@monaco-editor/react` latest `4.7.0` published `2025-02-13`
- `monaco-editor` latest `0.55.1` published `2025-11-20`; repo pin `0.52.2` published `2024-12-09`
- `react-json-view-lite` latest `2.5.0` published `2025-09-06`
- `@tanstack/react-virtual` latest `3.13.23` published `2026-03-16`; repo pin `3.13.12` published `2025-06-27`
- `@radix-ui/react-tabs` latest `1.1.13` published `2025-08-13`; repo pin `1.0.4` published `2023-05-26`
- `color-convert` latest `3.1.3` published `2025-11-14`; repo pin `3.1.0` published `2025-05-13`
- `date-fns` latest `4.1.0` published `2024-09-17`; repo pin `3.3.1` published `2024-01-22`
- `react` latest `19.2.4` published `2026-01-26`; repo pin `18.3.1` published `2024-04-26`
- `zustand` latest `5.0.12` published `2026-03-16`; repo pin `4.5.0` published `2024-01-20`

## Architecture Patterns

### Recommended Project Structure

```text
src/
├── lib/preview/
│   ├── entryPresentation.ts      # analysis-first semantic authority
│   ├── previewSummary.ts         # new: list/search density-aware summary contract
│   └── previewDescriptor.ts      # detail-only descriptor built from semantic core
├── components/ClipboardList/
│   ├── ClipboardList.tsx         # virtualization and selection only
│   └── ClipboardItem.tsx         # thin summary renderer, no subtype inference
└── components/DetailView/
    ├── scene/PrimaryPreviewRenderer.tsx   # detail-only dispatch from descriptor
    ├── scene/AlternateViews.tsx           # Raw / resolved tabs
    └── ContentRenderers/                  # renderer family, no store ownership
```

### Pattern 1: Semantic Core -> Density Adapters

**What:** 用一个同步、analysis-first 的 semantic model 承载 `semanticType`、`previewIntent`、`headline`、`secondarySummary`、metadata 和 raw 可达性，再由 list/search/detail 分别做密度投影。  
**When to use:** 所有 preview surface；这是 PREV-05 的核心合同。  
**Example:**

```typescript
// Source: local codebase pattern adapted from src/lib/preview/entryPresentation.ts
type PreviewDensity = 'list' | 'search';

interface SemanticPreviewModel {
  semanticType: string;
  previewIntent: string;
  headline: string;
  secondarySummary: string;
  rawContent: string | null;
  metadata: unknown;
}

interface PreviewSummaryDescriptor {
  semanticType: string;
  previewIntent: string;
  density: PreviewDensity;
  headline: string;
  secondarySummary: string;
}
```

### Pattern 2: Detail Stays Descriptor-Driven

**What:** 详情继续通过 `buildPreviewDescriptor()` 生成主视图、备用视图、inspector 和 actions，但它不再自己重新计算 headline/type/summary，而是消费 semantic core。  
**When to use:** 所有 detail 展示；尤其适合保留 Phase 2 的 URL-first、Raw 可达和显式高度合同。  
**Example:**

```typescript
// Source: local codebase pattern adapted from src/lib/preview/previewDescriptor.ts
const semantic = buildSemanticPreviewModel(entry);

return buildPreviewDescriptor({
  semantic,
  resolvedData,
  labels,
});
```

### Pattern 3: Summary Contract Must Be Synchronous

**What:** list 和后续 retrieval summary contract 只能依赖 `entry.analysis`、本地 metadata 和 raw 内容，不能依赖 `resolveEntryPreview()` 这类异步 resolved 数据。  
**When to use:** 列表首屏、高密度虚拟滚动和 Phase 4 检索结果。  
**Example:**

```typescript
// Source: local codebase pattern adapted from src/stores/clipboardStore.ts and src/components/ClipboardList/ClipboardList.tsx
const summary = buildPreviewSummary(entry, 'list');

// safe for virtualization: no async preview resolution required
return <ClipboardItem summary={summary} />;
```

### Pattern 4: URL Primary View Is Local, Resolved View Is Optional

**What:** URL 条目的 detail 主视图必须完全由 URL 自身语义和 analysis metadata 构成；远端 resolved JSON/text/media 只能作为 alternate views，而且 summary contract 不能依赖它存在。  
**When to use:** URL subtype 在 list/detail/search 三个表面都必须遵守。  
**Example:**

```typescript
// Source: local codebase pattern adapted from Phase 02 URL-first contract
if (semantic.semanticType === 'url') {
  primaryKind = 'url_card';
  alternateViews = [
    { key: 'raw', kind: 'raw', payload: semantic.rawContent },
    ...resolvedAlternates,
  ];
}
```

### Pattern 5: Search Density Is A Third Projection, Not A Third Semantics Source

**What:** 为 Phase 4 预留 `density: 'search'` 的 summary contract，让 retrieval 只增加信息密度和 query-context，不新增 subtype 推断逻辑。  
**When to use:** 现在先定义接口和测试；不要提前实现搜索能力本身。  
**Example:**

```typescript
// Source: local planning recommendation, derived from D-04 / D-09
const listSummary = buildPreviewSummary(entry, 'list');
const searchSummary = buildPreviewSummary(entry, 'search');

expect(searchSummary.semanticType).toBe(listSummary.semanticType);
expect(searchSummary.previewIntent).toBe(listSummary.previewIntent);
```

### Recommended Execution Order

1. 先把 `entryPresentation.ts` 提升成 semantic core，定义 `semanticType`、`previewIntent`、headline、secondary summary 与 density 合同。
2. 再新增 `previewSummary.ts` 并迁移 `ClipboardItem.tsx`，把列表从 subtype 分支式实现收口成统一两层摘要结构。
3. 然后修 detail 的 URL primary renderer，让 `url_card` 真正展示 protocol / host / path / query，而不是只显示 raw code block。
4. 接着补齐 JSON / color / code / command 的 summary 规则与 detail 测试，把高频体验先打磨完。
5. 最后扫过 markdown / email / ip / timestamp / base64 / plain_text / image / file，确保它们至少有正确的 semantic type、preview intent 和 density summary，不留孤儿分支。

### Coverage Strategy By Type

| Group                      | Types                                                        | Phase 3 Expectation                                                                                          |
| -------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| Heavy custom experience    | `json`, `url`, `color`, `code`, `command`                    | 独立 summary rule + detail renderer / workbench + contract tests                                             |
| Lightweight semantic cards | `timestamp`, `email`, `ip_address`, `markdown`, `plain_text` | 共享 summary contract + 现有轻量 renderer，不新增重型 view                                                   |
| Binary/derived text        | `base64`                                                     | 保持 semantic type = `base64`，summary 和 detail 都通过 decoded hint 补充 intent，但不改写 subtype authority |
| Non-text top-level         | image, file                                                  | 保留现有主路径，只把 headline / type label / density contract 统一到 shared summary 层                       |

### Anti-Patterns to Avoid

- **把 `PreviewDescriptor` 当列表 contract：** 它过于 detail-oriented，会把 actions、alternate payload 和 resolved async 依赖带进列表与检索。
- **让 list/search 等待远端 URL resolution：** 这会直接破坏 D-05..D-09 的高密度合同，也和 PREV-02 的本地结构化目标冲突。
- **在多个 surface 各自重新解析 subtype：** Phase 2 已经把 Rust analysis 锁成权威，前端再解析只会制造 drift。
- **继续保留 `ClipboardItem.tsx` 的 subtype-specific UI 分支：** 这会让新 contract 永远停留在“detail-only”。
- **把旧 `UrlRenderer` 当成现成答案：** 它是一个远端 enrichment 组件，不是当前锁定合同下的 URL 主视图。

## Don't Hand-Roll

| Problem                          | Don't Build                                              | Use Instead                                                                            | Why                                                                                          |
| -------------------------------- | -------------------------------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| 前端 subtype / metadata 语义判断 | `ClipboardItem`、detail、search 各自重新 parse raw 文本  | Rust `entry.analysis` + `getEntryAnalysisSubtype()` / `getEntryPresentationMetadata()` | 自定义判断会让 list/detail/search 漂移，丢掉 Phase 2 fallback diagnostics 与 precedence 合同 |
| JSON tree viewer                 | 自定义递归 JSON 树和折叠逻辑                             | `react-json-view-lite`                                                                 | 现成支持层级展开和样式；官方 README 也明确它不是编辑器，正好和 Monaco 互补                   |
| 代码 / 命令工作台                | `textarea` / `contenteditable` + 自写 copy / syntax hint | 现有 `UnifiedTextRenderer` + Monaco                                                    | 当前已经实现编辑、复制当前内容、主题切换与高度合同，复用成本最低                             |
| 视图切换条                       | 自写按钮组和焦点/键盘行为                                | Radix Tabs                                                                             | 无障碍、受控/非受控、键盘导航都已有约定，避免再造 UI 基础设施                                |
| 虚拟列表尺寸管理                 | subtype-specific 自定义滚动或复杂高度计算                | `@tanstack/react-virtual` + 固定紧凑高度合同                                           | 当前 list 已有虚拟化；Phase 3 应减少高度变异，而不是增加它                                   |
| URL 结构解析                     | 正则到处切协议/host/query                                | Rust analysis metadata + `URL` fallback                                                | URL 解析规则隐含边界很多，仓库和平台 API 已经有更可靠的来源                                  |

**Key insight:** 这个 domain 里最危险的“手搓”不是 UI，而是语义复制。只要 list/detail/search 各自重算 preview intent，PREV-05 就会退化成肉眼难发现的长期漂移。

## Common Pitfalls

### Pitfall 1: 列表仍在走 legacy subtype / metadata

**What goes wrong:** 列表 badge、headline 和 secondary summary 跟 detail 不一致，尤其是 URL、color、timestamp 这类结构化类型。  
**Why it happens:** `ClipboardItem.tsx` 目前直接用 `entry.content_subtype` 和 `parseContentMetadata(entry.metadata)`，没有完整复用 analysis-first helper。  
**How to avoid:** 让列表只消费 shared summary contract，不再直接碰 raw subtype branches。  
**Warning signs:** type badge 和 detail 不一致；同一条内容在 list/detail 显示不同 headline。

### Pitfall 2: 把旧 `UrlRenderer` 重新接回生产主路径

**What goes wrong:** URL detail 会默认抓远端文本、媒体和 FFprobe 元数据，重新模糊 URL-first / no-default-remote-dependency 合同。  
**Why it happens:** 旧组件看起来“功能更全”，而且有自己的测试，容易被误判成现成答案。  
**How to avoid:** 把 URL 主路径收敛为纯本地 `url_card` renderer；resolved 数据只经 `alternateViews` 进入 detail。  
**Warning signs:** URL 主视图出现“图片预览”“视频预览”或远端文本，而不是协议/host/path/query。

### Pitfall 3: 让 summary contract 依赖异步 resolvedData

**What goes wrong:** 列表和后续检索结果变成抖动、占位和网络状态混合体，虚拟滚动与高密度浏览体验一起退化。  
**Why it happens:** 直接复用 detail preview resolution 逻辑最省事，但它是异步且 remote-capable 的。  
**How to avoid:** summary contract 必须是同步、analysis-first、可纯函数生成。  
**Warning signs:** `ClipboardItem` 开始需要 loading state、preview cache 或 effect。

### Pitfall 4: 用 detail payload 设计 retrieval density

**What goes wrong:** Phase 4 检索结果会被迫复用 detail 布局或 raw alternate 模型，导致搜索结果过重。  
**Why it happens:** 当前代码只有 detail descriptor，没有 search-summary descriptor，容易把“已有类型”当“正确抽象”。  
**How to avoid:** Phase 3 现在就定义 `density: 'list' | 'search'` 的 summary contract，但不实现搜索功能。  
**Warning signs:** planner 开始讨论“搜索结果要不要显示 alternate views / action buttons”。

### Pitfall 5: 编辑型 workbench 状态泄漏

**What goes wrong:** 用户编辑代码/命令后切换条目，旧编辑内容残留到新条目，或者复制行为回退到 raw content。  
**Why it happens:** 编辑状态被提升到 store 或和 summary/retrieval contract 混在一起。  
**How to avoid:** 编辑状态只留在 detail renderer 局部，随 `content` 变化和组件卸载自动重置。  
**Warning signs:** `selectedEntry` 变化后，Monaco 里还留着上一条内容。

### Pitfall 6: 以“测试存在”为准，而不是以“生产路径存在”为准

**What goes wrong:** 旧 `UrlRenderer.test.tsx` 全绿，但真正在线上运行的是另一条 URL 渲染链路，导致假安全感。  
**Why it happens:** brownfield 组件多条平行路径共存。  
**How to avoid:** Phase 3 要么收编旧测试到新生产路径，要么明确废弃旧组件与旧测试。  
**Warning signs:** 测试断言的是 fetch/FFprobe 行为，但 detail 主路径根本不 render `UrlRenderer`。

## Code Examples

Verified patterns from official sources:

### Accessible View Switch Strip

```tsx
// Source: https://www.radix-ui.com/primitives/docs/components/tabs
<Tabs.Root value={activeKey} onValueChange={setActiveKey}>
  <Tabs.List>
    <Tabs.Trigger value="semantic">Semantic</Tabs.Trigger>
    <Tabs.Trigger value="raw">Raw</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="semantic">{semanticView}</Tabs.Content>
  <Tabs.Content value="raw">{rawView}</Tabs.Content>
</Tabs.Root>
```

### Virtualized Dense List Shell

```tsx
// Source: https://tanstack.com/virtual/latest/docs/api/virtualizer
const virtualizer = useVirtualizer({
  count: entries.length,
  getScrollElement: () => parentRef.current,
  estimateSize: () => 168,
  overscan: 4,
});
```

### Monaco As A Controlled Local Workbench

```tsx
// Source: https://www.npmjs.com/package/@monaco-editor/react
<MonacoEditor
  value={editedContent}
  onChange={(value) => setEditedContent(value || '')}
  beforeMount={defineMonacoThemes}
  height={editorHeight}
/>
```

### JSON Tree View For Read-Only Structure

```tsx
// Source: https://github.com/AnyRoad/react-json-view-lite
<JsonView data={parsedJson} shouldExpandNode={(level) => level < 3} style={jsonViewStyle} />
```

## State of the Art

| Old Approach                                        | Current Approach                                                     | When Changed                          | Impact                                          |
| --------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------- | ----------------------------------------------- |
| List/detail 各自按 subtype 分支渲染                 | 一个 semantic core 派生多个 density / detail 投影                    | 应在 Phase 03 落地                    | 才能真正满足 PREV-05，而不是局部对齐 badge      |
| URL 远端 MIME / preview kind 可能抢主视图           | URL 自身语义固定为主视图，远端内容只做 alternate                     | Phase 02 locked on 2026-03-28         | Phase 03 只能继续强化，不能回退                 |
| Raw-only alternate 在非 immersive detail 中会被隐藏 | Raw 在非 immersive detail 恒定可达                                   | Phase 02 plan 07 completed 2026-03-28 | Phase 03 必须沿用同一视图切换条合同             |
| JSON / code renderer 依赖父级 `100%` 高度           | 显式高度 shell + 内部滚动区                                          | Phase 02 plan 07 completed 2026-03-28 | Phase 03 新 renderer / summary 不应打破高度边界 |
| URL renderer 在组件内部自抓远端内容                 | preview resolution 应由 store 管理，且 summary contract 不可依赖远端 | current brownfield tension            | 这是当前最需要 planner 显式规避的实现风险       |

**Deprecated/outdated:**

- `ClipboardItem.tsx` 当前的 subtype-specific preview 分支：它是 Phase 3 需要被替换的对象，不应继续扩张。
- 旧 `UrlRenderer` 的默认抓取主路径思路：与已锁定的 URL-first / Raw / no-default-remote-dependency contract 不一致。

## Open Questions

1. **Phase 3 是否顺手收紧当前 eager URL resolution 行为？**
   - What we know: `resolveEntryPreview()` 对 URL 会默认调用 `resolveUrlPreview()`，而 `resolveUrlPreview()` 可以走远端解析和前端 fallback 抓取。
   - What's unclear: planner 是只要求“summary/detail 主合同不依赖远端”，还是要在本 phase 把默认 eager resolved fetch 本身一起收口。
   - Recommendation: 至少把 list/search contract 完全做成同步且与远端无关；若 URL detail 主路径需要调整，优先保证 `url_card` 可独立完成 PREV-02。

2. **旧 `UrlRenderer` 是迁移、重写还是删除？**
   - What we know: 它有自己的测试，但 detail 主路径并未使用它。
   - What's unclear: 团队更偏好保留一部分内部解析逻辑，还是直接清理掉这条 dead-ish path。
   - Recommendation: 作为 Phase 3 后段任务处理。先让生产路径正确，再决定是收编测试还是移除组件。

3. **PREV-04 的“read-only” requirement wording 如何处理？**
   - What we know: `REQUIREMENTS.md` 写的是 read-only，但 `03-CONTEXT.md` 的 D-14..D-17 已锁定 detail 使用可编辑本地工作台。
   - What's unclear: planner 是否需要显式记录“以 CONTEXT 为准”的验证修订说明。
   - Recommendation: 在 PLAN.md 的 verification 里直接以 CONTEXT 锁定合同为准，并补一条说明避免实现/验收口径不一致。

## Environment Availability

| Dependency          | Required By                            | Available | Version    | Fallback                                                        |
| ------------------- | -------------------------------------- | --------- | ---------- | --------------------------------------------------------------- |
| Node.js             | Frontend build/test, Vitest            | ✓         | `v24.13.0` | —                                                               |
| `pnpm`              | Frontend install/test commands         | ✓         | `10.0.0`   | `npm` 可用于 registry 查询，但不建议替代项目脚本                |
| `cargo`             | 如需补跑 Rust 回归或全量 phase gate    | ✓         | `1.91.0`   | Phase 03 以前端为主，可不作为日常 quick run 必需                |
| `rustc`             | Rust test/build toolchain completeness | ✓         | `1.91.0`   | —                                                               |
| npm registry access | Standard Stack version verification    | ✓         | live       | 本机 `~/.npm` 权限异常时使用 `npm_config_cache=/tmp/.npm-cache` |

**Missing dependencies with no fallback:**

- None.

**Missing dependencies with fallback:**

- `~/.npm` 当前存在 root-owned cache 污染，会让裸 `npm view` 失败；version verification 可改用临时 cache 继续执行。

## Validation Architecture

### Test Framework

| Property           | Value                                                                                                                                                                                                                                                                                                              |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Framework          | Vitest `4.1.2` + Testing Library                                                                                                                                                                                                                                                                                   |
| Config file        | `vitest.config.ts`                                                                                                                                                                                                                                                                                                 |
| Quick run command  | `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx` |
| Full suite command | `pnpm test`                                                                                                                                                                                                                                                                                                        |

**Current baseline:** 2026-03-28 已实际执行上述 quick run，`5` 个 test files、`37` 个 tests 全部通过。

### Phase Requirements → Test Map

| Req ID  | Behavior                                                                                      | Test Type                 | Automated Command                                                                                                                                | File Exists? |
| ------- | --------------------------------------------------------------------------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ------------ |
| PREV-01 | JSON 结构化主视图 + Raw 可切回                                                                | component + contract      | `pnpm exec vitest run src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx` | ✅           |
| PREV-02 | URL detail 主视图展示 protocol / host / path / query，且 summary/main view 不依赖默认远端抓取 | contract + component      | `pnpm exec vitest run src/components/DetailView/scene/PrimaryPreviewRenderer.test.tsx`                                                           | ❌ Wave 0    |
| PREV-03 | 颜色 swatch + 多种开发友好格式                                                                | component                 | `pnpm exec vitest run src/components/DetailView/ContentRenderers/ColorRenderer.test.tsx`                                                         | ❌ Wave 0    |
| PREV-04 | 代码/命令本地工作台保持格式、提示、重置与复制当前编辑内容                                     | component                 | `pnpm exec vitest run src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx`                                                   | ❌ Wave 0    |
| PREV-05 | list/detail/search 保持同一 semantic type 与 preview intent，仅密度不同                       | contract + list component | `pnpm exec vitest run src/lib/preview/previewSummary.test.ts src/components/ClipboardList/ClipboardItem.test.tsx`                                | ❌ Wave 0    |

### Sampling Rate

- **Per task commit:** `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx`
- **Per wave merge:** `pnpm exec vitest run src/lib/preview/entryPresentation.test.ts src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx`
- **Phase gate:** `pnpm test`

### Wave 0 Gaps

- [ ] `src/lib/preview/previewSummary.test.ts` — 锁定 `density: 'list' | 'search'` 的 shared summary contract，并覆盖 PREV-05
- [ ] `src/components/ClipboardList/ClipboardItem.test.tsx` — 锁定两层固定紧凑结构、长内容截断和核心五类摘要显示
- [ ] `src/components/DetailView/scene/PrimaryPreviewRenderer.test.tsx` 或新 `UrlCardRenderer.test.tsx` — 锁定 URL 结构卡片与 no-default-remote-dependency contract
- [ ] `src/components/DetailView/ContentRenderers/ColorRenderer.test.tsx` — 锁定 swatch、格式互转与复制行为
- [ ] `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx` — 锁定本地工作台编辑、切条目重置、复制当前编辑内容

## Sources

### Primary (HIGH confidence)

- Local codebase: `src/lib/preview/entryPresentation.ts`, `src/lib/preview/previewDescriptor.ts`, `src/components/ClipboardList/ClipboardItem.tsx`, `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`, `src/components/DetailView/scene/AlternateViews.tsx`, `src/stores/clipboardStore.ts`
- Local tests: `src/lib/preview/entryPresentation.test.ts`, `src/components/DetailView/DetailPreviewContract.test.tsx`, `src/components/DetailView/scene/AlternateViews.test.tsx`, `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`, `src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx`
- Local planning artifacts: `.planning/phases/03-unified-developer-previews/03-CONTEXT.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, `.planning/ROADMAP.md`, `.planning/phases/02-analysis-contracts-versioned-detection/02-VERIFICATION.md`, `.planning/phases/02-analysis-contracts-versioned-detection/02-UAT.md`, `.planning/phases/02-analysis-contracts-versioned-detection/02-06-SUMMARY.md`, `.planning/phases/02-analysis-contracts-versioned-detection/02-07-SUMMARY.md`
- TanStack Virtual API: https://tanstack.com/virtual/latest/docs/api/virtualizer
- Radix Tabs docs: https://www.radix-ui.com/primitives/docs/components/tabs
- Monaco React official package page: https://www.npmjs.com/package/@monaco-editor/react
- Monaco React repo: https://github.com/suren-atoyan/monaco-react
- `react-json-view-lite` repo: https://github.com/AnyRoad/react-json-view-lite
- npm registry verification executed locally on 2026-03-28 via `npm view` for `react`, `zustand`, `@monaco-editor/react`, `monaco-editor`, `react-json-view-lite`, `@tanstack/react-virtual`, `@radix-ui/react-tabs`, `color-convert`, and `date-fns`

### Secondary (MEDIUM confidence)

- None.

### Tertiary (LOW confidence)

- None.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - 主要基于现有仓库依赖，辅以 npm registry 与官方文档核对
- Architecture: HIGH - 直接来自当前生产路径、Phase 2 锁定合同和已通过的本地测试
- Pitfalls: HIGH - 大多来自代码中直接观察到的平行路径、UAT 历史和未接线组件

**Research date:** 2026-03-28
**Valid until:** 2026-04-27
