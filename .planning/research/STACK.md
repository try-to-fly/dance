# Technology Stack Research: v1.1 Developer Preview, Retrieval & Smoke Automation

**Project:** Dance  
**Researched:** 2026-03-29  
**Scope:** 仅研究本轮里程碑新增能力需要的技术增量，包括 `JWT/TOML/XML/CSV/TSV/log` 专用预览、retrieval ranking 与 benchmark 强化，以及桌面 smoke 自动化。  
**Baseline:** 保持现有 `Tauri 2 + React 18 + Rust + SQLite + sqlx + Monaco`，不做框架迁移，不扩张到云同步、团队协作或 AI-first 检索。

## Executive Recommendation

这轮最合理的技术路线不是换栈，而是在现有架构上做三类增量：

1. **Rust analysis contract 扩容，而不是把新格式解析塞回前端。**  
   `analysis/contract.rs` 和 `analysis/service.rs` 已经是现有权威语义入口。新增 `jwt`、`toml`、`xml`、`csv_tsv`、`log` 这类 subtype 及其 typed metadata，能直接复用现有 rebuild、retrieval、preview contract。

2. **前端新增少量专用 renderer，但尽量复用现有展示底座。**  
   `Monaco` 继续承担 raw/code/text 展示；`JsonRenderer` 可以复用到 JWT header/payload；CSV/TSV 用现有 React 组件和虚拟滚动做表格视图；日志视图不要引入新的重型日志平台。

3. **smoke automation 以 repo 已有 CI/build 流程为基础做收口，而不是另建第二套发布基础设施。**  
   现有 `.github/workflows/release.yml` 与 `.github/workflows/test-build.yml` 已经包含 `pnpm test`、`pnpm build`、`cargo test` 和 `tauri-action` 构建。v1.1 应补的是可重复 smoke 层，而不是重写发布 pipeline。

## Recommended Stack

| Area                     | Recommendation                                                                                                | Why                                                                                                    | Confidence  |
| ------------------------ | ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ----------- |
| Desktop shell            | 保持 `Tauri 2.x`                                                                                              | 当前瓶颈不在桌面壳层；已有 release/test-build workflow 也已围绕 Tauri 2 建立                           | HIGH        |
| Analysis authority       | 保持 Rust 为唯一解析/分类/检索权威层                                                                          | 仓库已经在 `src-tauri/src/analysis/`、`src-tauri/src/retrieval/` 建立 authority 路线，继续扩容成本最低 | HIGH        |
| TOML parsing             | 新增 Rust `toml` crate                                                                                        | 适合 developer config 内容，解析稳定，能直接产出结构化 metadata                                        | HIGH        |
| XML parsing              | 新增 Rust `roxmltree`                                                                                         | 只读树解析足够覆盖 preview 与 token 提取，不必引入更重的 DOM 写入模型                                  | HIGH        |
| CSV / TSV parsing        | 新增 Rust `csv` crate                                                                                         | 适合把表格型文本转换成 rows/columns metadata，支持 CSV/TSV 两类 delimiter 场景                         | HIGH        |
| JWT inspection           | 新增 Rust `jsonwebtoken` 的 decode-only 路径，或用 base64 + serde 做最小解析                                  | JWT 专用预览的价值在 header/payload 结构化展示，而不是签名验证；实现要避免引入网络或 secret 依赖       | MEDIUM-HIGH |
| Log preview parsing      | 先用 repo 内轻量 parser + 规则，不引入第三方日志平台                                                          | 目标是结构化 preview 与 search token，而不是做 observability 产品                                      | HIGH        |
| Raw / code view          | 保持 `Monaco`                                                                                                 | 现有 `UnifiedTextRenderer` 已经接入 Monaco mock/test 体系，继续复用最稳                                | HIGH        |
| JWT/JSON tree view       | 复用现有 `JsonRenderer`                                                                                       | JWT header/payload 本质上是 JSON 树，现有 UI 能承接                                                    | HIGH        |
| CSV table view           | 复用 React 组件 + 现有虚拟滚动模式，不引入 AG Grid/Handsontable                                               | 本产品只需要可读表格 preview，不需要电子表格级交互                                                     | HIGH        |
| Retrieval ranking        | 保持 SQLite FTS + 当前 Rust ranking 路线，补 structured tokens、representative corpus 与 highlight 调优       | 仓库已经有 `entry_search_documents` / `entry_search_fts` 与 retrieval tests，继续演进低风险            | HIGH        |
| Retrieval benchmarking   | 先扩展现有 `retrieval_tests.rs`、`performance_tests.rs` 与 fixture corpus，不急着引入独立 benchmark framework | 当前首要问题是代表性样本与回归门槛，而不是 benchmark 工具缺失                                          | HIGH        |
| GitHub release packaging | 保持 `tauri-apps/tauri-action@v0`                                                                             | 官方 GitHub pipeline 文档明确支持在构建前串接测试步骤，仓库也已在 release workflow 中使用              | HIGH        |
| Desktop smoke automation | Windows/Linux 可考虑 Tauri WebDriver；macOS 采用包产物启动/日志/assert 的轻量 smoke                           | Tauri 官方 WebDriver 文档明确说明 macOS 不支持 WebDriver 测试，需要单独策略                            | HIGH        |

## Integration Points For This Repo

| Repo area                                                          | Change needed                                                                              | Why it fits                                                |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ---------------------------------------------------------- |
| `src-tauri/src/analysis/contract.rs`                               | 扩展 subtype 与 metadata 定义                                                              | 这是当前 preview/retrieval 共用的权威 contract             |
| `src-tauri/src/analysis/service.rs`                                | 为新 subtype 生成 typed metadata 与 diagnostic                                             | 现有 JSON/URL/command 等都走这条线，继续扩容最一致         |
| `src-tauri/src/retrieval/mod.rs`                                   | 为新 metadata 追加 search text / structured term                                           | retrieval 质量提升必须建立在后端 tokenization 上           |
| `src/lib/preview/previewDescriptor.ts`                             | 决定新 subtype 映射到哪个 primary/alternate view                                           | 当前 detail surface 已有 descriptor contract，不应另起一套 |
| `src/components/DetailView/ContentRenderers/`                      | 新增 `JwtRenderer`、`CsvRenderer`、`LogRenderer`，并评估 TOML/XML 是否只需 raw + inspector | 让新能力最小化侵入现有 detail scene                        |
| `.github/workflows/release.yml`                                    | 在现有 build gate 后补 packaged/release smoke 步骤                                         | 不要绕开现有 workflow，新 smoke 应进入现有 release gate    |
| `.github/workflows/test-build.yml`                                 | 补 artifact smoke / startup smoke / logs assertions                                        | test-build 已覆盖两平台构建，是最自然的 smoke 落点         |
| `src-tauri/tauri.test.conf.json` / `src-tauri/src/test_support.rs` | 提供可隔离的 smoke/test root                                                               | 现有测试已经有 temp root 基础，能避免污染真实用户目录      |

## What NOT To Add This Milestone

- 不新增第二套桌面自动化框架，只为“看起来现代”引入 Playwright-only 或 Electron 专用测试栈。
- 不为了 CSV 预览引入重型 grid 依赖，例如 AG Grid、Handsontable。
- 不为了 retrieval benchmark 先上独立搜索引擎或外部服务。
- 不把 JWT 预览扩张成联网验证、JWKS 拉取、token introspection 服务。
- 不把日志 preview 做成完整日志管理器；目标是“更易读的 clipboard 日志内容”，不是“应用日志平台”。

## Primary Sources

### Repo Evidence

- `.planning/PROJECT.md`
- `.planning/codebase/STACK.md`
- `src-tauri/src/analysis/contract.rs`
- `src-tauri/src/retrieval/mod.rs`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`
- `.github/workflows/release.yml`
- `.github/workflows/test-build.yml`

### Official Sources

- SQLite FTS5 documentation: https://www.sqlite.org/fts5.html  
  Relevant to external-content indexing, `bm25`, `highlight`, `snippet`, tokenizer design.
- Tauri official GitHub pipelines docs: https://v2.tauri.app/distribute/pipelines/github/  
  Relevant to building on GitHub Actions and extending existing Tauri release workflows.
- Tauri official WebDriver docs: https://v2.tauri.app/develop/tests/webdriver/  
  Relevant to the constraint that WebDriver tests are not available on macOS.
- `toml` crate docs: https://docs.rs/crate/toml/latest
- `csv` crate docs: https://docs.rs/crate/csv/latest
- `roxmltree` crate docs: https://docs.rs/crate/roxmltree/latest

---

_Research completed: 2026-03-29_  
_Ready for requirements: yes_
