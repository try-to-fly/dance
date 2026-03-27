# Codebase Concerns

**Analysis Date:** 2026-03-27

## Tech Debt

**Desktop monitoring lifecycle is implemented as fire-and-forget tasks instead of controllable services:**

- Issue: `AppState::start_monitoring()` creates a `ClipboardMonitor`, starts an internal `tokio::spawn`, then separately starts another database subscriber task, but `AppState::stop_monitoring()` only drops the handle stored in memory and never cancels either background loop.
- Files: `src-tauri/src/state.rs`, `src-tauri/src/clipboard/monitor.rs`, `src-tauri/src/state_tests.rs`
- Impact: Monitoring can continue after the UI says it stopped, repeated stop/start cycles can stack multiple pollers and database subscribers, and production behavior diverges from the visible `is_monitoring` flag.
- Fix approach: Replace the current `Option<ClipboardMonitor>` handle with an owned service object that stores `JoinHandle`s or a cancellation token, and add integration coverage for start/stop/restart semantics.

**Application data paths are split across two storage roots (`dance` and `clipboard-app`):**

- Issue: config and database modules write to `config_dir/dance`, while image cache, icon cache, some commands, and cache-stat helpers read from `config_dir/clipboard-app`.
- Files: `src-tauri/src/config/mod.rs`, `src-tauri/src/database/mod.rs`, `src-tauri/src/state.rs`, `src-tauri/src/clipboard/processor.rs`, `src-tauri/src/utils/app_icon_extractor.rs`, `src-tauri/src/commands.rs`
- Impact: storage is harder to reason about, migration behavior is implicit, cache statistics can point at the wrong database file, and tests are more likely to leak into real user data.
- Fix approach: Introduce one shared app-path helper module and migrate every config, DB, image, icon, and log path through that single source of truth.

**Core workflows are concentrated in a few oversized modules with cross-cutting responsibilities:**

- Issue: command routing, preview resolution, media inspection, clipboard persistence, and preferences UI are each implemented in files that are hundreds to thousands of lines long.
- Files: `src-tauri/src/commands.rs`, `src-tauri/src/clipboard/content_detector.rs`, `src/stores/clipboardStore.ts`, `src/components/Preferences/PreferencesModal.tsx`
- Impact: changes in one feature area are likely to touch unrelated code, review scope is large, and regression risk rises because the files mix I/O, business rules, UI state, and serialization.
- Fix approach: Split by domain boundary first, not by line count. The highest-value seams are preview commands, file commands, analytics/update commands, store caching, and preferences sub-panels.

**Quality gates are fragmented across subprojects and miss major parts of the repo:**

- Issue: the root ESLint config ignores `src-tauri` and `website`, while CI workflows build release artifacts but do not run the existing TypeScript or Rust test suites.
- Files: `eslint.config.js`, `package.json`, `.github/workflows/release.yml`, `.github/workflows/test-build.yml`, `website/package.json`
- Impact: the most platform-sensitive code ships with weaker automated checks than the React surface, and regressions can survive until manual testing on macOS or Windows.
- Fix approach: add explicit `pnpm test`, `cargo test`, and lint steps to CI, then make per-subproject quality gates visible from the repo root.

**Clipboard self-suppression is only partially implemented:**

- Issue: `skip_next_change` can be set from image-copy flows, but no monitoring path actually reads it before polling and recording clipboard changes.
- Files: `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`
- Impact: the code suggests a loop-prevention mechanism exists, but future changes may rely on a flag that is currently dead state.
- Fix approach: either wire `skip_next_change` into `ClipboardMonitor::check_clipboard()` with a clear reset policy, or remove the flag entirely and document the real dedupe strategy.

## Known Bugs

**Stopping monitoring does not stop real clipboard polling:**

- Symptoms: the UI can show monitoring as disabled while the background loop created in `ClipboardMonitor::start_monitoring()` keeps running every 500 ms.
- Files: `src-tauri/src/state.rs`, `src-tauri/src/clipboard/monitor.rs`
- Trigger: start monitoring once, stop it, then copy new content or start monitoring again.
- Workaround: none in code; only a full app restart guarantees the spawned tasks are gone.

**Deleting image entries or clearing history leaves cached image files behind:**

- Symptoms: image rows disappear from the database, but files saved under the image cache directory remain on disk.
- Files: `src-tauri/src/state.rs`, `src-tauri/src/clipboard/processor.rs`, `src/stores/clipboardStore.ts`
- Trigger: delete an image entry through the list/detail actions, or use “Clear History” while image expiry is `Never`.
- Workaround: run expiry cleanup later or manually remove cached files from the app image directory.

**Cache statistics can report the wrong database size:**

- Symptoms: the Preferences cache panel shows a database size derived from `config_dir/clipboard-app/clipboard.db`, while the real SQLite database is created in `config_dir/dance/clipboard.db`.
- Files: `src-tauri/src/database/mod.rs`, `src-tauri/src/state.rs`, `src/stores/configStore.ts`, `src/components/Preferences/PreferencesModal.tsx`
- Trigger: open Preferences and inspect the cache statistics cards.
- Workaround: none in the UI; the statistics are only trustworthy for image counts and entry counts, not the database file path.

**Log viewer controls do not match the actual logger configuration:**

- Symptoms: the viewer reads `~/Library/Logs/com.dance.app/dance.log`, the logger is configured with file name `clipboard-app`, `set_log_level` is a placeholder, and `get_current_log_level` always returns `info`.
- Files: `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`, `src/components/LogViewer/LogViewer.tsx`
- Trigger: open the log viewer, try to change log level, or expect non-empty logs from the configured log target.
- Workaround: inspect the real OS log directory manually; runtime level switching is not supported by the current implementation.

**Repo-root website deployment configuration is out of sync with the actual website project layout:**

- Symptoms: the root `vercel.json` points `ignoreCommand` to `./scripts/should-build-website.sh`, but the only script lives in `website/scripts/should-build-website.sh`; the README also describes a different build/output layout from the current root config.
- Files: `vercel.json`, `website/scripts/should-build-website.sh`, `website/README.md`
- Trigger: deploy the website from the repository root using the checked-in `vercel.json`.
- Workaround: deploy from the `website` directory directly and bypass the root config.

## Security Considerations

**Several Tauri commands accept arbitrary filesystem paths from the renderer:**

- Risk: if the renderer is compromised, commands can read or open arbitrary local files because non-`imgs/` inputs are accepted as raw `PathBuf`s and only checked for existence.
- Files: `src-tauri/src/commands.rs`, `src-tauri/src/state.rs`
- Current mitigation: existence checks and relative-path handling for app-managed `imgs/` paths.
- Recommendations: canonicalize all paths, restrict file access to app-managed directories or explicit allowlists, and reject absolute paths from the renderer by default.

**Clipboard URL previews automatically perform backend network requests and media inspection:**

- Risk: selecting or previewing a copied URL can trigger `reqwest` requests and `ffprobe` inspection against arbitrary HTTP(S) targets, including internal services or sensitive localhost endpoints.
- Files: `src/components/DetailView/DetailView.tsx`, `src/stores/clipboardStore.ts`, `src-tauri/src/commands.rs`
- Current mitigation: only absolute HTTP(S) URLs are accepted, request bodies are capped for text previews, and a 30-second timeout is configured.
- Recommendations: gate remote preview behind explicit user consent, block private-network/loopback targets by default, and add cancellation so stale preview jobs do not continue after selection changes.

**Analytics opt-out is frontend-only while Rust-side tracking remains active:**

- Risk: users can disable analytics in the React preferences screen, but Rust still emits lifecycle and command telemetry through Aptabase because the opt-out state is never propagated to the backend.
- Files: `src/components/Preferences/PreferencesModal.tsx`, `src/services/analytics.ts`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- Current mitigation: frontend analytics events are locally gated and event properties are sanitized before sending.
- Recommendations: persist the analytics preference in `AppConfig`, read it during Rust startup, and guard every `track_event` call in the backend as well as the frontend.

**The desktop webview CSP is intentionally broad and keeps dangerous script allowances enabled:**

- Risk: `unsafe-inline`, `unsafe-eval`, and wide `http: https: data: blob:` allowances increase the blast radius of any renderer-side bug and make arbitrary path/file commands more valuable to an attacker.
- Files: `src-tauri/tauri.conf.json`
- Current mitigation: `object-src 'none'` is set and resource classes are at least scoped by CSP directives.
- Recommendations: narrow `connect-src`, remove `unsafe-eval` if Monaco can be isolated, and prefer Tauri opener APIs over raw `window.open` for external navigation.

## Performance Bottlenecks

**Clipboard monitoring polls continuously and does expensive work on the hot path:**

- Problem: every 500 ms the monitor checks the active app, opens clipboard handles for text and image separately, hashes content, and runs content detection before any DB persistence happens.
- Files: `src-tauri/src/clipboard/monitor.rs`, `src-tauri/src/clipboard/content_detector.rs`
- Cause: monitoring is timer-based rather than event-driven, and regexes/content heuristics are recreated during detection instead of being cached.
- Improvement path: move to OS-native change notifications where possible, add adaptive backoff when idle, and precompile frequently used regexes with lazy statics.

**Search uses full-table `%LIKE%` scans on every debounced term change:**

- Problem: user input triggers `get_clipboard_history()` after a 200 ms debounce, and the backend runs `%search_term%` matching on `content_data` and `source_app` with no FTS index.
- Files: `src/components/SearchBar/SearchBar.tsx`, `src-tauri/src/state.rs`, `src-tauri/src/database/mod.rs`
- Cause: the schema only indexes `created_at`, `content_hash`, `content_subtype`, and `app_bundle_id`; it does not support scalable text search.
- Improvement path: add SQLite FTS5 or a normalized searchable column, cancel stale searches, and avoid refetching the first page on every keystroke when only client-side filtering is needed.

**Preview resolution can duplicate remote work across backend and frontend:**

- Problem: URL preview resolution may fetch content and inspect media in Rust, then the renderer can call fallback fetches or `extractMediaMetadata()` again for the same URL.
- Files: `src/stores/clipboardStore.ts`, `src/components/DetailView/ContentRenderers/UrlRenderer.tsx`, `src-tauri/src/commands.rs`
- Cause: there is no single preview pipeline or ownership boundary for “who resolves metadata.”
- Improvement path: make the backend authoritative for preview resolution, return all metadata in one shape, and let the frontend only render/cache the response.

**Image storage and conversion can accumulate unnecessary disk and memory pressure:**

- Problem: image cache files are persisted separately from DB rows, deleted rows do not reclaim files, and image conversion returns large base64 data URLs back into renderer memory.
- Files: `src-tauri/src/clipboard/processor.rs`, `src-tauri/src/state.rs`, `src/components/DetailView/ImagePreview.tsx`
- Cause: file lifecycle is not coupled to entry lifecycle, and conversion favors inline transport rather than temporary file or stream-based handling.
- Improvement path: delete files transactionally with entry removal, cap conversion output size, and prefer temporary file handoff for large assets.

## Fragile Areas

**Frontend event subscriptions are easy to duplicate and hard to clean up correctly:**

- Files: `src/components/MenuEventHandler/MenuEventHandler.tsx`, `src/stores/clipboardStore.ts`, `src/App.tsx`
- Why fragile: `MenuEventHandler` builds async listeners but never returns the real cleanup function to React, and the store-level `setupEventListener()` registers `clipboard-update` listeners without exposing unsubscription.
- Safe modification: centralize Tauri event registration in one hook that returns explicit cleanup, and ensure App-level mounts, hot reloads, and tests cannot stack listeners.
- Test coverage: no automated test covers listener duplication, teardown, or hot-reload remount behavior.

**macOS paste/focus flows depend on AppleScript timing and a hard-coded process name:**

- Files: `src-tauri/src/state.rs`, `src-tauri/tauri.conf.json`, `src-tauri/src/tray.rs`
- Why fragile: the paste script assumes the process is named `clipboard-app`, while the packaged product name is `Dance`; success also depends on accessibility permissions and focus timing.
- Safe modification: derive the executable/app name from runtime metadata, isolate paste automation behind a platform adapter, and validate the packaged app behavior instead of only dev mode.
- Test coverage: no integration test exercises the full paste-to-foreground-app flow.

**State tests are not hermetic and can depend on real user directories:**

- Files: `src-tauri/src/state_tests.rs`, `src-tauri/src/state.rs`, `src-tauri/src/config/mod.rs`, `src-tauri/src/clipboard/processor.rs`
- Why fragile: test helpers create a temp SQLite pool, but `ContentProcessor`, `ConfigManager`, and path helpers still point at real config directories outside the temp test sandbox.
- Safe modification: inject data directories into `AppState`, `Database`, `ConfigManager`, and `ContentProcessor` so tests run against isolated temp roots.
- Test coverage: the existing tests skip the real monitoring lifecycle and do not validate path isolation.

**Website build/deploy behavior is defined in multiple places that disagree with each other:**

- Files: `vercel.json`, `website/README.md`, `website/next.config.js`, `website/scripts/should-build-website.sh`
- Why fragile: the root deployment config, website README, and actual script locations describe different root directories and output expectations.
- Safe modification: choose one supported deployment path, delete the conflicting config, and verify the final flow in CI.
- Test coverage: there is no deployment smoke test for the website path.

## Scaling Limits

**History search and stats scale linearly with dataset size:**

- Current capacity: pagination limits initial list fetches to 50 rows, but search and stats still issue broad SQL queries over the whole `clipboard_entries` table.
- Limit: large local histories will slow search, stats panels, and duplicate detection long before SQLite itself becomes the hard limit.
- Scaling path: introduce FTS, background compaction, and targeted aggregate tables for statistics instead of recomputing from raw entries.

**The renderer keeps heavyweight clipboard payloads and preview caches in memory:**

- Current capacity: `useClipboardStore` keeps entries, URL content cache, media metadata cache, and preview-resolution cache resident in a single long-lived Zustand store.
- Limit: repeated large JSON/base64/URL preview content can increase memory pressure and make navigation costlier as caches grow.
- Scaling path: normalize preview cache storage, add cache size caps or LRU eviction, and avoid holding full decoded payloads when a summarized preview is sufficient.

## Dependencies at Risk

**macOS-private APIs and unsafe Cocoa/Objective-C interop are concentrated in platform utility code:**

- Risk: platform updates, signing changes, or accessibility restrictions can break app detection, icon extraction, and app listing in ways that are hard to reproduce outside macOS.
- Impact: source-app attribution, excluded-app migration, icon rendering, and focus automation can all regress together.
- Migration plan: wrap platform code behind narrow interfaces and reduce reliance on `macos-private-api`, `unsafe` blocks, and AppleScript where Tauri or OS-supported APIs exist.

**Media metadata resolution depends on `ffprobe` existing on the user machine:**

- Risk: preview behavior silently degrades or becomes inconsistent when `ffprobe` is missing, slow, or behaves differently across platforms.
- Impact: URL/image/audio/video inspection in the detail view and fallback preview flows lose metadata richness or fail noisily.
- Migration plan: treat `ffprobe` as optional capability, memoize failures, and consider bundling or replacing the external binary for supported targets.

## Missing Critical Features

**There is no unified release gate that runs tests and lint before packaging artifacts:**

- Problem: the repository has unit/integration/performance tests and lint scripts, but CI workflows currently build packages rather than proving behavioral correctness.
- Blocks: safe refactors in `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`, `src/stores/clipboardStore.ts`, and `src/components/Preferences/PreferencesModal.tsx` remain high-risk because regression feedback is delayed until manual validation.

**The desktop security model lacks a backend-enforced permission boundary for renderer-triggered file access and telemetry:**

- Problem: the renderer can invoke powerful file/network commands, while privacy preferences do not control backend analytics events.
- Blocks: hardening the app for broader distribution and making privacy guarantees that hold even if the renderer or UI state becomes inconsistent.

## Test Coverage Gaps

**Monitoring start/stop/restart behavior is not tested:**

- What's not tested: actual cancellation of the poller, duplicate task prevention, and end-to-end correctness of monitoring state transitions.
- Files: `src-tauri/src/state.rs`, `src-tauri/src/clipboard/monitor.rs`, `src-tauri/src/state_tests.rs`
- Risk: clipboard capture can keep running after stop, or multiply after restart, without any regression alarm.
- Priority: High

**Filesystem path controls and storage-root consistency are not tested:**

- What's not tested: renderer-supplied absolute paths, canonicalization boundaries, cache-stat path correctness, and migration between `dance` and `clipboard-app` directories.
- Files: `src-tauri/src/commands.rs`, `src-tauri/src/state.rs`, `src-tauri/src/database/mod.rs`, `src-tauri/src/config/mod.rs`
- Risk: local file exposure, inaccurate stats, and non-hermetic tests can break quietly.
- Priority: High

**Logging and updater UX paths are not validated end to end:**

- What's not tested: real log-file discovery, runtime log-level switching behavior, update download progress semantics, and manual update error handling.
- Files: `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/updater/mod.rs`, `src/components/LogViewer/LogViewer.tsx`, `src/components/UpdateChecker/UpdateChecker.tsx`
- Risk: the app can present controls that appear to work while doing nothing or reading the wrong file.
- Priority: Medium

**Website deployment wiring has no smoke coverage:**

- What's not tested: root `vercel.json`, the website build-ignore script path, and the claimed repo-root deployment flow documented in the website README.
- Files: `vercel.json`, `website/scripts/should-build-website.sh`, `website/README.md`
- Risk: the marketing site can fail only after pushing to Vercel, with no repo-local or CI signal.
- Priority: Medium

---

_Concerns audit: 2026-03-27_
