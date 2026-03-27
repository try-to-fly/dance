---
status: partial
phase: 01-capture-reliability-storage-cohesion
source: [01-VERIFICATION.md]
started: 2026-03-27T15:37:48Z
updated: 2026-03-27T15:37:48Z
---

## Current Test

awaiting human testing

## Tests

### 1. Real Clipboard Stop/Restart Smoke

expected: 第一条复制内容在重启后仍可见，停止监听后的第二条不会被偷偷入库
result: pending

### 2. Marker-First Capture Smoke

expected: 普通文本进入历史；auto-generated / concealed / remote clipboard 场景不会进入历史
result: pending

### 3. Legacy Install Migration Smoke

expected: 旧安装中的历史、图片资产和日志查看器都能在新路径权威下继续工作，不出现路径失配
result: pending

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
