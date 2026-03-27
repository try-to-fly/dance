# Feature Landscape: Developer Clipboard Manager

**Domain:** 面向开发者的本地桌面剪贴板管理器
**Researched:** 2026-03-27
**Scope:** brownfield 优化，聚焦内容类型识别、结构化预览、搜索检索、筛选与模糊匹配
**Overall confidence:** MEDIUM-HIGH

## Executive Summary

当前主流剪贴板管理器已经把“历史记录 + 即时搜索 + 键盘召回 + 收藏/固定 + 本地隐私控制”做成默认预期。Raycast、Paste、Alfred、Maccy、CopyQ 这类产品虽然定位不同，但都把“快速找回上一次复制内容”视为基础能力，而不是卖点。

对开发者来说，基础预期比通用用户再往前一步。只把内容当成一段纯文本已经不够了。JSON、URL、颜色、代码片段、命令行、日志这类内容如果不能被识别成有语义的对象，用户会觉得产品只是“有历史记录的剪贴板”，而不是“能帮助理解和回用开发内容的工具”。

真正的差异化不在支持更多花哨类型，而在三个层面：第一，识别足够准，误判率低；第二，预览足够有结构，让用户在粘贴前就能判断对不对；第三，检索足够快，尤其是能跨内容类型、元数据和模糊输入找到目标。对本项目而言，这比云同步、团队协作、AI 功能更贴近核心价值，也更符合当前代码库边界。

## Decision Summary

**Table stakes:** 本地历史、键盘优先召回、即时搜索、类型筛选、基础开发类型识别、最低限度结构化预览、隐私与排除规则。

**Differentiators:** 更高置信度的开发内容识别、针对 JSON/URL/颜色/代码/命令/日志的深度预览、类型感知搜索、模糊匹配排序、检索结果的结构化摘要。

**Do not optimize for:** 云同步、团队共享、默认联网抓取 URL 内容、AI 语义检索优先、重型片段管理系统。

## Table Stakes

缺少这些能力，产品会被感知为“不完整”或“不够开发者友好”。

| Feature Cluster          | 用户最低预期                                                                            | Why Expected                                                                                  | Complexity | Confidence  |
| ------------------------ | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ---------- | ----------- |
| 历史记录与键盘召回       | 自动记录历史、键盘打开列表、回车复制或粘贴、支持收藏/固定/删除/清空                     | Raycast、Paste、Alfred、Maccy、CopyQ 都把这类能力当基础入口                                   | Low-Med    | HIGH        |
| 即时全文搜索             | 输入即过滤，200ms 级反馈，至少支持按内容文本检索                                        | 主流产品都强调“快速找到最近复制内容”，没有搜索几乎不可用                                      | Low        | HIGH        |
| 基础筛选                 | 按内容类型、最近时间、固定项、来源应用筛选                                              | Raycast 与 Paste 已经把类型和应用过滤做成标准交互                                             | Med        | HIGH        |
| 本地隐私控制             | 忽略指定应用、控制保留时长、默认本地存储                                                | 剪贴板内容常含密钥、令牌、账号信息，本地工具必须可控                                          | Med        | HIGH        |
| 基础内容类型识别         | 至少正确区分文本、图片、文件、链接、颜色；对开发者场景再加 JSON、代码、命令的粗粒度识别 | 通用产品已支持 links/colors/files/images；开发者产品如果仍全归类为 plain text，会直接失去定位 | Med        | HIGH        |
| 最低限度结构化预览       | 不只显示截断文本；至少能看到完整文本、来源应用、复制时间、基本类型徽标                  | 用户需要在粘贴前确认“这条是不是我要的”                                                        | Med        | HIGH        |
| JSON/代码/URL 的可读展示 | JSON 至少自动格式化；代码至少等宽字体与保留缩进；URL 至少拆出 host/path                 | 开发者复制内容往往长且相似，纯文本列表不足以区分                                              | Med        | MEDIUM-HIGH |
| 稳定的退化策略           | 识别失败时回落到纯文本，不应因为某个预览器失败导致条目不可查看                          | 开发者复制的内容形态噪声大，不能假设永远命中规则                                              | Med        | HIGH        |

## Differentiators

这些能力不是所有剪贴板管理器都有，但对“开发者向”定位非常加分，且最贴近本项目本轮目标。

| Feature Cluster      | Value Proposition                                       | 具体要求                                                                                          | Complexity | Confidence  |
| -------------------- | ------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ---------- | ----------- |
| 高置信度开发内容识别 | 降低误判，减少“看起来像 JSON/命令/代码但实际不是”的噪声 | 对 JSON、URL、颜色、代码、命令、日志建立冲突优先级与 fallback；给出明确 subtype 与置信信号        | Med-High   | HIGH        |
| JSON 深度预览        | 让复制 API 响应、配置片段、对象字面量时可直接理解结构   | 格式化、折叠层级、数组/对象摘要、无效 JSON 明确提示、保留 raw view                                | Med        | HIGH        |
| URL 结构化预览       | 让复制 API URL、文档链接、GitHub 链接时更快判断目标     | 分离协议、域名、路径、查询参数、锚点；优先本地解析，远程抓取应显式开启                            | Med        | HIGH        |
| 颜色开发视图         | 让设计 token、CSS 颜色、主题值可直接复用                | 色块预览、HEX/RGB/HSL/Alpha 同步显示、快速复制其他格式                                            | Med        | MEDIUM-HIGH |
| 代码片段预览         | 让用户在多个相似片段里快速选中正确版本                  | 自动语言猜测、语法高亮、保留缩进、长代码折叠、raw/code 双视图                                     | Med        | HIGH        |
| 命令行视图           | 让 shell 命令区别于普通文本，并降低误操作               | shell 类型提示、参数换行/折叠、危险命令警示、raw command 一键复制                                 | Med        | MEDIUM      |
| 日志视图             | 让日志、堆栈、错误输出可快速扫描                        | 识别时间戳、日志级别、堆栈行、JSON 日志与纯文本日志分流展示                                       | Med-High   | MEDIUM      |
| 类型感知搜索         | 不只是搜全文，而是搜索“被解析后的内容”                  | URL 可搜 host/path/query；颜色可搜任一颜色格式；JSON 可搜 key；代码可搜语言标签；命令可搜可执行名 | High       | MEDIUM-HIGH |
| 模糊匹配与排序       | 开发者常凭片段记忆搜索，未必记得完整原文                | 支持子序列、camelCase、kebab-case、路径片段、域名片段、最近使用加权排序                           | High       | MEDIUM      |
| 结果摘要与判别信息   | 在列表阶段就减少点开详情的次数                          | 列表项展示 subtype、来源应用、时间、结构摘要，例如 `GET api.example.com/users?page=2`             | Med        | HIGH        |
| 多视图切换           | 同一内容在 raw 和 semantic 之间切换                     | 例如 URL 原文 / 结构卡片，JSON raw / pretty，代码 raw / highlighted                               | Med        | HIGH        |

## Anti-Features

这些方向会稀释当前目标，或者在本阶段投入产出比很差。

| Anti-Feature                   | Why Avoid                                                        | What to Do Instead                                           |
| ------------------------------ | ---------------------------------------------------------------- | ------------------------------------------------------------ |
| 云同步与多设备同步             | 直接把产品问题从“识别和检索”扩展到账户、冲突、加密、同步一致性   | 保持 local-first，把单机体验做扎实                           |
| 团队共享、公共剪贴板、协作空间 | 与“个人开发者快速回用本地内容”不一致，会吞掉大量产品和安全复杂度 | 保留个人工作台定位                                           |
| 默认联网抓取所有 URL 预览      | 会引入隐私泄露、内网探测、性能抖动，与本地工具心智冲突           | 默认只做本地 URL 解析，联网元数据抓取设为显式操作            |
| AI 语义搜索作为主检索路径      | 复杂度高、排序不可解释、可能损害本地隐私，还不能替代确定性搜索   | 先做 deterministic search + type-aware index + fuzzy ranking |
| 强制手工分类、文件夹、标签体系 | 会拖慢“复制后立刻找回”的主路径                                   | 优先自动识别和自动筛选，手工 pin/save 可选                   |
| 把产品做成重型 snippet manager | 会把重点从“检索最近复制内容”转成“长期知识管理”                   | 后续可提供“保存为片段”，但不应反向主导核心流                 |

## What Matters Most For Solo Developers

这一节直接服务 requirements 定义，回答“哪些内容类型最值得优先优化”。

| Content Type | Importance | 用户真正要解决的问题                                     | Minimum Acceptable             | Great Version                                |
| ------------ | ---------- | -------------------------------------------------------- | ------------------------------ | -------------------------------------------- |
| JSON         | Very High  | 多个 API 响应或配置对象非常相似，纯文本难分辨            | 自动格式化、raw 切换、无效提示 | 树形折叠、key 搜索、结构摘要                 |
| URL          | Very High  | API 地址、文档链接、GitHub 页面、带 query 的链接容易混淆 | host/path/query 基础拆解       | URL 类型细分、参数表、列表摘要、类型感知搜索 |
| 代码         | Very High  | 代码片段常只有几行差异，用户要快速找到“对的那段”         | 等宽展示、保留缩进、语言猜测   | 语法高亮、长段折叠、语言过滤                 |
| 命令         | High       | shell 命令要能快速识别，避免当普通文本误用               | monospace 预览、复制原文       | shell 检测、危险命令提示、命令名检索         |
| 日志         | High       | 错误日志和堆栈很长，用户需要快速定位错误上下文           | raw 预览、保留换行             | 日志级别高亮、堆栈折叠、JSON log 解析        |
| 颜色         | Med-High   | CSS/设计 token/主题值经常在多种颜色格式间切换            | 色块 + 原始值                  | HEX/RGB/HSL 转换与复制                       |

## Search, Filter, And Fuzzy Matching Guidance

### 搜索

建议把搜索分成三层，而不是继续只做单一 `%LIKE%`：

1. **原文搜索**：匹配原始文本，保证简单直接。
2. **规范化搜索**：对 URL、颜色、命令、代码语言、JSON key 等提取 searchable tokens。
3. **排序层**：叠加最近使用、固定项、来源应用、subtype 命中权重。

### 筛选

建议优先提供以下 facet：

1. 内容大类：文本 / 图片 / 文件。
2. 开发子类：JSON / URL / 代码 / 命令 / 日志 / 颜色。
3. 来源应用：浏览器 / IDE / 终端 / 设计工具。
4. 状态：固定、最近、当前搜索命中。

### 模糊匹配

模糊匹配对开发者很有价值，但更适合作为增强排序，而不是替代精确搜索。

推荐支持：

1. 子序列匹配，例如 `ghpr` 命中 `github pull request`。
2. camelCase / snake_case / kebab-case token 拆分。
3. 路径与域名片段匹配，例如 `api usr pg` 命中 `api.example.com/users?page=2`。
4. 结果高亮与命中解释，避免“为什么这条排第一”不可理解。

结论：**全文搜索是 table stake，模糊排序是 developer-focused differentiator。**

## Feature Dependencies

```text
稳定监听与去重
  -> 规范化存储内容、来源应用、时间
  -> 内容类型识别与 metadata 提取
  -> 结构化预览模型
  -> 类型筛选与列表摘要
  -> 类型感知搜索索引
  -> 模糊匹配与排序优化
```

更细一点：

```text
JSON/URL/代码/命令/日志识别
  -> 对应 subtype 预览
  -> 对应 subtype 搜索 token
  -> 对应 subtype 筛选 facet

raw view
  -> 安全退化策略
  -> 调试识别误判

来源应用与复制时间
  -> 来源筛选
  -> 结果排序
  -> 列表判别信息
```

## MVP Recommendation

优先顺序建议如下：

1. **先把识别做准**：JSON、URL、颜色、代码、命令、日志的 subtype 判定与 fallback。
2. **再把预览做清楚**：raw 与 semantic 双视图，优先覆盖 JSON、URL、代码、颜色、命令。
3. **然后把检索做快**：类型筛选、来源筛选、规范化 token 搜索。
4. **最后再加模糊排序**：把它建立在类型元数据和规范化索引之上，而不是直接堆一个模糊库。

**Defer:** 联网 URL 抓取、AI 语义搜索、重型片段管理、云同步与协作。

## Confidence Notes

| Area                                         | Confidence | Notes                                                               |
| -------------------------------------------- | ---------- | ------------------------------------------------------------------- |
| 历史记录、搜索、筛选                         | HIGH       | 多个主流产品已形成稳定共识                                          |
| 开发类型识别与结构化预览                     | HIGH       | 与本项目定位和现有代码能力高度一致                                  |
| 模糊匹配属于 table stake 还是 differentiator | MEDIUM     | 更接近 launcher/开发者工具预期，不是所有 clipboard manager 都已内建 |
| 日志/命令深度语义视图                        | MEDIUM     | 开发者价值明确，但市场上并非统一标准配置                            |

## Sources

### Internal Evidence

- `/Users/smile/Documents/try-to-fly/clipboard-app/.planning/PROJECT.md`
- `/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/ARCHITECTURE.md`
- `/Users/smile/Documents/try-to-fly/clipboard-app/.planning/codebase/CONCERNS.md`
- `/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/clipboard/content_detector.rs`
- `/Users/smile/Documents/try-to-fly/clipboard-app/src/lib/clipboardFilters.ts`
- `/Users/smile/Documents/try-to-fly/clipboard-app/src/lib/preview/previewDescriptor.ts`
- `/Users/smile/Documents/try-to-fly/clipboard-app/src/components/SearchBar/SearchBar.tsx`
- `/Users/smile/Documents/try-to-fly/clipboard-app/src-tauri/src/state.rs`

### External Sources

- Raycast Manual, Clipboard History: https://manual.raycast.com/windows/clipboard-history
- Raycast Manual, Search Bar: https://manual.raycast.com/raycast/search-bar
- Paste Help, Search and Filters: https://pasteapp.io/help/search-and-filters
- Paste Help, Supported Data Types: https://pasteapp.io/help/supported-data-types
- Alfred Help, Clipboard History: https://www.alfredapp.com/help/features/clipboard/
- Alfred Help, Snippets: https://www.alfredapp.com/help/features/snippets/
- Maccy official site: https://maccy.app/
- CopyQ documentation: https://copyq.readthedocs.io/en/latest/
- CopyQ item model and notes/actions docs: https://copyq.readthedocs.io/en/latest/items.html
