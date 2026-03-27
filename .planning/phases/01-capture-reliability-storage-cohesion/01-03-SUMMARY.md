---
phase: 01-capture-reliability-storage-cohesion
plan: 03
subsystem: database
tags: [rust, tauri, sqlite, tokio, tokio-util, clipboard, suppression]
requires:
  - phase: 01-01
    provides: Rust 测试入口与 temp-root 测试夹具
  - phase: 01-02
    provides: AppPaths 存储权威与 legacy root migration
provides:
  - CaptureRuntime 统一拥有 monitor/save 生命周期
  - brownfield content_hash 去重迁移与唯一索引
  - backend-owned self-write suppression contract
affects: [01-04 frontend copy-routing, 01-05 capture policy, CAPT-01, CAPT-02]
tech-stack:
  added: [tokio-util]
  patterns: [CaptureRuntime ownership, sqlite dedupe migration, backend suppression contract]
key-files:
  created: [src-tauri/src/capture/mod.rs, src-tauri/src/capture/runtime.rs]
  modified:
    [
      src-tauri/src/state.rs,
      src-tauri/src/database/mod.rs,
      src-tauri/src/clipboard/monitor.rs,
      src-tauri/src/commands.rs,
      src-tauri/src/capture_runtime_tests.rs,
    ]
key-decisions:
  - '用 CaptureRuntime 作为唯一 runtime owner，把 monitor/save worker 的生命周期收拢到 state.start_monitoring()/stop_monitoring()。'
  - '在 Database::init() 里先事务合并 brownfield 重复 content_hash，再创建唯一索引，避免历史库直接升级失败。'
  - '把自写 suppression 放到后端 copy_to_clipboard 命令层，统一使用 SHA256 十六进制 content_hash 和 1500ms TTL。'
patterns-established:
  - 'Runtime ownership: start_monitoring 只创建一个 CaptureRuntime，stop_monitoring cancel 并 await 两个后台任务。'
  - 'Schema hardening: brownfield 数据先 merge_existing_content_hash_duplicates，再创建 idx_clipboard_entries_content_hash_unique。'
  - 'Copy contract: backend copy 命令先 register_suppression_for_text，再写入系统剪贴板。'
requirements-completed: [CAPT-01, CAPT-02]
duration: 21min
completed: 2026-03-27
---

# Phase 01 Plan 03: Lifecycle Runtime, Dedupe Migration, and Backend Suppression Summary

**CaptureRuntime lifecycle control with cancellation-aware workers, brownfield SQLite content_hash dedupe migration, and backend-owned suppression keys for copy flows**

## Performance

- **Duration:** 21 min
- **Started:** 2026-03-27T21:43:17+08:00
- **Completed:** 2026-03-27T14:04:30Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- 用 `CaptureRuntime` 接管监控和保存 worker，`stop_monitoring()` 现在会显式 cancel 并 await 后台任务。
- 在 `Database::init()` 内增加 brownfield 重复行合并事务，并在其后创建 `content_hash` 唯一索引。
- 建立后端自写 suppression contract：`copy_to_clipboard` 先注册 suppression key，再写入系统剪贴板。

## Task Commits

Each task was committed atomically:

1. **Task 1: 引入 CaptureRuntime 并收拢 monitor/save 生命周期所有权** - `61a27e8` (test), `a6e8222` (feat)
2. **Task 2: 实现 brownfield dedupe migration、唯一索引与 backend suppression contract** - `9e94b6b` (test), `95eb17a` (feat)
3. **Task support: 同步 Rust lockfile** - `e1a703b` (chore)

## Files Created/Modified

- `src-tauri/src/capture/mod.rs` - 暴露 capture runtime 相关接口与共享 hash/suppression 工具。
- `src-tauri/src/capture/runtime.rs` - 定义 `CaptureRuntime`、suppression registry、save loop upsert 和停止语义。
- `src-tauri/src/clipboard/monitor.rs` - 把监控改成 `poll_once()`，并消费 runtime 提供的 suppression key。
- `src-tauri/src/database/mod.rs` - 在初始化中执行 brownfield duplicate merge，并创建唯一索引。
- `src-tauri/src/state.rs` - 用 `capture_runtime` 替代旧 monitor owner，并提供 `register_suppression_for_text()`。
- `src-tauri/src/commands.rs` - `copy_to_clipboard` 命令改为先注册 suppression 再写剪贴板。
- `src-tauri/src/capture_runtime_tests.rs` - 补全 CAPT-01/CAPT-02 的真实 Rust 自动化验证。

## Decisions Made

- 让 `broadcast::Receiver` 的订阅权归 `CaptureRuntime` 管理，而不是继续在 `state.rs` 里零散 `spawn`，这样 `receiver_count`、save worker 和 stop/join 语义可以统一验证。
- 选择事务内合并旧重复行而不是直接在 schema 上硬加唯一索引，避免已有本地数据库因历史重复数据而升级失败。
- 将 suppression key 注册放在 backend 命令层，而不是继续依赖 renderer 或前端 copy 入口自行避免回流，方便 01-04 统一收敛前端路由。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] 去重迁移同时保留 `is_favorite` 与可回填的 `content_data`**

- **Found during:** Task 2
- **Issue:** 计划明确了 `source_app`、`content_subtype`、`metadata`、`app_bundle_id` 和 `file_path` 的 survivor/fallback 规则，但如果合并时丢掉 `is_favorite` 或内容主体，会造成真实数据丢失。
- **Fix:** 在 duplicate merge 里把 `is_favorite` 聚合为任一行为真即保留，并在 survivor 缺失 `content_data` 时回填最近非空值。
- **Files modified:** `src-tauri/src/database/mod.rs`
- **Verification:** `cargo test test_capture_runtime_dedupe_migration_merges_existing_duplicates -- --nocapture`
- **Committed in:** `95eb17a`

**2. [Rule 3 - Blocking] 补提交流水中遗漏的 `Cargo.lock` 变更**

- **Found during:** Summary / cleanup
- **Issue:** `Cargo.toml` 已添加 `tokio-util`，但生成的 `src-tauri/Cargo.lock` 还处于未提交状态，会让源码依赖图和锁文件不一致。
- **Fix:** 只提交锁文件中新增的 `tokio-util` 条目，保持 Rust 依赖图一致。
- **Files modified:** `src-tauri/Cargo.lock`
- **Verification:** `cargo test --no-run`
- **Committed in:** `e1a703b`

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** 两项偏差都属于正确性和可交付性修复，没有扩大 Phase 1 的范围。

## Issues Encountered

- 第一次创建红灯测试提交时遇到短暂的 `.git/index.lock` 阻塞；确认锁已消失后重试提交成功，没有造成工作树回滚。

## User Setup Required

None - no external service configuration required.

## Known Stubs

- `src-tauri/src/commands.rs:1703` - `set_log_level()` 仍保留预先存在的 placeholder 注释，说明运行时动态调整日志级别尚未真正实现；这不是 01-03 的 capture reliability 范围，因此本计划未扩展处理。

## Next Phase Readiness

- 01-04 可以直接把所有前端 copy 入口收敛到后端 `copy_to_clipboard` 合同，而不需要再在 renderer 自己拼 suppression 逻辑。
- 01-05 可以建立在现有 `CaptureRuntime` seam 上，把 marker-first capture policy 接到 monitor 前置决策里。

## Self-Check

PASSED
