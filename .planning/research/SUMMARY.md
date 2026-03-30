# Project Research Summary: v1.1 Developer Preview, Retrieval & Smoke Automation

**Project:** Dance  
**Researched:** 2026-03-29  
**Confidence:** HIGH

## Executive Summary

v1.1 最优路线是继续沿用现有 `Tauri 2 + React 18 + Rust + SQLite`，但把新增能力全部收口到现有 authority path 上：Rust 继续负责 subtype、metadata、retrieval tokens 和 ranking；前端只负责把这些权威结果映射到 dedicated preview surfaces；CI 与打包链路则在现有 release/test-build workflow 上补 smoke，而不是重做发布基础设施。

这轮里程碑真正要解决的是三件事同时成立：

1. 新开发者格式能被**稳定识别并专用预览**
2. 新格式能被**稳定检索、解释与 benchmark**
3. 新能力在 dev、packaged、GitHub Actions 三类路径下都有**更可信的 smoke 证据**

## Key Findings

### Stack additions

- Rust analysis 层新增 `JWT/TOML/XML/CSV/TSV/log` subtype 与 typed metadata
- `toml`、`csv`、`roxmltree` 适合承接本轮新增格式解析
- JWT 以本地 decode-only 结构化展示为主，不做远端 introspection
- 继续使用现有 `Monaco`、`JsonRenderer`、React 组件体系，不引入重型 grid 或第二套编辑器
- retrieval 继续沿用当前 SQLite FTS + Rust ranking 路线，重点补 structured terms、query corpus、highlight/ranking 回归
- smoke automation 建立在现有 `release.yml` / `test-build.yml` 上；Windows/Linux 可评估 WebDriver，macOS 采用 startup/log/assert 路线

### Feature table stakes

- 新 subtype 的权威检测、diagnostic 与 raw fallback
- JWT / CSV / 日志等真正带来判断价值的 dedicated preview
- 新 subtype 进入 retrieval search document 与 explainability 路径
- representative benchmark corpus 覆盖新旧开发者格式
- desktop smoke、packaged smoke、GitHub Actions smoke 三层入口定义清楚

### Watch out for

- 不要在前端重新解析新格式，否则 list/detail/search 语义会再次漂移
- 不要用过宽 heuristics 抢走 plain/code 内容，特别是 JWT、CSV、日志
- 不要把 retrieval benchmark 做成一次性脚本，必须进入固定回归基线
- 不要假设 WebDriver 覆盖 macOS；官方文档已明确 macOS 不支持
- 不要让 smoke 读写真实用户目录，必须走 test config / temp roots

## Implications For Requirements And Roadmap

最自然的里程碑切分是四段：

1. **Phase 6:** 扩 analysis contract、detector、metadata、search tokens
2. **Phase 7:** 落 dedicated preview renderer 与 raw/semantic 视图
3. **Phase 8:** 建 representative retrieval benchmark，调 ranking/highlight
4. **Phase 9:** 把 desktop/package/release smoke 收口进现有 workflow

这个顺序的好处是：

- preview 与 retrieval 都建立在统一 contract 上
- benchmark 能真正覆盖新类型
- smoke 最终验证的是已存在的真实链路，而不是未稳定的半成品

## Sources

### Repo Evidence

- `.planning/PROJECT.md`
- `.planning/research/STACK.md`
- `.planning/research/FEATURES.md`
- `.planning/research/ARCHITECTURE.md`
- `.planning/research/PITFALLS.md`
- `.github/workflows/release.yml`
- `.github/workflows/test-build.yml`
- `src-tauri/src/analysis/contract.rs`
- `src-tauri/src/retrieval/mod.rs`
- `src/lib/preview/previewDescriptor.ts`

### Official Sources

- SQLite FTS5 documentation: https://www.sqlite.org/fts5.html
- Tauri GitHub pipelines docs: https://v2.tauri.app/distribute/pipelines/github/
- Tauri WebDriver docs: https://v2.tauri.app/develop/tests/webdriver/
- `toml` crate docs: https://docs.rs/crate/toml/latest
- `csv` crate docs: https://docs.rs/crate/csv/latest
- `roxmltree` crate docs: https://docs.rs/crate/roxmltree/latest

---

_Research completed: 2026-03-29_  
_Ready for requirements: yes_
