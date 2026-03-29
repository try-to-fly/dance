# Project Retrospective

_A living document updated after each milestone. Lessons feed forward into future planning._

## Milestone: v1.0 — MVP

**Shipped:** 2026-03-29  
**Phases:** 5 | **Plans:** 20 | **Sessions:** 3

### What Was Built

- 把剪贴板 capture lifecycle、存储路径和 suppression contract 收口为更可靠的本地运行时。
- 建立 Rust authoritative analysis contract、history rebuild 和 analysis-first preview consumption。
- 统一 JSON、URL、颜色、代码、命令等开发者内容在 list/detail/retrieval 三个 surface 的语义和展示。
- 落地本地 retrieval、structured token search、组合筛选、snippet/match reason 和 rebuild/release gate。

### What Worked

- Phase-by-phase 的 contract-first 执行方式让 capture、analysis、preview、retrieval 这条依赖链可以稳定递进。
- 把 Rust 作为 authority，React 作为 thin client，显著减少了前后端语义分裂。
- 在每个 phase 上保留 verification 文档，让里程碑 audit 时能直接回收证据而不是重新推理。

### What Was Inefficient

- 部分 code changes 没有在 Phase 4/5 过程中同步形成最终归档 commit，导致里程碑 closeout 需要一次性处理较大的脏工作树。
- 仍有若干桌面 smoke 和远端 workflow smoke 依赖人工验证，自动化闭环还不够完整。

### Patterns Established

- Rust authoritative contract pattern for analysis, retrieval, and rebuild.
- Shared semantic preview core pattern for list/detail/retrieval surfaces.
- One rebuild entry point pattern for analysis + search reindex.
- Pre-package validation gate pattern for release and test-build workflows.

### Key Lessons

1. 当产品主线高度依赖语义一致性时，最先要锁定的是 authority 边界，而不是先做 UI 表层优化。
2. 如果 milestone 代码没有持续提交，最终的 archive/tag 会失真；里程碑 closeout 必须把代码状态和文档状态一起收口。

### Cost Observations

- Model mix: 未单独统计
- Sessions: 3
- Notable: 先闭合 contract、再做 surface 演进，比从 UI 倒推后端稳定得多。

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change                                |
| --------- | -------- | ------ | ----------------------------------------- |
| v1.0      | 3        | 5      | 建立了 authority-first 的本地桌面演进模式 |

### Cumulative Quality

| Milestone | Tests                           | Coverage                          | Zero-Dep Additions            |
| --------- | ------------------------------- | --------------------------------- | ----------------------------- |
| v1.0      | `pnpm test` + `cargo test` 通过 | phase-level verification complete | search/rebuild 继续复用现有栈 |

### Top Lessons (Verified Across Milestones)

1. 先明确 Rust authority，再让前端消费 contract，能显著降低 brownfield 演进中的回归成本。
