# Dance

## What This Is

Dance 是一个面向开发者的本地桌面剪贴板管理工具，当前以客户端能力为中心，负责稳定监听剪贴板、持久化历史记录、识别复制内容的类型，并为不同内容提供合适的详情预览。它不是通用型云端协作产品，而是一个帮助开发者更高效查看、理解、筛选和回用剪贴板内容的工作台。

## Core Value

开发者复制任意常见内容后，应用都能稳定记录、准确识别，并以最合适的结构化方式展示出来。

## Requirements

### Validated

- ✓ 稳定记录本地剪贴板历史并支持启动/停止监听控制 — existing
- ✓ 支持文本、图片、文件路径等多种剪贴板内容类型的基础采集与存储 — existing
- ✓ 支持按关键字、内容类型、来源应用等维度浏览和筛选历史记录 — existing
- ✓ 支持收藏、删除、清空历史和统计查看等基础管理能力 — existing
- ✓ 支持 JSON、URL、代码、颜色等部分开发常见内容的详情预览与解析展示 — existing
- ✓ 支持本地配置管理、自动更新、日志查看及桌面端偏好设置 — existing

### Active

- [ ] 提升各类开发常见内容的类型识别准确率，减少误判、漏判和错误子类型归类
- [ ] 提升不同内容类型的预览质量，让 JSON、URL、颜色及其他已支持格式都能优先以最优视图展示
- [ ] 强化搜索、筛选和模糊匹配能力，帮助用户快速定位目标剪贴板内容
- [ ] 优先修复影响稳定性和可靠性的监听、存储与预览链路问题，保证日常开发使用可依赖

### Out of Scope

- 云同步 — 当前只考虑本地客户端体验，避免引入账户、服务端和同步复杂度
- 多设备同步 — 不属于本轮目标，优先把单机使用体验做稳定
- 移动端 — 当前产品边界是桌面客户端，不扩展到手机和平板端
- 团队协作 — 当前主要服务个人开发者工作流，不做共享协作能力
- 分享能力 — 不是当前核心价值的一部分，避免偏离“查看、理解、检索剪贴板内容”的主线

## Context

当前仓库已经具备明确的 brownfield 基础：主应用是基于 Tauri 2、React 18、TypeScript、Rust 和 SQLite 的桌面客户端，现有能力覆盖剪贴板监听、历史记录、类型过滤、详情预览、偏好设置、统计和自动更新。代码库中已经存在 URL、JSON、颜色、代码等开发相关内容的识别和展示链路，因此本项目并不是从零构建“剪贴板管理器”，而是在已有基础上把“开发者内容理解”做深。

从代码库映射结果看，当前最大的实现风险不在 UI 壳层，而在稳定性和一致性层面，包括监听生命周期控制、存储路径不统一、若干命令与预览链路过于集中、部分检索能力仍停留在基础匹配、以及若干测试和质量门禁缺口。这与本轮目标一致：优先把开发者日常会复制的内容识别准确、展示清楚、检索可靠，而不是扩展到云同步或多端体系。

用户最看重的使用场景是开发过程中的高频复制内容查看与理解。当前项目中已经覆盖的内容类型，就是用户最常处理的类型集合。对用户来说，“最优格式展示”不是表面 UI，而是复制后立即得到可读、可判断、可再次利用的结构化视图，例如 JSON 自动格式化并在代码编辑器中查看，URL 根据类型展示合理内容，颜色直接看到色块与其他色值表示，其他格式也能按语义化方式预览。

## Constraints

- **Platform**: 仅考虑桌面客户端能力 — 当前目标明确排除云端、同步和移动端扩展
- **Primary Audience**: 面向开发者 — 功能优先围绕开发工作流中的内容识别、预览和检索
- **Existing Stack**: 基于当前 Tauri + React + Rust + SQLite 架构演进 — 避免脱离现有代码基础重做产品
- **Reliability**: 监听、存储、预览和检索链路必须可靠 — 这是用户持续使用该产品的前提
- **Scope Control**: 不扩展到团队协作、分享和多设备体系 — 防止主线目标被平台化诉求稀释

## Key Decisions

| Decision                                                     | Rationale                                                                 | Outcome   |
| ------------------------------------------------------------ | ------------------------------------------------------------------------- | --------- |
| 继续把项目定位为开发者本地剪贴板工具，而不是通用内容平台     | 用户的核心诉求是开发过程中的内容理解与回用，不是泛社交或协作场景          | — Pending |
| 本轮优先级聚焦在识别准确率、预览质量与搜索检索能力           | 这三项直接决定产品是否真正提升开发效率                                    | — Pending |
| 仅围绕客户端演进，不把云同步、多设备、移动端纳入当前主目标   | 先把单机体验打磨稳定，比提前扩展系统边界更有价值                          | — Pending |
| 现有已实现能力视为 Validated，后续迭代在此基础上持续收敛质量 | 仓库已经具备可运行产品形态，初始化应反映真实代码状态而不是假设 greenfield | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):

1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---

_Last updated: 2026-03-27 after initialization_
