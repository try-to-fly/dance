# 剪贴板详情预览改造技术方案

## 1. 背景与目标

当前应用的主工作区已经具备稳定的左右分栏结构，但右侧详情预览仍然以“通用头部 + 按 `content_type` / `content_subtype` 切换 renderer”的方式组织。这个结构能完成基础展示，但不能保证“用户复制了什么，就优先看到什么”。

这会直接导致几个体验问题：

- 用户复制 JSON，右侧不一定优先展示结构化内容。
- 用户复制图片 URL、视频 URL、音频 URL，右侧不是稳定地以媒体预览为主视图。
- 用户复制 Base64，当前检测器能识别，但前端没有把“解码后的内容”纳入主预览体系。
- 用户复制普通 URL，右侧更像“URL 信息卡”，而不是“URL 指向内容”的最佳预览。
- 用户选择文件类内容时，右侧几乎只有路径文本，没有预览价值。

本方案的核心目标只有一个：

**右侧详情区必须用对用户当前内容最有价值的方式进行展示。**

具体展开为四条设计目标：

- 主预览优先：首屏优先展示“用户最想看”的内容，不优先展示记录管理信息。
- 类型语义统一：列表项摘要、详情页标题、详情页主预览、辅助信息必须来自同一套内容语义模型。
- 检测与预览分层：剪贴板入库阶段只做轻量检测；重型解析、网络请求、解码、ffprobe 只在用户选中条目时按需执行。
- 可扩展：未来新增 `base64`、本地文件、富文本、HTML 片段、代码片段增强视图时，不再继续堆 `switch case`。

## 2. 当前现状与核心问题

### 2.1 结构问题

当前详情页入口在 `src/components/DetailView/DetailView.tsx`，它同时负责：

- 解析标题
- 生成顶部元信息 pills
- 处理复制 / 粘贴 / 收藏 / 删除
- 判断 `content_type`
- 判断 `content_subtype`
- 分发到具体 renderer
- 控制内容区的滚动模式

这导致 `DetailView` 变成一个过重的分发器，而不是一个“详情场景容器”。

### 2.2 信息架构问题

右侧当前更偏“记录管理面板”，而不是“内容理解面板”：

- 顶部主要展示来源、时间、复制次数
- 内容区再切换各种 renderer
- 不同类型没有统一的主预览区、辅助信息区、备用视图区

结果就是：

- 图片看起来像图片页
- JSON 看起来像编辑器页
- URL 看起来像 URL 卡片页
- 文件看起来像空白占位页

它们不是同一个产品语言。

### 2.3 类型系统问题

Rust 侧与前端侧的类型系统已经出现断层：

- Rust `ContentSubType` 已经支持 `Base64`
- 前端 `ContentSubType` 还没有 `base64`
- 筛选器和文案也没有 `base64`
- Rust `ContentType` 只有 `Text / Image / Unknown`
- 前端却把 `file` 当作主类型对待

这说明当前前端展示层已经超前假设了一部分类型，但后端采集和类型定义还没有完全支撑。

还有一个不能忽略的现实问题：

- 监听层当前会直接跳过 `data:image/...;base64,...`

这意味着如果用户复制的是图片型 data URL，条目甚至不会进入历史记录。后续如果要把 Base64 方案做完整，必须重新设计这一层的“循环保护”策略，不能继续用“一刀切跳过所有图片 data URL”的方式处理。

### 2.4 能力孤岛问题

当前已经存在一些有价值能力，但被孤立在局部组件内：

- `JsonRenderer` 已经有树视图 / 代码视图双模式
- `UrlRenderer` 已经有图片 / 音视频 / 文本 URL 预览能力
- `UrlRenderer` 已经接入 `ffprobe`
- Rust 侧已经有 Base64 检测能力
- Store 已经有 URL 内容缓存和媒体元数据缓存

问题不是没有能力，而是能力没有抽象成统一预览管线。

## 3. 改造原则

### 3.1 展示优先级原则

内容展示优先级必须遵循下面的顺序：

1. 用户真正想看的内容本体
2. 能帮助理解内容的结构化信息
3. 常用操作
4. 原始字符串和底层记录信息

例如：

- 图片 URL 的第一优先级是图片本身，不是 URL 字符串
- JSON 的第一优先级是结构化树，不是原始文本
- Base64 图片的第一优先级是解码后的图片，不是编码串
- 音频 URL 的第一优先级是播放器，不是 `ffprobe` 字段

### 3.2 检测与解析分层原则

建议把能力拆为三层：

- 入库检测层
  - 负责轻量内容归类
  - 只产生稳定 subtype 和轻量 metadata
- 详情解析层
  - 在用户选中条目后按需触发
  - 负责 URL 抓取、Base64 解码、ffprobe、文件属性探测
- 展示决策层
  - 把基础 entry + 异步解析结果组合为最终视图描述

### 3.3 “最佳预览”必须是决策结果，不是组件名

不再让 `DetailView` 直接决定“渲染哪个 renderer”，而是先生成一个统一的预览描述对象，再由场景层渲染。

## 4. 目标信息架构

### 4.1 新的详情页结构

建议把右侧详情统一为 `DetailScene`：

1. `IdentityBar`
   - 条目类型
   - 核心标签
   - 来源应用
   - 时间
   - 收藏状态
   - 通用动作

2. `PrimaryPreview`
   - 占据首屏最大视觉面积
   - 只放“用户最想看”的内容

3. `InspectorPanel`
   - 结构化元数据
   - 派生信息
   - 附加动作

4. `AlternateViews`
   - 原始内容
   - 树视图 / 代码视图切换
   - 解码前 / 解码后切换
   - URL 结构 / 响应内容切换

### 4.2 页面布局建议

桌面端右侧详情建议采用自适应两种模式：

- `single-column scene`
  - 适合纯文本、颜色、邮箱、IP、时间戳等轻量内容
- `preview + inspector scene`
  - 适合图片、视频、音频、JSON、URL、Base64、文件

建议布局规则：

- 如果 `PrimaryPreview` 是媒体或结构化数据，则主预览区优先
- 如果 `InspectorPanel` 信息密度高，则桌面端采用 `minmax(0, 1fr) 320px` 双列
- 如果 `AlternateViews` 内容多，则采用 tabs 放到底部

## 5. 统一预览决策模型

### 5.1 新增 Preview Descriptor

建议在前端引入统一视图描述对象。

```ts
type PreviewKind =
  | 'plain_text'
  | 'code'
  | 'markdown'
  | 'json'
  | 'image'
  | 'audio'
  | 'video'
  | 'url_card'
  | 'file_card'
  | 'email_card'
  | 'ip_card'
  | 'color_card'
  | 'timestamp_card'
  | 'base64_text'
  | 'base64_binary'
  | 'unsupported';

interface PreviewDescriptor {
  headline: string;
  typeLabel: string;
  badges: Array<{ label: string; tone?: 'default' | 'secondary' | 'warning' }>;
  primaryKind: PreviewKind;
  primaryPayload: unknown;
  inspectorSections: Array<{
    title: string;
    items: Array<{ label: string; value: string; mono?: boolean }>;
  }>;
  alternateViews: Array<{
    key: string;
    label: string;
    kind: PreviewKind | 'raw';
    payload: unknown;
  }>;
  actions: Array<'copy_raw' | 'copy_decoded' | 'open_url' | 'open_file' | 'download' | 'paste'>;
}
```

### 5.2 新增解析结果模型

建议把异步解析结果独立出来。

```ts
interface ResolvedPreviewData {
  sourceKind: 'local' | 'remote' | 'decoded';
  mime?: string;
  fileName?: string;
  extension?: string;
  sizeBytes?: number;
  textContent?: string;
  jsonContent?: unknown;
  imageUrl?: string;
  audioUrl?: string;
  videoUrl?: string;
  media?: {
    duration?: string;
    bitrate?: string;
    codec?: string;
    width?: number;
    height?: number;
    fps?: string;
    sampleRate?: string;
  };
  base64?: {
    decodedKind: 'text' | 'json' | 'image' | 'audio' | 'video' | 'binary' | 'unknown';
    mime?: string;
    textPreview?: string;
    dataUrl?: string;
  };
}
```

### 5.3 预览决策流程

```text
ClipboardEntry
  -> 基础 subtype / metadata
  -> PreviewResolver.resolve(entry)
  -> ResolvedPreviewData
  -> buildPreviewDescriptor(entry, resolvedData)
  -> DetailScene
```

建议把这层统一命名为 `PreviewResolver`，并明确规定：

- renderer 不直接发起 URL 探测
- renderer 不直接决定是否调用 ffprobe
- renderer 不直接承担 Base64 解码

renderer 只消费 `PreviewDescriptor`，不再负责决定“最佳展示形式”。

## 6. 内容类型最佳展示矩阵

下表是本次改造的核心产物。每个类型都必须有“最佳主预览形态”。

| 内容类型                  | 识别条件                                   | 主预览                                               | 辅助信息                                 | 主要操作                           | 当前差距                               |
| ------------------------- | ------------------------------------------ | ---------------------------------------------------- | ---------------------------------------- | ---------------------------------- | -------------------------------------- |
| 纯文本                    | `plain_text`                               | 可读文本视图，短文本优先大号排版，长文本用只读编辑器 | 字符数、来源、复制次数                   | 复制、粘贴                         | 当前一律走 Monaco，短文本过重          |
| 代码                      | `code`                                     | 只读代码编辑器                                       | 语言、行数、字符数                       | 复制、粘贴                         | 可保留现有能力，但应归入统一场景       |
| 命令                      | `command`                                  | 命令块视图，优先单行/多行 shell block                | shell 类型、长度、可执行风险提示         | 复制、粘贴                         | 当前和代码共用编辑器，没有命令语义增强 |
| Markdown                  | `markdown`                                 | 双视图：渲染视图 + 原始源码视图                      | 标题数、链接数、字符数                   | 复制、切换视图                     | 当前只当文本处理                       |
| JSON                      | `json`                                     | 树视图优先，代码视图作为备用                         | 顶层键、数组长度、结构摘要               | 复制格式化 JSON、切换视图          | 现有较好，但还未纳入统一 descriptor    |
| URL（普通网页）           | `url` 且未识别成媒体/结构化内容            | 页面摘要卡或 URL 结构卡                              | 协议、host、path、query、响应类型        | 打开链接、复制链接                 | 当前偏“URL 卡”，没有目标内容优先策略   |
| URL -> JSON / 文本 / 代码 | `url` + 远程响应为文本类                   | 响应内容预览；JSON 用树视图；代码用代码视图          | 状态码、响应类型、大小                   | 复制原 URL、复制响应、打开链接     | 当前只靠扩展名猜测，未消费响应头       |
| URL -> 图片               | `url` + 远程资源为图片                     | 图片本体                                             | 尺寸、格式、大小                         | 打开链接、复制 URL、下载           | 当前可预览，但只是 URL renderer 的一支 |
| URL -> 音频               | `url` + 远程资源为音频                     | 音频播放器                                           | 时长、码率、编码、采样率                 | 播放、复制 URL、打开链接           | 当前已有基础，但缺统一媒体场景         |
| URL -> 视频               | `url` + 远程资源为视频                     | 视频播放器                                           | 时长、分辨率、码率、编码、FPS            | 播放、复制 URL、打开链接           | 当前已有基础，但缺统一媒体场景         |
| Email                     | `email`                                    | 邮箱卡片                                             | 用户名、域名                             | 复制、`mailto:`                    | 可保留，但应接入统一 inspector         |
| IP                        | `ip_address`                               | IP 卡片                                              | IPv4/IPv6、分段结果                      | 复制、外部查询                     | 可保留，但应接入统一 inspector         |
| 颜色                      | `color`                                    | 大色块 + 多格式色值                                  | HEX/RGB/RGBA/HSL                         | 复制各种格式                       | 当前较合理，可直接迁移                 |
| 时间戳                    | `timestamp`                                | 可读时间卡                                           | 标准格式、ISO、相对时间、Unix 秒/毫秒    | 复制各格式                         | 当前较合理，但仍是独立卡片             |
| Base64 -> JSON            | `base64` + 解码后是 JSON                   | 树视图优先                                           | MIME、大小、解码后字符数                 | 复制原始、复制解码后、切视图       | 当前完全缺失                           |
| Base64 -> 文本 / 代码     | `base64` + 解码后是文本类                  | 文本或代码视图                                       | 解码类型、字符数、语言猜测               | 复制原始、复制解码后               | 当前完全缺失                           |
| Base64 -> 图片            | `base64` + 解码后是图片                    | 图片本体                                             | 格式、尺寸、大小                         | 复制原始、复制 data URL、下载      | 当前完全缺失                           |
| Base64 -> 音频 / 视频     | `base64` + 解码后是媒体                    | 播放器                                               | 时长、编码、码率                         | 播放、复制原始、下载               | 当前完全缺失                           |
| Base64 -> 二进制文件      | `base64` + 解码后是二进制                  | 文件摘要卡                                           | MIME、大小、magic bytes、文件扩展建议    | 复制原始、导出                     | 当前完全缺失                           |
| 本地图片                  | `content_type=image`                       | 大图预览                                             | 宽高、大小、格式                         | 打开系统、复制、下载/转换          | 当前能力较强，但结构未统一             |
| 本地文件                  | 未来新增 `content_type=file` 或 `file_ref` | 文件摘要卡；可预览时进一步升级为对应预览             | 名称、大小、扩展名、MIME、路径、修改时间 | 系统打开、显示原路径、快速复制路径 | 当前前端有假设，后端没有真正生产       |
| 未知内容                  | fallback                                   | 原始内容视图                                         | 原始元信息                               | 复制、粘贴                         | 作为兜底保留                           |

## 7. URL 内容的最佳展示策略

### 7.1 关键判断

用户复制 URL 时，目标不是“看见 URL 本身”，而是“看见 URL 指向的东西”。

因此 URL 必须分两层：

- 第一层：URL 结构
- 第二层：URL 指向的目标内容

主预览应该优先由第二层决定。

### 7.2 现有问题

当前 `UrlRenderer` 主要根据 URL 后缀猜类型，再决定是图片、音视频还是文本抓取。这有三个问题：

- 只靠后缀猜测，不可靠
- `fetch_url_content` 只返回字符串，不返回响应头
- `extract_media_metadata` 只在 URL renderer 内部消费

### 7.3 推荐方案

新增统一的后端命令：

```rust
resolve_url_preview(url) -> UrlPreviewResolution
```

返回内容建议包含：

- final_url
- status
- content_type
- content_length
- preview_kind
- text_content
- json_preview
- media_metadata
- title

推荐流程：

1. 先做 URL 归一化
2. 发起轻量请求获取响应头
3. 根据 `content-type` 决定预览路径
4. 文本类再按需 GET body
5. 媒体类再按需调用 ffprobe

优先判断顺序：

1. `image/*`
2. `video/*`
3. `audio/*`
4. `application/json`
5. `text/markdown`
6. `text/*`
7. `application/pdf`
8. 其他二进制文件

## 8. Base64 内容的最佳展示策略

### 8.1 关键判断

用户复制 Base64 时，目标不是“看编码”，而是“看编码里面是什么”。

因此 Base64 的主预览必须是“解码后的内容”，原始编码串只能放到备用视图。

### 8.2 当前问题

Rust 侧已经能检测 Base64，并产出：

- `estimated_original_size`
- `encoded_size`
- `content_hint`
- `encoding_efficiency`

但前端没有：

- `base64` subtype
- Base64 renderer
- 解码命令
- 解码缓存
- 解码后的最佳视图决策

另外需要注意：

- 当前监听层会跳过图片型 Base64 data URL

所以 Base64 能力补齐不能只改详情页，还要同步修正采集层的过滤策略。

### 8.3 推荐方案

新增后端命令：

```rust
decode_base64_preview(input) -> Base64PreviewResolution
```

建议返回：

- decoded_kind
- mime
- size_bytes
- text_content
- json_content
- data_url
- filename_suggestion

推荐解码策略：

- 只在用户选中条目后解码
- 设最大解码上限，避免大体积卡顿
- 文本类返回 UTF-8 预览
- JSON 直接返回结构化对象
- 图片 / 音视频返回 data URL 或临时文件 URL
- 二进制文件返回文件摘要，不直接内联展示十六进制大块内容

Base64 的展示规则：

- 解码后是 JSON -> JSON 视图
- 解码后是文本 / 代码 -> 文本 / 代码视图
- 解码后是图片 -> 图片视图
- 解码后是音视频 -> 播放器
- 解码后是二进制 -> 文件摘要卡

## 9. 图片、音频、视频的统一媒体方案

### 9.1 当前问题

当前媒体能力分散：

- 本地图片走 `ImagePreview`
- URL 图片 / 音视频走 `UrlRenderer`
- ffprobe 只在 URL renderer 中使用

这意味着媒体展示不是一套统一方案，而是多套 if/else 的拼接。

### 9.2 推荐方案

引入统一媒体场景：

- `MediaScene`
  - 支持 image / audio / video
  - 主预览区渲染媒体本体
  - inspector 展示格式、宽高、时长、码率、编码、采样率等
  - action 区展示打开、复制、下载、转码等

数据来源统一成：

- 本地图片 / 本地文件
- 远程 URL
- Base64 解码结果

建议新增统一探测命令：

```rust
inspect_media_source(source) -> MediaInspection
```

其中 `source` 可以是：

- 本地绝对路径
- 相对图片路径
- 远程 URL
- 临时解码文件路径

这样 `ffprobe` 不再只服务 URL，而成为整个媒体体系的底层能力。

## 10. 本地文件方案

### 10.1 当前现状

前端已经有 `file` 的展示和筛选分支，但 Rust 侧 `ContentType` 只有：

- `Text`
- `Image`
- `Unknown`

这意味着真正的文件型条目还没有完整打通。

### 10.2 推荐方案

如果产品要支持本地文件剪贴板项，应新增：

- Rust `ContentType::File`
- 文件采集逻辑
- 文件 metadata
- 文件探测命令

推荐 metadata：

```json
{
  "file_metadata": {
    "name": "report.pdf",
    "extension": "pdf",
    "mime": "application/pdf",
    "size_bytes": 123456,
    "modified_at": 1710000000000,
    "is_directory": false
  }
}
```

本地文件的最佳展示原则：

- 可预览就升级为内容预览
  - 图片 -> 图片视图
  - 音视频 -> 媒体视图
  - JSON / 文本 / Markdown -> 结构化文本视图
  - PDF -> 文档第一页缩略预览或文件摘要
- 不可预览则退化为高信息量文件卡

## 11. 前后端改造方案

### 11.1 类型定义改造

前端 `src/types/clipboard.ts` 建议补齐：

- `ContentSubType` 新增 `base64`
- `ContentMetadata` 新增：
  - `base64_metadata`
  - `file_metadata`
  - `resolved_preview_summary`

同时把目前只存在于组件里的临时类型提升到共享类型层。

### 11.2 Store 改造

当前 store 有：

- `urlContentCache`
- `mediaMetadataCache`

建议升级为统一缓存：

```ts
previewResolutionCache: Map<
  string,
  {
    data: ResolvedPreviewData;
    updatedAt: number;
  }
>;
```

然后新增：

- `resolveEntryPreview(entry)`
- `invalidatePreview(entryId | contentHash)`
- `prefetchEntryPreview(entry)` 可选

建议这部分优先基于已经接入的 React Query 实现，而不是继续在 renderer 里堆 `useEffect + isActive + local state`。原因很简单：

- URL 探测、文本抓取、ffprobe、Base64 解码都属于可缓存的异步资源
- 它们天然适合按 `entry.id` 或 `content_hash` 做 query key
- 这样可以统一 loading / stale / error / retry / cache TTL 语义

### 11.3 后端命令改造

建议新增或替换以下命令：

- `resolve_url_preview(url)`
- `decode_base64_preview(input)`
- `inspect_file(path)`
- `inspect_media_source(source)`

建议逐步废弃“只返回单一片段”的模式：

- `fetch_url_content`
- `extract_media_metadata`

不是立即删除，而是让它们逐步被统一命令包裹。

### 11.4 组件改造

建议组件层改成：

- `DetailView`
  - 只负责拿 entry、拿 resolved data、构建 descriptor、渲染 scene
- `PreviewResolver`
  - 只负责决定 URL / Base64 / 媒体 / 文件的解析路径
- `DetailScene`
  - 统一骨架
- `PrimaryPreviewRenderer`
  - 根据 `primaryKind` 渲染主视图
- `InspectorPanel`
  - 统一渲染结构化信息块
- `AlternateViews`
  - 统一渲染 tabs

原有 renderer 保留，但身份改变：

- `JsonRenderer` -> `AlternateViews` 的一种
- `UnifiedTextRenderer` -> 文本类基础 renderer
- `ImagePreview` -> `MediaScene` 的 image 分支
- `UrlRenderer` -> 拆成：
  - URL 解析逻辑
  - URL 卡片视图
  - 媒体 / 文本 / JSON 目标视图

## 12. 推荐实施顺序

### Phase 1：统一类型与 descriptor

目标：

- 前端补齐 `base64`
- 补齐共享类型
- 引入 `PreviewDescriptor`
- 引入 `ResolvedPreviewData`

产出：

- 不改 UI 外观，也能先把结构统一

### Phase 2：重构 DetailView 为统一场景层

目标：

- 新增 `DetailScene`
- 拆掉 `DetailView` 内部的大型 `switch`
- 统一 header / preview / inspector / alternate views

产出：

- 右侧布局从“renderer 页面”变成“统一详情页”

### Phase 3：打通 URL 与 Base64 解析

目标：

- 上线 `resolve_url_preview`
- 上线 `decode_base64_preview`
- 接入统一缓存

产出：

- URL 与 Base64 都变成“看内容本体”而不是“看字符串”

### Phase 4：统一媒体与文件能力

目标：

- 抽出 `MediaScene`
- 统一 ffprobe 接入
- 准备本地文件预览体系

产出：

- 图片 / 音频 / 视频 / 文件具备统一产品语言

### Phase 5：列表与详情共享摘要模型

目标：

- 抽出共享 preview summary builder
- 让列表摘要和详情标题来自同一语义层

产出：

- 左右区域视觉与语义统一

## 13. 测试与验证建议

### 13.1 单元测试

重点补：

- `buildPreviewDescriptor`
- `resolveEntryPreview`
- URL 响应类型决策
- Base64 解码类型决策

### 13.2 组件测试

新增场景：

- JSON 条目默认进入树视图
- 图片 URL 默认进入图片主预览
- 视频 URL 默认进入播放器主预览
- Base64 JSON 默认进入结构化视图
- Base64 图片默认进入图片视图
- 普通 URL 在无法解析目标内容时退回 URL 结构卡

### 13.3 集成测试

建议准备样例数据：

- 纯文本
- 多语言代码
- Markdown 文档
- JSON
- 图片 URL
- 视频 URL
- 音频 URL
- JSON API URL
- Base64 文本
- Base64 JSON
- Base64 PNG
- Base64 二进制

## 14. 关键代码现状参考

以下文件是本次改造的核心依据：

- `src/components/DetailView/DetailView.tsx`
  - 当前详情页入口，职责过重
- `src/components/DetailView/ContentRenderers/UrlRenderer.tsx`
  - 当前最复杂的局部能力中心，包含 URL 归一化、媒体探测、文本抓取
- `src/components/DetailView/ContentRenderers/JsonRenderer.tsx`
  - 当前最接近“最佳展示”的 renderer
- `src/components/DetailView/ImagePreview.tsx`
  - 当前本地图片预览能力承载点
- `src/components/ClipboardList/ClipboardItem.tsx`
  - 已存在的列表摘要模型，可作为共享 view model 的起点
- `src/stores/clipboardStore.ts`
  - 现有 URL / 媒体缓存入口
- `src/types/clipboard.ts`
  - 前端类型系统，当前缺失 `base64`
- `src-tauri/src/clipboard/content_detector.rs`
  - Rust 侧 subtype 检测入口，已经支持 `base64`
- `src-tauri/src/commands.rs`
  - URL 抓取、ffprobe 能力所在
- `src-tauri/src/models/mod.rs`
  - 当前 `ContentType` 仍未正式支持 `file`

## 15. 结论

这次改造不应该继续沿着“再补几个 renderer”推进，而应该直接建立统一的内容预览决策层。

如果只做局部修补，会继续出现以下问题：

- 新类型越来越多，`DetailView` 的 `switch` 越来越大
- URL、Base64、图片、文件继续各走各路
- 列表和详情继续维护两套摘要逻辑

最合理的方向是：

- 让剪贴板条目先变成可解析的统一预览对象
- 再让详情页根据这个对象渲染最佳场景

这样用户复制 URL、JSON、图片、音频、视频、Base64、文件时，右侧都能稳定地优先展示“真正想看的内容”。
