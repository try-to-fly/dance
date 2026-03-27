# Testing Patterns

**Analysis Date:** 2026-03-27

## Test Framework

**Runner:**

- 主前端使用 `vitest` `^4.1.2`，配置在 `vitest.config.ts`。
- `vitest.config.ts` 设置 `environment: 'jsdom'`、`setupFiles: ['./src/test/setup.ts']`、`css: true`、`clearMocks: true`、`mockReset: true`、`restoreMocks: true`。
- `src/test/setup.ts` 统一接入 `@testing-library/jest-dom/vitest`，并 stub `ResizeObserver`、`IntersectionObserver`、`window.matchMedia`、`HTMLElement.prototype.scrollIntoView`。
- Rust 后端没有单独测试 runner 配置文件，测试通过 `cargo test` 发现 `#[cfg(test)]` 模块和 `#[tokio::test]` 异步测试，入口位于 `src-tauri/src/lib.rs`。

**Assertion Library:**

- 前端断言使用 Vitest `expect` 与 `@testing-library/jest-dom` matcher。
- 前端渲染与交互测试依赖 `@testing-library/react`、`@testing-library/user-event`（当前测试主要使用 `render`、`screen`、`fireEvent`、`waitFor`、`act`）。
- Rust 侧直接使用标准 `assert!` / `assert_eq!`，结合 `sqlx` 查询结果断言。

**Run Commands:**

```bash
pnpm test                # 运行 `src/` 下所有 Vitest 用例
pnpm test:watch          # 启动 Vitest watch 模式
cd src-tauri && cargo test   # 运行 Rust 单元/集成测试模块
```

## Test File Organization

**Location:**

- 前端测试采用就近放置，测试文件与被测模块同目录：
  - `src/stores/clipboardStore.test.ts`
  - `src/components/DetailView/DetailView.test.tsx`
  - `src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`
  - `src/components/DetailView/scene/AlternateViews.test.tsx`
- Rust 测试放在源码树内部，而不是独立 `tests/` 目录：
  - 模块内测试：`src-tauri/src/database/mod.rs`
  - 聚合测试文件：`src-tauri/src/state_tests.rs`、`src-tauri/src/integration_tests.rs`、`src-tauri/src/performance_tests.rs`

**Naming:**

- 前端统一使用 `*.test.ts` / `*.test.tsx`。
- Rust 测试使用 `#[cfg(test)] mod tests` 或以 `*_tests.rs`、`integration_tests.rs`、`performance_tests.rs` 命名。

**Structure:**

```text
src/
  stores/
    clipboardStore.ts
    clipboardStore.test.ts
  components/
    DetailView/
      DetailView.tsx
      DetailView.test.tsx
      DetailPreviewContract.test.tsx
      ContentRenderers/
        JsonRenderer.tsx
        JsonRenderer.test.tsx
src-tauri/src/
  database/mod.rs                # 内嵌 #[cfg(test)] mod tests
  state_tests.rs                 # 独立 Rust 测试模块
  integration_tests.rs           # 端到端数据流测试
  performance_tests.rs           # 默认忽略的性能测试
```

## Test Structure

**Suite Organization:**

```typescript
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';

const mockedDeps = vi.hoisted(() => ({
  child: vi.fn(() => <div data-testid="child" />),
}));

vi.mock('../dependency', () => ({
  dependency: mockedDeps.child,
}));

describe('FeatureName', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('描述用户可见行为，而不是实现细节', async () => {
    render(<Component />);
    await waitFor(() => {
      expect(screen.getByTestId('child')).toBeInTheDocument();
    });
  });
});
```

**Patterns:**

- 测试名称直接写业务行为，且当前仓库大多使用中文描述场景，例如 `src/components/DetailView/DetailView.test.tsx`、`src/stores/clipboardStore.test.ts`。
- 每个文件先定义 `baseEntry`、`createStoreState`、`createDeferred` 之类的夹具工厂，再在 `describe` 内通过 `beforeEach` 重置 mock。
- UI 测试优先断言可见文本、ARIA role、DOM 属性和 class 组合，而不是直接调用组件内部函数，见 `src/components/DetailView/DetailView.test.tsx`、`src/components/DetailView/scene/AlternateViews.test.tsx`。
- 针对异步竞争条件，常用 `rerender` + deferred promise + `waitFor` / `act` 验证“旧结果不会覆盖新结果”，见 `src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx`、`src/components/DetailView/DetailView.test.tsx`。
- Rust 测试通常先创建临时 SQLite 环境，再写入数据并验证查询或状态变化，见 `src-tauri/src/state_tests.rs`、`src-tauri/src/integration_tests.rs`、`src-tauri/src/database/mod.rs`。

## Mocking

**Framework:** `vi.mock`、`vi.fn`、`vi.hoisted`、`vi.mocked`、`vi.stubGlobal`。

**Patterns:**

```typescript
const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => dictionary[key] ?? key,
  }),
}));

beforeEach(() => {
  invokeMock.mockReset();
});
```

**What to Mock:**

- 平台边界：`@tauri-apps/api/core`、`@tauri-apps/api/event`、`@tauri-apps/plugin-clipboard-manager`，见 `src/stores/clipboardStore.test.ts`、`src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`。
- 浏览器环境缺失能力：`ResizeObserver`、`IntersectionObserver`、`matchMedia`、`scrollIntoView`，见 `src/test/setup.ts`。
- 重型依赖或渲染器：`@monaco-editor/react`、`monaco-editor`、`react-json-view-lite`，见 `src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx`、`src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx`。
- 子组件和 store hook：`useClipboardStore`、`ContentRenderers` 聚合导出、`ImagePreview`，见 `src/components/DetailView/DetailView.test.tsx`、`src/components/DetailView/scene/AlternateViews.test.tsx`。
- 国际化：当前测试普遍 mock `react-i18next` 并内建少量字典，避免依赖真实语言包，见 `src/components/DetailView/DetailView.test.tsx`、`src/components/DetailView/ContentRenderers/UnifiedTextRenderer.test.tsx`。

**What NOT to Mock:**

- 仓库倾向直接测试自有纯函数与契约逻辑，而不是把它们 mock 掉，例如 `src/components/DetailView/DetailPreviewContract.test.tsx` 直接调用 `buildPreviewDescriptor`。
- 对于轻量的纯文本/结构转换，优先让真实实现参与测试，例如 `src/lib/preview/previewDescriptor.ts`、`src/lib/preview/entryPresentation.ts` 的行为通过上层契约测试覆盖。
- Rust 侧不使用 mock 框架，直接构建真实数据库、真实 `AppState` 和真实内容处理器，见 `src-tauri/src/state_tests.rs`、`src-tauri/src/integration_tests.rs`。

## Fixtures and Factories

**Test Data:**

```typescript
const baseEntry: ClipboardEntry = {
  id: 'entry-1',
  content_hash: 'hash-1',
  content_type: 'text/plain',
  content_data: 'hello',
  source_app: 'Terminal',
  created_at: Date.now(),
  copy_count: 1,
  file_path: null,
  is_favorite: false,
  content_subtype: 'plain_text',
  metadata: null,
  app_bundle_id: null,
};

const createStoreState = () => ({
  copyToClipboard: vi.fn(),
  fetchUrlContent: vi.fn(),
  extractMediaMetadata: vi.fn(),
});
```

**Location:**

- 前端夹具通常内联在测试文件顶部，不存在统一的 fixture 目录。
- 共享测试环境只在 `src/test/setup.ts` 和 `src/test/mocks/monaco-editor.ts`。
- Rust 夹具通过工厂函数创建临时环境：`create_test_db`、`create_test_state`、`create_integration_test_env`、`create_perf_test_env`，见 `src-tauri/src/database/mod.rs`、`src-tauri/src/state_tests.rs`、`src-tauri/src/integration_tests.rs`、`src-tauri/src/performance_tests.rs`。

## Coverage

**Requirements:** 未检测到覆盖率门槛、覆盖率脚本或 CI 强制测试步骤。`package.json` 只有 `test` 与 `test:watch`，没有 `coverage` 命令；`.github/workflows/test-build.yml` 和 `.github/workflows/release.yml` 只做构建，不执行测试。

**View Coverage:**

```bash
Not configured
```

**Current State:**

- 前端当前共有 7 个 Vitest 文件，约 1253 行测试代码，覆盖面集中在 `src/components/DetailView/**` 与 `src/stores/clipboardStore.ts`。
- `src/components/DetailView/DetailPreviewContract.test.tsx` 是最大的契约测试文件，负责验证 preview descriptor 与场景渲染切换。
- `website/` 未检测到任何 `*.test.*` 或 `*.spec.*` 文件。
- Rust 侧有 3 个集中测试模块加多个内嵌 `#[cfg(test)]` 模块，但它们没有被 GitHub workflow 自动执行。

## Test Types

**Unit Tests:**

- React renderer 级测试覆盖 `JsonRenderer`、`UnifiedTextRenderer`、`UrlRenderer`、`AlternateViews` 等局部组件，见 `src/components/DetailView/ContentRenderers/*.test.tsx`、`src/components/DetailView/scene/AlternateViews.test.tsx`。
- Zustand store 行为测试覆盖 fallback、缓存 TTL 与筛选副作用，见 `src/stores/clipboardStore.test.ts`。

**Integration Tests:**

- `src/components/DetailView/DetailView.test.tsx` 与 `src/components/DetailView/DetailPreviewContract.test.tsx` 更接近组件集成/契约测试，验证多个 renderer、描述器和操作按钮之间的协作。
- Rust 的 `src-tauri/src/integration_tests.rs` 搭建真实 `AppState`、真实 SQLite 和真实内容检测流程，验证“检测 -> 建模 -> 入库 -> 查询”的完整链路。
- `src-tauri/src/state_tests.rs` 与 `src-tauri/src/database/mod.rs` 覆盖状态层和数据库层的真实读写行为。

**E2E Tests:**

- 未检测到 Playwright、Cypress、Tauri 驱动 UI E2E 或浏览器级端到端测试配置。
- `.github/workflows/test-build.yml` 只验证能否构建桌面包，不属于行为级 E2E。

**Performance Tests:**

- `src-tauri/src/performance_tests.rs` 提供内容识别、数据库写入、查询吞吐和并发行为测试。
- 这些测试全部通过 `#[ignore]` 标记，需手动 `cargo test -- --ignored` 才会运行。

## Common Patterns

**Async Testing:**

```typescript
const request = createDeferred<string>();
store.fetchUrlContent.mockReturnValue(request.promise);

const { rerender } = render(<UrlRenderer content="example.com/first.json" />);
rerender(<UrlRenderer content="example.com/second.json" />);

await act(async () => {
  request.resolve('{"id":"second"}');
  await request.promise;
});

await waitFor(() => {
  expect(screen.getByTestId('unified-text-renderer')).toHaveTextContent(/second/);
});
```

- 使用 `createDeferred` 控制请求完成时机，专门测试竞态与过期结果丢弃，见 `src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx`。
- 组件异步渲染一般组合 `waitFor`、`mockResolvedValue`、`rerender` 和 `act`，而不是手写 `setTimeout`。

**Error Testing:**

```typescript
invokeMock.mockImplementation((command: string) => {
  if (command === 'resolve_url_preview') {
    throw 'HTTP error: 503';
  }
  return Promise.resolve({ width: 1920, height: 1080 });
});

const result = await useClipboardStore.getState().resolveUrlPreview?.('https://example.com/x.png');

expect(result?.url?.error).toBe('HTTP error: 503');
expect(
  useClipboardStore.getState().previewResolutionCache?.get('url:https://example.com/x.png')?.ttlMs
).toBe(30_000);
```

- 失败场景主要通过 mock 抛错、Promise reject 或无效输入驱动，然后断言 fallback UI 或缓存策略，见 `src/stores/clipboardStore.test.ts`、`src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx`。
- Rust 失败路径更多通过真实依赖返回 `Result` 或 `unwrap()` 触发 panic，不存在专门的错误 mock 层。

## Prescriptive Guidance

- 给 `src/` 新增测试时，继续采用“与实现同目录放置 `*.test.ts(x)`”的方式，不要新建顶层测试目录。
- 需要隔离平台能力时，优先 mock Tauri API、浏览器 observer 和 Monaco，而不是 mock 自己的纯函数。
- 测试名称继续写成完整行为描述，当前仓库以中文为主，保持一致。
- 涉及异步 effect 或缓存竞争时，优先使用 deferred promise 模式，参考 `src/components/DetailView/ContentRenderers/UrlRenderer.test.tsx` 与 `src/components/DetailView/DetailView.test.tsx`。
- 给 `website/` 新增功能时需要同步补齐测试基建；当前该子项目没有任何测试基线。
- 若希望测试真正成为门禁，需要把 `pnpm test` 和 `cargo test` 接入 `.github/workflows/*`；当前 workflow 仅验证构建成功。

---

_Testing analysis: 2026-03-27_
