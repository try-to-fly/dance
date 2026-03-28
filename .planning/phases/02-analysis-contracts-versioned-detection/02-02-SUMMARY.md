---
phase: 02-analysis-contracts-versioned-detection
plan: 02
subsystem: database
tags: [rust, sqlite, sqlx, tauri, analysis, history]
requires:
  - phase: 02-01
    provides: Rust authoritative `AnalysisSnapshot` contract 与 `TextAnalysisService`
provides:
  - `entry_analysis` companion table migration with JSON-valid constraints and version indexes
  - repository helpers for analysis UPSERT、joined history reads 和 stale-row selection
  - `ClipboardEntry.analysis` shared DTO plus authoritative-analysis-first history reads
affects: [02-03, 02-04, 02-05, entry_analysis, history-read-model]
tech-stack:
  added: []
  patterns:
    [
      companion-table analysis persistence,
      authoritative-analysis-first history reads,
      legacy field compatibility projection,
    ]
key-files:
  created: [src-tauri/src/analysis/repository.rs]
  modified:
    [
      src-tauri/src/analysis/contract.rs,
      src-tauri/src/analysis/mod.rs,
      src-tauri/src/database/mod.rs,
      src-tauri/src/models/mod.rs,
      src-tauri/src/state.rs,
      src-tauri/src/state_tests.rs,
      src/types/clipboard.ts,
    ]
key-decisions:
  - '`entry_analysis` 保持为 companion table，而不是把新的 analysis 语义字段继续塞回 `clipboard_entries`。'
  - '历史读取优先 join authoritative analysis，并把 legacy `content_subtype` / `metadata` 当成兼容投影，而不是继续当语义权威。'
  - '前后端共享 DTO 直接暴露 `analysis` snapshot，供后续 capture/runtime 和 detail preview 复用同一份 contract。'
patterns-established:
  - 'Joined read model: `AppState::get_clipboard_history()` 统一走 repository join helper，不再直接 `SELECT * FROM clipboard_entries`。'
  - 'Compatibility projection: authoritative `AnalysisMetadata` 会在返回给前端前投影成 legacy metadata JSON，保证现有 UI 可以渐进迁移。'
requirements-completed: [DETE-02, DETE-03, DETE-04]
duration: 13 min
completed: 2026-03-28
---

# Phase 02 Plan 02: Companion Persistence Summary

**`entry_analysis` 现在已经成为 analysis 的持久化权威，历史读取也能把 authoritative snapshot 和 legacy 兼容字段一起返回给前端**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-28T03:05:57Z
- **Completed:** 2026-03-28T03:18:57Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- 在 SQLite migration 中新增 `entry_analysis` companion table，落下 `contract_version`、`analysis_version`、JSON-valid constraints、外键和版本索引。
- 新增 `src-tauri/src/analysis/repository.rs`，提供 analysis UPSERT、joined history read 和 stale-row selector，为后续 rebuild/service 接线铺平路径。
- 把 `ClipboardEntry.analysis` 接到 Rust 和 TypeScript 共享合同上，并让 history read model 以 authoritative analysis 优先、legacy fallback 次之的规则返回数据。

## Task Commits

Each task was committed atomically:

1. **Task 1: 落 `entry_analysis` schema、repository helper 和共享 Rust 读模型** - `83499f1` (feat)
2. **Task 2: 暴露前端共享 `analysis` snapshot 类型** - `96576bf` (feat)

## Files Created/Modified

- `src-tauri/src/analysis/repository.rs` - companion-table UPSERT、joined history read 和 stale selector helper。
- `src-tauri/src/database/mod.rs` - `entry_analysis` migration、JSON-valid constraints、外键和索引，以及对应数据库测试。
- `src-tauri/src/models/mod.rs` - 给 `ClipboardEntry` 增加 `analysis` 字段和 compatibility attach helper。
- `src-tauri/src/state.rs` - `get_clipboard_history()` 改成 authoritative-analysis-first 的 joined read model。
- `src-tauri/src/state_tests.rs` - 覆盖 authoritative analysis 优先与 legacy fallback 共存的回归测试。
- `src/types/clipboard.ts` - 定义 `EntryAnalysisSnapshot`、diagnostics 和 typed metadata 的前端共享类型。
- `src-tauri/src/analysis/contract.rs` - 增加 subtype/status string mapping 和 legacy metadata projection helper。
- `src-tauri/src/analysis/mod.rs` - 导出 repository helper 给 state/runtime 后续计划复用。

## Decisions Made

- 继续把 raw clipboard payload 留在 `clipboard_entries`，让 `entry_analysis` 独立承载可重建的语义版本信息，避免再次混淆“原始数据”和“分析结果”。
- 让 Rust 在 joined read 时把 authoritative metadata 投影回 legacy JSON，而不是要求前端在本计划里同步重写所有 renderer。
- 提前暴露 stale selector helper，即使本计划还不接 Preferences UI，也为 `02-05` 的 rebuild 命令保留最短接线路径。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] repository helper 在接入 rebuild 前会被 Rust `-D warnings` 判定为 dead code**

- **Found during:** Task 1 提交
- **Issue:** `upsert_entry_analysis(...)` 和 `list_stale_entry_ids(...)` 在 `02-02` 完成时还没全部进入生产链路，但后续 `02-03` / `02-05` 已经依赖它们。
- **Fix:** 在 `src-tauri/src/analysis/repository.rs` 顶部添加最小范围 `#![allow(dead_code)]`，只压住这一阶段的过渡态警告。
- **Files modified:** `src-tauri/src/analysis/repository.rs`
- **Verification:** `cd src-tauri && cargo test --no-run`、`cargo test test_database_merges_analysis_fields -- --nocapture`
- **Committed in:** `83499f1`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** 仅处理了 Phase 2 波次推进中的编译阻塞，没有改变 companion-table 主线，也没有扩 scope。

## Issues Encountered

- `git add` 过程中两次遇到残留 `.git/index.lock`，但锁文件都在重试前消失，没有造成代码丢失或提交污染。
- 运行并行 `cargo test` 时依旧会看到构建目录锁等待，这是当前仓库体量下的正常现象，不影响本 plan 的验证结论。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `02-03` 现在可以直接把 capture/runtime save flow 接到 `upsert_entry_analysis(...)`，不需要再设计 companion-table 结构。
- `02-04` 已经有 `ClipboardEntry.analysis` 作为 authoritative source，可以开始把 detail/store 从 legacy subtype inference 收回来。
- `02-05` 可以直接消费 `list_stale_entry_ids(...)` 做 version-aware rebuild，而不需要重新设计 stale-row 选择规则。

## Self-Check

PASSED
