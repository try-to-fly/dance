# Domain Pitfalls: v1.1 Developer Preview, Retrieval & Smoke Automation

**Project:** Dance  
**Researched:** 2026-03-29  
**Focus:** brownfield 条件下，为现有开发者剪贴板应用新增专用 preview、retrieval benchmark/ranking 与 smoke automation 时最容易踩的坑。  
**Overall confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: 在前端 renderer 里重新解析 JWT / TOML / XML / CSV / 日志

**Why it matters here:**  
当前仓库已经建立了 Rust authoritative analysis contract。若新 preview 类型又在 `PrimaryPreviewRenderer` 或各个 renderer 内自行 parse，list/detail/search 很快会再次出现语义漂移。

**What goes wrong:**

- 同一条记录在列表里是 `plain_text`，详情页却被某个 renderer 临时识别成 `xml`
- retrieval tokens 不包含前端临时解析出来的结构信息
- fallback / diagnostics 无法统一，debug 成本急剧上升

**Prevention strategy:**

- 新 subtype 先进入 `analysis/contract.rs` 与 `analysis/service.rs`
- renderer 只消费 normalized content + typed metadata
- 每个新 preview 类型都保留 raw fallback 与 analysis diagnostics

**Recommended phase ownership:**  
Phase 6 负责 contract 与 detection；Phase 7 只接入消费端 renderer。

### Pitfall 2: 用过于宽松的启发式把普通文本误判成新开发者格式

**Why it matters here:**  
`JWT`、`CSV/TSV`、日志文本都很容易和普通文本、代码片段或 URL 参数串混淆。这个项目的核心价值是“识别准”，误判会直接伤害信任。

**What goes wrong:**

- 随便一个 `a.b.c` 就被当作 JWT
- 普通逗号分隔文本都进入 CSV 表格视图
- 一段 shell 输出被误判成日志专用视图
- XML/TOML 判定一宽松，就会吞掉本来应该走 code/plain 的内容

**Prevention strategy:**

- 为每种新 subtype 建立显式 priority 和 minimum confidence 条件
- 给误判风险高的类型保留 diagnostic code，而不是静默 fallback
- 用 representative corpus 覆盖“长得像但不该命中”的负样本

**Recommended phase ownership:**  
Phase 6。

### Pitfall 3: 把 retrieval benchmark 做成一次性脚本，而不是固定回归基线

**Why it matters here:**  
本轮有明确的 retrieval quality 目标。如果 benchmark 只是临时跑几条 query，后续排名变化无法解释，也无法纳入 release gate。

**What goes wrong:**

- 每次“优化”都只对作者手头那几条 query 有利
- highlight / snippet 变化没人发现
- 新格式加入后，旧 query 回归没有证据

**Prevention strategy:**

- 建 representative query corpus，覆盖 URL、JSON、JWT、CSV、XML、日志、代码等场景
- 为 Top N、match reason、highlight/snippet 建固定断言
- benchmark 文件进入仓库，不依赖本地临时数据

**Recommended phase ownership:**  
Phase 8。

### Pitfall 4: 假设 Tauri WebDriver 能覆盖所有 packaged smoke，尤其是 macOS

**Why it matters here:**  
这轮要把 smoke automation 推进到更接近真实桌面路径。Tauri 官方文档明确说明 WebDriver 测试不支持 macOS，如果路线假设错误，最终 CI 设计会卡死。

**What goes wrong:**

- roadmap 里设计出一套只在 Windows/Linux 能跑的 packaged smoke
- macOS release 继续只有 build，没有真实 smoke
- 团队误以为“有 WebDriver 计划”就等于 release gate 已闭环

**Prevention strategy:**

- 从一开始就把 smoke 分成 dev、packaged、release/CI 三层
- 明确 macOS 使用 startup/log/assert 这类轻量 smoke，而不是依赖 WebDriver
- 把“平台差异是设计输入”写进 roadmap，而不是执行阶段临时补救

**Recommended phase ownership:**  
Phase 9。

### Pitfall 5: smoke automation 直接读写真实 app data、日志目录或用户配置

**Why it matters here:**  
当前仓库已经存在 app path、log path、test isolation 相关 concern。smoke 如果落到真实目录，会把回归测试变成污染源。

**What goes wrong:**

- CI / 本地 smoke 改写真实剪贴板数据库或日志
- 测试结果依赖开发者机器现状，无法稳定复现
- package smoke 与 dev smoke 互相污染缓存和 config

**Prevention strategy:**

- 复用 `src-tauri/src/test_support.rs` 与 `src-tauri/tauri.test.conf.json` 的隔离思路
- 为 smoke 明确 test root / test config / test env
- 所有 smoke 证据输出到独立 artifact 目录，不复用真实用户目录

**Recommended phase ownership:**  
Phase 9，必要时由 Phase 6 先补基础 path/test seam。

## Why These Pitfalls Matter In This Repo

当前仓库的几个现状会放大上述风险：

- `previewDescriptor` 与 `PrimaryPreviewRenderer` 已经是 detail surface 的关键汇流点，任何“前端临时解析”都会立即外溢。
- `retrieval/mod.rs` 已经有 FTS 与 ranking 逻辑，benchmark 若不固定，会造成隐蔽回归。
- `.github/workflows/release.yml` 与 `.github/workflows/test-build.yml` 已经包含 build/test/package gate，这意味着 smoke 设计错误会直接污染现有发布链。
- `.planning/codebase/CONCERNS.md` 已经记录了 path split、test 不 hermetic、monitoring lifecycle 等问题；新 smoke 不处理这些 seam，会把问题放大。

## Primary Sources

### Repo Evidence

- `.planning/PROJECT.md`
- `.planning/codebase/CONCERNS.md`
- `.planning/codebase/TESTING.md`
- `src/lib/preview/previewDescriptor.ts`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx`
- `src-tauri/src/retrieval/mod.rs`
- `src-tauri/src/test_support.rs`
- `src-tauri/tauri.test.conf.json`
- `.github/workflows/release.yml`
- `.github/workflows/test-build.yml`

### Official Sources

- Tauri WebDriver docs: https://v2.tauri.app/develop/tests/webdriver/
- Tauri GitHub pipelines docs: https://v2.tauri.app/distribute/pipelines/github/
- SQLite FTS5 documentation: https://www.sqlite.org/fts5.html

---

_Research completed: 2026-03-29_  
_Ready for roadmap: yes_
