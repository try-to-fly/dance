---
status: complete
phase: 02-analysis-contracts-versioned-detection
source: [02-VERIFICATION.md]
started: 2026-03-28T05:43:16Z
updated: 2026-03-28T08:11:17Z
---

## Current Test

[testing complete]

## Tests

### 1. Live Clipboard Capture

expected: 新历史项显示稳定 subtype；畸形输入降级为 plain_text 且保留 diagnostics；raw 内容仍可查看
result: pass

### 2. Desktop Rebuild + Detail UI Sanity

expected: 重建结果摘要出现并刷新历史；URL 仍以 url_card 为主视图；JSON Raw 入口可见，长内容列与 Monaco 代码视图可滚动
result: pass

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
