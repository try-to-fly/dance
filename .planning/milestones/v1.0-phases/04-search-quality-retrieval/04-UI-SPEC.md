---
phase: 04
slug: search-quality-retrieval
status: draft
shadcn_initialized: true
preset: default-slate-cssvars
created: 2026-03-29
---

# Phase 04 — UI Design Contract

> Visual and interaction contract for retrieval-focused UI in Phase 04. This spec preserves the current desktop shell, shadcn/Tailwind token system, and the Phase 03 semantic preview contract while adding a denser, explainable retrieval state.

---

## Design System

| Property          | Value                                                                                                                                                         |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tool              | `shadcn` 已初始化并落在仓库内，沿用现有 repo-local 组件封装；来源：`components.json`、`src/components/ui/*`                                                   |
| Preset            | `default` style + `slate` baseColor + CSS variables；来源：`components.json`（`npx shadcn info` 于 2026-03-29 受本机 npm cache 权限阻塞，因此以仓库配置为准） |
| Component library | Radix UI primitives + repo-local shadcn wrappers；来源：`src/components/ui/`                                                                                  |
| Icon library      | `lucide-react`；来源：`src/App.tsx`、`src/components/SearchBar/SearchBar.tsx`、`src/components/ClipboardList/ClipboardItem.tsx`                               |
| Font              | 主字体 `IBM Plex Sans`；等宽字体 `JetBrains Mono` / `IBM Plex Mono` fallback；来源：`src/index.css`                                                           |

---

## Spacing Scale

Declared values (must be multiples of 4):

| Token | Value | Usage                                          |
| ----- | ----- | ---------------------------------------------- |
| xs    | 4px   | 高亮片段内边距、细小图标与文本间距             |
| sm    | 8px   | 过滤条 chip 间距、紧凑控件横向间距             |
| md    | 12px  | 结果卡内部纵向 gap、过滤条内边距、列表项内边距 |
| lg    | 16px  | 默认组件间距、搜索与过滤模块分隔               |
| xl    | 24px  | 卡片主体 padding、状态卡内容留白               |
| 2xl   | 32px  | 大块空状态与无结果状态垂直留白                 |
| 3xl   | 48px  | 状态页顶部/底部主留白                          |
| 4xl   | 64px  | 页面级安全留白上限，不用于结果卡内部           |

Exceptions: 保留当前桌面密度下的固定高度，且全部仍为 4 的倍数：搜索与紧凑 filter 控件高 `36px`，大桌面高 `40px`；retrieval 结果卡高 `156px`，大桌面高 `164px`；`load more` 哨兵高 `68px`。

---

## Typography

| Role                        | Size | Weight | Line Height |
| --------------------------- | ---- | ------ | ----------- |
| Meta / Match Reason         | 11px | 400    | 1.45        |
| Secondary Summary / Snippet | 12px | 400    | 1.5         |
| Body / Input / Filter Label | 14px | 400    | 1.5         |
| Heading / State Title       | 16px | 600    | 1.25        |

补充约束：

- 检索结果 `headline` 使用 `14px / 600 / 1.4`，不新增第五档字号。
- `json_structured`、`code_workbench`、`command_workbench`、`base64_summary` 在 `headline` 与 `snippet` 上切到等宽字体，但字号仍使用上表四档。
- 输入框 placeholder、过滤器默认态、无结果正文都使用 `14px / 400`，不下探到 12px，避免可读性被桌面高密度压坏。

---

## Color

| Role            | Value                                                           | Usage                                                                                                  |
| --------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Dominant (60%)  | `hsl(170 45% 97%)` / dark `hsl(193 28% 9%)`                     | 工作区主背景、搜索壳层、列表大面背景；来源：`src/index.css --background`                               |
| Secondary (30%) | `hsl(180 24% 93%)` + `hsl(0 0% 100%)` / dark `hsl(193 24% 13%)` | 结果卡、过滤条容器、空状态卡、次级胶囊与 hover surface；来源：`--secondary`、`--card`                  |
| Accent (10%)    | `hsl(176 84% 33%)` / dark `hsl(172 75% 44%)`                    | 搜索 focus ring、激活 filter、选中结果卡左轨与边框、literal highlight、加载 spinner；来源：`--primary` |
| Destructive     | `hsl(0 78% 58%)` / dark `hsl(0 72% 52%)`                        | 仅用于删除单条、清空历史、错误 banner 的强调 icon/边框；来源：`--destructive`                          |

Accent reserved for:

- 搜索主输入聚焦态与 pending spinner。
- 激活中的 type/source/favorites/recency filter。
- retrieval 结果选中态左侧竖条、选中边框、当前行 hover 的高优先反馈。
- 查询 literal match 的高亮底色与文字强调。
- `load more` 与首次 retrieval loading 的 spinner。

Accent never reserved for:

- 时间戳、来源应用、复制次数等被动元信息。
- 默认 badge、未激活 filter、空结果正文。
- 删除、清空等 destructive 操作。

---

## Copywriting Contract

Canonical locale: `zh-CN`。其他语言要保持语义和节奏一致，不按字面对齐长度。

| Element                                  | Copy                                                         |
| ---------------------------------------- | ------------------------------------------------------------ |
| Search placeholder                       | `搜索内容、URL host、JSON key、命令或应用...`                |
| Primary CTA (empty history)              | `开始监听`                                                   |
| Reset CTA (no results / retrieval error) | `清除筛选`                                                   |
| Empty state heading                      | `暂无剪切板记录`                                             |
| Empty state body                         | `开始监听后，新的复制内容会出现在这里。`                     |
| No results heading                       | `未找到匹配结果`                                             |
| No results body                          | `尝试缩短关键词、切换类型，或清除来源、收藏与时间筛选。`     |
| Initial retrieval loading                | `正在搜索剪贴板...`                                          |
| Load more                                | `继续加载匹配结果...`                                        |
| Inline retrieval error                   | `搜索暂时不可用，请稍后重试。`                               |
| Full retrieval error body                | `可以先清除筛选回到最近记录；如果问题持续，再检查应用日志。` |
| Delete single entry                      | `删除`，无额外 modal；仅保留 destructive 菜单样式与危险色    |
| Clear history confirmation               | `确定要清空所有剪切板历史记录吗？此操作不可恢复。`           |

文案约束：

- 搜索是 live retrieval，不使用“提交”“执行搜索”这类按钮文案。
- `No results` 先说明“没找到”，再给下一步动作，避免只给空白或技术错误。
- `Match reason` 使用短标签，不出现“score”“rank”“BM25”等实现词。
- `Snippet` 不重复整段原文，不超过两行，不用句号堆砌。

---

## Registry Safety

| Registry        | Blocks Used                                                                   | Safety Gate                                              |
| --------------- | ----------------------------------------------------------------------------- | -------------------------------------------------------- |
| shadcn official | `button`、`input`、`card`、`badge`、`select`、`dropdown-menu`、`context-menu` | 已检查仓库内现有实现，2026-03-29；无第三方 registry 引入 |
| third-party     | none                                                                          | 不适用；2026-03-29 未声明第三方 registry                 |

---

## Retrieval State Scope

retrieval state 在以下任一条件成立时启用：

- `searchTerm` 非空。
- `type` 不为 `all`。
- `source app` 不为 `all`。
- `favorites` 为激活态。
- `recency` 不为 `all time`。

进入 retrieval state 后，以下约束同时生效：

- 左侧列表切到 `retrieval` 信息密度，不再沿用普通 chronological list 的紧凑卡片高度。
- 结果卡继续复用 `buildPreviewSummary(entry, 'retrieval')` 的 `semanticType`、`previewIntent`、`headline`、`secondarySummary`。
- 右侧 detail pane 不改变布局层级，不新增 retrieval-only detail 视图。
- 搜索输入、过滤条、结果列表共享同一 query model，不允许前端再做独立二次 includes 过滤作为视觉真相。

---

## Search And Filter Layout

### Layout Hierarchy

| Surface           | Placement                                    | Size Contract                                                                    | Priority   |
| ----------------- | -------------------------------------------- | -------------------------------------------------------------------------------- | ---------- |
| Search input      | 顶部工具条主位，位于 app icon 与设置按钮之间 | 宽度 `1fr`，最小 `320px`，最大 `760px`；高度 `36px`，`>=1200px` 高度 `40px`      | Highest    |
| Type filter       | 与搜索输入同一行，固定跟在输入右侧           | 宽度 `160px`，`>=1200px` 宽 `176px`；高度与输入一致                              | High       |
| Source app filter | 左侧列表卡内部，sticky 过滤条第一位          | `140px` 到 `160px` 的 compact select；高度 `36px`                                | Medium     |
| Favorites filter  | sticky 过滤条第二位                          | 单个 toggle pill，最小宽 `88px`，高 `36px`                                       | Medium     |
| Recency filter    | sticky 过滤条第三位，靠右                    | 四档 segmented pills：`全部` / `24 小时` / `7 天` / `30 天`；单个 pill 高 `36px` | Medium-low |

### Priority Rules

- 主输入永远是第一视觉锚点，任何状态下都不降级为次级控件。
- `type` 是最高优先级过滤器，因为它直接映射当前已验证的 semantic type / subtype contract。
- `source app` 是第二优先级，因为它能高效缩小开发者工作流里的工具上下文。
- `favorites` 是第三优先级，必须是一个一键切换的 binary filter，不进二级菜单。
- `recency` 是第四优先级，用于加速最近回查，不抢占 type/source 的横向空间。

### Responsive Preservation

- 在当前桌面断点内，搜索输入与 `type` 保持同一行，不移入弹层。
- 若左侧列表宽度不足以完整容纳 sticky filter strip，首先压缩 `recency` 为 dropdown，其次压缩 `source app` 文案宽度；`type` 与 `favorites` 不隐藏。
- 不允许把所有 filters 收进一个 “More filters” 菜单，这会破坏 retrieval 的快速扫描节奏。

---

## Interaction Rhythm

| Interaction       | Contract                                                                                                                                       |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Free-text query   | 继续使用现有 `200ms` debounce + `useDeferredValue`；来源：`SearchBar.tsx`                                                                      |
| Filter changes    | `type`、`source app`、`favorites`、`recency` 全部即时提交，不再额外 debounce                                                                   |
| Pending feedback  | query 提交后 `150ms` 以内不显示 loading chrome；超过 `150ms` 在搜索输入右侧显示 spinner；超过 `400ms` 且当前无结果时才切换到居中 loading state |
| Result continuity | query/filter 更新期间保留上一次结果，直到新结果返回；不得先清空列表再重绘                                                                      |
| Selection         | 若之前选中的条目仍在新结果里，保持选中；否则自动选中首条结果，沿用当前 store 行为                                                              |
| Scroll reset      | 新 query 或任一 filter 改变时，结果列表滚动重置到顶部；仅 `load more` 保留当前位置                                                             |
| Infinite scroll   | 继续在滚动到列表高度 `90%` 时触发 `load more`，并使用底部哨兵，不阻塞已加载结果                                                                |
| Motion            | hover、选中、highlight 和 banner 过渡维持现有 `200ms` 级别；遵守 `prefers-reduced-motion`，不新增弹跳或 spring                                 |

交互禁令：

- 不使用“按回车才触发搜索”的表单节奏。
- 不在每次 query 变化时把右侧 detail 强制清空。
- 不在 retrieval state 中增加新的 modal、popover 向导或全屏遮罩。

---

## Retrieval Result Row Contract

### Row Shell

| Property  | Contract                                                                   |
| --------- | -------------------------------------------------------------------------- |
| Height    | `156px`，`>=1200px` 为 `164px`                                             |
| Shape     | 沿用现有圆角卡片语言：默认 `18px`，大桌面 `20px`                           |
| Selection | 左侧 `4px` accent rail + 轻度 accent border/ring；不整卡纯色填充           |
| Padding   | 卡片内容 `12px 12px 12px 12px`，大桌面 `12px 14px`                         |
| Actions   | 右上角仍保留更多操作按钮；retrieval state 不新增 inline destructive button |

### Content Hierarchy

每个 retrieval row 固定为四层信息，自上而下不可交换：

1. 元信息层：类型 badge、时间、来源应用、收藏星标、复制次数。
2. 主摘要层：`summary.headline`，单行，最高对比度。
3. 次摘要层：`summary.secondarySummary`，单行，说明 semantic context。
4. 检索解释层：`snippet` + `match reason`，解释“为什么命中”。

### Summary Contract

| Field              | Requirement                                                                                                                     |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------- |
| `headline`         | 必填，直接来自 `buildPreviewSummary(entry, 'retrieval').headline`；单行截断；不允许因为 query 改写 headline 文义                |
| `secondarySummary` | 必填，直接来自 `buildPreviewSummary(entry, 'retrieval').secondarySummary`；单行截断；作为 semantic summary，而不是 match reason |
| Font treatment     | `code_workbench`、`command_workbench`、`json_structured`、`base64_summary` 使用等宽字体；其他类型使用主字体                     |

### Snippet Contract

| Rule             | Contract                                                                                                                                               |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Presence         | 只有在存在 free-text query 且 backend 能返回具体命中上下文时显示；仅靠 filters 命中时可省略                                                            |
| Length           | 最多两行，总字符建议不超过 `140`；超出使用尾部省略                                                                                                     |
| Source priority  | 1. literal structured token 命中上下文；2. 原始文本匹配行；3. 归一化 token 上下文；4. 若只有 filter 命中则不显示                                       |
| Field preference | URL 先 `host/path`；JSON 先 `key path`；command 先 `command name` 或命中行；code 先命中行；color 先 alternate format；plain text/markdown 先正文命中行 |
| Tone             | 使用 `12px / 400 / 1.5`，颜色为 `muted-foreground`，不高于 `secondarySummary` 的视觉权重                                                               |

### Match Reason Contract

| Rule       | Contract                                                                                                                                                |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Presence   | retrieval state 必须展示至少一个 `match reason`；free-text query 优先展示 query 命中原因，其次才是 filter 命中原因                                      |
| Format     | 单行低强调 chip 或 inline pill，最多显示 `2` 个 reason；其余收为 `+N`                                                                                   |
| Vocabulary | 仅允许使用：`文本`、`URL Host`、`URL Path`、`JSON Key`、`命令名`、`颜色格式`、`来源应用`、`收藏`、`最近 24 小时`、`最近 7 天`、`最近 30 天`、`模糊匹配` |
| Ordering   | Structured exact > raw substring > fuzzy > filter-only                                                                                                  |
| Prohibited | 不显示分数、权重、SQL 字段名、token ID、`LIKE` / `FTS` / `BM25` 等实现词                                                                                |

### Highlight Contract

- 仅对 literal exact / substring 命中做高亮，允许落在 `headline` 或 `snippet`，不默认高亮 `secondarySummary`。
- `模糊匹配` 只通过 `match reason` 呈现，不伪造高亮区间。
- 单条结果最多显示 `3` 个 highlight ranges；超过后只保留前 `3` 个最强命中。
- highlight 使用 accent-tinted 背景 + 正文前景色，不使用纯红、纯黄或整段底色。

---

## Retrieval States

| State                         | Trigger                                     | UI Contract                                                                         | Copy / CTA                                                                                                     |
| ----------------------------- | ------------------------------------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Empty history                 | 历史为空，且无任何结果可展示                | 沿用现有居中空状态卡；图标、标题、正文、单主按钮                                    | `暂无剪切板记录` / `开始监听后，新的复制内容会出现在这里。` / CTA `开始监听`                                   |
| No results                    | 有历史，但当前 query/filter 返回 0 项       | 不显示空白列表；在列表卡内居中显示 no-results card，并保留顶部搜索与 filters 可操作 | `未找到匹配结果` / `尝试缩短关键词、切换类型，或清除来源、收藏与时间筛选。` / CTA `清除筛选`                   |
| Initial retrieval loading     | 当前无可复用结果，且 retrieval 请求正在进行 | 显示居中 spinner card，不渲染伪结果 skeleton                                        | `正在搜索剪贴板...`                                                                                            |
| Refresh with existing results | 已有旧结果，新 query/filter 请求进行中      | 保留旧结果，搜索输入显示 spinner；不全量清空列表                                    | 无额外正文                                                                                                     |
| Load more                     | 已有结果，向下滚动触发追加加载              | 在列表底部显示 `68px` 哨兵行；结果仍可继续浏览                                      | `继续加载匹配结果...`                                                                                          |
| Inline retrieval error        | 已有旧结果，但本次 refresh 失败             | 在 sticky filter strip 下方显示 `36px` inline banner，不替换旧结果                  | `搜索暂时不可用，请稍后重试。` + 次动作 `清除筛选`                                                             |
| Full retrieval error          | 无旧结果，且 retrieval 请求失败             | 用 full-state card 替换列表内容，保留顶部搜索与 filters                             | `搜索暂时不可用，请稍后重试。` / `可以先清除筛选回到最近记录；如果问题持续，再检查应用日志。` / CTA `清除筛选` |

状态切换约束：

- `Empty history` 与 `No results` 必须区分开；前者说明“没有任何记录”，后者说明“有记录但当前没命中”。
- `Load more` 永远是增量状态，不可遮挡已加载结果。
- `Inline retrieval error` 只在存在 last-good-results 时使用；没有 last-good-results 时必须升级为 `Full retrieval error`。

---

## Implementation Notes For Planner / Executor

- `TypeFilter` 继续复用现有 taxonomy，但 Phase 04 要把它并入统一 retrieval query model，而不是停留在 `getFilteredEntries()` 的前端二次过滤。
- `source app`、`favorites`、`recency` 的视觉壳层应与当前 `SearchBar` / `TypeFilter` 保持同一玻璃感、边框透明度和圆角体系。
- retrieval row 的 `headline` / `secondarySummary` 必须继续来自 `buildPreviewSummary(entry, 'retrieval')`；query-specific `snippet` 与 `match reason` 只能附加在下层，不得重写 semantic summary。
- 空状态、无结果、错误态继续沿用当前 `Card` 语言，不引入新的 illustration system。

---

## Source Notes

| Source                                                                                    | Decisions Used                                                                                                                      |
| ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `04-CONTEXT.md`                                                                           | Rust/SQLite retrieval authority、统一 query model、filters 范围、结果需解释命中原因、复用 `buildPreviewSummary(entry, 'retrieval')` |
| `components.json`                                                                         | shadcn 已初始化；`default` style、`slate` baseColor、CSS variables                                                                  |
| `tailwind.config.js` + `src/index.css`                                                    | 当前颜色 token、圆角、字体与暗色模式基础                                                                                            |
| `src/App.tsx`                                                                             | 顶部搜索 + type filter 的现有布局锚点，保持桌面视觉语言                                                                             |
| `src/components/SearchBar/SearchBar.tsx`                                                  | `200ms` debounce、`useDeferredValue`、清空按钮位置                                                                                  |
| `src/components/TypeFilter/TypeFilter.tsx` + `src/lib/clipboardFilters.ts`                | type filter 的现有控件样式与 taxonomy                                                                                               |
| `src/components/ClipboardList/ClipboardItem.tsx` + `ClipboardList.tsx` + `EmptyState.tsx` | 当前列表卡视觉语言、元信息层、空状态壳层、load-more 哨兵模式                                                                        |
| `src/lib/preview/previewSummary.ts` + `previewSummary.test.ts`                            | retrieval summary 只能增加信息量，不改变 semantic type / preview intent                                                             |
| This UI-SPEC defaults                                                                     | source app/favorites/recency 的具体顺序、pending threshold、retrieval row 高度、no-results/error/loading-more 文案                  |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
