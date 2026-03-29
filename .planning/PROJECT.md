# Dance

## What This Is

Dance 是一个面向开发者的本地桌面剪贴板工作台。v1.0 已经把剪贴板监听、历史持久化、类型识别、结构化预览、本地检索和升级后的重建安全收口成一条完整本地链路。它不是通用内容平台，也不是云同步产品，而是帮助开发者更快查看、理解、筛选和回用剪贴板内容的桌面工具。

## Core Value

开发者复制任意常见内容后，应用都能稳定记录、准确识别，并以最合适的结构化方式展示出来。

## Current State

- **Shipped milestone:** v1.0 MVP on 2026-03-29
- **Archive:** `.planning/milestones/v1.0-ROADMAP.md`, `.planning/milestones/v1.0-REQUIREMENTS.md`, `.planning/milestones/v1.0-MILESTONE-AUDIT.md`
- **Execution history:** `.planning/milestones/v1.0-phases/`
- **Current focus:** 定义下一个 milestone，而不是继续扩张 v1.0 的执行文档

## Current Milestone: v1.1 Developer Preview, Retrieval & Smoke Automation

**Goal:** 把更多开发者常见内容从 generic preview 提升为结构化专用预览，同时用更可靠的自动化验证和更真实的数据集继续提升 retrieval 质量。

**Target features:**

- JWT、TOML、XML、CSV/TSV、日志类内容的专用 developer preview
- 桌面 smoke、packaged smoke 与 GitHub Actions release smoke 的稳定自动化收口
- 基于真实历史样本的 retrieval ranking、highlight 与 representative query benchmark 强化

## Requirements

### Validated

- ✓ 稳定记录本地剪贴板历史，并支持启动/停止监听控制与单一存储生命周期 — v1.0
- ✓ Rust authoritative analysis contract、typed metadata、fallback diagnostics 与历史 rebuild — v1.0
- ✓ JSON、URL、颜色、代码、命令等开发者内容的统一预览语义与 detail/list/retrieval 一致性 — v1.0
- ✓ 本地检索能力，支持类型或 subtype、来源应用、收藏、时间窗口筛选，以及可解释的 snippet/match reason — v1.0
- ✓ analysis/search rebuild safety 与 release/test-build 打包前自动化 gate — v1.0

### Active

- [ ] 为 JWT、TOML、XML、CSV/TSV、日志类内容提供专用 developer preview，并让 list/detail/search surface 共享一致语义
- [ ] 把桌面 smoke、packaged smoke 和 GitHub Actions release smoke 收敛成更稳定的自动化验证层，缩小 CI 通过与真实桌面行为之间的差距
- [ ] 基于真实大历史样本继续打磨 retrieval ranking、highlight 和 representative query benchmark，提升 recall、解释性与回归可观测性

### Out of Scope

- 云同步 — 当前只考虑本地客户端体验，避免引入账户、服务端和同步复杂度
- 多设备同步 — 继续优先把单机体验打磨稳定
- 移动端 — 当前产品边界是桌面客户端，不扩展到手机和平板端
- 团队协作 — 当前主要服务个人开发者工作流，不做共享协作能力
- 分享能力 — 不属于“查看、理解、检索和回用本地内容”的核心主线
- AI-first semantic search — 当前优先做可解释、可本地运行的确定性搜索与模糊匹配

## Context

v1.0 以现有 Tauri + React + Rust + SQLite 架构为基础，先闭合 runtime capture、存储路径、authoritative analysis、semantic preview、retrieval 和 rebuild safety，再把 release/test-build 包装进自动化 gate。当前代码库已经不只是“基础剪贴板管理器”，而是一个具备开发者内容识别、结构化展示和本地回查能力的桌面工作台。

当前最明确的下一步不是扩张产品边界，而是继续加深 developer-specific content support，并把现在仍依赖人工 smoke 的桌面链路往自动化验证推进。v1.0 的 shipped 能力已经证明“本地优先 + Rust authority + React thin client”这条路线是成立的。

## Next Milestone Goals

- 把更多开发者常见格式从 generic text preview 提升为 dedicated structured preview
- 继续提升 retrieval 的 explainability 和真实数据集下的 recall/ranking 质量
- 缩小“自动化验证通过”和“真实桌面 smoke”之间的差距

## Constraints

- **Platform**: 仅考虑桌面客户端能力，当前不扩张到云端、同步和移动端
- **Primary Audience**: 面向开发者，优先围绕开发工作流中的内容识别、预览和检索
- **Existing Stack**: 基于当前 Tauri + React + Rust + SQLite 架构演进
- **Reliability**: 监听、存储、预览和检索链路必须可靠
- **Scope Control**: 不扩展到团队协作、分享和多设备体系

## Key Decisions

| Decision                                                                    | Rationale                                          | Outcome               |
| --------------------------------------------------------------------------- | -------------------------------------------------- | --------------------- |
| 继续把产品定位为开发者本地剪贴板工作台                                      | 用户核心诉求是本地内容理解与回用，不是平台化协作   | Validated in v1.0     |
| Rust 继续作为 capture、analysis、retrieval 与 rebuild 语义的权威层          | 避免前端继续分裂 subtype、query 与 rebuild 语义    | Validated in v1.0     |
| URL detail 采用 local-first 结构卡，远端 resolved 内容只进入备用视图        | 条目自身语义优先于远端资源 MIME                    | Validated in Phase 02 |
| 统一 semantic preview core，让 list/detail/retrieval 只在密度上不同         | 避免不同 surface 各自解释同一条目                  | Validated in v1.0     |
| retrieval 只允许一个 authoritative query path                               | backend 结果必须成为最终事实，不能被前端再二次裁剪 | Validated in v1.0     |
| 不新开独立 search rebuild command                                           | 维护入口越少，analysis/search 一致性越容易保证     | Validated in v1.0     |
| release/test-build 必须在打包前经过 `pnpm test`、`pnpm build`、`cargo test` | 把回归挡在产物生成前，而不是发布后                 | Validated in v1.0     |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):

1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):

1. Full review of all sections
2. Core Value check -> still the right priority?
3. Audit Out of Scope -> reasons still valid?
4. Update Context with current state

---

_Last updated: 2026-03-29 after starting v1.1 milestone_
