# Project Milestones: Dance

## v1.0 MVP (Shipped: 2026-03-29)

**Delivered:** 一个面向开发者的本地桌面剪贴板工作台，已经具备稳定 capture、authoritative analysis、统一 preview、本地 retrieval，以及 rebuild/release safety。

**Phases completed:** 1-5 (20 plans total)

**Key accomplishments:**

- 收口了剪贴板 capture lifecycle、存储路径和 suppression contract，建立更可靠的本地持久化基础。
- 建立 Rust authoritative analysis contract、typed metadata、fallback diagnostics 与 history rebuild。
- 统一 JSON、URL、颜色、代码、命令等 developer content 在 list/detail/retrieval 三个 surface 的语义和展示。
- 落地本地 retrieval，支持 structured token、模糊片段、来源应用/收藏/时间窗口筛选，以及 snippet/match reason。
- 把 search rebuild 串进现有 rebuild 入口，并在 release/test-build workflow 中加入打包前验证 gate。

**Stats:**

- 104 files included in the shipped milestone commit
- 86,685 lines of TypeScript/TSX/Rust in the current desktop codebase
- 5 phases, 20 plans, 40 tasks
- 3 days from first milestone commit to ship

**Git range:** `feat(01-01)` -> `chore: complete v1.0 milestone`

**What's next:** 定义下一个 milestone，继续扩展 developer-specific previews、retrieval quality 和 smoke automation。

---
