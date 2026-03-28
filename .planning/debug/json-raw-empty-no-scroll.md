---
status: diagnosed
trigger: '复制合法 JSON 后，详情页会进入 JSON 结构化预览，同时仍保留 Raw 视图可切换查看原始内容。实际：复制的json可以正确识别，但是右侧详情切换代码视图，内容为空。为啥超长无法滚动查看（看起来像是详情card的通用bug，都是超长无法滚动查看。）'
created: 2026-03-28T04:22:49Z
updated: 2026-03-28T04:27:04Z
---

## Current Focus

hypothesis: 当前实现同时违背了 Test 3 的两个关键点。第一，普通 JSON 只保留 raw 备用视图时，会被 DetailScene/AlternateViews 显式隐藏，导致根本没有 Raw 切换入口。第二，用户退而求其次使用主 JsonRenderer 的树/代码切换时，代码模式和长内容又被错误的高度/滚动设计卡住：JsonRenderer 父容器只有 minHeight 而 Monaco 用 100% 高度，DetailScene 左列也没有公共滚动容器。
test: 核对 1) raw-only 是否被代码和测试显式隐藏；2) JSON 主预览是否确实进入 JsonRenderer；3) JsonRenderer 与 DetailScene 的高度/overflow 是否足以解释代码空白和超长不滚动。
expecting: 会同时看到 raw-only hide 的明确分支、主 JSON 一定走 JsonRenderer、以及缺失确定高度与左列滚动容器的布局实现。
next_action: 输出 ROOT CAUSE FOUND 诊断，不做代码修改。

## Symptoms

expected: 复制合法 JSON 后，详情页默认进入 JSON 结构化预览，同时保留 Raw/代码视图切换，切换后应显示原始文本；超长内容应可完整滚动查看。
actual: JSON 能被识别，但右侧详情切换到代码视图时内容为空；超长内容在详情卡片里无法滚动查看。
errors: 无明确报错；用户报告为“代码视图内容为空”“超长无法滚动查看”。
reproduction: 运行 .planning/phases/02-analysis-contracts-versioned-detection/02-UAT.md 的 Test 3，复制合法 JSON，打开右侧详情并切换代码视图，再尝试查看超长内容。
started: Phase 02 UAT Test 3 报出，2026-03-28。

## Eliminated

## Evidence

- timestamp: 2026-03-28T04:24:53Z
  checked: src/lib/preview/previewDescriptor.ts
  found: 合法 JSON 条目会先走 primaryKind='json'，并在 entry.content_data 存在时总是追加 key='raw' 的 alternate view；若还有 resolvedData.jsonContent，则再追加 key='resolved-json'。
  implication: “Raw 视图内容为空”不是因为数据未进入 descriptor，Raw 原文在渲染前的数据层已经保留下来。

- timestamp: 2026-03-28T04:24:53Z
  checked: src/components/DetailView/DetailPreviewContract.test.tsx
  found: 契约测试明确断言“JSON 条目默认进入结构化主预览并保留 raw 备用视图”，说明当前产品意图就是主 JSON + Raw 备用切换。
  implication: 用户在 UAT 中没法正常看到/使用 Raw，更可能是视图可达性或布局问题，而不是产品合同本身没实现。

- timestamp: 2026-03-28T04:24:53Z
  checked: src/components/DetailView/scene/DetailScene.tsx
  found: detail 卡片、shell 和 content wrapper 多层都使用了 overflow-hidden；右侧 inspector 单独加了 overflow-y-auto，但左侧主预览列没有任何公共滚动容器，且非 immersive 布局下左列不是 flex-1 滚动区，只是 space-y 堆叠。
  implication: 左列只要主预览 + 备用视图总高度超过可用高度，就会被共同祖先裁掉，超长内容和位于下方的 Raw 卡片都会变成“看得到一部分/完全到不了但也滚不动”。

- timestamp: 2026-03-28T04:24:53Z
  checked: src/components/DetailView/ContentRenderers/JsonRenderer.tsx
  found: JsonRenderer 把代码/树视图内容放在 style={{ minHeight: contentHeight }} 的容器里，代码视图中的 MonacoEditor 却固定使用 height='100%'；该父容器没有明确 height。
  implication: Monaco 的 100% 高度缺少确定的参照高度，真实浏览器布局下容易折叠或测量为 0，从而出现“切到代码视图内容为空”；同一容器的 tree 分支也缺少确定滚动高度，长 JSON 不会在这里形成稳定滚动区。

- timestamp: 2026-03-28T04:26:10Z
  checked: src/components/DetailView/scene/DetailScene.tsx, src/components/DetailView/scene/AlternateViews.tsx, src/components/DetailView/DetailView.test.tsx
  found: DetailScene 先用 showAlternateViews 显式隐藏“只有 raw 一个备用视图”的情况；AlternateViews 也对 views.length === 1 && key === 'raw' 直接 return null；而 DetailView 测试还把“raw-only 场景下隐藏备用视图”当成期望行为。
  implication: 普通 JSON 在没有 resolved-json/resolved-text 补充视图时，Raw 入口会被整个产品实现主动拿掉，这正是 Test 3 要求“仍保留 Raw 视图”却无法满足的直接原因。

- timestamp: 2026-03-28T04:26:10Z
  checked: src/components/DetailView/scene/PrimaryPreviewRenderer.tsx
  found: kind === 'json' 时，PrimaryPreviewRenderer 一定把主详情渲染到 JsonRenderer，内容优先取 resolvedData.jsonContent，否则回退到 raw content。
  implication: 用户在详情里能看到并操作的“树形/代码视图”按钮，确实来自 JsonRenderer，而不是 Raw 备用视图；因此空白代码视图的责任点就在 JsonRenderer 本身。

## Resolution

root_cause:
普通 JSON 的 Raw 入口在当前实现里被刻意隐藏：descriptor 虽然会生成 raw alternate view，但 DetailScene/AlternateViews 对 raw-only 场景直接不渲染，和 Test 3 的“仍保留 Raw 视图”要求相冲突。用户因此只能使用主 JsonRenderer 的树/代码切换；而 JsonRenderer 的代码模式把 Monaco 放在只有 minHeight、没有明确 height 的父容器里，同时 DetailScene 左列又没有公共 overflow-y-auto 滚动区，最终表现为代码视图可切换但内容空白、长内容和下方备用视图都无法滚动到。
fix:
verification:
files_changed: []
