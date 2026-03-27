# Technology Stack

**Analysis Date:** 2026-03-27

## Languages

**Primary:**

- TypeScript `~5.6.2` - Desktop frontend lives in `src/`; website source lives in `website/app/`, `website/components/`, and `website/lib/`; build/test config also uses TS in `vite.config.ts` and `vitest.config.ts`
- Rust `edition = 2021` - Native desktop backend, updater, clipboard processing, database access, and OS integration live in `src-tauri/src/`

**Secondary:**

- JavaScript - Tooling and release automation live in `eslint.config.js`, `tailwind.config.js`, `postcss.config.js`, `website/next.config.js`, `website/tailwind.config.js`, `website/postcss.config.js`, and `scripts/update-version.js`
- SQL (SQLite dialect) - Schema creation, migrations, and queries are embedded in `src-tauri/src/database/mod.rs`, `src-tauri/src/state.rs`, and `src-tauri/src/integration_tests.rs`
- Shell/YAML - CI/CD and deployment automation live in `website/scripts/should-build-website.sh`, `.github/workflows/release.yml`, and `.github/workflows/test-build.yml`

## Runtime

**Environment:**

- Node.js - JavaScript toolchain for the repo root and `website/`; `website/package.json` explicitly requires `>=22`, while `.github/workflows/release.yml` and `.github/workflows/test-build.yml` install Node `lts/*`
- Rust stable toolchain - Native layer is built from `src-tauri/Cargo.toml` and locked by `src-tauri/Cargo.lock`
- Tauri `2.x` desktop runtime - Desktop shell is configured in `src-tauri/tauri.conf.json` and bootstrapped in `src-tauri/src/lib.rs`

**Package Manager:**

- `pnpm` - Root desktop app uses `package.json` plus `pnpm-lock.yaml`; website subproject uses `website/package.json` plus `website/pnpm-lock.yaml`
- `cargo` - Native desktop backend uses `src-tauri/Cargo.toml` plus `src-tauri/Cargo.lock`
- Lockfile: present in all active JS and Rust package layers

## Frameworks

**Core:**

- Tauri `2.x` - Desktop shell, IPC commands, tray, updater, dialog, clipboard, autostart, and global shortcut plugins are wired from `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, and `src-tauri/tauri.conf.json`
- React `18.3.1` - Desktop UI entry points are `src/main.tsx` and `src/App.tsx`
- Vite `6.0.3` - Desktop frontend dev server and bundle pipeline are configured in `vite.config.ts`
- Next.js `15.1.6` with React `19.0.0` - Marketing website runs from `website/app/` and is configured by `website/package.json` and `website/next.config.js`
- Tailwind CSS `3.4.0` - Styling pipelines are configured in `tailwind.config.js`, `postcss.config.js`, `website/tailwind.config.js`, and `website/postcss.config.js`
- Zustand `4.5.0` - Client state stores live in `src/stores/clipboardStore.ts` and `src/stores/configStore.ts`
- TanStack React Query `5.18.0` and React Virtual `3.13.12` - Query provider is created in `src/App.tsx`; virtualization is used in `src/components/ClipboardList/ClipboardList.tsx`

**Testing:**

- Vitest `4.1.2` - Frontend tests are configured in `vitest.config.ts` and bootstrapped in `src/test/setup.ts`
- Testing Library (`@testing-library/react`, `@testing-library/jest-dom`, `@testing-library/user-event`) - Component tests live under `src/components/**/*.test.tsx`
- Rust `cargo test` with async/sqlite coverage - Backend tests live in `src-tauri/src/state_tests.rs`, `src-tauri/src/integration_tests.rs`, and `src-tauri/src/performance_tests.rs`

**Build/Dev:**

- TypeScript `~5.6.2` - Compiler settings live in `tsconfig.json`, `tsconfig.node.json`, and `website/tsconfig.json`
- ESLint `9.33.0` plus `typescript-eslint` `8.39.0` - Desktop lint rules live in `eslint.config.js`
- Prettier `3.6.2` - Formatting rules live in `.prettierrc`
- Tauri CLI `2.x` - Desktop dev/build commands are exposed through root `package.json`
- GitHub Actions - Release and test-build automation live in `.github/workflows/release.yml` and `.github/workflows/test-build.yml`
- Custom version automation - `scripts/update-version.js` updates `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.lock`

## Key Dependencies

**Critical:**

- `@tauri-apps/api` `^2` and `tauri` `2` - Frontend/backend bridge, window events, and command invocation used in `src/App.tsx`, `src/stores/clipboardStore.ts`, and `src-tauri/src/lib.rs`
- `sqlx` `0.7` with SQLite - Local persistence, schema init, and queries live in `src-tauri/src/database/mod.rs` and `src-tauri/src/state.rs`
- `reqwest` `0.11` - Remote URL preview resolution and content fetching are implemented in `src-tauri/src/commands.rs`
- `@aptabase/tauri` `^0.4.1` and `tauri-plugin-aptabase` `1` - Product analytics live in `src/services/analytics.ts` and `src-tauri/src/lib.rs`
- `@tauri-apps/plugin-updater` `^2.9.0` and `tauri-plugin-updater` `2` - In-app update checks/install live in `src/components/UpdateChecker/UpdateChecker.tsx`, `src-tauri/src/updater/mod.rs`, and `src-tauri/tauri.conf.json`
- `@monaco-editor/react` `^4.7.0` and `monaco-editor` `^0.52.2` - Rich code/JSON preview renderers live in `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.tsx` and `src/components/DetailView/ContentRenderers/JsonRenderer.tsx`

**Infrastructure:**

- `@tauri-apps/plugin-clipboard-manager`, `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`, `tauri-plugin-dialog`, `tauri-plugin-log`, and `tauri-plugin-opener` - Native desktop capabilities are registered in `src-tauri/src/lib.rs`
- `arboard`, `image`, `infer`, `uuid`, `chrono`, `serde`, `serde_json`, `serde_with`, `regex`, `url`, and `dirs` - Clipboard parsing, media handling, serialization, and path management live across `src-tauri/src/clipboard/`, `src-tauri/src/models/`, `src-tauri/src/commands.rs`, and `src-tauri/src/config/mod.rs`
- `@radix-ui/*`, `class-variance-authority`, `lucide-react`, `tailwind-merge`, and `tailwindcss-animate` - UI primitives and styling helpers live in `src/components/ui/` and `website/components/ui/`
- `i18next`, `react-i18next`, and `i18next-browser-languagedetector` - Internationalization is initialized in `src/i18n/config.ts` and backed by locale files in `src/locales/`

## Configuration

**Environment:**

- `src-tauri/.env` is present and is loaded manually in debug builds via `dotenvy::dotenv()` in `src-tauri/src/lib.rs`
- `APTABASE_APP_KEY` configures analytics in `src-tauri/src/lib.rs`; CI injects it in `.github/workflows/release.yml` and `.github/workflows/test-build.yml`
- `TAURI_DEV_HOST` customizes Vite/Tauri dev host and HMR behavior in `vite.config.ts`
- `GITHUB_TOKEN`, `TAURI_SIGNING_PRIVATE_KEY`, and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` are required by `.github/workflows/release.yml` and `.github/workflows/test-build.yml` for signed release builds
- Additional env files are present at `dance-sync-server/.env` and `dance-sync-server-node/.env`; no active package manifest was detected alongside them during this scan

**Build:**

- Desktop frontend/build config lives in `package.json`, `vite.config.ts`, `tsconfig.json`, `tailwind.config.js`, `postcss.config.js`, `eslint.config.js`, `.prettierrc`, and `vitest.config.ts`
- Native desktop config lives in `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/tauri.test.conf.json`, and `src-tauri/capabilities/default.json`
- Website config lives in `website/package.json`, `website/next.config.js`, `website/tsconfig.json`, `website/tailwind.config.js`, and `website/postcss.config.js`
- Deployment/build automation lives in `vercel.json`, `website/scripts/should-build-website.sh`, `.github/workflows/release.yml`, `.github/workflows/test-build.yml`, and `scripts/update-version.js`

## Platform Requirements

**Development:**

- `pnpm` plus a working Node.js toolchain is required for the JS layers; `website/package.json` is the only manifest that explicitly enforces `node >=22`
- Rust stable, Cargo, and Tauri system dependencies are required for `src-tauri/`
- macOS is the primary native integration target: `src-tauri/Cargo.toml` enables `macos-private-api`, `src-tauri/src/lib.rs` sets `ActivationPolicy::Accessory`, and `src-tauri/src/utils/app_list.rs` plus `src-tauri/src/utils/app_icon_extractor.rs` call macOS `NSWorkspace` APIs
- Windows code paths and build targets are present via `winapi` in `src-tauri/Cargo.toml`, Windows branches in `src-tauri/src/utils/app_list.rs` and `src-tauri/src/utils/app_icon_extractor.rs`, and Windows jobs in `.github/workflows/release.yml`
- `ffprobe` is an optional external binary for media metadata extraction; runtime checks live in `src-tauri/src/commands.rs`

**Production:**

- Desktop bundles are produced as signed Tauri artifacts with updater metadata via `src-tauri/tauri.conf.json` and `.github/workflows/release.yml`
- `src-tauri/tauri.conf.json` enables `createUpdaterArtifacts`, so release builds are expected to emit GitHub-hosted updater manifests
- The website is configured as a static Next.js export in `website/next.config.js`; repo-level Vercel settings are defined in `vercel.json`

---

_Stack analysis: 2026-03-27_
