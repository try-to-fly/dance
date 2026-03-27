# Requirements: Dance

**Defined:** 2026-03-27
**Core Value:** 开发者复制任意常见内容后，应用都能稳定记录、准确识别，并以最合适的结构化方式展示出来。

## v1 Requirements

本轮 requirements 只覆盖当前 brownfield 迭代范围，不重复声明已经在 `.planning/PROJECT.md` 中作为 Validated 记录的既有能力。当前 v1 聚焦把现有客户端打磨成更可靠的开发者剪贴板工作台。

### Capture Reliability

- [ ] **CAPT-01**: User can start and stop clipboard monitoring without hidden background listeners continuing after stop
- [ ] **CAPT-02**: User sees each clipboard change recorded once without duplicate history entries caused by repeated listeners or self-generated copy flows
- [ ] **CAPT-03**: User can keep ignored, transient, concealed, or otherwise non-persistent clipboard events out of saved history
- [ ] **CAPT-04**: User can rely on one consistent local storage lifecycle for history, cache, image assets, and related metadata

### Detection

- [ ] **DETE-01**: User copying supported developer content gets a stable subtype classification for URL, JSON, code, command, color, markdown, email, IP, timestamp, base64, or plain text fallback
- [ ] **DETE-02**: User sees subtype-specific metadata extracted for supported content, including URL parts, color formats, detected language, timestamp formats, and related structured hints
- [ ] **DETE-03**: User benefits from improved detection rules on existing history without needing to recopy items after parser or classifier upgrades
- [ ] **DETE-04**: User can still inspect copied content when analysis fails because the app degrades gracefully to raw content and preserves failure diagnostics for later repair

### Preview

- [ ] **PREV-01**: User can inspect JSON entries in a formatted structured view and switch back to the raw representation
- [ ] **PREV-02**: User can inspect URL entries in a structured preview showing at least protocol, host, path, and query details without requiring default remote fetching
- [ ] **PREV-03**: User can inspect color entries with a visual swatch and alternate color formats suitable for development work
- [ ] **PREV-04**: User can inspect code and command entries in a read-only developer-oriented view with preserved formatting and language or shell hints when available
- [ ] **PREV-05**: User sees the same semantic type and preview intent for an entry across the list view, detail view, and follow-up retrieval flows

### Retrieval

- [ ] **RETR-01**: User can retrieve history through indexed interactive search that remains responsive on large local datasets
- [ ] **RETR-02**: User can narrow results by content type or subtype, source app, favorites, and recency-oriented filters
- [ ] **RETR-03**: User can find entries with fuzzy fragments, abbreviations, or partial developer tokens when exact text is unknown
- [ ] **RETR-04**: User can search normalized structured tokens where available, such as URL host or path fragments, JSON keys, command names, and alternate color values
- [ ] **RETR-05**: User sees ranked results with enough snippet, highlight, or summary context to distinguish similar matches quickly

### Reliability & Maintenance

- [ ] **RELY-01**: User keeps existing history usable after detection or search upgrades because analysis and search indexes can be rebuilt without clearing stored entries
- [ ] **RELY-02**: User experiences fewer regressions across releases because monitoring, preview, and retrieval critical paths are covered by automated validation and packaged smoke checks

## v2 Requirements

这些能力对开发者有潜在价值，但不应挤占当前主线。

### Extended Developer Views

- **EXTD-01**: User can inspect JWT tokens in a dedicated structured security-oriented view
- **EXTD-02**: User can inspect TOML, XML, CSV, and TSV content in dedicated structured previews instead of generic text views
- **EXTD-03**: User can inspect log-heavy clipboard entries in a dedicated log viewer with level and stack-oriented cues

### Optional Enrichment

- **ENRH-01**: User can explicitly request remote URL enrichment when they want additional metadata beyond local URL parsing

## Out of Scope

Explicitly excluded from the current roadmap.

| Feature                         | Reason                                                              |
| ------------------------------- | ------------------------------------------------------------------- |
| Cloud sync                      | 当前只做本地客户端体验，避免引入账户、服务端和同步一致性复杂度      |
| Multi-device sync               | 本轮先把单机可靠性、识别和检索做扎实，不扩展跨设备状态              |
| Mobile app                      | 当前产品边界是桌面客户端，不在手机和平板端扩张                      |
| Team collaboration              | 当前服务对象是个人开发者，不做共享工作区或多人协作                  |
| Sharing workflows               | 与当前“查看、理解、检索、回用本地内容”的核心主线不一致              |
| Default remote URL fetching     | 默认联网抓取会引入隐私、安全和性能风险，当前只做本地结构化 URL 解析 |
| AI-first semantic search        | 当前更需要可解释、可控、可本地运行的确定性搜索与模糊匹配            |
| Heavy snippet-manager workflows | 当前目标是高频剪贴板回查，不做长期知识库或复杂标签系统              |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status  |
| ----------- | ----- | ------- |
| CAPT-01     | TBD   | Pending |
| CAPT-02     | TBD   | Pending |
| CAPT-03     | TBD   | Pending |
| CAPT-04     | TBD   | Pending |
| DETE-01     | TBD   | Pending |
| DETE-02     | TBD   | Pending |
| DETE-03     | TBD   | Pending |
| DETE-04     | TBD   | Pending |
| PREV-01     | TBD   | Pending |
| PREV-02     | TBD   | Pending |
| PREV-03     | TBD   | Pending |
| PREV-04     | TBD   | Pending |
| PREV-05     | TBD   | Pending |
| RETR-01     | TBD   | Pending |
| RETR-02     | TBD   | Pending |
| RETR-03     | TBD   | Pending |
| RETR-04     | TBD   | Pending |
| RETR-05     | TBD   | Pending |
| RELY-01     | TBD   | Pending |
| RELY-02     | TBD   | Pending |

**Coverage:**

- v1 requirements: 20 total
- Mapped to phases: 0
- Unmapped: 20 ⚠️

---

_Requirements defined: 2026-03-27_
_Last updated: 2026-03-27 after initial definition_
