# Requirements: Dance

**Defined:** 2026-03-29  
**Milestone:** v1.1 Developer Preview, Retrieval & Smoke Automation  
**Core Value:** 开发者复制任意常见内容后，应用都能稳定记录、准确识别，并以最合适的结构化方式展示出来。

## v1.1 Requirements

本轮 requirements 只覆盖 v1.1 的新增范围，不重复声明已经在 `.planning/PROJECT.md` 中列为 Validated 的 v1.0 基线能力。

### Developer Previews

- [ ] **PREV-06**: User can inspect compact JWT tokens in a dedicated structured view that distinguishes decoded header or claims information from unverified raw token content and degrades safely when the token is malformed or encrypted
- [ ] **PREV-07**: User can inspect TOML entries in a structured configuration view with table or key hierarchy, raw fallback, and parse diagnostics
- [ ] **PREV-08**: User can inspect XML entries in a structured tree view with root, attribute, and namespace cues plus raw fallback and well-formedness diagnostics
- [ ] **PREV-09**: User can inspect CSV and TSV entries in a table-oriented preview with delimiter awareness, header cues, row or column summaries, and raw fallback
- [ ] **PREV-10**: User can inspect log-heavy entries in a log-oriented preview that surfaces severity, timestamp, stack, trace, or logger cues when detected and falls back safely to raw text
- [ ] **PREV-11**: User sees the same subtype summary, preview intent, and diagnostic status for the new developer formats across list, detail, and retrieval surfaces

### Retrieval Quality

- [ ] **RETR-06**: User can retrieve JWT, TOML, XML, CSV, TSV, and log entries through structured tokens derived from their metadata instead of relying only on raw text matches
- [ ] **RETR-07**: User sees match reasons, snippets, or highlights for the new developer formats that explain why an entry ranked highly in retrieval results
- [ ] **RETR-08**: User benefits from ranking that favors clear structured matches for new developer formats before weaker fuzzy matches when the query strongly aligns with extracted metadata
- [ ] **RETR-09**: User keeps retrieval quality stable across parser and ranking changes because representative benchmark queries cover both new and existing developer formats

### Reliability & Validation

- [ ] **RELY-03**: Maintainer can run repeatable desktop smoke checks against seeded test data without touching real user history, cache, or log directories
- [ ] **RELY-04**: Maintainer can run packaged artifact smoke checks against the same built app artifacts that will be distributed rather than rebuilding a second binary just for smoke verification
- [ ] **RELY-05**: Maintainer can run GitHub Actions smoke gates that capture startup, preview, retrieval, and release-path regressions before or alongside publishing
- [ ] **RELY-06**: User keeps existing history usable after v1.1 analysis and retrieval upgrades because rebuild or backfill flows can populate new metadata and search documents without requiring items to be recopied

## v2 Requirements

这些能力有价值，但不应挤占 v1.1 主线。

### Extended Security And Data Tooling

- **EXTD-04**: User can validate JWT signatures or JWKS-backed trust relationships instead of only inspecting decoded token structure
- **EXTD-05**: User can inspect XML with schema-aware or XPath-oriented tooling beyond well-formed structured preview
- **EXTD-06**: User can edit, sort, or export CSV and TSV previews with spreadsheet-like controls

### Advanced Automation And Analysis

- **EXTD-07**: Maintainer can run full macOS packaged UI smoke with zero manual spot checks
- **EXTD-08**: User can inspect logs with cross-entry analytics, aggregation, or timeline-oriented tooling

## Out of Scope

Explicitly excluded from the current roadmap.

| Feature                                            | Reason                                                                |
| -------------------------------------------------- | --------------------------------------------------------------------- |
| Cloud sync                                         | 当前主线仍是本地开发者工作台，不引入账户、服务端和同步一致性复杂度    |
| Multi-device sync                                  | 本轮优先把单机 preview、retrieval 与 smoke 验证做深，不扩展跨设备状态 |
| Mobile app                                         | 当前产品边界仍是桌面客户端                                            |
| Team collaboration                                 | 当前服务对象是个人开发者工作流，不做共享工作区或多人协作              |
| AI-first semantic retrieval                        | 当前优先做可解释、可 benchmark、可本地运行的确定性检索与排序          |
| JWT signature verification or remote introspection | v1.1 只做本地 decode-oriented preview，不扩展到 trust validation 平台 |
| Heavy spreadsheet editing                          | v1.1 目标是可读、可搜、可回用的 table preview，不是表格编辑器         |
| Full log analytics platform                        | v1.1 只做复制日志内容的结构化理解，不做跨 entry 观测分析系统          |
| Default remote fetching for new preview types      | 本轮聚焦本地解析与本地渲染，不引入新的联网默认行为                    |

## Traceability

Which phases cover which requirements. Will be finalized during roadmap creation.

| Requirement | Phase | Status  |
| ----------- | ----- | ------- |
| PREV-06     | TBD   | Pending |
| PREV-07     | TBD   | Pending |
| PREV-08     | TBD   | Pending |
| PREV-09     | TBD   | Pending |
| PREV-10     | TBD   | Pending |
| PREV-11     | TBD   | Pending |
| RETR-06     | TBD   | Pending |
| RETR-07     | TBD   | Pending |
| RETR-08     | TBD   | Pending |
| RETR-09     | TBD   | Pending |
| RELY-03     | TBD   | Pending |
| RELY-04     | TBD   | Pending |
| RELY-05     | TBD   | Pending |
| RELY-06     | TBD   | Pending |

**Coverage:**

- v1.1 requirements: 14 total
- Mapped exactly once: 0
- Unmapped: 14
- Duplicate mappings: 0
- Coverage status: roadmap pending

---

_Requirements defined: 2026-03-29_  
_Last updated: 2026-03-29 after milestone v1.1 requirement definition_
