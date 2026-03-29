# Phase 1: Capture Reliability & Storage Cohesion - Research

**Researched:** 2026-03-27
**Domain:** Tauri/Rust clipboard capture lifecycle, macOS pasteboard policy, and app-scoped local storage
**Confidence:** MEDIUM

<user_constraints>

## User Constraints

No phase-specific `CONTEXT.md` exists for this phase. Active constraints come from the user request, `REQUIREMENTS.md`, `ROADMAP.md`, `STATE.md`, and the additional planning context.

### Locked Constraints

- Must answer: "What do I need to know to PLAN this phase well?"
- Must address `CAPT-01`, `CAPT-02`, `CAPT-03`, `CAPT-04`.
- Phase goal is fixed: users can trust clipboard capture to start and stop cleanly, avoid unwanted entries, and keep local data under one coherent storage lifecycle.
- Planning must focus on known current concerns:
  - monitoring lifecycle currently leaks background tasks or cannot cancel cleanly
  - storage paths are split across `dance` and `clipboard-app`
  - capture policy should account for transient, concealed, auto-generated, or remote clipboard markers
  - planning output should help create executable plans, not generic advice
- This remains a brownfield, local-only desktop client on the existing `Tauri + React + Rust + SQLite` stack.
- Do not expand scope into cloud sync, multi-device sync, collaboration, mobile, or default remote URL fetching.

### Claude's Discretion

- Choose the narrowest implementation seams that make Phase 1 executable without forcing a rewrite.
- Recommend the minimum new dependencies needed to make lifecycle control, path authority, and tests reliable.
- Define how much of marker-aware capture policy must be native-platform specific now versus left as a no-op on non-macOS targets.

### Deferred Ideas (OUT OF SCOPE)

- Detection upgrades beyond what Phase 1 needs to classify capture-policy decisions.
- Preview contract refactors not required for capture or storage reliability.
- Search/index upgrades beyond what is required to preserve current history semantics.
  </user_constraints>

<phase_requirements>

## Phase Requirements

| ID      | Description                                                                                                                               | Research Support                                                                                            |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| CAPT-01 | User can start and stop clipboard monitoring without hidden background listeners continuing after stop                                    | `CaptureRuntime` ownership, cooperative cancellation, explicit task joining, lifecycle tests                |
| CAPT-02 | User sees each clipboard change recorded once without duplicate history entries caused by repeated listeners or self-generated copy flows | single persistence worker, atomic `UPSERT`, backend-owned self-write suppression, frontend listener cleanup |
| CAPT-03 | User can keep ignored, transient, concealed, or otherwise non-persistent clipboard events out of saved history                            | marker-first capture policy, macOS pasteboard type adapter, `Persist`/`CurrentOnly`/`Skip` dispositions     |
| CAPT-04 | User can rely on one consistent local storage lifecycle for history, cache, image assets, and related metadata                            | `AppPaths` authority, explicit legacy migration from `dance` and `clipboard-app`, temp-root test fixtures   |

</phase_requirements>

## Summary

Phase 1 is not a feature-add phase. It is a trust-repair phase for the input pipeline. The current codebase already shows the three structural faults that planning must treat as one problem: `AppState::start_monitoring()` and `ClipboardMonitor::start_monitoring()` spawn work that `stop_monitoring()` does not actually cancel; app-triggered copy flows often happen in the renderer through `@tauri-apps/plugin-clipboard-manager`, so backend suppression never has a chance to prevent self-generated entries; and storage is split between `~/.../dance` and `~/.../clipboard-app`, so database, config, image assets, icon cache, and cache statistics disagree about the app’s canonical root.

The correct phase shape is incremental, not a rewrite. Keep the existing `Tauri + React + Rust + SQLite` stack. Add one Rust-owned `CaptureRuntime` seam for lifecycle control, one `CapturePolicy` seam for marker-aware skip rules and self-write suppression, and one `AppPaths` seam for all filesystem locations plus a one-time migration path from the two legacy roots. That gets CAPT-01 through CAPT-04 into planable, testable units without dragging later preview or search phases into this one.

The biggest planning mistake would be to treat CAPT-03 as “just add more string heuristics.” Transient, concealed, auto-generated, source, and remote-clipboard markers are metadata on the pasteboard, not properties of the text payload. `arboard` is still fine for general text/image I/O, but its documented API does not expose macOS pasteboard marker types. For macOS, a small native adapter using the repo’s existing `cocoa`/`objc` dependencies is the safest path. For non-macOS targets, the planner should explicitly keep the adapter capability-based and no-op where markers are unavailable.

**Primary recommendation:** Plan Phase 1 around three concrete deliverables only: `CaptureRuntime`, `CapturePolicy`, and `AppPaths` with explicit legacy migration.

## Project Constraints (from CLAUDE.md)

- Reply and document in Chinese simplified.
- Run `pnpm`, `npm`, and `node` commands from the project root.
- Run `cargo` commands from `src-tauri/`.
- Always verify the working directory before executing commands that assume root or `src-tauri/`.
- Stay on the existing `Tauri + React + Rust + SQLite` architecture; do not recommend a rewrite.
- Keep scope local-only desktop; do not introduce cloud sync, multi-device sync, or collaboration work.
- Reliability of monitoring, storage, preview, and retrieval is a first-class constraint.
- New desktop capabilities should follow the existing Tauri command pattern:
  - add `#[tauri::command]` in `src-tauri/src/commands.rs`
  - register it in `src-tauri/src/lib.rs`
  - add the matching frontend action in `src/stores/clipboardStore.ts`
  - keep shared data shapes aligned with `src/types/clipboard.ts`
- Frontend formatting/linting stays on the root Prettier and ESLint setup.
- Rust formatting/linting stays on `cargo fmt` and `cargo clippy`.
- Research and implementation should stay inside the GSD workflow; do not bypass planning artifacts.

## Standard Stack

### Core

| Library                                                                                 | Version                                                                         | Purpose                                                           | Why Standard                                                                             |
| --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `tauri` PathResolver (`app_config_dir`, `app_data_dir`, `app_cache_dir`, `app_log_dir`) | Repo pinned `2.x`; current docs `2.10.3` published `2026-03-04`                 | Canonical app-scoped directories and migration targets            | Official, bundle-identifier-aware path resolution that matches the current desktop stack |
| `tokio` + `tokio-util::sync::CancellationToken`                                         | Repo pinned `tokio 1.x`; `tokio-util 0.7.18` verified on crates.io `2026-03-27` | Cooperative task cancellation for monitor and persistence workers | Standard async lifecycle control with child tokens and explicit stop semantics           |
| SQLite `UPSERT` (`INSERT ... ON CONFLICT DO UPDATE`)                                    | SQLite `3.24.0+`; docs current as of `2024-04-11`                               | Atomic dedupe/update by `content_hash`                            | Eliminates query-then-insert races when duplicate listeners or concurrent saves happen   |

### Supporting

| Library          | Version                                     | Purpose                                                            | When to Use                                                                                              |
| ---------------- | ------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| `arboard`        | Repo pinned `3.3`; current docs `3.6.1`     | Cross-platform clipboard text/image I/O                            | Keep existing generic read/write path for text and images                                                |
| `cocoa` + `objc` | Repo pinned                                 | macOS pasteboard type inspection and marker read/write             | Use only for CAPT-03 marker-aware policy on macOS because `arboard` does not document marker-type access |
| `tempfile`       | `3.27.0` verified on crates.io `2026-03-27` | Hermetic temp roots for storage and migration tests                | Use for Wave 0 test isolation and filesystem migration validation                                        |
| `vitest`         | `4.1.2`, modified `2026-03-26`              | Frontend contract tests around listener cleanup and command wiring | Use only where Rust-side fixes alter frontend event or copy behavior                                     |

### Alternatives Considered

| Instead of                                                  | Could Use                                           | Tradeoff                                                                                                               |
| ----------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Hard-coded `dirs::config_dir()` joins with app-name strings | Tauri `PathResolver` app directories                | `PathResolver` is app-scoped and consistent with the current bundle identifier; manual joins are already causing drift |
| Replacing clipboard access wholesale                        | Keep `arboard` and add a small macOS marker adapter | Smaller brownfield change; only replace `arboard` later if marker support needs exceed the adapter seam                |
| Drop-only monitor ownership                                 | Boolean flags or ad-hoc `watch` loops               | `CancellationToken` plus owned `JoinHandle`s is clearer to reason about and easier to test                             |

**Installation:**

```bash
cd src-tauri && cargo add tokio-util
```

No frontend package changes are required for Phase 1.

**Version verification:** Verified during research.

```bash
cargo search tokio-util --limit 1
cargo search tempfile --limit 1
npm_config_cache=/tmp/.npm-cache npm view @tauri-apps/api version time.modified
npm_config_cache=/tmp/.npm-cache npm view @tauri-apps/plugin-clipboard-manager version time.modified
npm_config_cache=/tmp/.npm-cache npm view vitest version time.modified
```

Verified outputs:

- `tokio-util` -> `0.7.18`
- `tempfile` -> `3.27.0`
- `@tauri-apps/api` -> `2.10.1`, modified `2026-02-03`
- `@tauri-apps/plugin-clipboard-manager` -> `2.3.2`, modified `2026-02-02`
- `vitest` -> `4.1.2`, modified `2026-03-26`

## Architecture Patterns

### Recommended Project Structure

```text
src-tauri/src/
├── app_paths.rs          # canonical directories + legacy migration
├── capture/
│   ├── runtime.rs        # owned service, cancellation, join handles
│   ├── policy.rs         # marker/source/self-write suppression rules
│   ├── monitor.rs        # clipboard polling + platform adapters
│   └── persistence.rs    # single save worker + atomic UPSERT
├── state.rs              # wires AppState to CaptureRuntime/AppPaths
└── test_support.rs       # temp roots and fixture builders
```

This is the recommended seam layout, not a demand to rewrite all current files at once. The planner can stage the work by introducing `app_paths.rs` first, then `capture/runtime.rs`, then moving specific logic out of `state.rs` and `clipboard/monitor.rs`.

### Pattern 1: Owned Capture Runtime

**What:** `AppState` owns exactly one runtime object that encapsulates monitor task(s), persistence task(s), cancellation token(s), and stop/join behavior.

**When to use:** Every `start_monitoring`, `stop_monitoring`, restart, shutdown, or “is monitoring?” path.

**Why:** Current code stores `Option<ClipboardMonitor>` but the actual work is in spawned tasks. Dropping the struct handle does not stop those tasks.

**Example:**

```rust
// Source: tokio_util CancellationToken docs + tokio JoinHandle docs + local AppState pattern
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct CaptureRuntime {
    cancel: CancellationToken,
    monitor_task: JoinHandle<()>,
    save_task: JoinHandle<()>,
}

impl CaptureRuntime {
    pub fn spawn(ctx: CaptureContext) -> Self {
        let cancel = CancellationToken::new();
        let monitor_cancel = cancel.child_token();
        let save_cancel = cancel.child_token();

        let monitor_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = monitor_cancel.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                        ctx.poll_once().await;
                    }
                }
            }
        });

        let save_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = save_cancel.cancelled() => break,
                    maybe_entry = ctx.recv_entry() => {
                        if let Some(entry) = maybe_entry {
                            ctx.persist(entry).await;
                        } else {
                            break;
                        }
                    }
                }
            }
        });

        Self {
            cancel,
            monitor_task,
            save_task,
        }
    }

    pub async fn stop(self) {
        self.cancel.cancel();
        let _ = self.monitor_task.await;
        let _ = self.save_task.await;
    }
}
```

### Pattern 2: Single Storage Authority + Explicit Legacy Migration

**What:** One `AppPaths` module owns every filesystem root and exposes migration from both current legacy roots: `dance` and `clipboard-app`.

**When to use:** Any code touching config, database, images, icons, logs, cache stats, temp files, cleanup, or migration.

**Why:** Current code mixes `config_dir/dance` and `config_dir/clipboard-app`, so “the app’s storage” does not mean one thing.

**Recommended storage contract:**

- Config JSON -> `app_config_dir`
- Persistent database -> `app_data_dir`
- Persistent image assets referenced by history -> `app_data_dir/imgs`
- Ephemeral icon cache or derived preview cache -> `app_cache_dir`
- Logs -> `app_log_dir`

This is a coherent lifecycle even though it uses multiple app-scoped subdirectories. The important rule is one authority and one documented contract.

**Example:**

```rust
// Source: Tauri PathResolver docs + current tauri.conf identifier com.dance.app
use tauri::{AppHandle, Manager};
use std::path::PathBuf;

pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppPaths {
    pub fn from_app(app: &AppHandle) -> anyhow::Result<Self> {
        let resolver = app.path();
        Ok(Self {
            config_dir: resolver.app_config_dir()?,
            data_dir: resolver.app_data_dir()?,
            cache_dir: resolver.app_cache_dir()?,
            log_dir: resolver.app_log_dir()?,
        })
    }
}
```

### Pattern 3: Marker-First Capture Policy

**What:** Build a pure policy function that decides `Persist`, `CurrentOnly`, or `Skip` before content subtype detection or database writes.

**When to use:** Every clipboard change after raw pasteboard metadata is collected.

**Why:** `TransientType`, `ConcealedType`, `AutoGeneratedType`, `org.nspasteboard.source`, and `com.apple.is-remote-clipboard` are independent of payload text. The policy decision belongs before subtype detection and before persistence.

**Example:**

```rust
// Source: NSPasteboard.org marker guidance + Maccy ignore-type defaults
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureDisposition {
    Persist,
    CurrentOnly,
    Skip,
}

pub struct PasteboardMarkers {
    pub is_transient: bool,
    pub is_concealed: bool,
    pub is_auto_generated: bool,
    pub is_remote: bool,
    pub source_bundle_id: Option<String>,
}

pub fn decide_capture(markers: &PasteboardMarkers, self_generated: bool) -> CaptureDisposition {
    if self_generated || markers.is_transient || markers.is_concealed || markers.is_remote {
        return CaptureDisposition::Skip;
    }

    if markers.is_auto_generated {
        return CaptureDisposition::CurrentOnly;
    }

    CaptureDisposition::Persist
}
```

`CurrentOnly` is optional in implementation, but the planner should decide it explicitly. If the app does not need a “show current without history” state in Phase 1, collapse it to `Skip` and document the choice.

### Pattern 4: Deterministic Self-Write Suppression

**What:** Replace the current global `skip_next_change: bool` idea with a short-lived suppression registry keyed by content hash and timestamp.

**When to use:** Any app-originated copy flow from history recall, copy buttons, image re-copy, or paste automation.

**Why:** A boolean skip is too coarse even if it were wired in. If another app changes the clipboard before the next poll, the wrong event gets skipped.

**Example:**

```rust
// Source: local code constraints; this is a recommended pattern, not copied code
pub struct SuppressionEntry {
    pub content_hash: String,
    pub expires_at_ms: i64,
}

pub fn should_suppress(now_ms: i64, current_hash: &str, pending: &[SuppressionEntry]) -> bool {
    pending
        .iter()
        .any(|entry| entry.content_hash == current_hash && entry.expires_at_ms >= now_ms)
}
```

### Anti-Patterns to Avoid

- **Drop-only stop logic:** setting `Option<ClipboardMonitor>` to `None` does not cancel spawned tasks in `state.rs` and `clipboard/monitor.rs`.
- **One persistence subscriber per start cycle:** repeated `start_monitoring()` calls must not spawn multiple database consumers on the same broadcast channel.
- **Frontend-only clipboard writes for recall flows:** backend suppression cannot work if the renderer writes directly through `writeText`.
- **Hard-coded storage roots:** `"dance"` and `"clipboard-app"` literals across modules guarantee drift and broken stats.
- **Content-only ignore heuristics:** `trim()`, `data:image` checks, and bundle-ID heuristics alone cannot cover transient/concealed/remote markers.

## Don't Hand-Roll

| Problem                  | Don't Build                                               | Use Instead                                      | Why                                                               |
| ------------------------ | --------------------------------------------------------- | ------------------------------------------------ | ----------------------------------------------------------------- |
| Async task shutdown      | ad-hoc booleans and drop semantics                        | `CancellationToken` + owned `JoinHandle`s        | Explicit stop semantics are easier to reason about and test       |
| Storage roots            | repeated `dirs::config_dir()` joins with app-name strings | one `AppPaths` module using Tauri `PathResolver` | Avoids path drift and makes migration/cleanup centralized         |
| Dedupe under concurrency | separate `SELECT` then `INSERT`/`UPDATE` per worker       | unique constraint plus SQLite `UPSERT`           | Atomic dedupe under repeated listeners or save-worker races       |
| Self-write ignore logic  | one global `skip_next_change` flag                        | short-lived hash/time suppression registry       | Prevents skipping the wrong clipboard event                       |
| Marker policy            | regexes on content text only                              | native marker adapter + pure policy function     | Clipboard history rules depend on metadata, not just payload text |
| Hermetic storage tests   | real user config directories in tests                     | injected temp roots with `tempfile`              | Current tests are not safe enough for migration/path work         |

**Key insight:** Phase 1 is mostly about coordination bugs, not domain novelty. The correct primitives already exist; the planner should spend effort on ownership boundaries and verification, not inventing new control mechanisms.

## Common Pitfalls

### Pitfall 1: “Stopped” UI State Without Stopped Work

**What goes wrong:** The UI and `is_monitoring()` report “off” while the poll loop and database save task keep running.

**Why it happens:** Current code stores monitor presence as `Option<ClipboardMonitor>`, but the real work is spawned and detached.

**How to avoid:** Make stop consume or mutate an owned runtime object that can cancel and join every task it created.

**Warning signs:** New history still appears after stop; repeated start/stop cycles increase duplicate saves; logs show multiple save-task starts.

### Pitfall 2: Duplicate Rows Caused by Duplicate Save Workers

**What goes wrong:** The same clipboard event is persisted more than once after repeated starts or duplicate listeners.

**Why it happens:** Each `start_monitoring()` currently subscribes a new broadcast receiver and spawns a new save loop. The table has no uniqueness constraint on `content_hash`.

**How to avoid:** Guarantee only one save worker per runtime and make dedupe atomic with a unique key plus `UPSERT`.

**Warning signs:** Same `content_hash` appears in multiple rows; copy counts and rows diverge; database logs show parallel inserts for the same hash.

### Pitfall 3: Migration by Path Switch Causes Silent Data Loss

**What goes wrong:** New code reads only the new canonical root and “loses” existing database files, configs, or images that still live under the old roots.

**Why it happens:** Current storage is already split across `dance` and `clipboard-app`. Simply changing helper functions will strand existing files.

**How to avoid:** Plan explicit discovery and migration order:

- detect both legacy roots
- choose canonical new root
- migrate or adopt existing DB/config/images/icons
- write a migration marker so the process is idempotent

**Warning signs:** Empty history after upgrade; missing image files for old rows; cache stats suddenly reset despite old data existing on disk.

### Pitfall 4: `arboard` Abstraction Gap Hides Marker Metadata

**What goes wrong:** CAPT-03 gets implemented as more content heuristics, but transient or concealed items still leak into history.

**Why it happens:** `arboard` documents text, image, HTML, and file-list access, but not macOS pasteboard marker type inspection.

**How to avoid:** Treat marker inspection as a macOS platform adapter. Keep `arboard` for payload I/O, but do not expect it to answer marker-policy questions.

**Warning signs:** Password managers, text expanders, or Universal Clipboard entries still appear in history despite new rules.

### Pitfall 5: Non-Hermetic Tests Touch Real User Storage

**What goes wrong:** Tests pass or fail based on the developer machine’s real config directories, and migration tests risk polluting user data.

**Why it happens:** Current test factories use temp SQLite pools, but `ContentProcessor::new()` and `ConfigManager::new()` still resolve real app directories.

**How to avoid:** Inject `AppPaths` or base directories into `Database`, `ConfigManager`, `ContentProcessor`, and `AppState`.

**Warning signs:** Temp tests create files under real `~/Library/Application Support`; failures reproduce only on some machines.

### Pitfall 6: Backend Suppression Never Fires Because Renderer Wrote the Clipboard

**What goes wrong:** Copying from inside the app creates new history entries for the app’s own content.

**Why it happens:** The renderer frequently uses `writeText()` directly, so Rust cannot register a suppression fingerprint before the clipboard changes.

**How to avoid:** Route history-affecting copy flows through a backend command or an explicit backend suppression-registration command.

**Warning signs:** Re-copying an old item creates a fresh row or increments copy count unexpectedly; `skip_next_change` exists but has no effect.

## Code Examples

Verified patterns from official sources and direct codebase evidence:

### Atomic Dedupe Write

```sql
-- Source: SQLite UPSERT docs + local clipboard_entries schema
CREATE UNIQUE INDEX IF NOT EXISTS idx_clipboard_entries_content_hash
ON clipboard_entries(content_hash);

INSERT INTO clipboard_entries (
  id,
  content_hash,
  content_type,
  content_data,
  source_app,
  created_at,
  copy_count,
  file_path,
  is_favorite,
  content_subtype,
  metadata,
  app_bundle_id
)
VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, ?)
ON CONFLICT(content_hash) DO UPDATE SET
  copy_count = clipboard_entries.copy_count + 1,
  created_at = excluded.created_at,
  source_app = excluded.source_app,
  content_subtype = excluded.content_subtype,
  metadata = excluded.metadata,
  app_bundle_id = excluded.app_bundle_id;
```

### App-Scoped Path Resolution

```rust
// Source: Tauri PathResolver docs
use tauri::{AppHandle, Manager};

pub fn image_assets_dir(app: &AppHandle) -> anyhow::Result<std::path::PathBuf> {
    Ok(app.path().app_data_dir()?.join("imgs"))
}

pub fn icon_cache_dir(app: &AppHandle) -> anyhow::Result<std::path::PathBuf> {
    Ok(app.path().app_cache_dir()?.join("icons"))
}
```

### Backend-Owned Copy for Suppression

```rust
// Source: recommended pattern derived from current state.rs + clipboardStore.ts split
#[tauri::command]
pub async fn copy_to_clipboard(
    state: tauri::State<'_, AppState>,
    content: String,
) -> Result<(), String> {
    state
        .register_suppression_for_text(&content)
        .await
        .map_err(|e| e.to_string())?;

    state
        .copy_to_clipboard(content)
        .await
        .map_err(|e| e.to_string())
}
```

## State of the Art

| Old Approach                                                           | Current Approach                                     | When Changed                                                                                                  | Impact                                                                                            |
| ---------------------------------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Drop a struct handle and assume background work stops                  | Cooperative cancellation plus owned task handles     | Current Tokio 1.x ecosystem                                                                                   | Start/stop/restart semantics become explicit and testable                                         |
| Hard-code app paths with `dirs::config_dir()` and product-name strings | Use Tauri `PathResolver` app directories             | Tauri 2 stable (`2024-10-02`)                                                                                 | Paths become bundle-aware and consistent across config/data/cache/log                             |
| Query for duplicates, then insert/update in separate steps             | `UPSERT` on a uniqueness constraint                  | SQLite `3.24.0` (`2018-06-04`), generalized `3.35.0` (`2021-03-12`)                                           | Dedupe remains correct under concurrent events                                                    |
| Infer “should skip” only from text payload or current foreground app   | Inspect pasteboard markers and source metadata first | Established macOS clipboard-manager practice; remote clipboard marker documented by 2025 community references | Prevents history pollution from transient, concealed, auto-generated, or remote clipboard content |

**Deprecated/outdated:**

- Drop-only monitor ownership: does not satisfy CAPT-01.
- Global boolean “skip next change”: too coarse for CAPT-02 in a polling monitor.
- Storage-root string literals in multiple modules: incompatible with CAPT-04.

## Open Questions

1. **Should image assets be treated as persistent data or disposable cache?**
   - What we know: CAPT-04 requires history to survive restart with image assets intact.
   - What's unclear: whether current “cache” language in UI/settings means all images are disposable.
   - Recommendation: treat image files referenced by history rows as persistent data under `app_data_dir/imgs`; only derived icons/thumbnails belong in `app_cache_dir`.

2. **How much cross-platform marker parity belongs in Phase 1?**
   - What we know: the user’s known concern is specifically about transient/concealed/auto-generated/remote markers; the strongest evidence found is macOS-centric.
   - What's unclear: equivalent marker support expectations on Windows for this phase.
   - Recommendation: implement full marker-aware policy on macOS now, keep a no-op adapter elsewhere, and document the capability boundary so later phases can extend it deliberately.

3. **Should `AutoGeneratedType` be `Skip` or `CurrentOnly` in v1?**
   - What we know: community guidance suggests it often should not enter persistent history, but may still represent the current clipboard.
   - What's unclear: whether the product needs a visible “current clipboard but not history” concept in Phase 1.
   - Recommendation: decide this explicitly during planning; do not leave it implicit inside monitor code.

4. **How broad should self-write suppression be?**
   - What we know: history recall copy flows and automatic paste flows should not create new history entries.
   - What's unclear: whether generic text selection copied inside app chrome should also be suppressed in v1.
   - Recommendation: Phase 1 should at minimum suppress product-triggered recall/copy/paste flows; broader in-app copy behavior can be a follow-up if user experience demands it.

## Environment Availability

| Dependency | Required By                                            | Available | Version    | Fallback                                        |
| ---------- | ------------------------------------------------------ | --------- | ---------- | ----------------------------------------------- |
| Node.js    | Frontend tests and repo scripts                        | ✓         | `v24.13.0` | —                                               |
| `pnpm`     | Frontend test runner and Tauri scripts                 | ✓         | `10.0.0`   | Avoid fallback; repo lockfile is pnpm-first     |
| `cargo`    | Rust tests and native build/test flow                  | ✓         | `1.91.0`   | —                                               |
| `rustc`    | Native compilation and test runtime                    | ✓         | `1.91.0`   | —                                               |
| macOS      | Real clipboard, pasteboard markers, and focus behavior | ✓         | `26.4`     | No faithful fallback for native marker behavior |

**Missing dependencies with no fallback:**

- None detected for planning and local validation.

**Missing dependencies with fallback:**

- None detected.

## Validation Architecture

### Test Framework

| Property           | Value                                                                                        |
| ------------------ | -------------------------------------------------------------------------------------------- |
| Framework          | Vitest `4.1.2` for frontend contracts; Rust `cargo test` for backend lifecycle/storage logic |
| Config file        | `vitest.config.ts`; Rust has no separate config file                                         |
| Quick run command  | `cd src-tauri && cargo test capture_ -- --nocapture`                                         |
| Full suite command | `pnpm test` and `cd src-tauri && cargo test`                                                 |

### Phase Requirements -> Test Map

| Req ID  | Behavior                                                                                                  | Test Type                   | Automated Command                                                                              | File Exists? |
| ------- | --------------------------------------------------------------------------------------------------------- | --------------------------- | ---------------------------------------------------------------------------------------------- | ------------ |
| CAPT-01 | Start, stop, and restart monitoring cancel old tasks and never leave background polling active            | Rust integration            | `cd src-tauri && cargo test test_capture_runtime_stop_cancels_tasks -- --nocapture`            | ❌ Wave 0    |
| CAPT-02 | Repeated starts and app-originated copy flows do not create duplicate history rows or duplicate listeners | Rust integration            | `cd src-tauri && cargo test test_capture_runtime_single_worker_and_suppression -- --nocapture` | ❌ Wave 0    |
| CAPT-03 | Transient, concealed, auto-generated, remote, and self-generated events are filtered according to policy  | Rust unit + manual smoke    | `cd src-tauri && cargo test test_capture_policy_marker_matrix -- --nocapture`                  | ❌ Wave 0    |
| CAPT-04 | Config, DB, image assets, and caches resolve through one authority and migrate legacy roots safely        | Rust filesystem integration | `cd src-tauri && cargo test test_app_paths_migrate_legacy_roots -- --nocapture`                | ❌ Wave 0    |

### Sampling Rate

- **Per task commit:** `cd src-tauri && cargo test capture_ -- --nocapture`
- **Per wave merge:** `pnpm test` and `cd src-tauri && cargo test`
- **Phase gate:** full frontend and Rust suites green, plus manual macOS smoke for marker-aware capture behavior

### Wave 0 Gaps

- [ ] `src-tauri/src/test_support.rs` (or equivalent) to inject temp roots into `AppPaths`, `Database`, `ConfigManager`, `ContentProcessor`, and `AppState`
- [ ] `src-tauri/src/capture_runtime_tests.rs` for CAPT-01 and CAPT-02 lifecycle/dedupe behavior
- [ ] `src-tauri/src/capture_policy_tests.rs` for CAPT-03 pure policy matrix and self-write suppression cases
- [ ] `src-tauri/src/app_paths_tests.rs` for CAPT-04 legacy-root discovery and migration
- [ ] `src/stores/clipboardStore.test.ts` extension if frontend event listener setup/cleanup or copy-routing contracts change

## Sources

### Primary (HIGH confidence)

- Local requirements and phase docs:
  - `.planning/REQUIREMENTS.md`
  - `.planning/ROADMAP.md`
  - `.planning/STATE.md`
  - `.planning/research/SUMMARY.md`
  - `.planning/codebase/ARCHITECTURE.md`
  - `.planning/codebase/CONCERNS.md`
  - `.planning/codebase/TESTING.md`
- Local implementation evidence:
  - `src-tauri/src/state.rs`
  - `src-tauri/src/clipboard/monitor.rs`
  - `src-tauri/src/config/mod.rs`
  - `src-tauri/src/database/mod.rs`
  - `src-tauri/src/clipboard/processor.rs`
  - `src/stores/clipboardStore.ts`
  - `src/App.tsx`
  - `src/components/MenuEventHandler/MenuEventHandler.tsx`
  - `src-tauri/tauri.conf.json`
- Tauri PathResolver docs: `https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html`
- Tokio `CancellationToken` docs: `https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html`
- Tokio `JoinHandle::abort` docs: `https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html`
- SQLite UPSERT docs: `https://sqlite.org/lang_upsert.html`
- arboard clipboard docs: `https://docs.rs/arboard/latest/arboard/struct.Clipboard.html`

### Secondary (MEDIUM confidence)

- NSPasteboard marker guidance: `https://nspasteboard.org/`
- Maccy README and release notes as current ecosystem practice:
  - `https://github.com/p0deje/Maccy`
  - `https://github.com/p0deje/Maccy/releases`

### Tertiary (LOW confidence)

- None. I did not rely on unverified one-off blog posts for critical recommendations.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - driven by official docs plus the repo’s locked architecture
- Architecture: HIGH - directly supported by current code hotspots and failure modes
- Pitfalls: MEDIUM - marker-policy details rely partly on current ecosystem references rather than Apple’s primary docs

**Research date:** 2026-03-27
**Valid until:** 2026-04-26
