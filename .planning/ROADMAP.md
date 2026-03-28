# Roadmap: Dance

## Overview

This roadmap keeps the existing desktop client on its current Tauri + React + Rust + SQLite foundation and orders work around the user promise that matters most: copied content must be captured reliably, analyzed consistently, previewed clearly, found quickly, and remain trustworthy across upgrades. The phase sequence therefore follows the dependency chain from runtime reliability to analysis contracts, then preview unification, then search quality, and finally rebuild and release safeguards.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 1: Capture Reliability & Storage Cohesion** - Make monitoring start and stop trustworthy while keeping persisted clipboard data under one consistent local lifecycle.
- [x] **Phase 2: Analysis Contracts & Versioned Detection** - Establish Rust-side analysis as the single source of truth for subtype classification, metadata extraction, and graceful fallback.
- [ ] **Phase 3: Unified Developer Previews** - Render supported developer content through one consistent preview pipeline across raw and semantic representations.
- [ ] **Phase 4: Search Quality & Retrieval** - Deliver responsive indexed, filtered, fuzzy, and ranked retrieval for large local clipboard histories.
- [ ] **Phase 5: Rebuild Safety & Release Gates** - Preserve history across upgrades and prevent regressions with rebuild tooling and packaged validation.

## Phase Details

### Phase 1: Capture Reliability & Storage Cohesion

**Goal**: Users can trust clipboard capture to start and stop cleanly, avoid unwanted entries, and keep local data under one coherent storage lifecycle.
**Depends on**: Nothing (first phase)
**Requirements**: CAPT-01, CAPT-02, CAPT-03, CAPT-04
**Success Criteria** (what must be TRUE):

1. User can start monitoring, stop monitoring, and confirm no new clipboard items are saved after monitoring is turned off.
2. User sees each eligible clipboard change recorded once, even after repeated start/stop cycles or app-driven copy flows.
3. User can keep ignored, transient, concealed, or other non-persistent clipboard events out of saved history.
4. User can restart the app and still access history, cached previews, and image assets without path-related mismatches or missing data.
   **Plans**: 6/6 plans complete

Plans:

- [x] 01-01-PLAN.md — 建立 validation/test scaffolding、temp-root helper 与固定测试入口
- [x] 01-02-PLAN.md — 建立 AppPaths 存储权威并完成 legacy migration
- [x] 01-03-PLAN.md — 引入 lifecycle runtime、brownfield dedupe migration 与 backend suppression contract
- [x] 01-04-PLAN.md — 统一所有前端 copy 入口到 backend copy-routing 合同
- [x] 01-05-PLAN.md — 接入 marker-first capture policy 与 macOS pasteboard marker adapter
- [x] 01-06-PLAN.md — 闭合 CAPT-04 的 AppPaths residual bypass 并补日志路径回归测试

### Phase 2: Analysis Contracts & Versioned Detection

**Goal**: Users get stable developer-content analysis that can be re-applied to history and still falls back cleanly when parsing fails.
**Depends on**: Phase 1
**Requirements**: DETE-01, DETE-02, DETE-03, DETE-04
**Success Criteria** (what must be TRUE):

1. User copying supported developer content sees a stable subtype classification for URL, JSON, code, command, color, markdown, email, IP, timestamp, base64, or plain-text fallback.
2. User sees subtype-specific metadata extracted for supported content, including URL parts, color formats, detected language, timestamp formats, and other structured hints.
3. User can re-run improved detection on existing history and get updated classifications without copying items again.
4. User can still inspect copied content when analysis fails because the app falls back to raw content and preserves failure diagnostics for later repair.
   **Plans**: 7/7 plans complete

Plans:

- [x] 02-01-PLAN.md — 建立 `AnalysisSnapshot` contract、stable subtype precedence 和 fallback diagnostics shape
- [x] 02-02-PLAN.md — 建立 `entry_analysis` companion table、joined read model 与 authoritative persistence
- [x] 02-03-PLAN.md — 把 authoritative analysis 接入 capture/runtime 主链路并持久化 diagnostics
- [x] 02-04-PLAN.md — 让前端 detail/store 改为 analysis-first 消费，不再继续推断 subtype 语义
- [x] 02-05-PLAN.md — 增加历史 reanalysis service、Tauri command 与 Preferences rebuild 入口
- [x] 02-06-PLAN.md — 恢复 URL-first detail preview 合同，并让 resolved alternate views 真正可渲染
- [x] 02-07-PLAN.md — 恢复 JSON raw-only 入口、detail 滚动能力与 JsonRenderer 显式高度合同

### Phase 3: Unified Developer Previews

**Goal**: Users can inspect supported developer content through one consistent preview system across list, detail, and later retrieval contexts.
**Depends on**: Phase 2
**Requirements**: PREV-01, PREV-02, PREV-03, PREV-04, PREV-05
**Success Criteria** (what must be TRUE):

1. User can open any JSON item in a formatted structured view and switch back to the raw representation.
2. User can inspect URL items in a structured preview that shows protocol, host, path, and query details without automatic remote fetching.
3. User can inspect color items with a visual swatch and alternate development-friendly color formats.
4. User can inspect code and command items in a read-only developer-oriented view with preserved formatting and language or shell hints when available.
5. User sees the same semantic type and preview intent for an entry in the list, detail, and retrieval flows.
   **Plans**: TBD
   **UI hint**: yes

### Phase 4: Search Quality & Retrieval

**Goal**: Users can retrieve the right clipboard entry quickly from large local history through responsive, type-aware, and ranked search.
**Depends on**: Phase 3
**Requirements**: RETR-01, RETR-02, RETR-03, RETR-04, RETR-05
**Success Criteria** (what must be TRUE):

1. User can search large local history interactively and get responsive results without noticeable full-list lag.
2. User can narrow matches by content type or subtype, source app, favorites, and recent time windows in the search interface.
3. User can find entries from fuzzy fragments, abbreviations, or partial developer tokens when exact text is unknown.
4. User can search normalized structured tokens such as URL host or path fragments, JSON keys, command names, and alternate color values.
5. User can distinguish similar matches quickly because results include ranked snippets, highlights, or concise summary context in the list view.
   **Plans**: TBD
   **UI hint**: yes

### Phase 5: Rebuild Safety & Release Gates

**Goal**: Users can keep history usable across analysis and search upgrades and trust releases because rebuild and validation safeguards are in place.
**Depends on**: Phase 4
**Requirements**: RELY-01, RELY-02
**Success Criteria** (what must be TRUE):

1. User can rebuild analysis results and search indexes on existing history without clearing stored entries or losing past clipboard items.
2. User can install upgraded builds and still see monitoring, preview, and retrieval behave consistently because critical desktop paths are protected by automated validation and packaged smoke checks.
   **Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase                                       | Plans Complete | Status      | Completed  |
| ------------------------------------------- | -------------- | ----------- | ---------- |
| 1. Capture Reliability & Storage Cohesion   | 6/6            | Complete    | 2026-03-28 |
| 2. Analysis Contracts & Versioned Detection | 7/7            | Complete    | 2026-03-28 |
| 3. Unified Developer Previews               | 0/TBD          | Not started | -          |
| 4. Search Quality & Retrieval               | 0/TBD          | Not started | -          |
| 5. Rebuild Safety & Release Gates           | 0/TBD          | Not started | -          |
