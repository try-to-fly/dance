# 详情预览重构审查报告

审查日期：2026-03-27

审查基线：`0238cce refactor: 底层重构`

关联方案文档：`docs/detail-preview-redesign-plan.md`

## 1. 审查范围

本次审查基于最新一次提交进行，因为审查时工作区没有未提交改动。

- `git status --short` 为空
- `git diff HEAD` 为空
- 实际审查范围为 `HEAD~1..HEAD`

重点关注以下内容：

- 详情预览是否按方案落地为 `PreviewDescriptor -> DetailScene`
- URL / Base64 / 图片等异步解析链路是否闭环
- 前后端类型、缓存、状态切换是否存在回归风险
- 测试是否覆盖了关键行为和错误路径

## 2. 总体结论

这次重构方向是对的。

- 详情区已经从旧的 subtype 直切 renderer，收敛为 `PreviewDescriptor + DetailScene`
- URL / Base64 的按需解析能力已经从零散组件逻辑提升到了 store 和 Tauri 命令层
- 基础 contract test、scene test、JSON 预览测试都已经补上

当前主要问题不在“能力是否存在”，而在以下几个层面还没有彻底接通：

- 错误路径处理不完整
- descriptor 到最终 UI 动作的闭环还没完成
- 快速切换条目时的状态一致性有问题
- 部分边界值和大对象场景仍然会退化成错误展示

## 3. 主要问题

### 3.1 高优先级

#### 3.1.1 切换条目时会短暂复用上一条的解析结果

涉及文件：

- `src/components/DetailView/DetailView.tsx:68`
- `src/components/DetailView/DetailView.tsx:83`
- `src/components/DetailView/DetailView.tsx:106`
- `src/components/DetailView/DetailView.tsx:158`
- `src/components/DetailView/DetailView.tsx:212`

问题说明：

`resolvedPreview` 在 `selectedEntry` 切换时不会先清空，只有在完全没有选中条目时才会被置为 `null`。这会导致新条目首帧渲染时，使用“当前条目 + 上一条解析结果”去构建 descriptor。

直接影响：

- 详情区会短暂显示上一条的图片、媒体信息或 inspector 内容
- `handleOpenUrl` 会优先使用旧的 `resolvedPreview.url.finalUrl`
- 快速切换条目时，用户看到的内容可能和当前选中项不一致

建议：

- 在 `selectedEntry.id` 或 `selectedEntry.content_hash` 变化时立即重置 `resolvedPreview`
- 或者给 `resolvedPreview` 绑定来源 key，渲染前验证它仍然匹配当前条目

#### 3.1.2 URL 解析失败时没有进入前端 fallback，错误结果还会被缓存

涉及文件：

- `src-tauri/src/commands.rs:1012`
- `src-tauri/src/commands.rs:1015`
- `src-tauri/src/commands.rs:1060`
- `src/stores/clipboardStore.ts:547`
- `src/stores/clipboardStore.ts:565`
- `src/stores/clipboardStore.ts:632`

问题说明：

Rust 侧 `resolve_url_preview` 在网络失败或 HTTP 非 2xx 时返回的是 `Ok(resolution)`，只是填充了 `error` 字段。前端 `resolveUrlPreview` 只有在 `invoke` 抛异常时才会进入扩展名和媒体 heuristics fallback。

直接影响：

- 离线、超时、DNS 失败时，明显的图片或音视频 URL 会退化成普通 `url_card`
- 这个退化结果会被 `previewResolutionCache` 缓存 5 分钟
- 用户短时间内即使网络恢复，也可能继续看到错误预览

建议：

- 方案一：Tauri 侧在网络失败和 HTTP 错误时直接返回 `Err`
- 方案二：前端把 `response.error` 视为 degraded result，并继续走 fallback
- 不要缓存错误结果，或者为错误结果使用更短 TTL

### 3.2 中优先级

#### 3.2.1 `PreviewDescriptor.actions` 已产出，但 UI 没有真正消费

涉及文件：

- `src/lib/preview/previewDescriptor.ts:321`
- `src/lib/preview/previewDescriptor.ts:323`
- `src/lib/preview/previewDescriptor.ts:328`
- `src/components/DetailView/scene/DetailScene.tsx:162`
- `src/components/DetailView/DetailView.tsx:186`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx:187`

问题说明：

descriptor 已经生成 `open_url`、`open_file`、`copy_decoded` 等动作，但 `DetailScene` 顶部按钮仍然是固定的复制、粘贴、收藏、删除。`DetailView.handleCopy()` 也始终只复制 raw content。当前只有 `url_card` 和 `file_card` 分支手写了局部按钮。

直接影响：

- 媒体型 URL 进入主预览后，反而丢失“打开原始链接”的入口
- Base64 已经定义了 `copy_decoded`，但用户无法触发
- descriptor 里的动作信息已经存在，但没有形成真正的场景层闭环

建议：

- 把动作栏真正收口到 scene 层
- 由 `descriptor.actions` 驱动按钮渲染和 handler 绑定
- 避免继续在具体 preview renderer 中分散补按钮

#### 3.2.2 JSON 边界值 `0`、`false`、`null` 会被误判为“没有 JSON”

涉及文件：

- `src/lib/preview/previewDescriptor.ts:153`
- `src/lib/preview/previewDescriptor.ts:272`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx:135`
- `src-tauri/src/commands.rs:1106`

问题说明：

前端对 `resolvedData.jsonContent` 使用了 truthy 判断。后端 `parse_json_if_possible()` 返回的是任意合法 JSON，因此 `0`、`false`、`null` 这种合法 JSON 会在前端被误判为“没有结构化结果”。

直接影响：

- URL/Base64 命中这些 JSON 时，主预览会错误地退回到纯文本
- 备用 JSON 视图不会出现
- 已经解析成功的结构化结果无法被正确呈现

建议：

- 改为 `resolvedData.jsonContent !== undefined`
- URL 分支的主预览决策也应尊重 `resolvedData.url?.previewKind === 'json'`

#### 3.2.3 筛选结果为空时，右侧仍可能保留旧详情

涉及文件：

- `src/stores/clipboardStore.ts:753`
- `src/stores/clipboardStore.ts:756`
- `src/components/ClipboardList/ClipboardList.tsx:35`
- `src/components/DetailView/DetailView.tsx:118`

问题说明：

`setSelectedType()` 只会在过滤结果非空时更新 `selectedEntry`。`ClipboardList` 也只会在“当前没有选中项”时自动选中第一条。这样当搜索或筛选结果为空时，左侧列表已经空了，右侧仍然可能显示之前的条目详情。

直接影响：

- 当前工作区的左右信息不同步
- 用户会误以为右侧显示的内容仍在当前筛选结果内

建议：

- 在过滤结果为空时显式清空 `selectedEntry`
- 或者让 `selectedEntry` 始终从当前过滤后的 entries 中派生

#### 3.2.4 文本类 URL 预览虽然限制展示大小，但实际仍会全量下载

涉及文件：

- `src-tauri/src/commands.rs:1099`
- `src-tauri/src/commands.rs:1100`
- `src-tauri/src/commands.rs:1102`
- `src-tauri/src/commands.rs:1108`

问题说明：

当前逻辑是先 `response.bytes().await` 把响应体完整下载到内存，再通过 `URL_PREVIEW_MAX_BYTES` 截断最终展示。也就是说，限制的只是“显示给用户的内容大小”，不是“实际读取和占用的资源大小”。

直接影响：

- 大型文本响应仍会被完整下载
- 内存和网络开销高于设计目标中的“按需轻量预览”

建议：

- 改成 streaming 或带上限的读取
- 在读取阶段而不是展示阶段做截断

#### 3.2.5 大尺寸 Base64 媒体会进入自相矛盾的预览状态

涉及文件：

- `src-tauri/src/commands.rs:1166`
- `src-tauri/src/commands.rs:1179`
- `src-tauri/src/commands.rs:1185`
- `src-tauri/src/commands.rs:1196`
- `src/lib/preview/previewDescriptor.ts:168`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx:68`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx:80`

问题说明：

当 Base64 被识别为图片、音频或视频，但体积超过 `BASE64_DATA_URL_MAX_BYTES` 时，后端仍然保留 `decoded_kind = image/audio/video`，但不会返回 `data_url`，只会写入 `error`。前端又忽略了这个 `error`，继续按媒体主预览来渲染。

直接影响：

- scene 会认为它是媒体预览
- renderer 却拿不到 `src`
- 图片分支最终可能直接退回原始 Base64 文本
- 音视频分支则只剩空播放器或未知占位

建议：

- 后端在超限时显式降级成 `base64_binary`、`file_card` 或 `unsupported`
- 或者前端根据 `error + 无 dataUrl` 做一致性降级

### 3.3 次要问题

#### 3.3.1 新文案存在硬编码，国际化没有完全接通

涉及文件：

- `src/lib/preview/previewDescriptor.ts:44`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx:197`
- `src/components/ClipboardList/ClipboardItem.tsx:193`

问题说明：

新 UI 里仍有不少硬编码文本，例如 inspector 的 `Protocol / Host / Path`、URL 卡片里的“复制 / 打开”、列表里的“图片预览不可用”“时间解析”“(无内容)”等。

直接影响：

- 多语言环境下会出现中英文混杂
- 新增 descriptor / scene 层后，文案注入方式还没有统一

建议：

- scene、inspector、alternate views、列表项预览统一改成通过 `t()` 或 labels 注入

#### 3.3.2 列表项摘要与详情语义模型尚未统一

涉及文件：

- `src/lib/preview/entryPresentation.ts:96`
- `src/components/DetailView/DetailView.tsx:158`
- `src/components/ClipboardList/ClipboardItem.tsx:110`
- `docs/detail-preview-redesign-plan.md:22`

问题说明：

详情侧已经使用 `buildEntrySemanticSummary()` 统一标题语义，但列表项仍然是自己按 subtype 和 raw text 拼装摘要，没有复用同一套语义模型。

直接影响：

- 同一条 URL 在详情里显示 host/path，在列表里仍可能是原始长链接
- 时间戳、Base64 等类型在左右区域的表达风格不一致
- 与方案里“列表项摘要、详情标题、主预览、辅助信息来自同一套语义模型”的目标仍有差距

建议：

- 让列表项也基于 `buildEntrySemanticSummary()` 生成 headline / summary
- 再叠加列表自己的压缩样式，而不是重写一套语义逻辑

#### 3.3.3 图片预览失败时缺少明确的失败占位

涉及文件：

- `src/components/DetailView/DetailView.tsx:89`
- `src/components/DetailView/scene/PrimaryPreviewRenderer.tsx:68`

问题说明：

只要 `content_type` 包含 `image`，descriptor 就会优先进入图片主预览；但如果 `getImageUrl()` 失败，当前图片分支没有明确的 unavailable fallback。

直接影响：

- 图片条目在失败路径下可能出现空面板或退回无意义内容

建议：

- 图片分支在没有可用 `src` 时显式渲染失败占位
- 或者在 descriptor 层不要过早锁死为 `image`

## 4. 已做验证

本次审查过程中，已运行以下定向验证：

- `npm test -- src/components/DetailView/DetailPreviewContract.test.tsx src/components/DetailView/DetailView.test.tsx src/components/DetailView/ContentRenderers/JsonRenderer.test.tsx src/components/DetailView/scene/AlternateViews.test.tsx`
- `cargo test preview_kind_from_mime --manifest-path src-tauri/Cargo.toml`
- `cargo test detect_decoded_kind --manifest-path src-tauri/Cargo.toml`

验证结果：

- 前端相关定向测试通过
- Rust 侧定向测试通过

需要注意：

- 现有测试没有覆盖“快速切换条目”
- 现有测试没有覆盖“Rust 返回 `error` 但前端未 fallback”
- 现有测试没有覆盖“大尺寸 Base64 媒体降级行为”
- 现有测试没有覆盖“筛选结果为空时右侧详情是否清空”
- 现有测试没有覆盖“descriptor.actions 是否真正映射到 UI 交互”

## 5. 建议的修复优先级

建议按以下顺序处理：

1. 先修状态一致性和错误路径
2. 再补 descriptor.actions 到 scene 的动作闭环
3. 再修 JSON falsy 值和 Base64 大对象降级
4. 最后处理国际化统一、列表语义统一和图片失败占位

其中最值得优先落地的是：

- 切换条目时清空旧的 `resolvedPreview`
- URL 解析失败时继续走前端 fallback
- 筛选结果为空时同步清空 `selectedEntry`
- 把 `copy_decoded` / `open_url` / `open_file` 真正接到 UI 上

## 6. 结论

这版 detail preview 重构已经完成了最难的一步，也就是把详情预览从分散的 renderer 逻辑，推进成统一的描述模型和解析链路。结构已经走在正确方向上。

现在剩下的问题，主要集中在“最后一公里”：

- 解析结果和当前选中项是否严格一致
- descriptor 中定义的能力是否真的可被用户触达
- 错误和边界场景是否能稳定降级

把这几处补齐之后，这套 detail preview 设计才算真正和方案目标闭环。
