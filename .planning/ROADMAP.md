# Roadmap: Dance

## Milestones

- ✅ **v1.0 MVP** — Phases 1-5, 20 plans, shipped 2026-03-29. Archive: `.planning/milestones/v1.0-ROADMAP.md`
- 🚧 **v1.1 Developer Preview, Retrieval & Smoke Automation** — Phases 6-9, 14 requirements, planning started 2026-03-29

## Active Milestone

**Milestone v1.1: Developer Preview, Retrieval & Smoke Automation**

**Goal:** 把更多开发者常见文本从 generic preview 提升为结构化专用预览，同时让 retrieval 质量和桌面 smoke 验证一起变得更稳定、可解释、可回归。

**14 requirements** | **4 phases** | All mapped ✓

| #   | Phase                                   | Goal                                                                                                                  | Requirements                                         | Success Criteria |
| --- | --------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------- | ---------------- |
| 6   | Developer Type Analysis & Search Tokens | 扩展 Rust authoritative analysis、structured terms 与 rebuild/backfill，使新 developer formats 先变成稳定可检索的事实 | RETR-06, RELY-06                                     | 4                |
| 7   | Dedicated Developer Preview Surfaces    | 为 JWT/TOML/XML/CSV/TSV/log 落 dedicated preview surfaces，并保证 list/detail/retrieval 语义一致                      | PREV-06, PREV-07, PREV-08, PREV-09, PREV-10, PREV-11 | 4                |
| 8   | Retrieval Benchmark & Explainability    | 用 representative benchmark、structured ranking 和 explainability 把 retrieval 质量从“能搜”推进到“稳定可信”           | RETR-07, RETR-08, RETR-09                            | 4                |
| 9   | Desktop & Release Smoke Automation      | 让 desktop/package/release smoke 进入现有 workflow，并用同一产物验证关键路径                                          | RELY-03, RELY-04, RELY-05                            | 4                |

## Phase Details

### Phase 6: Developer Type Analysis & Search Tokens

**Goal:** 让 JWT/TOML/XML/CSV/TSV/log 先进入 Rust authority、typed metadata、search document 和 rebuild 语义，而不是先在前端做孤立 renderer。

**Requirements:** `RETR-06`, `RELY-06`

**Success criteria:**

1. Rust analysis contract 能稳定分类并提取 JWT/TOML/XML/CSV/TSV/log 的摘要级 metadata 与 diagnostics。
2. Retrieval search documents 能为新 subtype 产出 structured tokens，而不是只退回 raw text。
3. 历史 rebuild/backfill 能把旧条目补齐新 metadata 与 search documents，不要求用户重新复制内容。
4. 新 subtype 的检测优先级、负样本与 rebuild 行为都有自动化 contract coverage。

### Phase 7: Dedicated Developer Preview Surfaces

**Goal:** 把新 developer formats 从 generic text 正式升级为有判断价值的 dedicated preview，同时保持 raw fallback 与 shared semantic summary。

**Requirements:** `PREV-06`, `PREV-07`, `PREV-08`, `PREV-09`, `PREV-10`, `PREV-11`

**Success criteria:**

1. JWT、TOML、XML、CSV/TSV、日志在 detail view 都有 dedicated preview，而不是只显示 generic text。
2. 每个新 preview 都保留 raw fallback、diagnostic 状态和安全退化路径。
3. List/detail/retrieval 对同一条目的 subtype summary、preview intent 和诊断语义保持一致。
4. 新 renderer 只消费 authority contract，不在前端自行发明第二套解析逻辑。

### Phase 8: Retrieval Benchmark & Explainability

**Goal:** 让 retrieval 对新 developer formats 的 ranking、snippet 与 highlight 变得可解释且可回归，而不是靠体感调分。

**Requirements:** `RETR-07`, `RETR-08`, `RETR-09`

**Success criteria:**

1. Retrieval results 能对新 developer formats 给出稳定的 reason label、snippet 或 highlight 解释。
2. 对明确 structured match 的查询，排名优先于较弱的 fuzzy 命中。
3. Representative benchmark corpus 覆盖新旧 developer formats，并定义 top-k 或 match-kind 期望。
4. Ranking 或 parser 调整后，可以通过 benchmark 输出明确看到退化或提升，而不是只看人工体感。

### Phase 9: Desktop & Release Smoke Automation

**Goal:** 把 v1.1 的 preview/retrieval 关键路径纳入 hermetic smoke 层，并在本地、打包产物与 GitHub Actions release 路径上复用。

**Requirements:** `RELY-03`, `RELY-04`, `RELY-05`

**Success criteria:**

1. Desktop smoke 能在隔离的 test root 下运行，不污染真实用户历史、cache 或 logs。
2. Packaged smoke 复用同一 build job 产物，而不是为了 smoke 重新构建第二个 binary。
3. GitHub Actions release/test-build workflow 能产出 smoke evidence，并在关键失败时阻止错误产物继续发布。
4. 平台策略明确区分 Windows/Linux 与 macOS 的自动化能力边界，不假设 macOS 有完整 WebDriver UI smoke。

## Archived Snapshot

<details>
<summary>✅ v1.0 MVP (Phases 1-5)</summary>

- [x] Phase 1: Capture Reliability & Storage Cohesion (6/6 plans) — completed 2026-03-28
- [x] Phase 2: Analysis Contracts & Versioned Detection (7/7 plans) — completed 2026-03-28
- [x] Phase 3: Unified Developer Previews (5/5 plans) — completed 2026-03-29
- [x] Phase 4: Search Quality & Retrieval (1/1 plan) — completed 2026-03-29
- [x] Phase 5: Rebuild Safety & Release Gates (1/1 plan) — completed 2026-03-29

</details>

## Progress

| Milestone                                            | Phases | Plans | Status   | Shipped    |
| ---------------------------------------------------- | ------ | ----- | -------- | ---------- |
| v1.0 MVP                                             | 5      | 20    | Complete | 2026-03-29 |
| v1.1 Developer Preview, Retrieval & Smoke Automation | 4      | 0     | Planned  | —          |
