---
status: partial
phase: 01-capture-reliability-storage-cohesion
source: [01-VERIFICATION.md]
started: 2026-03-27T15:37:48Z
updated: 2026-03-28T02:10:04Z
---

## Current Test

human feedback recorded; accepted with partial coverage

## Tests

### 1. Real Clipboard Stop/Restart Smoke

expected: 第一条复制内容在重启后仍可见，停止监听后的第二条不会被偷偷入库
result: passed - 用户确认测试通过

### 2. Marker-First Capture Smoke

expected: 普通文本进入历史；auto-generated / concealed / remote clipboard 场景不会进入历史
result: partial - 用户确认普通文本会进入历史，auto-generated / concealed / remote clipboard 场景暂无可复现测试步骤

### 3. Legacy Install Migration Smoke

expected: 旧安装中的历史、图片资产和日志查看器都能在新路径权威下继续工作，不出现路径失配
result: skipped - 用户明确表示当前里程碑不再考虑兼容旧 case

## Summary

total: 3
passed: 1
issues: 0
pending: 1
skipped: 1
blocked: 0

## Gaps

- Marker-first 的 auto-generated / concealed / remote clipboard 手工 smoke 仍未完成，但用户已接受当前 partial 覆盖继续推进。
