# Feature Landscape: v1.1 Developer Preview, Retrieval & Smoke Automation

**Domain:** 面向开发者的本地桌面剪贴板工作台  
**Researched:** 2026-03-29  
**Scope:** 只研究本轮里程碑需要纳入的功能边界，不重复定义 v1.0 已验证能力。  
**Overall confidence:** HIGH

## Executive Summary

v1.1 的主线不是“加更多内容类型”这么简单，而是把三条能力一起做实：

1. **开发者格式支持继续变深。**  
   让 `JWT/TOML/XML/CSV/TSV/log` 不再只是 generic text，而是能在粘贴前被快速判断、快速理解、快速回用。

2. **retrieval 质量从“能搜”提升到“稳定、可解释、可回归”。**  
   本轮不需要重写搜索引擎，但必须让新格式能贡献 structured tokens，并把 representative queries 与 highlight/ranking 变成可验证基线。

3. **自动化验证必须更接近真实桌面路径。**  
   现有 CI 已经会跑 tests/build/package；这一轮的目标是把“构建成功”向“应用至少能启动、关键链路能冒烟验证”推进。

## Table Stakes For This Milestone

以下内容如果不做，v1.1 很容易沦为“加了几种 subtype 名字，但体验没有本质提升”。

| Feature cluster                                 | Expected behavior                                                                                          | Why it is table stake now                                          | Confidence |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------- |
| Dedicated preview detection                     | 复制 JWT、TOML、XML、CSV/TSV、日志文本后，系统能稳定识别为专用 subtype，而不是继续落到 plain/code          | 新里程碑的第一价值就是扩大 developer-specific preview 覆盖面       | HIGH       |
| Raw fallback for every new format               | 每种新格式都必须保留 raw 原文视图与 diagnostic/fallback 路径                                               | developer clipboard 内容噪声大，不能把专用 renderer 建在脆弱假设上 | HIGH       |
| Structured preview where it adds decision value | JWT 能看到 header/payload；CSV/TSV 能看到表格；日志能看到级别/时间/stack 候选；TOML/XML 至少能格式化和高亮 | “更快判断是不是我想要的那条”是 preview 的核心价值                  | HIGH       |
| Retrieval token support for new preview types   | 新 subtype 的关键字段能进入 search document，例如 JWT claims key、XML tag、CSV header、日志 level/token    | 只做渲染不做检索，会让新 preview 与搜索体验脱节                    | HIGH       |
| Representative benchmark corpus                 | 能用固定样本跑 query corpus，验证 recall、ranking、highlight 不回退                                        | retrieval 调优如果没有固定样本，后续改动会反复漂移                 | HIGH       |
| Repeatable smoke entrypoints                    | 至少能定义并运行 `desktop smoke`、`packaged smoke`、`GitHub Actions smoke` 三类入口                        | 现有 workflow 已有 build gate，本轮必须往行为级验证再推进一步      | HIGH       |

## Differentiators Worth Including Now

这些能力不是所有剪贴板管理器都必须有，但它们最贴近 Dance 的开发者定位，且值得在 v1.1 就纳入。

| Feature cluster                     | User value                                               | Concrete expectation                                               | Confidence  |
| ----------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------ | ----------- |
| JWT dedicated inspector             | 快速判断 token 内容是否正确，避免复制错误 header/payload | 展示 header 与 payload 的结构化视图，保留原始 token 与 diagnostic  | HIGH        |
| CSV / TSV tabular preview           | 快速理解列结构和数据行，而不是在长文本里肉眼找分隔符     | 表头、行数、列数明确，支持 raw 与表格切换                          | HIGH        |
| Log-focused preview                 | 快速从错误日志、堆栈或 structured log 中定位关键信息     | 提取 level、timestamp、logger/module、stack-like lines，高亮关键行 | MEDIUM-HIGH |
| XML / TOML formatted developer view | 配置与接口片段更易读，更容易与 plain text 区分           | 至少支持 pretty/raw 双视图和基本 inspector                         | HIGH        |
| Retrieval benchmark visibility      | 让 ranking/highlight 变更有回归证据，不再靠体感          | 为代表性 query 给出稳定期望结果与命中解释                          | HIGH        |
| Packaged smoke evidence in CI       | 缩小“测试通过”和“真实用户拿到包后能跑”的差距             | 至少对产物启动、关键日志、基础命令/窗口行为有 smoke 级断言         | MEDIUM-HIGH |

## Anti-Features / Defer List

这些方向容易稀释 v1.1，应该明确延后或排除。

| Anti-feature                      | Why avoid now                                   | Do instead                                       |
| --------------------------------- | ----------------------------------------------- | ------------------------------------------------ |
| 云同步 / 多设备同步               | 会把问题从内容理解扩张到账户、同步与安全体系    | 继续把本地开发者体验做深                         |
| AI-first semantic retrieval       | 排序不可解释、成本高、与本地优先目标冲突        | 先把 deterministic retrieval + benchmark 做实    |
| 重型日志分析功能                  | 会把“剪贴板日志内容预览”做成 observability 产品 | 只做 clipboard log text 的结构化阅读体验         |
| JWT 在线验证 / 远端 introspection | 会引入网络和 secret 依赖，偏离本地预览主线      | 只做本地 decode 与安全展示                       |
| Spreadsheet 级 CSV 编辑           | 超出“预览与回用”的产品边界                      | 只做只读表格 preview + raw copy                  |
| 第二套发布/测试基础设施           | 现有 release/test-build workflow 已经存在       | 在现有 workflow 上补 smoke 步骤与 artifacts 验证 |

## Dependencies Between Feature Groups

```text
New subtype detection and metadata
  -> dedicated preview descriptors
  -> structured retrieval tokens
  -> benchmark corpus for new types

Representative retrieval corpus
  -> highlight/ranking tuning
  -> regression gate in CI

Packaged/dev smoke entrypoints
  -> confidence that new preview/retrieval changes survive real desktop execution
```

更具体地说：

1. **preview 先依赖 analysis contract。**  
   没有稳定 subtype 和 metadata，前端 renderer 只能重新猜语义。

2. **retrieval benchmark 依赖新格式 tokenization。**  
   否则 benchmark 只会验证旧能力，无法覆盖 v1.1 的新增价值。

3. **smoke automation 应该覆盖前两者的关键路径。**  
   它不是独立产品功能，而是新能力进入 release gate 的验证层。

## Primary Sources

### Repo Evidence

- `.planning/PROJECT.md`
- `.planning/MILESTONES.md`
- `src-tauri/src/analysis/contract.rs`
- `src-tauri/src/retrieval/mod.rs`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`
- `.github/workflows/release.yml`
- `.github/workflows/test-build.yml`

### Supporting References

- SQLite FTS5 documentation: https://www.sqlite.org/fts5.html
- Tauri GitHub pipelines docs: https://v2.tauri.app/distribute/pipelines/github/
- Tauri WebDriver docs: https://v2.tauri.app/develop/tests/webdriver/

---

_Research completed: 2026-03-29_  
_Ready for requirements: yes_
