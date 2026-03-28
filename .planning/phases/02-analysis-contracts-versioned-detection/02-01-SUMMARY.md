---
phase: 02-analysis-contracts-versioned-detection
plan: 01
subsystem: analysis
tags: [rust, tauri, serde, clipboard, detection]
requires:
  - phase: 01-capture-reliability-storage-cohesion
    provides: marker-first capture boundary、single-owner runtime 和稳定的本地存储生命周期
provides:
  - Rust authoritative `AnalysisSnapshot` contract
  - `TextAnalysisService` 统一分析入口
  - subtype corpus、precedence 与 fallback serialization contract tests
affects: [02-02, 02-03, 02-04, entry_analysis, preview-contract]
tech-stack:
  added: []
  patterns:
    [
      tagged analysis metadata contract,
      service-over-detector adapter,
      contract-first subtype regression tests,
    ]
key-files:
  created:
    [
      src-tauri/src/analysis/contract.rs,
      src-tauri/src/analysis/service.rs,
      src-tauri/src/analysis_contract_tests.rs,
    ]
  modified:
    [src-tauri/src/analysis/mod.rs, src-tauri/src/lib.rs]
key-decisions:
  - '先用 `AnalysisSnapshot` + `TextAnalysisService` 立住 Rust 语义边界，再把 companion table、capture path 和前端消费往这条线收敛。'
  - 'analysis 合同在还没接入生产路径前使用局部 `allow(dead_code)`/`allow(unused_imports)` 压住 `-D warnings`，避免为了过编译把半成品接口提前塞进错误位置。'
  - 'contract tests 只通过 `TextAnalysisService` 断言 subtype、precedence 和 fallback serialization，不再直接把 tuple detector 当作公开合同测试。'
patterns-established:
  - 'Analysis authority: Rust 侧所有后续 analysis 持久化和消费都围绕 `AnalysisSnapshot`，不是继续扩张 `content_subtype`/`metadata` 字符串接口。'
  - 'Subtype lock: URL/JSON/code/command/color/markdown/email/IP/timestamp/base64/plain_text 的语义与 precedence 必须先被具名测试锁死。'
requirements-completed: [DETE-01, DETE-02, DETE-04]
duration: 12 min
completed: 2026-03-28
---

# Phase 02 Plan 01: Analysis Contract Summary

**Rust 侧现在已经有一份可测试的 `AnalysisSnapshot` 合同和 `TextAnalysisService` 入口，Phase 2 后续实现不再需要围绕 tuple detector 和旧字符串字段打补丁**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-28T02:50:14Z
- **Completed:** 2026-03-28T03:02:34Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- 新增 `src-tauri/src/analysis/` 模块，定义 `AnalysisSubtype`、`AnalysisStatus`、tagged `AnalysisMetadata`、`AnalysisDiagnostic` 和 `AnalysisSnapshot`。
- 建立 `TextAnalysisService` 作为 Rust authoritative text-analysis 入口，先以 adapter 方式复用现有 `ContentDetector`，但不把 tuple 结果继续当作公开合同。
- 用独立 contract tests 锁定 11 类 subtype、歧义 precedence 和 fallback diagnostics serialization。

## Task Commits

每个任务都做了原子提交：

1. **Task 1: 建立 analysis 模块与 authoritative contract/service 骨架** - `4116904` (feat)
2. **Task 2: 新增 contract tests 并锁定 subtype/preference/fallback serialization** - `0680541` (test)

## Files Created/Modified

- `src-tauri/src/analysis/contract.rs` - 定义 Phase 2 的稳定 analysis contract、typed metadata 和 diagnostics 结构。
- `src-tauri/src/analysis/service.rs` - 提供 `TextAnalysisService` 统一入口，并把现有 detector 适配进 contract。
- `src-tauri/src/analysis/mod.rs` - 导出 analysis 域模块。
- `src-tauri/src/analysis_contract_tests.rs` - 锁定 subtype corpus、precedence 和 fallback serialization。
- `src-tauri/src/lib.rs` - 注册 `analysis` 模块与 contract tests。

## Decisions Made

- 先建立 contract/service seam，再接 schema 和 capture，避免把 Phase 2 变成“边迁移边猜接口”的工作流。
- 当前 contract 尚未接到生产路径，所以用最小范围 `allow` 压住 `-D warnings`，等后续 plans 真正消费这些类型后再自然消化。
- contract tests 统一走 `TextAnalysisService`，不让 `ContentDetector::detect()` 的 tuple 继续泄漏成外部依赖。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] pre-commit 的 `-D warnings` 阻塞了尚未接线的 analysis contract 提交**

- **Found during:** Task 1 提交
- **Issue:** 新建的 contract/service 在本 plan 结束时还未接入生产路径，pre-commit 的 Rust 检查把 unused imports 和 dead code 当成错误。
- **Fix:** 在 `src-tauri/src/analysis/mod.rs`、`src-tauri/src/analysis/contract.rs`、`src-tauri/src/analysis/service.rs` 上添加局部 `allow`，只压住当前 Phase 2 过渡态的编译噪音。
- **Files modified:** `src-tauri/src/analysis/mod.rs`, `src-tauri/src/analysis/contract.rs`, `src-tauri/src/analysis/service.rs`
- **Verification:** `cd src-tauri && cargo test --no-run` 通过，随后两个 task commit 都通过 Rust 检查。
- **Committed in:** `4116904`

---

**Total deviations:** 1 auto-fixed
**Impact on plan:** 仅处理过渡态编译阻塞，没有扩张范围，也没有改变 Phase 2 的 contract 主线。

## Issues Encountered

- 最初的 Wave 1 executor 在 `git add` 阶段碰到 `.git/index.lock` 后停住，留下了未提交的 Task 1 代码；后续由主线程接管并完成验证与提交。
- `cargo test` 输出里持续有来自现有代码的 3 条 `new()` dead_code warning，但不影响本 plan 的测试和提交通过。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `AnalysisSnapshot` 和 `TextAnalysisService` 已经就位，Wave 2 可以直接开始接 `entry_analysis` companion table、repository 和 joined read model。
- 当前 contract 已经锁住 subtype 与 fallback 语义，后续 02-02/02-03 不应再改动 variant 名称或 status 取值。

## Self-Check

PASSED
