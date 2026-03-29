---
status: resolved
phase: 02-analysis-contracts-versioned-detection
source: [02-01-SUMMARY.md, 02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-04-SUMMARY.md, 02-05-SUMMARY.md]
started: 2026-03-28T04:08:29Z
updated: 2026-03-28T08:11:17Z
---

## Current Test

[testing complete]

## Tests

### 1. Plain Text Baseline

expected: 复制一段普通文本后，条目会进入历史。打开详情时仍以文本/raw 为主，不会被错误识别成 URL、JSON、命令或其他结构化类型。
result: pass

### 2. URL Structured Preview

expected: 复制一个 HTTPS URL 后，条目会以 URL 类型显示。打开详情可看到 URL 的结构化信息，例如协议、主机和路径，并且可以执行打开链接操作。
result: issue
reported: "我复制url，显示的是url类型。但是右侧详情里，展示了 JSON、树形视图。 下面展示的又是无效的json。 在这个下面展示的URL解析出来的card数据，协议、host、path等，这个才是预期内的"
severity: major

### 3. JSON Structured Preview

expected: 复制合法 JSON 后，详情页会进入 JSON 结构化预览，同时仍保留 Raw 视图可切换查看原始内容。
result: issue
reported: "复制的json可以正确识别，但是右侧详情切换 代码视图，内容为空。 为啥超长无法滚动查看（看起来像是详情card的通用bug，都是超长无法滚动查看。）"
severity: major

### 4. Degraded Structured Fallback

expected: 复制损坏 JSON 或无效 base64 风格文本后，条目仍会进入历史。打开详情时 raw 文本仍可直接查看，并能看到 fallback 或 diagnostics 提示。
result: pass

### 5. Manual Rebuild From Preferences

expected: 打开 Preferences 的 system/cache 区域并触发重建后，界面会显示重建结果摘要，历史列表和预览缓存会刷新，且不会报错或改写原始内容。
result: pass

## Summary

total: 5
passed: 3
issues: 2
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "复制一个 HTTPS URL 后，条目会以 URL 类型显示。打开详情可看到 URL 的结构化信息，例如协议、主机和路径，并且可以执行打开链接操作。"
  status: resolved
  reason: "User reported: 我复制url，显示的是url类型。但是右侧详情里，展示了 JSON、树形视图。 下面展示的又是无效的json。 在这个下面展示的URL解析出来的card数据，协议、host、path等，这个才是预期内的"
  severity: major
  test: 2
  root_cause: "URL 条目的主预览判定把远端资源的 `previewKind` 放在了 URL 自身语义之前；当 resolve_url_preview 返回 `previewKind=json` 时，descriptor 会把主视图切到 JSON，而渲染器在缺少真实 JSON 对象时又回退到原始 URL 文本，最终显示为 JSON/树形视图加无效 JSON，同时底部仍保留正确的 URL 结构卡片。"
  resolution: "Closed by 02-06-PLAN.md and confirmed in desktop human verification on 2026-03-28."
  artifacts:
  - path: "src/lib/preview/previewDescriptor.ts"
    issue: "URL 条目在远端 `previewKind=json` 或存在 `jsonContent` 时直接把 `primaryKind` 提升为 `json`。"
  - path: "src/stores/clipboardStore.ts"
    issue: "URL 条目在 URL 专属解析前先写入 `resolved.textContent = entry.content_data`，为错误的 JSON 回退提供原始 URL 字符串。"
  - path: "src/components/DetailView/scene/PrimaryPreviewRenderer.tsx"
    issue: "JSON 主视图在没有真实 JSON 数据时回退使用 `resolvedData.textContent || content`，把 URL 字符串交给 JsonRenderer。"
  - path: "src/components/DetailView/DetailPreviewContract.test.tsx"
    issue: "现有契约测试把“URL + previewKind=json => primaryKind=json”当成正确行为，锁定了错误语义。"
    missing: []
    debug_session: ".planning/debug/url-detail-renders-json-tree.md"
- truth: "复制合法 JSON 后，详情页会进入 JSON 结构化预览，同时仍保留 Raw 视图可切换查看原始内容。"
  status: resolved
  reason: "User reported: 复制的json可以正确识别，但是右侧详情切换 代码视图，内容为空。 为啥超长无法滚动查看（看起来像是详情card的通用bug，都是超长无法滚动查看。）"
  severity: major
  test: 3
  root_cause: "普通 JSON 的 raw 备用视图虽然已生成，但 raw-only 场景被 DetailScene 和 AlternateViews 主动隐藏，导致用户拿不到 Raw 入口；用户只能使用主 JsonRenderer 的树/代码切换，而 JsonRenderer 把 Monaco 放到只有 `minHeight`、没有明确 `height` 的父容器里，同时详情左列也缺少公共 `overflow-y-auto` 滚动容器，最终表现为代码视图空白且超长内容无法滚动查看。"
  resolution: "Closed by 02-07-PLAN.md and confirmed in desktop human verification on 2026-03-28."
  artifacts:
  - path: "src/components/DetailView/scene/DetailScene.tsx"
    issue: "左侧主预览列被多层 `overflow-hidden` 包裹，且对 raw-only alternate views 做了显式隐藏。"
  - path: "src/components/DetailView/scene/AlternateViews.tsx"
    issue: "当唯一备用视图是 `raw` 时直接 `return null`，使 Raw 入口不可达。"
  - path: "src/components/DetailView/ContentRenderers/JsonRenderer.tsx"
    issue: "Monaco 编辑器使用 `height='100%'`，但父容器只有 `minHeight` 没有确定高度，浏览器里容易折叠为空白。"
  - path: "src/components/DetailView/DetailView.test.tsx"
    issue: "测试把 raw-only 场景隐藏备用视图当成预期，锁定了错误行为。"
    missing: []
    debug_session: ".planning/debug/json-raw-empty-no-scroll.md"
