# OPENDOG 功能介绍

OPENDOG 是一个面向多项目工作区的工程观察与 AI 决策辅助系统。它持续记录项目文件的快照、访问、验证和风险信号，并通过 CLI、MCP、daemon 和结构化 JSON 输出，帮助人类操作者与 AI agent 判断下一步应该检查什么、验证什么、清理什么，以及什么时候必须回到项目自身的 `git`、测试、lint、build 或源码语义中确认。

它的定位不是替代版本控制、测试系统或静态分析工具，而是在这些工具之上补充一层可复用的观察证据和行动路由。对 AI 来说，OPENDOG 像一个工程工作区雷达：先给出项目状态、证据新鲜度、风险候选和推荐动作，再引导 AI 进入正确的仓库、文件和验证流程。

## 项目规模

| 指标 | 数据 |
|------|------|
| 源文件 | 213 个 Rust 源文件，约 51 000 行 |
| MCP 工具 | 26 个 AI 调用工具 + 4 个只读 resource |
| CLI 命令 | 23 个顶层入口命令 |
| JSON 合约 | 32 个版本化 MCP payload contract |
| 测试 | 1 581 个单元/集成测试，0 clippy 警告 |
| 源码测试覆盖 | 104 / 213 个源文件包含 `#[cfg(test)]` 模块（49%） |
| 提交历史 | 200+ 次 atomic commits |

## 核心价值

现代 AI 辅助开发经常面对同一类问题：项目很多，文件很多，证据分散，AI 不知道哪些文件是真正活跃的核心文件，也不知道最近一次验证是否仍然可信。OPENDOG 将这些散落在文件系统、运行时、验证命令和历史操作中的信号统一沉淀下来，让项目状态可以被查询、比较、导出和交给 AI 消费。

它重点解决以下问题：

- 哪些项目已经纳入观察，哪些项目还缺少快照或监控证据。
- 哪些文件经常被 AI 或开发流程访问，哪些文件长期没有被观察到。
- 最近的测试、lint、build 结果是否存在、是否通过、是否已经过期。
- 哪些项目或文件更值得优先检查。
- 哪些文件可能涉及 mock、hardcoded data、混合用途或清理风险。
- 在修改、重构或删除前，需要先补充哪些外部验证。
- AI 下一步应该使用 MCP、CLI、shell、测试还是人工审查。

## 功能结构

OPENDOG 当前分为三层能力。

第一层是观测内核。它负责项目注册、SQLite 持久化、全量文件快照、文件访问统计、`/proc` 文件描述符扫描、`inotify` 变更监控、未访问文件识别、时间窗口报告、快照对比、证据导出和 retained evidence 清理。它回答的是"项目里发生过什么"和"哪些文件被观察到过"。

第二层是服务入口和运行协调。OPENDOG 同时提供面向人的 CLI、面向 AI 的 MCP server、长期运行的 daemon，以及用于复用 daemon 状态的本地控制面。CLI 适合人工操作、脚本、导出和维护；MCP 适合 AI agent 获取结构化状态；daemon 让监控和项目状态跨会话保持稳定。

第三层是 AI 决策辅助。它把底层观察数据进一步组织成 workspace observation、project overview、attention score、verification gates、repo risk findings、data-risk focus、recommended next action、mandatory shell checks、external truth boundary 等字段。AI 不需要从一堆底层数字开始猜测，而是可以先读取一个经过压缩的工程态势图。

## 已实现能力

OPENDOG 已经形成完整的本地产品闭环，包括：

- 项目注册、列出、删除和配置隔离。
- 全量文件快照、快照运行历史和快照对比。
- `/proc` + `inotify` 混合监控，用于捕捉访问和变更信号。
- 文件使用统计、热点文件、冷门文件和从未访问文件列表。
- 时间窗口报告和使用趋势，用于判断最近工作集中区域。
- JSON/CSV 证据导出。
- retained evidence 清理、dry-run 和存储维护提示。
- 全局和项目级配置查看、修改和 reload。
- daemon-first 的本地控制面，支持 MCP 会话间复用运行状态。
- test/lint/build 验证结果记录、查询和执行。
- AI guidance 和 decision brief，用于给出下一步行动建议。
- workspace attention 和多项目优先级排序。
- mock、hardcoded data、mixed file、cleanup/refactor 风险候选识别。
- orphan 扫描和删除计划验证。
- governance lanes/nodes，用于能力边界和任务状态观察。
- MCP 工具面、只读 MCP resources 和版本化 JSON contract。
- OpenDog binary 自更新检查和手动重建入口。

## AI 决策辅助详解

### Guidance 系统

OPENDOG 的 AI 决策辅助通过 `get_guidance` MCP 工具暴露（`detail = "summary"` 返回工作区态势，`detail = "decision"` 返回单项目决策包）。CLI 侧对应 `opendog agent-guidance` 和 `opendog decision-brief`。Guidance 系统内部由以下模块协作：

- **workspace observation layer** — 汇总多项目的快照、活动、验证新鲜度，输出 analysis_state（empty / insufficient_activity / stale / ready）。
- **execution strategy layer** — 将 project recommendation 排序后，生成 verification summary、observation summary、repo truth summary、stabilization summary、data risk focus summary 和 risk-strategy coupling，指导 AI 选择正确的工具和操作序列。
- **constraints and boundaries layer** — 列出 direct observations、inferences、blind spots、guardrails 和 mandatory shell checks，明确告诉 AI 什么时候必须离开 OPENDOG、回到项目原生工具。
- **workspace portfolio layer** — 多项目优先级排序、attention queue、dirty / high-risk / failing-verification 项目标记。
- **storage maintenance layer** — 检测 retained evidence 中的 reclaimable bytes 和 vacuum 候选，在大型清理前提醒 AI 先处理 OPENDOG 自身的存储开销。
- **verification evidence layer** — 汇总各项目的 test/lint/build 通过状态，标记 failing runs 和 missing evidence。
- **project toolchain layer** — 识别各项目的语言和工具链（Rust/Cargo、Python/pip、Node/npm 等）。
- **governance layer** — 观察 lanes 和 nodes 的状态，用于 FT ownership 和任务映射。

### Attention 评分

每个项目根据 repo risk level、verification status、coverage state、snapshot/activity/verification freshness、hardcoded/mock candidate count 和 safe_for_cleanup/safe_for_refactor 门控，计算出一个 attention score 和对应的 attention band（low / medium / high / critical）。AI 可以直接按 attention score 排序决定先处理哪个项目。

### Decision Brief

`get_decision_brief` 返回单个项目的压缩决策包，包含 recommended next action、execution sequence、repo truth gaps、mandatory shell checks、external truth boundary、data risk focus、risk profile 和 action profile。AI 读取一个 JSON 就能获得完整的项目行动路线。

### Risk-Strategy Coupling

OPENDOG 会自动将最高优先级项目的 repo risk finding 与 workspace strategy mode 耦合。如果 top project 存在未解决的 repo risk，整个 workspace 的 strategy mode 和 preferred tool chain 会受到影响。这种耦合帮助 AI 理解为什么当前建议使用某个工具而不是另一个。

## Orphan 检测与删除计划

OPENDOG 的 orphan 检测系统采用多信号证据模型：

1. **Candidate Collector** — 扫描项目中的源文件，生成 SupportsUnused 证据（confidence 0.70）。
2. **Internal Scanners** — 包括 entrypoint scanner、docs/ownership gate、frontend literal scanner、Python import graph 等，每个 scanner 对候选文件产生 Veto（发现引用）或 SupportsUnused（未发现引用）证据。
3. **External Scanners** — 支持外部 scanner 报告接入（如 python_import_graph、fastapi_route_auditor、openapi_contract）。
4. **Classification Engine** — 汇总所有证据后，根据 veto 信号、used signal threshold、required scanner 健康状态和证据时效，将候选分类为 RemoveCandidate、ReviewRequired 或 Blocked。
5. **Deletion Plan Verification** — 在实际删除前，先通过 `verify_deletion_plan` 确认所有 target 都被分类为 RemoveCandidate、没有 blocked target、verification commands 已提供，才会输出 `status: "ready"`。

## 质量保障

OPENDOG 自身通过以下手段保障代码质量：

- **1581 个自动化测试**，覆盖纯函数逻辑、serde 序列化、SQLite 集成、分类引擎、证据匹配、配置解析、错误路径和边界条件。
- **0 clippy 警告**，所有编译器和 linter 检查均通过。
- **104 个源文件包含测试模块**，其中观测内核、存储层、MCP 辅助模块和 orphan 检测系统的测试密度最高。
- **版本化 JSON contract**（32 个），确保 CLI 和 MCP 输出的 schema 稳定可消费。
- **Per-project SQLite 隔离**，每个项目拥有独立的 `.db` 文件、配置和监控状态，WAL 模式支持并发读取。

## 对其他项目提供的信息

接入 OPENDOG 后，一个普通项目会多出一层可查询的工程上下文。

| 信息类型 | 具体内容 | 用途 |
|---|---|---|
| 项目清单 | 已注册项目、根路径、隔离边界、配置状态 | 让 AI 知道工作区里有哪些项目 |
| 文件基线 | 文件清单、大小、元数据、快照记录 | 做重构、迁移、清理前的资产盘点 |
| 文件活跃度 | 访问次数、最近访问、hot/cold、never accessed | 区分核心文件和低信号文件 |
| 文件变化 | 快照差异、时间窗口活动、趋势 | 判断最近工作集中在哪些区域 |
| 监控状态 | daemon、monitor、会话复用情况 | 避免重复启动或误判观察状态 |
| 验证证据 | test/lint/build 命令、状态、退出码、摘要、时间 | 修改前判断验证是否可信 |
| 证据新鲜度 | missing、stale、fresh snapshot 或 verification | 避免基于过期证据行动 |
| 风险候选 | mock、hardcoded data、mixed file、unused、orphan | 给审查、清理和重构提供候选 |
| 多项目优先级 | attention score、attention band、priority projects | 决定先处理哪个项目 |
| 下一步动作 | recommended next action、recommended flow、execution sequence | 给 AI 可执行的行动路线 |
| 外部真相边界 | mandatory shell checks、repo truth gaps、external truth boundary | 指明什么时候必须回到 git、测试或源码确认 |
| 治理状态 | lanes、nodes、FT ownership、任务映射 | 让能力变更有结构化归属 |
| 可移植证据 | JSON、CSV、MCP payload、只读 resources | 供其他工具稳定消费 |

这些信息让 AI 不必每次从零遍历仓库，而是先通过 OPENDOG 获取一份压缩后的项目态势，再决定是否进入源码、运行验证或执行清理流程。

## 典型使用场景

新 AI agent 接手陌生项目时，可以先通过 OPENDOG 判断项目是否已经注册、是否有最新快照、是否有验证证据、哪些文件最活跃、哪些区域风险最高。这样 agent 的第一步不再是盲目扫描整个仓库，而是沿着证据路线进入项目。

准备清理文件时，OPENDOG 可以提供 never-accessed 文件、orphan candidate、路径分类、验证门控和 data-risk 候选。它不会直接断言文件可以删除，而是给出候选和风险提示，最终仍然需要项目原生测试、git diff、源码语义或人工审查确认。

准备大规模重构时，OPENDOG 可以先暴露最近活跃区域、验证证据是否过期、是否存在 blocked gate、是否有 mock 或 hardcoded data 风险、是否需要 mandatory shell checks。重构因此可以先过证据门，再进入具体代码修改。

管理多项目工作区时，OPENDOG 可以根据缺失快照、过期验证、数据风险、attention score 和存储维护信号，帮助人或 AI 决定哪个项目应该优先处理。

构建自动化工具时，OPENDOG 的 JSON contract、MCP 工具和只读 resources 可以作为稳定输入。仪表盘、审查助手、CI 辅助脚本、迁移工具和清理工具都可以复用这些结构化信号，而不需要解析人类可读文本。

## 推荐接入方式

最小初始化流程：

```bash
opendog register --path <repo>
opendog snapshot --id <id>
opendog start --id <id>
```

AI agent 的推荐第一入口是 `get_guidance`。它可以先读取工作区或单项目 summary，再根据返回的推荐动作调用更具体的工具，例如 `get_stats`、`get_unused_files`、`get_verification_status`、`get_data_risk_candidates` 或 `get_time_window_report`。

如果 AI 需要一个更聚焦的单项目决策包，可以使用 `get_decision_brief`（MCP）或 `opendog decision-brief`（CLI），直接获得 recommended next action、execution sequence 和 risk profile。

人类操作者可以优先使用以下 CLI：

```bash
opendog agent-guidance --json
opendog decision-brief --json
opendog stats --id <id> --json
opendog verification --id <id> --json
opendog data-risk --id <id> --json
opendog report window --id <id> --json
```

当 OPENDOG 明确指出 evidence stale、repo truth gap 或 mandatory shell check 时，应切换到项目自身的 `git`、测试、lint、build 或源码检查。

## 与现有工具的关系

OPENDOG 不替代 `git`、测试、lint、build 或静态分析。

`git` 提供版本事实，测试和构建提供正确性事实，静态分析提供结构事实。OPENDOG 提供的是观察事实和决策辅助：哪些文件被访问过、哪些证据缺失或过期、哪里风险更高、下一步应该验证什么。

因此，OPENDOG 的输出应该被视为 advisory evidence。它适合帮助 AI 和人类缩小范围、排序风险、发现证据缺口和选择下一步工具，但不应该单独作为删除文件、判定业务逻辑无用或确认修改安全的最终依据。

## 使用边界

可以信任 OPENDOG 做：

- 工作区和项目观察。
- 文件访问证据汇总。
- 快照、趋势、统计和未访问候选。
- 验证证据记录和新鲜度判断。
- mock、hardcoded data、mixed file 风险候选识别。
- 多项目优先级排序。
- 下一步行动建议。
- 非破坏性清理和重构辅助。

不应该让 OPENDOG 单独决定：

- 文件一定可以删除。
- 业务逻辑一定无用。
- 测试通过就一定安全。
- mock 或 hardcoded data 一定是问题。
- unused 文件一定是 dead code。
- OPENDOG 观察状态等同于 git 或源码的最终真实状态。

## 一句话总结

OPENDOG 是其他项目的 AI 工程观察仪表盘和行动路由器。它把多项目环境中的文件访问、快照、验证、风险和治理信号转成可复用的结构化证据，帮助 AI 和人类更快、更稳地决定下一步该看哪里、验证什么、清理什么，以及什么时候必须回到项目原生工具确认真相。
