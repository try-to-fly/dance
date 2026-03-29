# Phase 05: Rebuild Safety & Release Gates - Research

**Researched:** 2026-03-29  
**Domain:** 本地 history rebuild 与 GitHub Actions 打包门禁  
**Confidence:** MEDIUM

## Summary

当前代码基已经具备 Phase 05 所需的大部分基础设施：Phase 02 提供 `rebuild_entry_analysis`，Phase 04 提供 search document rebuild 能力和 retrieval authoritative path。缺口不在“重新设计维护系统”，而在把这些现有能力正确串起来，并确保发版工作流不会绕过测试和构建门禁。

最小且正确的 Phase 05 路径有两个：

1. 把 `rebuild_search_documents(pool)` 串到 `EntryAnalysisRebuilder::rebuild()`，并把 `search_reindexed` / `search_failed` 暴露到 Tauri command 与 Preferences 结果摘要。
2. 在 `.github/workflows/test-build.yml` 和 `.github/workflows/release.yml` 的每个平台打包前增加 `pnpm test`、`pnpm build`、`cargo test`。

**Primary recommendation:** 不再新增单独的 search rebuild UI 或命令，而是复用现有 analysis rebuild 入口；CI gate 直接落在现有打包 job 内，保持路径最短、责任最清晰。

## Current State Audit

- `src-tauri/src/analysis/rebuild.rs` 当前已经拥有扫描 stale history 并重建 analysis 的主链路，改动面最小。
- `src-tauri/src/retrieval/mod.rs` 已实现 `rebuild_search_documents(pool)`，说明 search document rebuild 不需要额外 schema 或新模块。
- `src/components/Preferences/PreferencesModal.tsx` 已展示 rebuild 结果统计，适合补充 search rebuild 反馈而非新增页面。
- `.github/workflows/test-build.yml` 和 `.github/workflows/release.yml` 之前都可以直接进入打包阶段，缺少统一的 frontend/backend gate。

## Recommended Patterns

### Pattern 1: One Rebuild Entry Point

- 用户只触发一个 rebuild action。
- backend 先更新 analysis，再同步重建 search documents。
- 结果对象同时返回 analysis 与 search 两类统计。

### Pattern 2: Pre-Package Gate

- 所有正式打包 job 在 Tauri build 前先运行 `pnpm test`、`pnpm build`、`cargo test`。
- gate 失败时直接阻止产物生成，避免错误被打包放大。

## Risks And Mitigations

- **Risk:** rebuild 只更新 analysis，不更新 search documents，导致历史分类已变但检索仍命中过时索引。  
  **Mitigation:** 在 `EntryAnalysisRebuilder::rebuild()` 内强制调用 `rebuild_search_documents(pool)` 并返回统计。
- **Risk:** release/test-build 工作流路径不一致，某一条漏掉 gate。  
  **Mitigation:** 同步改两份 workflow，并在 verification 文档里把两者都作为必要 artifact。
- **Risk:** 用户不知道 search rebuild 是否成功。  
  **Mitigation:** Preferences 结果摘要增加 `search_reindexed` / `search_failed`。

## Verification Targets

- 本地 `pnpm test`
- 本地 `pnpm build`
- 本地 `cargo test`
- 真机 smoke：已有历史触发 rebuild 后，搜索结果仍可用
- 远端 smoke：实际 CI/release 运行时 gate 会在打包前执行

---

_Research prepared from current codebase state on 2026-03-29._
