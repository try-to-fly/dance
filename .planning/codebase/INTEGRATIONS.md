# External Integrations

**Analysis Date:** 2026-03-27

## APIs & External Services

**Product analytics:**

- Aptabase - captures app lifecycle and feature-usage events from the desktop runtime
  - SDK/Client: `@aptabase/tauri` in `src/services/analytics.ts` and `tauri-plugin-aptabase` in `src-tauri/src/lib.rs`
  - Auth: `APTABASE_APP_KEY` loaded in `src-tauri/src/lib.rs` and referenced by `.github/workflows/release.yml` plus `.github/workflows/test-build.yml`

**Application updates and binary distribution:**

- GitHub Releases - serves the updater manifest and downloadable desktop artifacts
  - SDK/Client: `@tauri-apps/plugin-updater` in `src/components/UpdateChecker/UpdateChecker.tsx` and `tauri-plugin-updater` in `src-tauri/src/updater/mod.rs`
  - Auth: public updater endpoint is configured in `src-tauri/tauri.conf.json`; publishing uses `GITHUB_TOKEN` in `.github/workflows/release.yml`

**Remote URL inspection:**

- Arbitrary user-copied HTTP(S) endpoints - fetched to resolve preview type, final URL, body snippets, and media metadata
  - SDK/Client: `reqwest` commands `resolve_url_preview`, `fetch_url_content`, and media inspection logic in `src-tauri/src/commands.rs`; frontend callers live in `src/stores/clipboardStore.ts`
  - Auth: none detected; commands only accept absolute `http/https` URLs in `src-tauri/src/commands.rs`

**Website hosting and delivery:**

- Vercel - deployment config is present for the website surface
  - SDK/Client: no runtime SDK detected; deployment files are `vercel.json`, `website/next.config.js`, and `website/scripts/should-build-website.sh`
  - Auth: repository does not store Vercel credentials

**Auxiliary sync directories:**

- `dance-sync-server/` and `dance-sync-server-node/` - directories exist, but this scan only found `.env` files plus compiled output in `dance-sync-server-node/dist/`; no active package manifest or root integration point was detected
  - SDK/Client: Not detected
  - Auth: `.env` files are present at `dance-sync-server/.env` and `dance-sync-server-node/.env`; contents were not read

## Data Storage

**Databases:**

- Embedded SQLite database on local filesystem
  - Connection: no env var; `src-tauri/src/database/mod.rs` constructs `sqlite:{config_dir}/dance/clipboard.db?mode=rwc`
  - Client: `sqlx::SqlitePool` in `src-tauri/src/database/mod.rs`
- Additional local database path references
  - Connection: no env var; cache-statistics and cleanup helpers in `src-tauri/src/state.rs` reference `dirs::config_dir()/clipboard-app/clipboard.db`
  - Client: direct filesystem inspection plus `sqlx` queries in `src-tauri/src/state.rs`

**File Storage:**

- Local filesystem only
  - Config: `dirs::config_dir()/dance/config.json` in `src-tauri/src/config/mod.rs`
  - Clipboard images: `dirs::config_dir()/clipboard-app/imgs` in `src-tauri/src/clipboard/processor.rs`
  - Cached app icons: `dirs::config_dir()/clipboard-app/icons` in `src-tauri/src/utils/app_icon_extractor.rs`
  - Logs: `~/Library/Logs/com.dance.app/dance.log` in `src-tauri/src/commands.rs`
  - Webview preference storage: `localStorage['analytics_enabled']` in `src/services/analytics.ts`

**Caching:**

- No external cache service was detected from imports and config across `src/`, `src-tauri/`, `website/`, and `.github/workflows/`
- In-memory preview cache with TTL logic lives in `src/stores/clipboardStore.ts`
- Filesystem cache is used for saved clipboard images in `src-tauri/src/clipboard/processor.rs` and app icons in `src-tauri/src/utils/app_icon_extractor.rs`

## Authentication & Identity

**Auth Provider:**

- None for end users
  - Implementation: no login, OAuth, session store, or identity provider references were detected in `src/`, `src-tauri/`, or `website/`
- Release signing identity
  - Implementation: `.github/workflows/release.yml` and `.github/workflows/test-build.yml` use `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` for bundle signing

## Monitoring & Observability

**Error Tracking:**

- No dedicated external error tracker was detected in `src/`, `src-tauri/`, `website/`, or `.github/workflows/`
- Aptabase acts as the only external telemetry sink, with calls in `src/services/analytics.ts` and `src-tauri/src/lib.rs`

**Logs:**

- Tauri log plugin writes to stdout, log directory, and webview targets from `src-tauri/src/lib.rs`
- Log read/clear commands live in `src-tauri/src/commands.rs`
- The desktop log viewer UI lives in `src/components/LogViewer/LogViewer.tsx`

## CI/CD & Deployment

**Hosting:**

- Desktop binaries are published to GitHub Releases via `.github/workflows/release.yml`
- Website deployment is intended through Vercel via `vercel.json` and `website/next.config.js`
- Current repo state keeps website source in `website/`, while root `vercel.json` is also present; the build-skip helper exists at `website/scripts/should-build-website.sh`

**CI Pipeline:**

- `.github/workflows/release.yml` builds macOS Apple Silicon, macOS Intel, and Windows bundles and uploads them to a release
- `.github/workflows/test-build.yml` runs manual test builds with optional signing
- `scripts/update-version.js` bumps version numbers locally, commits them, and creates the release tag that CI consumes

## Environment Configuration

**Required env vars:**

- `APTABASE_APP_KEY` - analytics provider key consumed by `src-tauri/src/lib.rs`
- `TAURI_DEV_HOST` - optional Vite/Tauri dev host override read by `vite.config.ts`
- `GITHUB_TOKEN` - GitHub release publishing token referenced in `.github/workflows/release.yml`
- `TAURI_SIGNING_PRIVATE_KEY` - Tauri bundle signing key referenced in `.github/workflows/release.yml` and `.github/workflows/test-build.yml`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - signing-key password referenced in `.github/workflows/release.yml` and `.github/workflows/test-build.yml`

**Secrets location:**

- Repository or organization secrets are referenced from `.github/workflows/release.yml` and `.github/workflows/test-build.yml`
- Local development env config is present at `src-tauri/.env` and is loaded in debug mode by `src-tauri/src/lib.rs`
- Auxiliary env files are present at `dance-sync-server/.env` and `dance-sync-server-node/.env`; contents were not read

## Webhooks & Callbacks

**Incoming:**

- None detected for HTTP webhooks or third-party callbacks in `src/`, `src-tauri/`, or `website/`
- Internal desktop IPC events exist between Rust and the webview, including `clipboard-update`, `update-download-progress`, `show_statistics`, and menu events emitted from `src-tauri/src/lib.rs` and consumed in `src/stores/clipboardStore.ts`, `src/components/UpdateChecker/UpdateChecker.tsx`, and `src/App.tsx`

**Outgoing:**

- Updater requests to `https://github.com/try-to-fly/dance/releases/latest/download/latest.json` are configured in `src-tauri/tauri.conf.json`
- Aptabase analytics submissions originate from `src/services/analytics.ts` and `src-tauri/src/lib.rs`
- Remote HTTP(S) GET requests to user-provided URLs are issued by `resolve_url_preview` and `fetch_url_content` in `src-tauri/src/commands.rs`
- Browser navigation to external resources such as `https://github.com/try-to-fly/dance`, `https://github.com/try-to-fly/dance/releases/latest`, and `https://aptabase.com` is triggered from `website/app/page.tsx` and `src/components/Preferences/PreferencesModal.tsx`

---

_Integration audit: 2026-03-27_
