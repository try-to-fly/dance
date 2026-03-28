---
status: diagnosed
trigger: '我复制url，显示的是url类型。但是右侧详情里，展示了 JSON、树形视图。下面展示的又是无效的json。在这个下面展示的URL解析出来的card数据，协议、host、path等，这个才是预期内的。'
created: 2026-03-28T04:22:44Z
updated: 2026-03-28T04:24:23Z
---

## Current Focus

hypothesis: 已确认，URL 条目被“远端内容预览”错误覆盖了主视图语义：只要远端解析结果给出 `previewKind=json`，详情页就把 URL 主预览改成 JSON
test: 已完成 descriptor / store / renderer / tauri command / contract tests 全链路核对
expecting: 主预览误入 JSON，且底部仍保留 URL inspector / alternate view，这与用户报告完全一致
next_action: return root cause only

## Symptoms

expected: 复制一个 HTTPS URL 后，条目会以 URL 类型显示。打开详情可看到 URL 的结构化信息，例如协议、主机和路径，并且可以执行打开链接操作。
actual: 条目列表显示为 URL 类型，但右侧详情的主区域先展示 JSON / 树形视图，并提示无效 JSON；更下面同时还能看到 URL 解析出来的协议、host、path 等卡片，这部分才是预期。
errors: 无控制台错误已知；界面上出现“无效 JSON”提示。
reproduction: 按 .planning/phases/02-analysis-contracts-versioned-detection/02-UAT.md 的 Test 2，复制一个 HTTPS URL，打开详情面板观察主预览与 inspector 区域。
started: Phase 02 UAT（2026-03-28）中发现；是否更早存在未知。

## Eliminated

## Evidence

- timestamp: 2026-03-28T04:24:23Z
  checked: src/stores/clipboardStore.ts resolveEntryPreview
  found: URL 条目在进入 URL 专属解析前，先被写入 `resolved.textContent = entry.content_data`；随后再 merge `resolveUrlPreview()` 的结果。
  implication: 即使没有可展示的 JSON 对象，后续 JSON 渲染器也会拿到原始 URL 字符串作为后备内容。

- timestamp: 2026-03-28T04:24:23Z
  checked: src/lib/preview/previewDescriptor.ts resolvePrimaryKind
  found: `subType === 'url'` 时，只要 `resolvedData.jsonContent !== undefined` 或 `resolvedData.url.previewKind === 'json'`，主预览就直接返回 `json`，而不是 `url_card`。
  implication: URL 条目的详情主视图会被远端内容类型升级覆盖，丢失“URL 条目应优先展示 URL 卡片”的语义。

- timestamp: 2026-03-28T04:24:23Z
  checked: src/components/DetailView/scene/PrimaryPreviewRenderer.tsx 和 src/components/DetailView/ContentRenderers/JsonRenderer.tsx
  found: `kind === 'json'` 时，渲染器优先用 `jsonContent`，否则退回 `resolvedData.textContent || content`；`JsonRenderer` 对非 JSON 文本会显示“invalid json”。
  implication: 当 URL 被判成 JSON 但实际没有可解析 JSON 对象时，主区域就会出现用户看到的“JSON/树形视图 + 无效 JSON”。

- timestamp: 2026-03-28T04:24:23Z
  checked: src/lib/preview/previewDescriptor.ts 和 src/components/DetailView/scene/DetailScene.tsx
  found: 对 `subType === 'url'`，descriptor 无论主预览是什么，都会追加 `url-structure` alternate view，并且 URL inspector 会单独显示在右侧。
  implication: 这解释了为什么同一个 URL 条目会同时出现错误的 JSON 主视图和正确的协议/host/path 结构信息。

- timestamp: 2026-03-28T04:24:23Z
  checked: src-tauri/src/commands.rs resolve_url_preview / preview_kind_from_mime 与 src/components/DetailView/DetailPreviewContract.test.tsx
  found: 后端会按响应 `content-type` 或 URL 路径把远端资源标成 `PreviewKind::Json`；前端契约测试还显式断言“URL + previewKind=json => primaryKind=json，并保留 url-structure”。
  implication: 这不是偶发数据问题，而是 Phase 02 里被写进前后端契约的设计性误判。

## Resolution

root_cause: URL 条目的主预览判定把“远端资源内容的 preview kind”放在了“条目本身是 URL”之前；当 resolve_url_preview 返回 `previewKind=json`（来自 content-type、+json 或路径推断）时，descriptor 会把主视图切到 JSON，而渲染器又会在缺少真实 JSON 对象时回退到原始 URL 文本，最终呈现为“JSON/树形视图 + 无效 JSON”，同时右侧仍显示正确的 URL 结构信息。
fix:
verification: 通过 UAT 描述、descriptor 判定、store 解析、JSON 渲染器回退逻辑、右侧 inspector 渲染方式以及现有契约测试交叉验证，现象与机制一致。
files_changed: []
