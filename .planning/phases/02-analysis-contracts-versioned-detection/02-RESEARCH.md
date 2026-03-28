# Phase 2: Analysis Contracts & Versioned Detection - Research

**Researched:** 2026-03-28
**Domain:** Rust-side clipboard analysis contracts, versioned detection, brownfield SQLite rebuilds
**Confidence:** HIGH

<user_constraints>

## User Constraints

- No phase-local `CONTEXT.md` exists for this phase.
- Use `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, the current codebase, and Phase 1 outputs as the only planning inputs.
- Focus this phase on `DETE-01`, `DETE-02`, `DETE-03`, and `DETE-04`.
- Map current brownfield detection behavior and its gaps against `DETE-01..04`.
- Recommend a stable Rust-side analysis contract covering subtype enum, metadata schema, versioning/rebuild strategy, and fallback/error diagnostics.
- Identify Phase 1 migration constraints that must be preserved.
- Propose validation architecture and concrete automated tests for `DETE-01..04`.
- Keep recommendations grounded in the current repo state. Do not expand scope to new platforms, cloud, sync, or speculative content families.
  </user_constraints>

<phase_requirements>

## Phase Requirements

| ID      | Description                                                                                                                                                                       | Research Support                                                                                                                         |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| DETE-01 | User copying supported developer content gets a stable subtype classification for URL, JSON, code, command, color, markdown, email, IP, timestamp, base64, or plain text fallback | Recommends a Rust-authoritative subtype enum, explicit detection precedence, and joined read model so all consumers see the same subtype |
| DETE-02 | User sees subtype-specific metadata extracted for supported content, including URL parts, color formats, detected language, timestamp formats, and related structured hints       | Recommends a typed metadata contract per subtype plus SQLite JSON-valid persistence and consumer-safe fallback rules                     |
| DETE-03 | User benefits from improved detection rules on existing history without needing to recopy items after parser or classifier upgrades                                               | Recommends `entry_analysis` versioning, batch rebuild commands, stale-row selection, and idempotent UPSERT semantics                     |
| DETE-04 | User can still inspect copied content when analysis fails because the app degrades gracefully to raw content and preserves failure diagnostics for later repair                   | Recommends explicit analysis status/fallback reason/diagnostics fields and raw-content-first read behavior                               |

</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Frontend commands must run from the repo root. Rust commands must run from `src-tauri/`.
- New Tauri commands must be added in `src-tauri/src/commands.rs`, registered in `src-tauri/src/lib.rs`, exposed through the frontend store, and reflected in `src/types/clipboard.ts`.
- Database schema changes must update both `src-tauri/src/database/mod.rs` and `src-tauri/src/models/mod.rs`.
- Stay on the current `Tauri + React + Rust + SQLite` architecture. This phase is a brownfield refactor, not a rewrite.
- Use the actual codebase as the source of truth when `CLAUDE.md` architecture prose is stale. Example: the current monitor is a 500 ms runtime poller in `src-tauri/src/capture/runtime.rs`, even though `CLAUDE.md` describes a non-polling integration.

## Summary

Phase 2 should not introduce new parser ecosystems or broaden content scope. The repo already has the required subtype surface area in `src-tauri/src/clipboard/content_detector.rs`, but the current contract is only a tuple return `(ContentSubType, Option<ContentMetadata>)` that is written directly into `clipboard_entries.content_subtype` and `clipboard_entries.metadata`. That design partially satisfies capture-time classification, but it cannot express analysis versioning, rebuild stale history, or distinguish a true `plain_text` match from a parser fallback. It also leaves semantic authority split between Rust and React because `src/stores/clipboardStore.ts` still re-infers URL/Base64 preview semantics on the frontend.

The recommended Phase 2 shape is a Rust-authoritative `EntryAnalysis` snapshot stored in a new companion table, with `contract_version`, `analysis_version`, `status`, typed metadata, and persisted diagnostics. Capture remains marker-first and raw-first: `clipboard_entries` keeps the raw clipboard payload and lifecycle semantics preserved by Phase 1, while `entry_analysis` becomes the only authoritative place for subtype meaning. Existing read paths should join analysis rows and only fall back to legacy `content_subtype` / `metadata` when a row has not been backfilled yet.

`DETE-03` should be solved by versioned, batched reanalysis of existing history rather than by mutating raw content or requiring the user to recopy entries. `DETE-04` should be solved by making fallback explicit in the contract, not by silently returning `plain_text` and dropping context. The current frontend already proves the user-facing fallback principle for URL/Base64 preview failures; Phase 2 needs to move that discipline down into the persisted analysis layer.

**Primary recommendation:** Add an authoritative `entry_analysis` companion table plus a typed Rust `AnalysisSnapshot` contract with `contract_version`, `analysis_version`, `status`, subtype-specific metadata, and diagnostics; rebuild stale rows in batches and stop introducing new subtype inference in React.

## Brownfield Audit

### Current Behavior vs DETE-01..04

| Requirement | Current brownfield behavior                                                                                                                                              | Gap                                                                                                                                                                                              |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| DETE-01     | `ClipboardMonitor::process_text_capture_with_detector()` runs `ContentDetector::detect()` for persisted text captures and stores a string subtype on `clipboard_entries` | Classification exists, but there is no version field, no explicit public precedence contract, and no backend-only authority because frontend preview code still reclassifies URL/Base64 behavior |
| DETE-02     | Metadata is only produced for URL, color, timestamp, base64, and code; it is serialized into an unversioned JSON string                                                  | Metadata coverage is incomplete for command/email/IP/markdown, schema is not versioned, and parse/serialization failures are silently collapsed to `None`                                        |
| DETE-03     | No rebuild command, no stale-row selector, no analysis version, and no companion analysis storage                                                                        | Existing history only changes when the user recopies content or edits the DB manually                                                                                                            |
| DETE-04     | URL/Base64 preview failures degrade to raw frontend fallback, but detection itself has no persisted failure state or diagnostics                                         | The app cannot distinguish real `plain_text` from parser fallback, and later repair has no persisted reason code to inspect                                                                      |

### Current Detection Pipeline

| Layer                | Current implementation                                                                                                                                    | Planning implication                                                                                                |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Capture boundary     | `src-tauri/src/clipboard/monitor.rs` runs marker-first capture policy before text analysis                                                                | Preserve this ordering. Do not analyze or persist non-persistent clipboard events                                   |
| Detector             | `src-tauri/src/clipboard/content_detector.rs` returns `(ContentSubType, Option<ContentMetadata>)` with hard-coded precedence                              | Wrap current heuristics behind a versioned service instead of letting tuple semantics leak further                  |
| Persistence          | `src-tauri/src/capture/runtime.rs` UPSERTs subtype/metadata into `clipboard_entries` on `content_hash` conflict                                           | Raw table is not sufficient for rebuildable analysis; add a companion table rather than mutating semantics in place |
| Read model           | `AppState::get_clipboard_history()` reads only `clipboard_entries` and search is `%LIKE%` over raw text/source app                                        | Phase 2 should add a joined analysis read model without trying to solve Phase 4 search indexing                     |
| Frontend consumption | `src/lib/preview/entryPresentation.ts` downgrades unknown subtypes to `plain_text`; `src/stores/clipboardStore.ts` re-infers URL/Base64 preview semantics | Stop growing frontend semantic logic. Consumers should render backend-provided semantics or raw fallback only       |

### Current Detection Precedence

Current `ContentDetector::detect()` order is:

1. URL
2. IP address
3. Email
4. Color
5. JSON
6. Command
7. Timestamp
8. Markdown
9. Base64
10. Code
11. Plain text fallback

This ordering must become an explicit, tested contract in Phase 2. Right now it only exists implicitly in code.

### Brownfield Notes

- The prompt listed `src-tauri/src/models/clipboard_entry.rs`, but the current repo defines `ClipboardEntry` in `src-tauri/src/models/mod.rs`. Planning should use the real file layout.
- There are no project-local skills under `.claude/skills/` or `.agents/skills/` in this repo.

## Standard Stack

### Core

| Library / System                                                      | Version                                            | Purpose                                                                             | Why Standard                                                                                                                |
| --------------------------------------------------------------------- | -------------------------------------------------- | ----------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Rust + `serde`                                                        | repo-locked `serde = 1`                            | Stable typed analysis contract serialization                                        | Existing repo standard for Rust <-> Tauri DTOs; official tagged-enum support makes contract evolution explicit              |
| `serde_json`                                                          | repo-locked `1`                                    | Persist analysis metadata/diagnostics as canonical JSON text                        | Already in use; matches current `metadata TEXT` storage model and avoids introducing a new storage format in Phase 2        |
| `sqlx` + SQLite                                                       | repo-locked `sqlx = 0.7`; local `sqlite3 = 3.51.0` | Brownfield migration, companion-table persistence, JSON validation, rebuild UPSERTs | Current persistence layer already uses SQLite; official JSON and UPSERT support are enough for this phase without new infra |
| Existing detector primitives (`regex`, `url`, `base64`, `serde_json`) | repo-locked current                                | Keep current subtype family working behind one authoritative service                | Grounded in current codebase and avoids speculative parser expansion in a contract-first phase                              |

### Supporting

| Library / System          | Version              | Purpose                                                                                     | When to Use                                                                            |
| ------------------------- | -------------------- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `tokio`                   | repo-locked `1`      | Batch rebuild tasks, bounded background processing                                          | Use for chunked reanalysis jobs and cancellation-aware command execution               |
| Vitest                    | repo-locked `4.1.2`  | Frontend contract tests for raw fallback rendering and consumer safety                      | Use only for renderer/store consumption rules, not for authoritative subtype detection |
| Rust `cargo test` modules | current repo pattern | Unit/integration verification of detector precedence, rebuilds, and diagnostics persistence | Primary validation surface for this phase                                              |

### Alternatives Considered

| Instead of                                     | Could Use                                                             | Tradeoff                                                                                                                                              |
| ---------------------------------------------- | --------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Authoritative `entry_analysis` companion table | Keep mutating `clipboard_entries.content_subtype` and `metadata` only | Easier migration short-term, but cannot model versioned rebuilds or fallback diagnostics cleanly                                                      |
| Canonical text JSON + `json_valid()`           | SQLite JSONB blobs                                                    | JSONB is newer and faster, but Phase 2 does not need its storage savings and text JSON fits the current repo and brownfield debugging workflow better |
| Explicit companion-table columns / indexes     | Generated columns for every extracted field                           | Generated columns add ALTER/compatibility constraints and are overkill before Phase 4 search work clarifies which tokens need indexing                |
| Backend-only subtype authority                 | Continue frontend fallback inference for semantic truth               | Low immediate cost, but keeps semantics split and makes DETE-01 stability impossible to guarantee                                                     |

**Installation:**

```bash
# No new phase-specific package is recommended for Phase 2.
# Reuse the current repo stack.
```

**Version verification:** No new external dependency is recommended for this phase. Repo-locked versions were verified from `src-tauri/Cargo.toml` and `package.json`; local tool availability was verified as `node v24.13.0`, `pnpm 10.0.0`, `cargo 1.91.0`, `rustc 1.91.0`, and `sqlite3 3.51.0`.

## Architecture Patterns

### Recommended Project Structure

```text
src-tauri/src/
├── analysis/                  # New authoritative analysis domain
│   ├── mod.rs
│   ├── contract.rs            # subtype enum, status, metadata, diagnostics
│   ├── detector.rs            # precedence orchestration over current heuristics
│   ├── rebuild.rs             # batched reanalysis of stale history
│   └── repository.rs          # entry_analysis storage helpers / joins
├── clipboard/
│   └── monitor.rs             # capture boundary only; delegates to analysis service
├── commands.rs                # analysis rebuild command(s), joined history read DTO
├── database/mod.rs            # migration creates entry_analysis + indexes
└── models/mod.rs              # compatibility DTOs returned to frontend
```

### Pattern 1: Stable Analysis Snapshot

**What:** A typed Rust snapshot that separates user-visible contract shape from detector-rule evolution.

**When to use:** For every persisted text clipboard row and every rebuild pass.

**Recommended contract fields:**

| Field              | Type                                      | Purpose                                                                                 |
| ------------------ | ----------------------------------------- | --------------------------------------------------------------------------------------- |
| `contract_version` | integer                                   | Bump when public meaning or JSON shape changes                                          |
| `analysis_version` | integer                                   | Bump when detection precedence/heuristics/parsers change but the shape stays compatible |
| `status`           | `matched` or `fallback`                   | Distinguish true subtype matches from degraded raw fallback                             |
| `subtype`          | stable enum                               | The authoritative subtype for all consumers                                             |
| `metadata`         | typed Rust struct/enum serialized to JSON | Subtype-specific structured hints                                                       |
| `diagnostics`      | array of typed diagnostics                | Persist parser failures, ambiguity, or coercion notes                                   |
| `analyzed_at`      | timestamp                                 | Operational traceability for rebuilds                                                   |

**Example:**

```rust
// Source: repo DTO patterns + Serde enum representation docs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisSubtype {
    PlainText,
    Url,
    IpAddress,
    Email,
    Color,
    Code,
    Command,
    Timestamp,
    Json,
    Markdown,
    Base64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    Matched,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum AnalysisMetadata {
    PlainText(PlainTextMetadata),
    Url(UrlMetadata),
    IpAddress(IpMetadata),
    Email(EmailMetadata),
    Color(ColorMetadata),
    Code(CodeMetadata),
    Command(CommandMetadata),
    Timestamp(TimestampMetadata),
    Json(JsonMetadata),
    Markdown(MarkdownMetadata),
    Base64(Base64Metadata),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSnapshot {
    pub contract_version: i32,
    pub analysis_version: i32,
    pub status: AnalysisStatus,
    pub subtype: AnalysisSubtype,
    pub metadata: AnalysisMetadata,
    pub diagnostics: Vec<AnalysisDiagnostic>,
    pub analyzed_at: i64,
}
```

### Pattern 2: Subtype Metadata Schema

**What:** Keep metadata minimal but typed. Only persist fields that later phases or current UI actually need.

**When to use:** Whenever a subtype match succeeds or a fallback wants to preserve context about why it degraded.

| Subtype      | Required metadata                                            | Optional metadata                     | Notes                                                                                                      |
| ------------ | ------------------------------------------------------------ | ------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `plain_text` | `normalized_preview`                                         | `line_count`                          | Use for true plain text and fallback plain text; differentiate by `status` and diagnostics, not by subtype |
| `url`        | `normalized_url`, `protocol`, `host`, `path`, `query_params` | `port`                                | Matches current preview/list needs directly                                                                |
| `ip_address` | `version` (`v4`/`v6`)                                        | —                                     | Enough for stable display and future filtering                                                             |
| `email`      | `local_part`, `domain`                                       | —                                     | Avoid parsing beyond what Phase 3 will need                                                                |
| `color`      | canonical `hex` plus available alternate formats             | `rgb`, `rgba`, `hsl`                  | Keep current renderer compatibility                                                                        |
| `code`       | `detected_language`                                          | `confidence`, `line_count`            | Current UI already reads `detected_language` from metadata                                                 |
| `command`    | `command_name`                                               | `shell_family`, `argv_preview`        | Needed to avoid treating commands as generic code forever                                                  |
| `timestamp`  | `unix_ms`                                                    | `iso8601`, `date_string`              | Matches current Time renderer expectations                                                                 |
| `json`       | `root_kind` (`object`/`array`/`scalar`)                      | `top_level_key_count`                 | Do not persist full parsed JSON tree in analysis metadata                                                  |
| `markdown`   | `has_heading`, `has_list`, `has_code_fence`                  | `link_count`                          | Enough for preview intent without introducing a markdown AST dependency                                    |
| `base64`     | `encoded_size`, `estimated_original_size`, `decoded_kind`    | `content_hint`, `encoding_efficiency` | Preserve current base64 signal while clarifying decoded intent                                             |

### Pattern 3: Companion Analysis Table

**What:** Store analysis separately from raw clipboard rows so raw history stays immutable and analysis can be rebuilt independently.

**When to use:** Immediately in Phase 2. This is the minimum structure that makes `DETE-03` possible.

**Recommended schema:**

```sql
-- Source: current SQLite/sqlx brownfield design + SQLite JSON/UPSERT docs
CREATE TABLE IF NOT EXISTS entry_analysis (
    entry_id TEXT PRIMARY KEY REFERENCES clipboard_entries(id) ON DELETE CASCADE,
    content_hash TEXT NOT NULL,
    contract_version INTEGER NOT NULL,
    analysis_version INTEGER NOT NULL,
    status TEXT NOT NULL,
    subtype TEXT NOT NULL,
    metadata_json TEXT NOT NULL CHECK (json_valid(metadata_json)),
    diagnostics_json TEXT NOT NULL CHECK (json_valid(diagnostics_json)),
    analyzed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entry_analysis_version
    ON entry_analysis(analysis_version, contract_version);

CREATE INDEX IF NOT EXISTS idx_entry_analysis_subtype
    ON entry_analysis(subtype);
```

Phase 2 should treat this table as the source of truth. Existing `clipboard_entries.content_subtype` and `clipboard_entries.metadata` should become compatibility fields only. Read paths should prefer joined analysis rows and fall back to legacy columns only when no analysis row exists yet.

### Pattern 4: Batched Rebuild by Version

**What:** Re-run detection for stale or missing analysis rows without changing the original raw clipboard data.

**When to use:** On schema migration/backfill and whenever `CURRENT_ANALYSIS_VERSION` increases.

**Recommended rebuild rule:**

- Select rows where `entry_analysis` is missing, or `analysis_version < CURRENT_ANALYSIS_VERSION`, or `contract_version < CURRENT_CONTRACT_VERSION`.
- Process in deterministic batches ordered by `created_at DESC, id DESC` or by primary key cursor.
- Persist analysis with UPSERT on `entry_id`.
- Never increment `copy_count`, emit new capture events, or mutate `content_hash`.

**Example:**

```sql
INSERT INTO entry_analysis (
    entry_id,
    content_hash,
    contract_version,
    analysis_version,
    status,
    subtype,
    metadata_json,
    diagnostics_json,
    analyzed_at
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(entry_id) DO UPDATE SET
    content_hash = excluded.content_hash,
    contract_version = excluded.contract_version,
    analysis_version = excluded.analysis_version,
    status = excluded.status,
    subtype = excluded.subtype,
    metadata_json = excluded.metadata_json,
    diagnostics_json = excluded.diagnostics_json,
    analyzed_at = excluded.analyzed_at
WHERE excluded.analysis_version > entry_analysis.analysis_version
   OR excluded.contract_version > entry_analysis.contract_version;
```

### Anti-Patterns to Avoid

- **Keep semantic truth in `clipboardStore.ts`:** React should not remain responsible for fallback subtype inference such as `guessUrlPreviewCategory()` or `decodeBase64Fallback()` once a backend contract exists.
- **Treat `plain_text` as both success and failure:** without a separate `status`, `DETE-04` cannot tell normal text from degraded text.
- **Only mutate `clipboard_entries.metadata`:** this blocks rebuildability and makes it impossible to keep old and new analyses conceptually separate.
- **Adopt generated columns as the Phase 2 default:** SQLite cannot `ALTER TABLE ADD COLUMN` a STORED generated column, and generated-column compatibility is stricter than this phase needs.

## Don't Hand-Roll

| Problem                                | Don't Build                                                     | Use Instead                                                                           | Why                                                                |
| -------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| Versioned analysis storage             | Ad-hoc in-place updates on `clipboard_entries` only             | `entry_analysis` companion table + UPSERT                                             | Raw history stays intact and rebuilds become idempotent            |
| Subtype contract                       | Stringly typed subtype + free-form blob with implicit meaning   | Rust enum + typed metadata + explicit `status`                                        | Prevents drift across Rust, DB, and TypeScript consumers           |
| Failure diagnostics                    | Silent `None` metadata or implicit fallback                     | Persisted diagnostic records with stage/code/message                                  | Required for later repair and for distinguishing degraded output   |
| Legacy JSON validation                 | Manual string checks before SQL writes                          | `serde_json` serialization + `json_valid()` + optional `json_error_position()` audits | Catches malformed payloads and supports repair tooling             |
| Search-oriented projections in Phase 2 | Premature FTS/generated-column indexing of every metadata field | Defer token projection to Phase 4 after contract stabilizes                           | Keeps this phase bounded to detection contracts and rebuild safety |

**Key insight:** Phase 2 is not a parser arms race. Its success comes from separating raw clipboard facts from analysis snapshots and making degradation first-class.

## Runtime State Inventory

| Category            | Items Found                                                                                                                             | Action Required                                                                                                                                                                      |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Stored data         | Existing `clipboard_entries` rows already store raw text/image payloads plus brownfield `content_subtype` / `metadata` strings          | Add `entry_analysis` backfill migration. This is both a schema migration and a data migration. Raw rows stay authoritative for payload; analysis rows become rebuildable projections |
| Live service config | None verified. The desktop app is local-only; analysis behavior is not configured in an external service UI                             | None                                                                                                                                                                                 |
| OS-registered state | None verified for analysis contracts. Global shortcuts/autostart exist, but Phase 2 detection changes do not require OS re-registration | None                                                                                                                                                                                 |
| Secrets/env vars    | None verified for analysis contracts. Existing env vars are analytics/release-related, not detection-related                            | None                                                                                                                                                                                 |
| Build artifacts     | None verified that cache analysis semantics outside the DB. Frontend preview caches are in-memory only                                  | None                                                                                                                                                                                 |

## Common Pitfalls

### Pitfall 1: String Drift Between Rust and TypeScript

**What goes wrong:** Rust introduces a new subtype or metadata meaning, but TypeScript still treats unknown values as `plain_text`.

**Why it happens:** Current consumers normalize unknown subtype strings in `getEntrySubType()` and continue rendering, which is user-safe but hides contract drift.

**How to avoid:** Keep subtype values exactly aligned with the current union during Phase 2 and add serialization tests that round-trip the Rust enum into the frontend string set.

**Warning signs:** New subtype strings appear in persisted rows while the UI quietly shows `Text`.

### Pitfall 2: Rebuild Logic Accidentally Touches Capture Semantics

**What goes wrong:** A rebuild job increments `copy_count`, emits `clipboard-update` as if a new copy happened, or bypasses Phase 1 suppression/capture boundaries.

**Why it happens:** The current save path couples capture and persistence through one broadcast pipeline.

**How to avoid:** Rebuilds should operate on stored rows only, through a dedicated analysis repository/service, and must not write to the clipboard or capture runtime.

**Warning signs:** Old entries jump to the top of history or `copy_count` changes after a rebuild.

### Pitfall 3: Frontend Preview Remains a Semantic Backdoor

**What goes wrong:** Backend analysis says one thing, but `resolveEntryPreview()` infers another preview intent from URL extensions or Base64 shape.

**Why it happens:** Today the frontend still contains semantic fallback heuristics for URL/Base64 preview behavior.

**How to avoid:** In Phase 2, stop adding new subtype inference in React. The frontend may keep raw fallback rendering, but not semantic reclassification.

**Warning signs:** Different screens disagree about the same entry's subtype or preview intent.

### Pitfall 4: JSON Storage Assumptions Leak Into SQL Queries

**What goes wrong:** Planner assumes SQLite stores a real JSON type or that `json_extract()` always returns JSON text.

**Why it happens:** SQLite stores JSON as ordinary text, and single-path `json_extract()` often returns SQL scalars.

**How to avoid:** Store canonical JSON text, use `json_valid()` constraints, and test any SQL JSON query against the real SQLite behavior used in this repo.

**Warning signs:** Queries fail on malformed JSON or compare quoted JSON strings instead of scalars.

## Code Examples

Verified patterns from repo structure and official docs:

### Stable Analysis Snapshot

```rust
// Source: https://serde.rs/enum-representations.html
// Source: current repo DTO style in src-tauri/src/models/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDiagnostic {
    pub stage: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSnapshot {
    pub contract_version: i32,
    pub analysis_version: i32,
    pub status: AnalysisStatus,
    pub subtype: AnalysisSubtype,
    pub metadata: AnalysisMetadata,
    pub diagnostics: Vec<AnalysisDiagnostic>,
    pub analyzed_at: i64,
}
```

### JSON-Safe Companion Storage

```sql
-- Source: https://www.sqlite.org/json1.html
CREATE TABLE entry_analysis (
    entry_id TEXT PRIMARY KEY REFERENCES clipboard_entries(id) ON DELETE CASCADE,
    contract_version INTEGER NOT NULL,
    analysis_version INTEGER NOT NULL,
    status TEXT NOT NULL,
    subtype TEXT NOT NULL,
    metadata_json TEXT NOT NULL CHECK (json_valid(metadata_json)),
    diagnostics_json TEXT NOT NULL CHECK (json_valid(diagnostics_json)),
    analyzed_at INTEGER NOT NULL
);
```

### Version-Aware UPSERT

```sql
-- Source: https://www.sqlite.org/lang_upsert.html
INSERT INTO entry_analysis (...)
VALUES (...)
ON CONFLICT(entry_id) DO UPDATE SET
    contract_version = excluded.contract_version,
    analysis_version = excluded.analysis_version,
    status = excluded.status,
    subtype = excluded.subtype,
    metadata_json = excluded.metadata_json,
    diagnostics_json = excluded.diagnostics_json,
    analyzed_at = excluded.analyzed_at
WHERE excluded.analysis_version > entry_analysis.analysis_version
   OR excluded.contract_version > entry_analysis.contract_version;
```

### Legacy JSON Audit Query

```sql
-- Source: https://www.sqlite.org/json1.html
SELECT id,
       json_valid(metadata) AS is_valid,
       json_error_position(metadata) AS first_error_pos
FROM clipboard_entries
WHERE metadata IS NOT NULL;
```

## State of the Art

| Old Approach                                                              | Current Approach                                                                     | When Changed   | Impact                                             |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | -------------- | -------------------------------------------------- |
| One-shot tuple return from `ContentDetector::detect()`                    | Typed `AnalysisSnapshot` with explicit `status`, versions, metadata, and diagnostics | Phase 2 target | Makes subtype meaning stable and rebuildable       |
| `clipboard_entries` stores both raw payload and latest analysis semantics | Raw payload stays in `clipboard_entries`; analysis moves to companion table          | Phase 2 target | Enables reanalysis without rewriting raw history   |
| Frontend preview fallback doubles as semantic inference                   | Backend stays authoritative; frontend only renders or degrades to raw                | Phase 2 target | Prevents list/detail/search semantic drift         |
| Silent fallback to `plain_text` with no reason                            | Persisted diagnostics and fallback reason                                            | Phase 2 target | Satisfies DETE-04 and later repair/debug workflows |

**Deprecated/outdated:**

- Treating `content_subtype` + free-form `metadata TEXT` on `clipboard_entries` as the long-term analysis contract.
- Growing new semantic inference paths inside `src/stores/clipboardStore.ts`.
- Designing Phase 2 around generated columns or JSONB before the contract shape is stable.

## Open Questions

1. **Should `entry_analysis` key on `entry_id` only, or also cache by `content_hash`?**
   - What we know: current raw rows are unique on `content_hash`, but the durable row identity exposed to the UI is `id`.
   - What's unclear: whether later rebuild tooling wants content-hash-level dedupe caching.
   - Recommendation: key storage on `entry_id` for correctness and optionally add a service-level memoization by `content_hash` later if profiling proves it useful.

2. **How far should Phase 2 go in removing frontend semantic fallback?**
   - What we know: current frontend fallback keeps the UI usable for URL/Base64 preview failures.
   - What's unclear: whether planner should fully delete those code paths in Phase 2 or first demote them to raw-only rendering guards.
   - Recommendation: stop adding new semantic inference immediately, but keep raw-only rendering fallback until Phase 3 unifies preview resolution.

3. **Should legacy `clipboard_entries.content_subtype/metadata` remain populated?**
   - What we know: current frontend and tests still consume those fields heavily.
   - What's unclear: whether Phase 2 should dual-write them or switch reads to a joined projection right away.
   - Recommendation: make `entry_analysis` authoritative and prefer a joined read model. Dual-write only as a short-lived compatibility bridge if the migration would otherwise become too wide.

## Environment Availability

| Dependency    | Required By                                           | Available | Version    | Fallback                                                   |
| ------------- | ----------------------------------------------------- | --------- | ---------- | ---------------------------------------------------------- |
| Node.js       | Frontend tests and store contract checks              | ✓         | `v24.13.0` | —                                                          |
| `pnpm`        | Vitest runs and repo scripts                          | ✓         | `10.0.0`   | npm could install deps, but repo is pnpm-first             |
| Cargo         | Rust tests, schema changes, analysis modules          | ✓         | `1.91.0`   | —                                                          |
| `rustc`       | Backend compilation                                   | ✓         | `1.91.0`   | —                                                          |
| `sqlite3` CLI | Local JSON/SQL smoke checks during planning/execution | ✓         | `3.51.0`   | Rust/sqlx tests can validate queries if CLI use is skipped |

**Missing dependencies with no fallback:**

- None verified.

**Missing dependencies with fallback:**

- None verified.

## Validation Architecture

### Test Framework

| Property           | Value                                                                               |
| ------------------ | ----------------------------------------------------------------------------------- |
| Framework          | Rust `cargo test` + frontend Vitest `4.1.2`                                         |
| Config file        | Rust: `src-tauri/src/lib.rs` test-module registration; frontend: `vitest.config.ts` |
| Quick run command  | `cd src-tauri && cargo test analysis_ -- --nocapture`                               |
| Full suite command | `pnpm test` plus `cd src-tauri && cargo test`                                       |

### Phase Requirements → Test Map

| Req ID  | Behavior                                                                                                                        | Test Type                            | Automated Command                                                                                   | File Exists? |
| ------- | ------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------- | ------------ |
| DETE-01 | Supported developer content maps to one stable subtype enum with fixed precedence                                               | Rust unit + integration              | `cd src-tauri && cargo test test_analysis_supported_content_maps_to_stable_subtypes -- --nocapture` | ❌ Wave 0    |
| DETE-02 | Each supported subtype emits the expected typed metadata schema and survives serialization/joined reads                         | Rust unit + repository contract      | `cd src-tauri && cargo test test_analysis_metadata_contract_round_trip -- --nocapture`              | ❌ Wave 0    |
| DETE-03 | Existing history can be backfilled/reanalyzed when `analysis_version` increases, without recopying or changing raw row identity | Rust integration                     | `cd src-tauri && cargo test test_rebuild_analysis_updates_existing_history -- --nocapture`          | ❌ Wave 0    |
| DETE-04 | Parser failures persist diagnostics while the UI still renders raw content safely                                               | Rust integration + frontend contract | `cd src-tauri && cargo test test_analysis_fallback_persists_diagnostics -- --nocapture`             | ❌ Wave 0    |

### Sampling Rate

- **Per task commit:** `cd src-tauri && cargo test analysis_ -- --nocapture`
- **Per wave merge:** `pnpm test` plus `cd src-tauri && cargo test`
- **Phase gate:** Full suite green, plus targeted rebuild/fallback tests green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src-tauri/src/analysis_contract_tests.rs` — stable subtype enum, metadata schema, diagnostics serialization, and precedence coverage for `DETE-01`, `DETE-02`, `DETE-04`
- [ ] `src-tauri/src/analysis_rebuild_tests.rs` — brownfield backfill, version bump, idempotent UPSERT, and non-mutating rebuild coverage for `DETE-03`
- [ ] `src/components/DetailView/DetailView.test.tsx` extension — verify fallback analysis still renders raw content without breaking current detail UI
- [ ] `src/lib/preview/entryPresentation.test.ts` — verify unknown or fallback analysis states degrade predictably to plain-text presentation until Phase 3 consumes richer analysis DTOs

## Sources

### Primary (HIGH confidence)

- Local repo files:
  - `.planning/ROADMAP.md`
  - `.planning/REQUIREMENTS.md`
  - `.planning/STATE.md`
  - `.planning/phases/01-capture-reliability-storage-cohesion/01-VERIFICATION.md`
  - `.planning/phases/01-capture-reliability-storage-cohesion/01-06-SUMMARY.md`
  - `src-tauri/src/clipboard/content_detector.rs`
  - `src-tauri/src/clipboard/monitor.rs`
  - `src-tauri/src/capture/runtime.rs`
  - `src-tauri/src/database/mod.rs`
  - `src-tauri/src/models/mod.rs`
  - `src-tauri/src/commands.rs`
  - `src-tauri/src/state.rs`
  - `src/stores/clipboardStore.ts`
  - `src/lib/preview/entryPresentation.ts`
  - `src/lib/preview/previewDescriptor.ts`
  - `src/components/DetailView/DetailView.tsx`
  - `src/components/DetailView/DetailPreviewContract.test.tsx`
  - `src/components/DetailView/DetailView.test.tsx`
- Serde enum representations: https://serde.rs/enum-representations.html
- SQLite JSON functions: https://www.sqlite.org/json1.html
- SQLite generated columns: https://www.sqlite.org/gencol.html
- SQLite UPSERT: https://www.sqlite.org/lang_upsert.html

### Secondary (MEDIUM confidence)

- Local tool availability probes: `node --version`, `pnpm --version`, `cargo --version`, `rustc --version`, `sqlite3 --version`

### Tertiary (LOW confidence)

- None

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - This phase can and should reuse the current repo stack; no speculative dependency leap is required.
- Architecture: HIGH - The recommendation is directly driven by current code paths and official SQLite/Serde behavior.
- Pitfalls: HIGH - Each pitfall is observable in the current brownfield implementation or in the official storage/serialization semantics.

**Research date:** 2026-03-28
**Valid until:** 2026-04-27
