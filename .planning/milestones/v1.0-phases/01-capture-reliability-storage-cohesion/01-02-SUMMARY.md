---
phase: 01-capture-reliability-storage-cohesion
plan: 02
subsystem: database
tags: [tauri, rust, sqlite, app-paths, migration]
requires:
  - phase: 01-01
    provides: temp-root helper与 Phase 1 路径测试脚手架
provides:
  - AppPaths 统一配置、数据库、图片、图标缓存和日志目录契约
  - Database、ConfigManager、ContentProcessor、AppState 的注入式路径构造
  - dance / clipboard-app legacy root 显式迁移与幂等 marker
  - commands 与图标缓存链路的 AppPaths 相对路径解析
affects: [01-03-PLAN.md, 01-04-PLAN.md, CAPT-04]
tech-stack:
  added: []
  patterns: [Arc<AppPaths> injection, copy-if-missing legacy migration]
key-files:
  created: [src-tauri/src/app_paths.rs]
  modified:
    [
      src-tauri/src/database/mod.rs,
      src-tauri/src/config/mod.rs,
      src-tauri/src/clipboard/processor.rs,
      src-tauri/src/state.rs,
      src-tauri/src/lib.rs,
      src-tauri/src/commands.rs,
      src-tauri/src/utils/app_icon_extractor.rs,
      src-tauri/src/app_paths_tests.rs,
    ]
key-decisions:
  - '把 AppPaths 作为唯一存储权威，通过 AppState 和 new_in(paths) 传入核心模块，而不是让模块自行解析目录。'
  - 'legacy root 迁移采用 copy-if-missing + capt04 marker，保证目标根优先且重复执行幂等。'
patterns-established:
  - '核心存储服务统一提供 new_in(paths: Arc<AppPaths>)，让运行时和 temp-root 测试共用同一套路径契约。'
  - '所有相对图片/资产路径在命令层和状态层都先经过 AppPaths::resolve_relative_asset_path。'
requirements-completed: [CAPT-04]
duration: 16 min
completed: 2026-03-27
---

# Phase 1 Plan 2: AppPaths Storage Authority Summary

**AppPaths 统一了本地存储根目录、接管 legacy root 迁移，并让核心存储模块与命令层都消费同一套注入式路径契约**

## Performance

- **Duration:** 16 min
- **Started:** 2026-03-27T13:15:43Z
- **Completed:** 2026-03-27T13:31:16Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- 新增 `AppPaths` 权威层，固定配置、数据库、图片、图标缓存、日志和 migration marker 的目录布局。
- `Database`、`ConfigManager`、`ContentProcessor` 与 `AppState` 改成消费注入式 `Arc<AppPaths>`，temp-root 测试可以完整驱动真实路径行为。
- 启动链路现在会显式迁移 `dance` / `clipboard-app` 遗留根目录，并让命令层和图标缓存全部走 `AppPaths` 做相对路径解析。

## Task Commits

Each task was committed atomically:

1. **Task 1: 建立 AppPaths 权威层并把核心存储构造改成注入式** - `24d425d` (`feat`)
2. **Task 2: 实现 legacy migration 并替换残留的路径字符串拼接** - `afd0d60` (`feat`)

## Files Created/Modified

- `src-tauri/src/app_paths.rs` - 定义 AppPaths、相对资源解析和 legacy migration marker 逻辑
- `src-tauri/src/database/mod.rs` - 数据库路径改成由 AppPaths 注入并支持 hermetic 测试构造
- `src-tauri/src/config/mod.rs` - 配置文件路径改成由 AppPaths 注入
- `src-tauri/src/clipboard/processor.rs` - 图片持久化目录改成由 AppPaths 注入
- `src-tauri/src/state.rs` - AppState 保存 AppPaths 并用它计算数据库、图片和粘贴路径
- `src-tauri/src/commands.rs` - 图片打开、图片 URL、图片转换和图标读取都通过 AppPaths 解析路径
- `src-tauri/src/utils/app_icon_extractor.rs` - 图标缓存目录改成由 AppPaths 注入
- `src-tauri/src/app_paths_tests.rs` - 新增 temp-root 注入与 legacy migration 的 CAPT-04 自动化验证

## Decisions Made

- 使用 `AppPaths::from_app()` 在 Tauri 启动时解析正式运行目录，再把 `Arc<AppPaths>` 传给状态层和核心服务，避免运行时散落的字符串拼接继续分叉。
- legacy migration 只认 `dirs::config_dir()/dance` 和 `dirs::config_dir()/clipboard-app` 两个旧根，并且只在目标缺失时复制文件，防止覆盖已迁入的新目录数据。
- 命令层的相对图片路径统一复用 `AppPaths::resolve_relative_asset_path()`，这样 CAPT-04 不只覆盖写入路径，也覆盖读取和打开路径。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] 收窄 dead_code 抑制以通过提交钩子的 `-D warnings`**

- **Found during:** Task 1
- **Issue:** 为测试和后续 Task 2 预留的注入式构造与访问器，在生产构建中尚未被完全消费，提交钩子把这些 warning 视为错误。
- **Fix:** 仅对过渡期需要的 helper 和字段添加精确的 `cfg_attr(not(test), allow(dead_code))`，不改变运行时行为。
- **Files modified:** `src-tauri/src/app_paths.rs`, `src-tauri/src/database/mod.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/clipboard/processor.rs`
- **Verification:** `cd src-tauri && cargo check`，随后两个任务提交都通过 Rust 检查
- **Committed in:** `24d425d`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** 该修正只为消除由当前任务引入的过渡期告警，不影响功能边界，也没有额外扩 scope。

## Issues Encountered

- 提交钩子会在 `cargo check` 阶段把未使用代码升级成错误，必须在 Task 1 完成后先清理告警，才能继续保持每个任务独立提交。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 01 后续计划现在可以直接依赖 `AppPaths` 和 temp-root helper 做 capture runtime、dedupe migration 和 copy routing 改造。
- CAPT-04 已经有稳定自动化信号，后续如果新增任何存储入口，都应继续走 `new_in(paths)` 或 `state.paths`。

## Known Stubs

- `src-tauri/src/utils/app_icon_extractor.rs:235` Windows 分支仍使用占位 PNG 作为图标兜底。这是 pre-existing stub，本计划只改了图标缓存目录权威，不影响 CAPT-04 的路径一致性目标。
- `src-tauri/src/commands.rs:1702` 运行时切换日志级别仍是 placeholder 实现。这是 pre-existing stub，因为本计划修改了 `commands.rs` 的路径解析而被一并扫描到，但不阻塞存储根目录统一与迁移能力。

## Self-Check: PASSED

- `01-02-SUMMARY.md` exists in `.planning/phases/01-capture-reliability-storage-cohesion/`
- Task commit `24d425d` exists in git history
- Task commit `afd0d60` exists in git history
