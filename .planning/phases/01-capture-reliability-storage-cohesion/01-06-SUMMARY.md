---
phase: 01-capture-reliability-storage-cohesion
plan: 06
subsystem: infra
tags: [rust, tauri, app-paths, logging, clipboard]
requires:
  - phase: 01-02
    provides: AppPaths 权威路径、legacy migration 和 temp-root 路径测试骨架
provides:
  - monitor 图片 metadata 回读改走 AppPaths 相对资源解析
  - 日志读取与清理命令统一走 AppPaths::log_dir()/clipboard-app.log
  - CAPT-04 gap regression tests 覆盖 nested imgs 路径与日志路径 helper
affects: [CAPT-04, clipboard monitor, log viewer, local storage lifecycle]
tech-stack:
  added: []
  patterns:
    [
      AppPaths-owned asset resolution,
      shared log path helpers,
      temp-root filesystem regression testing,
    ]
key-files:
  created: []
  modified:
    [
      src-tauri/src/clipboard/processor.rs,
      src-tauri/src/clipboard/monitor.rs,
      src-tauri/src/commands.rs,
      src-tauri/src/app_paths_tests.rs,
    ]
key-decisions:
  - '不把 `AppPaths` 直接塞进 `ClipboardMonitor` 状态，而是让 monitor 通过 `ContentProcessor` 复用 `resolve_relative_asset_path()`，维持现有构造签名。'
  - '日志命令先抽成 `app_log_file_path` / `read_log_content_in` / `clear_log_file_in` helper，再让 Tauri command 委托给 helper，这样 temp-root 测试可以直接覆盖生产路径合同。'
  - '日志文件名固定收口到 `clipboard-app.log`，与 `tauri_plugin_log` 当前 `file_name: Some(\"clipboard-app\")` 的落盘命名保持一致。'
patterns-established:
  - 'Storage authority: 任何 `imgs/...` 相对资源回读都必须经由 `AppPaths::resolve_relative_asset_path()` 或其封装 helper，不允许手写 legacy 根目录。'
  - 'Log command contract: 日志读取与清理统一复用 `app_log_file_path()`，缺失日志返回空字符串，清理缺失日志保持 no-op。'
requirements-completed: [CAPT-04]
duration: 5 min
completed: 2026-03-27
---

# Phase 01 Plan 06: CAPT-04 Gap Closure Summary

**AppPaths 现在已经真正接管 monitor 图片 metadata 回读和日志命令路径，CAPT-04 的最后两条生产旁路也有 temp-root 回归测试锁定**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-27T15:25:28Z
- **Completed:** 2026-03-27T15:30:26Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- `ContentProcessor` 现在保存 `Arc<AppPaths>`，并向 monitor 暴露 `resolve_relative_asset_path()` 封装，图片 metadata 回读不再手写 `dirs::config_dir()/clipboard-app/...`。
- `get_log_content` 和 `clear_logs` 改成委托 `AppPaths::log_dir()/clipboard-app.log` 的共享 helper，删除了 `~/Library/Logs/com.dance.app/dance.log` 硬编码。
- 新增 CAPT-04 gap 回归测试，分别锁定 nested `imgs/...` 路径解析和日志 helper 的 temp-root 行为。

## Task Commits

代码改动以一个原子提交落地：

1. **Task 1 + Task 2: CAPT-04 gap tests 与 AppPaths 生产接线** - `91c9a43` (fix)

## Files Created/Modified

- `src-tauri/src/clipboard/processor.rs` - 为图片处理器保存 `AppPaths`，并提供相对资源路径解析 helper。
- `src-tauri/src/clipboard/monitor.rs` - 把图片 metadata 文件大小回读改为复用 processor 的 `resolve_relative_asset_path()`。
- `src-tauri/src/commands.rs` - 新增日志路径 helper，并让 `get_log_content` / `clear_logs` 从 `AppState.paths` 解析 `clipboard-app.log`。
- `src-tauri/src/app_paths_tests.rs` - 新增 nested `imgs/...` 路径解析测试和日志路径 helper 测试。

## Decisions Made

- 继续复用 `ContentProcessor` 作为图片资产路径的最近权威，而不是额外扩大 `ClipboardMonitor::new(...)` 的构造依赖。
- 日志 helper 设计成纯函数接口，优先服务 temp-root regression test，而不是把路径逻辑埋进 Tauri command 体内。
- 保留日志缺失时返回空字符串、清理缺失日志无副作用的原有命令语义，避免前端日志查看器出现兼容性回归。

## Deviations from Plan

### Auto-fixed Issues

**1. Shared helper surface required a combined implementation commit**

- **Found during:** Task 1 和 Task 2
- **Issue:** 日志路径回归测试直接依赖命令层 helper，而这些 helper 同时是 Task 2 生产接线的一部分，两个任务在代码上共享同一接口面。
- **Fix:** 采用一个原子代码提交同时交付测试和实现，避免制造一个无法编译的中间提交状态。
- **Files modified:** `src-tauri/src/commands.rs`, `src-tauri/src/app_paths_tests.rs`, `src-tauri/src/clipboard/processor.rs`, `src-tauri/src/clipboard/monitor.rs`
- **Verification:** 两条新增测试、legacy migration 测试和 `cargo test --no-run` 全部通过。
- **Committed in:** `91c9a43`

---

**Total deviations:** 1 auto-fixed
**Impact on plan:** 仅影响提交粒度，不影响计划范围，也没有引入额外功能。

## Issues Encountered

- `git commit` 第一次尝试被瞬时存在的 `.git/index.lock` 打断；确认锁消失后重试提交成功，没有额外清理仓库文件。
- `cargo test` / `cargo fmt` 输出里持续出现 `(eval):5: parse error near 'end'`，但命令均返回 0，属于当前 shell wrapper 噪音，不影响测试结果。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 1 的自动化 blocker 已经清零，当前 re-verification 状态为 `human_needed`；只剩真实 macOS 剪贴板与 legacy 安装迁移 smoke 待人工确认。
- 后续任何涉及图片资产或日志路径的新命令，都应优先复用 `AppPaths` 或已建立的 helper，不要再引入新的本地路径推导分叉。

## Self-Check

PASSED
