---
phase: 05-rebuild-safety-release-gates
verified: 2026-03-29T16:03:27+08:00
status: human_needed
score: 2/2 must-haves verified
human_verification:
  - test: '在已有历史的桌面实例里触发 Preferences rebuild，然后立刻搜索先前存在的 URL/JSON/命令条目'
    expected: 'history 不丢失，rebuild 结果显示 analysis 与 search 统计，检索结果仍正常命中'
    why_human: '真实历史数据、桌面 rebuild 反馈和搜索回查只能在运行中的 Tauri 应用里确认'
    result: pending
  - test: '实际触发 GitHub Actions test-build 或 release workflow'
    expected: '在 Tauri build 前先执行 `pnpm test`、`pnpm build`、`cargo test`，gate 失败时打包被阻止'
    why_human: 'workflow 的远端执行顺序和 secrets 环境只能在 GitHub Actions 中验证'
    result: pending
---

# Phase 05: Rebuild Safety & Release Gates Verification Report

**Phase Goal:** Users can keep history usable across analysis and search upgrades and trust releases because rebuild and validation safeguards are in place.  
**Verified:** 2026-03-29T16:03:27+08:00  
**Status:** human_needed

## Goal Achievement

### Observable Truths

| #   | Truth                                                                                                      | Status     | Evidence                                                                                                                                                                                                                               |
| --- | ---------------------------------------------------------------------------------------------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | 用户可以通过现有 rebuild 入口同时刷新 analysis 与 search documents，而不需要清空历史或再找第二个维护命令。 | ✓ VERIFIED | `src-tauri/src/analysis/rebuild.rs` 在完成 analysis rebuild 后调用 `rebuild_search_documents(pool)`；结果对象新增 `search_reindexed` / `search_failed`；`src/components/Preferences/PreferencesModal.tsx` 与其测试展示并断言这些字段。 |
| 2   | 正式打包路径在进入 Tauri build 前，都会先执行 frontend tests、frontend build 和 Rust tests。               | ✓ VERIFIED | `.github/workflows/test-build.yml` 与 `.github/workflows/release.yml` 都在打包步骤前新增 `pnpm test`、`pnpm build`、`cargo test`。                                                                                                     |

**Score:** 2/2 truths verified

### Required Artifacts

| Artifact                                               | Expected                               | Status     | Details                                                                                            |
| ------------------------------------------------------ | -------------------------------------- | ---------- | -------------------------------------------------------------------------------------------------- |
| `src-tauri/src/analysis/rebuild.rs`                    | one rebuild path for analysis + search | ✓ VERIFIED | `RebuildEntryAnalysisResult` 已包含 search rebuild 统计，并串联 `rebuild_search_documents(pool)`。 |
| `src/components/Preferences/PreferencesModal.tsx`      | user-visible search rebuild feedback   | ✓ VERIFIED | rebuild 结果摘要现在显示 search reindex 成功和失败数量。                                           |
| `src/components/Preferences/PreferencesModal.test.tsx` | UI contract regression coverage        | ✓ VERIFIED | 测试中断言了 `search_reindexed` / `search_failed` 新字段。                                         |
| `.github/workflows/test-build.yml`                     | pre-package validation gate            | ✓ VERIFIED | Windows/macOS test build 都先跑 tests/build。                                                      |
| `.github/workflows/release.yml`                        | release-time validation gate           | ✓ VERIFIED | release matrix job 在 Tauri build 前先跑 tests/build。                                             |

### Key Link Verification

| From                                              | To                               | Via                                | Status  | Details                                                 |
| ------------------------------------------------- | -------------------------------- | ---------------------------------- | ------- | ------------------------------------------------------- |
| `src-tauri/src/analysis/rebuild.rs`               | `src-tauri/src/retrieval/mod.rs` | `rebuild_search_documents(pool)`   | ✓ WIRED | analysis rebuild 现在直接触发 search rebuild。          |
| `src/components/Preferences/PreferencesModal.tsx` | `src-tauri/src/commands.rs`      | `invoke('rebuild_entry_analysis')` | ✓ WIRED | UI 继续复用既有 rebuild command，只是消费更完整的结果。 |
| `.github/workflows/test-build.yml`                | `.github/workflows/release.yml`  | matching validation gates          | ✓ WIRED | 两条工作流都在打包前跑同样的三道 gate。                 |

### Behavioral Spot-Checks

| Behavior                                     | Command      | Result | Status |
| -------------------------------------------- | ------------ | ------ | ------ |
| frontend regression suite                    | `pnpm test`  | passed | ✓ PASS |
| frontend production build                    | `pnpm build` | passed | ✓ PASS |
| backend + retrieval/rebuild regression suite | `cargo test` | passed | ✓ PASS |

### Requirements Coverage

| Requirement | Description                                                        | Status      | Evidence                                                                                   |
| ----------- | ------------------------------------------------------------------ | ----------- | ------------------------------------------------------------------------------------------ |
| `RELY-01`   | rebuild analysis/search without clearing stored history            | ✓ SATISFIED | one rebuild path + search reindex counters + non-destructive rebuild behavior。            |
| `RELY-02`   | release safeguards for monitoring/preview/retrieval critical paths | ✓ SATISFIED | test-build/release workflows now gate packaging on automated frontend/backend validation。 |

## Human Verification Required

### 1. Desktop Rebuild Smoke

**Test:** 在已有历史的桌面实例里触发 Preferences rebuild，然后搜索旧 URL、JSON、命令和颜色条目。  
**Expected:** 历史仍保留，rebuild 摘要出现 analysis 与 search 统计，检索结果可正常命中。  
**Why human:** 真正的历史规模、桌面交互与搜索回查只能在运行中的应用里确认。

### 2. Remote Workflow Smoke

**Test:** 在 GitHub Actions 中运行 test-build 或 release workflow。  
**Expected:** `pnpm test`、`pnpm build`、`cargo test` 会先于 Tauri build 执行；如果任一 gate 失败，打包停止。  
**Why human:** 远端 workflow 顺序、secrets 和 runner 环境必须在实际 CI 中观察。

## Gaps Summary

自动化层面没有发现 blocker。当前剩余的是桌面 rebuild 与远端 workflow 的真实环境 smoke，因此状态保持为 `human_needed`。

---

_Verified: 2026-03-29T16:03:27+08:00_  
_Verifier: Codex_
