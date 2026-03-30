# Architecture Research: v1.1 Integration Plan

**Project:** Dance  
**Researched:** 2026-03-29  
**Scope:** 研究 v1.1 新增 preview 类型、retrieval benchmark/ranking、以及 smoke automation 如何融入现有 Tauri + React + Rust + SQLite 架构。  
**Overall confidence:** HIGH

## Executive Summary

这轮不需要新建第二条架构主线。仓库已经有相对清晰的 authority 路径：

- Rust `analysis` 负责 subtype 与 metadata
- Rust `retrieval` 负责 search document 与排序
- 前端 `previewDescriptor` / `DetailView` 负责把权威语义映射到 renderer

所以 v1.1 的正确集成方式是：

1. **继续在 Rust analysis contract 内扩充新 subtype 和 typed metadata。**
2. **让 retrieval 直接消费这些 metadata，补齐 structured terms、highlight 与 representative benchmark。**
3. **前端只增加最少量 renderer 和 descriptor 分支，不重新实现解析逻辑。**
4. **smoke automation 贴着现有 release/test-build workflow 加，不另起炉灶。**

## Integration Points Table

| Component                                                          | Change                                                                                                   | Why                                                        |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| `src-tauri/src/analysis/contract.rs`                               | 增加 `jwt`、`toml`、`xml`、`csv_tsv`、`log` subtype 与对应 metadata                                      | 当前 preview/retrieval contract 就从这里出发，扩容后最一致 |
| `src-tauri/src/analysis/service.rs`                                | 为新 subtype 生成 metadata、diagnostic、fallback 语义                                                    | 防止前端自行解析，保持 authority 在 Rust                   |
| `src-tauri/src/clipboard/content_detector.rs`                      | 增加新格式 detector 规则并控制优先级冲突                                                                 | 需要稳定把新 developer text 从 plain/code 中分流出来       |
| `src-tauri/src/retrieval/mod.rs`                                   | 让 search document 吸收新 metadata tokens，并补 representative benchmark / highlight 断言                | v1.1 的 retrieval 价值来自“新格式可搜且排序更准”           |
| `src/lib/preview/previewDescriptor.ts`                             | 决定新 subtype 如何映射为 primary view / alternate view / inspector                                      | 这是当前 detail surface 的统一分发层                       |
| `src/components/DetailView/ContentRenderers/`                      | 新增 `JwtRenderer`、`CsvRenderer`、`LogRenderer`；评估 TOML/XML 是否走 `UnifiedTextRenderer + inspector` | 避免为每个新格式重做整套 detail scene                      |
| `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`       | 注册新 renderer 并处理 raw/semantic 入口                                                                 | 当前 primary renderer 是 detail surface 的装配点           |
| `src-tauri/src/test_support.rs` / `src-tauri/tauri.test.conf.json` | 为 smoke/test 提供隔离路径与 test config                                                                 | smoke automation 不能污染真实用户目录或依赖生产配置        |
| `.github/workflows/test-build.yml`                                 | 在打包后加入 artifact smoke / startup smoke                                                              | test-build 已是最自然的“产物是否可用”切入点                |
| `.github/workflows/release.yml`                                    | 在 release packaging 前后加入 smoke gate 或 evidence collection                                          | 把 smoke 纳入 release gate，避免只验证 build success       |

## Data Flow / Contract Changes

### 1. New preview types

推荐数据流：

```text
Clipboard text
  -> content_detector
  -> analysis.service
  -> analysis contract metadata
  -> retrieval document builder
  -> previewDescriptor
  -> dedicated renderer / raw fallback
```

关键约束：

- `JWT` 不在前端 decode，前端只渲染 Rust 返回的结构化信息。
- `CSV/TSV` 的列信息、delimiter、row/column count 应由后端 metadata 给出，前端只消费表格数据。
- `TOML/XML` 至少应共享“pretty/raw/diagnostic”这一套 contract，不要每个 renderer 自己定义状态字段。
- `log` 预览只负责结构化阅读，不应该和应用内 `LogViewer` 混成一套实现。

### 2. Retrieval quality work

推荐数据流：

```text
analysis metadata
  -> search document builder
  -> entry_search_documents / entry_search_fts
  -> retrieval query
  -> ranking + snippet/highlight
  -> list/detail reason labels
```

需要新增或强化的 contract：

- 新 subtype 的 structured term 类型与 token 生成规则
- representative query corpus
- ranking / snippet / highlight 断言
- 对新格式 query 的 explainability 输出

### 3. Smoke automation

推荐数据流：

```text
workflow trigger
  -> build/test gate
  -> packaged artifact
  -> smoke launcher/assertions
  -> logs / screenshots / failure evidence
```

关键约束：

- Windows/Linux 若后续接入 WebDriver，可复用官方 Tauri 路线。
- macOS 不能假设有 WebDriver，因此 packaged smoke 更像“启动 + 基础断言 + 日志检查”的轻量方案。
- smoke test 必须走 test config / temp roots，不能读写真实 app data。

## Suggested Build Order

### Phase 6: Analysis Coverage For New Developer Types

先扩 analysis contract、detector、metadata、search token foundations。

原因：

- preview 与 retrieval 都依赖这层 authority
- 当前代码库已经有 `analysis` 与 `retrieval` 基础，可以自然扩容

### Phase 7: Dedicated Preview Surfaces

在 authority contract 稳定后补前端 renderer、descriptor、alternate views、inspector。

原因：

- 避免先做 UI，再被后端 contract 返工
- 可以一次性把 raw/semantic/fallback 路径整理完整

### Phase 8: Retrieval Benchmark And Ranking Calibration

在新格式 tokens 落地后补 representative corpus、highlight/ranking 调优、benchmark 回归门槛。

原因：

- 这时 query corpus 才能覆盖新类型
- 能直接基于 Phase 6 的 metadata / search document 迭代

### Phase 9: Desktop / Packaged / Release Smoke Automation

最后把 dev/package/CI smoke 收口进现有 workflows，并让 v1.1 新链路进入 release gate。

原因：

- smoke automation 应该验证已经存在的真实能力，而不是先于能力建设空跑
- 现有 release/test-build workflow 已经具备可插入点

## Risks Introduced By Each Change Area

| Change area           | Main risk                                             | Mitigation                                                       |
| --------------------- | ----------------------------------------------------- | ---------------------------------------------------------------- |
| New subtype detection | detector 互相抢分类，导致 plain/code 回退异常         | 先定义优先级与 diagnostics，再加 renderer                        |
| New renderers         | 前端重新解析内容，和 Rust contract 漂移               | renderer 只消费 typed metadata 和 normalized content             |
| Retrieval tuning      | benchmark 没有代表性，导致“优化”只是换一组 query 胜出 | 建固定 corpus 与 explainability 断言                             |
| Packaged smoke        | 测试路径污染真实 app data，或只在 CI 某平台有效       | 使用 `tauri.test.conf.json`、temp roots、按平台拆策略            |
| macOS smoke           | 误以为 WebDriver 覆盖 macOS                           | 明确采用 startup/log/assert 路线，不把 macOS 绑死在 WebDriver 上 |

## Primary Sources

### Repo Evidence

- `.planning/PROJECT.md`
- `.planning/codebase/ARCHITECTURE.md`
- `.planning/codebase/CONCERNS.md`
- `.planning/codebase/STRUCTURE.md`
- `src-tauri/src/analysis/contract.rs`
- `src-tauri/src/analysis/service.rs`
- `src-tauri/src/retrieval/mod.rs`
- `src/lib/preview/previewDescriptor.ts`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`
- `.github/workflows/release.yml`
- `.github/workflows/test-build.yml`

### Official Sources

- SQLite FTS5 documentation: https://www.sqlite.org/fts5.html
- Tauri GitHub pipelines docs: https://v2.tauri.app/distribute/pipelines/github/
- Tauri WebDriver docs: https://v2.tauri.app/develop/tests/webdriver/

---

_Research completed: 2026-03-29_  
_Ready for roadmap: yes_
