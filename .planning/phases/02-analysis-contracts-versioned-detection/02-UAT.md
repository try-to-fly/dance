---
status: complete
phase: 02-analysis-contracts-versioned-detection
source: [02-01-SUMMARY.md, 02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-04-SUMMARY.md, 02-05-SUMMARY.md]
started: 2026-03-28T04:08:29Z
updated: 2026-03-28T04:18:55Z
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
  status: failed
  reason: "User reported: 我复制url，显示的是url类型。但是右侧详情里，展示了 JSON、树形视图。 下面展示的又是无效的json。 在这个下面展示的URL解析出来的card数据，协议、host、path等，这个才是预期内的"
  severity: major
  test: 2
  artifacts: []
  missing: []
- truth: "复制合法 JSON 后，详情页会进入 JSON 结构化预览，同时仍保留 Raw 视图可切换查看原始内容。"
  status: failed
  reason: "User reported: 复制的json可以正确识别，但是右侧详情切换 代码视图，内容为空。 为啥超长无法滚动查看（看起来像是详情card的通用bug，都是超长无法滚动查看。）"
  severity: major
  test: 3
  artifacts: []
  missing: []
