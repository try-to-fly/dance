# Phase 3: Unified Developer Previews - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 03-unified-developer-previews
**Areas discussed:** 列表预览密度、跨场景一致性、详情主视图与 Raw 入口、代码/命令编辑体验、统一覆盖范围

---

## 列表预览密度

| Option       | Description                                                    | Selected |
| ------------ | -------------------------------------------------------------- | -------- |
| 紧凑语义摘要 | 列表保持高密度，两层结构，固定高度，只显示 headline 与语义摘要 | ✓        |
| 完整语义卡片 | 列表直接展示更完整的 subtype-specific 卡片内容                 |          |
| 原文优先截断 | 主要展示原始文本截断，弱化 subtype-specific 摘要               |          |

**User's choice:** 选择紧凑语义摘要。
**Notes:** 用户明确表示列表应更方便快速查看，不需要展开成完整卡片；第二层内容要优先体现语义摘要，而不是简单截断原文。

---

## 跨场景一致性

| Option                                   | Description                                                            | Selected |
| ---------------------------------------- | ---------------------------------------------------------------------- | -------- |
| 只统一标签外观                           | 统一 badge、icon、label，列表和详情继续各自实现                        |          |
| 统一 preview contract 与 renderer family | 列表、详情、检索共享同一 semantic type 与 preview intent，只按密度区分 | ✓        |
| 仅统一核心几个类型                       | 只统一 JSON、URL、颜色，其他类型继续分散实现                           |          |

**User's choice:** 选择统一 preview contract 与 renderer family。
**Notes:** 用户明确要求统一不应停留在 badge/label 层，而是“同一语义，不同密度”；列表看摘要，详情看完整，检索结果用中间密度。

---

## 详情主视图与 Raw 入口

| Option                     | Description                                      | Selected |
| -------------------------- | ------------------------------------------------ | -------- |
| 语义主视图 + Raw 恒定可达  | 默认展示语义视图，Raw 作为统一切换条里的稳定入口 | ✓        |
| 语义主视图 + 独立 Raw 按钮 | 默认展示语义视图，Raw 通过独立按钮或次级操作打开 |          |
| 仅在失败时显示 Raw         | 正常匹配时不提供 Raw，只有解析失败才暴露         |          |

**User's choice:** 选择语义主视图 + Raw 恒定可达。
**Notes:** 用户强调“最优格式展示”同时必须随时能看 Raw；弱语义视图不值得占主位时，应自动退回 Raw。Raw 不应是零散特例，而应进入统一切换条。URL 继续保持 URL-first，协议、host、path 等结构信息才是预期主体验。

---

## 代码与命令编辑体验

| Option           | Description                                    | Selected |
| ---------------- | ---------------------------------------------- | -------- |
| 只读查看器       | 只允许查看格式化后的代码/命令，不允许编辑      |          |
| 临时可编辑工作台 | 详情默认进入可编辑编辑器，但编辑仅本地临时生效 | ✓        |
| 持久化编辑       | 允许在详情中直接修改并覆盖历史中的原始内容     |          |

**User's choice:** 选择临时可编辑工作台。
**Notes:** 用户明确说明编辑“仅仅是本地，不更新当前元素的原始内容，仅用来临时编辑”；切换条目或关闭详情时应重置；复制默认取当前编辑内容；辅助信息以语言或 shell 提示为主。

---

## 统一覆盖范围

| Option                  | Description                                                           | Selected |
| ----------------------- | --------------------------------------------------------------------- | -------- |
| 只覆盖本 phase 新增类型 | 只处理本 phase 明确改造到的少数类型                                   |          |
| 覆盖核心高频类型        | 只覆盖 JSON、URL、颜色、代码、命令五类                                |          |
| 覆盖所有当前已识别类型  | 用一套统一规则覆盖所有当前 recognized types，核心体验优先打磨高频类型 | ✓        |

**User's choice:** 选择覆盖所有当前已识别类型。
**Notes:** 用户额外补充“覆盖所有当前已识别类型”，但核心体验仍优先围绕 JSON、URL、颜色、代码、命令展开。

---

## the agent's Discretion

- 视图切换条的具体视觉样式与布局细节。
- 摘要长度阈值、截断策略和不同密度的具体排版。
- 哪些弱语义类型只做 contract-level 统一、不新增重型专属视图。
- renderer 内部具体布局，只要不违背 URL-first、Raw 恒定可达和显式滚动合同。

## Deferred Ideas

None
