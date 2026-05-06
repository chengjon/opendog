# OpenDog 项目全景解读与落地规划

> 与当前项目状态对齐版本  
> 对齐时间：2026-04-28  
> 主要参考：`.planning/PROJECT.md`、`.planning/STATE.md`、`FUNCTION_TREE.md`、`docs/capability-index.md`

## 1. 项目一句话定义

OpenDog 不是单个仓库内部的业务工具，而是一个运行在项目之上的、多项目复用的 AI 观察与决策支持系统。

它持续采集 AI 实际访问过哪些文件、哪些文件长期未被触达、当前验证证据是否充分、仓库状态是否稳定，然后通过 CLI、MCP 和 daemon 统一对外提供结构化建议，帮助 AI 和用户决定下一步该观察、验证、审查还是清理什么。

## 2. 设计理念与方向

### 2.1 核心设计理念

1. **观察先于结论**

   OpenDog 的基本立场不是“直接判断哪个文件该删、哪个模块该改”，而是先采集足够的 snapshot、activity、verification、repo-state 证据，再把这些证据组织成 AI 可消费的决策信息。

2. **多项目复用优先**

   它要解决的是“多个项目同时被多个 AI 工具协作修改时，缺少统一观察层和决策层”的问题。因此它从一开始就是 per-project isolation + workspace-level aggregation 的设计，而不是单仓库临时脚本。

3. **能力优先，不是接口优先**

   CLI、MCP、daemon 都只是交付面，不是功能本体。真正稳定的东西应该是业务能力本身，例如：

   - 是否具备可用 observation
   - 是否具备可信 verification evidence
   - 是否存在 repo risk
   - 哪个项目当前最值得优先处理

4. **决策支持重于控制**

   `git`、tests、lint、build 仍然是外部真理源；OpenDog 输出属于决策辅助证据，遇到需要确认的场景应切换到 shell 或项目原生验证。它也不自动删除文件，定位始终是提供“可解释、可组合、可审计”的决策支持，而不是直接操作目标仓库。

5. **边界必须显式表达**

   项目明确区分：

   - 直接观察到的事实
   - 基于事实推导出的建议
   - 必须切换到 shell 或项目原生验证才能确认的事项

   这也是 Phase 6 持续强化 `constraints_boundaries`、freshness、coverage、attention、risk findings 的原因。

### 2.2 当前产品方向

项目已经从“文件监控后端”演进为“AI 工作流基础设施”，当前主要方向是把以下信息沉淀成跨项目复用能力：

- 工作区观察状态
- 仓库状态与风险摘要
- AI 下一步执行策略
- 验证证据与安全门禁
- 多项目优先级排序
- 清理与重构候选
- 项目类型与工具链建议
- 约束、盲区与权威边界
- MOCK / 硬编码伪业务数据审查

## 3. 架构总览

## 3.1 基础运行模型

当前架构在运行时可以理解为三层，在项目治理上额外叠加一层治理覆盖层：

1. **Observation Layer**

   - snapshot 基线扫描
   - `/proc/<pid>/fd` 周期扫描
   - `notify`/inotify 文件变更监听
   - file stats、usage evidence、verification evidence、retained evidence 存储

2. **Core Intelligence Layer**

   - unused / hotspot / trend / compare / export
   - observation freshness / evidence coverage
   - repo risk summary
   - verification reasoning
   - attention scoring
   - data-risk detection

3. **Delivery Layer**

   - CLI operator surface
   - MCP AI surface
   - daemon runtime
   - local control plane

4. **Governance Overlay**

   - `FUNCTION_TREE.md`
   - `REQUIREMENTS.md`
   - `ROADMAP.md`
   - task cards
   - planning validators

这里需要特别说明：

- 对运行时系统本身，最重要的是“采集层 -> 智能计算层 -> 交付层”
- `Governance Overlay` 不是线上运行链路的一部分，而是为了控制项目演进复杂度的规划与治理附加层

这样理解更贴近当前实际，也更容易避免把 OpenDog 误解成一个“治理框架先于产品本体”的项目。

## 3.2 关键技术取向

- **语言**：Rust
- **运行环境**：WSL2 / Linux
- **数据库**：per-project SQLite
- **MCP 协议实现**：rmcp stdio
- **后台运行**：daemon + systemd integration
- **状态一致性策略**：CLI/MCP 尽量走 local control plane 复用 daemon-owned state

## 3.3 核心约束

- 只支持 WSL / Linux 能力边界内的观察模型
- 不做自动清理源文件
- `git`、tests、lint、build 仍然是外部真理源；OpenDog 输出属于决策辅助证据，遇到需要确认的场景应切换到 shell 或项目原生验证
- repo risk、mock detection、attention scoring 都属于 advisory output，而不是 authority

## 4. FUNCTION_TREE 的规划与意义

## 4.1 FUNCTION_TREE 的角色

`FUNCTION_TREE.md` 现在是项目的**业务能力主索引**，而不是普通功能清单。

它位于三类文档之间：

- 上接 `PROJECT.md`：定义产品意图
- 中接 `REQUIREMENTS.md`：定义详细需求
- 下接 `ROADMAP.md`、task cards：定义具体落地动作

它解决的是“项目继续长大后，能力归属、变更影响和治理边界如何保持稳定”的问题。

## 4.2 层级模型

当前采用 3 层结构：

- `L1`：领域能力
- `L2`：模块能力
- `L3`：原子能力

治理规则是：

- requirement 最终应映射到至少一个 `L3`
- roadmap phase / task card 必须声明它变更了哪些 `L3`
- 讨论能力时优先用 capability 名称，不用 CLI/MCP 命令名代替

## 4.3 当前 FUNCTION_TREE 结构

当前共有：

- 3 个 `L1` 领域能力
- 26 个 `L3` 叶子能力

### FT-01 Observation and Intelligence Capture

这一层负责“采集和沉淀证据”，包括：

- 项目注册与隔离
- snapshot 基线管理
- runtime monitoring 与 attribution
- usage analytics
- export / reporting
- retained evidence lifecycle

它对应的是 OpenDog 的数据底座和观察底座。

### FT-02 Service Delivery, Runtime and Coordination

这一层负责“如何稳定交付能力”，包括：

- CLI operator workflow
- MCP AI workflow
- daemon lifecycle
- local control plane coordination

它对应的是 OpenDog 的交付与运行底座。

### FT-03 AI Decision Support and Governance

这一层负责“如何把已有观察能力转成 AI 真能消费的决策层”，包括：

- `FT-03.01` Workspace Observation
- `FT-03.02` Repository Risk and Execution Strategy
- `FT-03.03` Verification Evidence
- `FT-03.04` Multi-Project Portfolio Prioritization
- `FT-03.05` Cleanup and Refactor Review
- `FT-03.06` Project Type and Toolchain Guidance
- `FT-03.07` Constraints and Boundaries
- `FT-03.08` Mock and Hardcoded Data Review

这也是当前项目最主要的演进方向。

## 4.4 FUNCTION_TREE 的治理目标

FUNCTION_TREE 不是为了画树而画树，而是为了形成以下约束：

- 不允许 orphan requirements
- 不允许 orphan task cards
- 不允许无人负责的叶子能力
- 不允许接口层覆盖能力层

这意味着未来项目继续迭代时，新增工作应该先回答：

1. 这是哪个能力的增强？
2. 它属于哪个 `FT-*` 叶子？
3. requirement、task card、code、doc 是否对齐？

## 5. 当前已落地的能力版图

## 5.1 已完成的基础阶段

### Phase 1

- 项目注册、隔离、snapshot 基线

### Phase 2

- `/proc` 扫描 + inotify/notify 组合观察模型
- 近似 attribution 证据链

### Phase 3

- per-file usage evidence
- unused / hotspot / summary / trends

### Phase 4

- MCP server
- CLI operator surface

### Phase 5

- daemon
- systemd integration
- local control plane

这些阶段已经完成，说明项目的“采集、存储、交付、运行”底盘是成立的。

## 5.2 当前已交付的 Phase 6 核心成果

在 AI decision-support 层，当前已经落地的关键能力包括：

- observation freshness 与 evidence coverage
- workspace portfolio attention scoring
- structured repository risk findings
- verification evidence recording / querying / execution
- mock / hardcoded pseudo-data detection
- retained-evidence cleanup and storage maintenance
- data-risk project/workspace views
- decision-brief / agent-guidance 统一入口

换句话说，Phase 6 已经不是“从零开始设计”，而是**core shipped, hardening active**。

## 6. 当前所在位置

根据当前 `STATE.md`，项目位置是：

- 当前 milestone：`v1.0`
- 当前 phase：`Phase 6`
- phase 状态：`In Progress`
- 总体状态：`v1 complete; Phase 6 core capabilities shipped and now in refinement/hardening`

当前关键指标：

- `114` 条 requirements，全部已 phase-mapped
- `3` 个 L1 capability domains
- `26` 个 L3 leaf capabilities
- `9` 张 task cards，全部完成
- `106` 个 unit tests
- `22` 个 integration tests

这说明项目已经越过“是否能工作”的阶段，进入“是否足够稳定、可解释、可治理”的阶段。

## 7. 当前设计评审：是否漂移、是否开口过大、是否过度设计

这一节用于回答一个更现实的问题：站在当前 v1.0 与 Phase 6 hardening 的位置回看，OpenDog 的设计是否已经开始偏离初衷，或者出现了范围过大、抽象过重的问题。

### 7.1 是否发生了项目方向漂移

结论是：**没有漂移**。

原因很直接：

- 原始目标就是“多项目 AI 文件行为观察 + MCP 服务 + AI 配套辅助能力”
- 当前核心底座仍然是 snapshot、monitoring、usage evidence、project isolation
- 当前主要交付面仍然是 MCP，CLI 和 daemon 是支撑面
- 当前新增能力虽然更深，但都仍然围绕“让 AI 基于文件与仓库状态做更好的下一步决策”

也就是说，项目不是从“文件观察工具”拐向别的赛道，而是从浅观察升级到了决策支持层。

### 7.2 是否开口过大

结论是：**开口偏大，但合理且可控**。

它看起来开口大的原因包括：

- 一开始就按多项目工作区设计
- 不只做文件监控，还做 verification、risk、strategy、portfolio、mock-data
- 同时覆盖运行时、交付面、治理面

但它并不是无边界扩张，因为当前仍然有很强的收口约束：

- 只建议，不自动改 repo
- `git`、tests、lint、build 仍然是外部真理源
- 明确 direct observation 与 inference 边界
- 运行环境明确限制在 WSL/Linux
- 所有能力都挂回 `FUNCTION_TREE`

所以它属于“战略型扩界”，不是“功能堆叠型失控”。

### 7.3 是否存在过度设计

结论是：**存在局部轻度过度设计，但还没有伤到主架构**。

主要体现在三个方面：

1. **治理体系超前于当前体量**

   现在已经有：

   - 114 requirements 全映射
   - 三级 capability tree
   - task card 绑定
   - validators 与 planning governance

   对长期项目这是优点，但对当前体量来说，部分治理成本已经略高于即时收益。

2. **AI 决策层结构化抽象偏重**

   freshness、coverage、attention、risk findings、verification evidence、selection reasons、execution templates 这些结构对长期非常有价值，但对“只想知道哪些文件闲置、哪些文件高频”的轻量用户来说，默认理解成本偏高。

3. **部分增值能力落地较早**

   例如：

   - mock / hardcoded data review
   - retained-evidence lifecycle
   - workspace-level attention / risk layering

   这些不是错方向，但确实属于“比最小可行能力集更超前一些”的投入。

整体判断是：**项目存在轻度超前设计，但属于可接受、可回收、可降配的超前，而不是架构性错误**。

## 8. 哪些设计必须保留

有几类设计虽然看上去“重”，但实际上是这个项目的核心壁垒，不能为了轻量化而砍掉：

### 8.1 多项目隔离 + per-project SQLite

这是 OpenDog 从一开始就区别于单仓库脚本的基础。

### 8.2 `/proc` fd 扫描 + inotify/notify 双观测模型

这是“AI 实际访问”和“文件实际变化”两条证据链的核心组合，也是当前观察层最关键的技术基础。

### 8.3 MCP 作为核心交付面

OpenDog 的独特价值就是给 AI 直接提供结构化决策支持，这一点不能退化成“只有 CLI 的本地工具”。

### 8.4 daemon + local control plane

长期后台运行、复用 daemon-owned monitor state、避免 CLI/MCP 状态漂移，这些都属于基础设施级能力，不是可选糖衣。

### 8.5 “观察先于结论、建议不越权”的边界

这是项目长期可控的关键。没有这个边界，OpenDog 会很快变成一个职责混乱、风险过高的自动化工具。

## 9. 当前真正的工作重心

从当前规划看，项目现在的重点不是新增大量散功能，而是围绕 Phase 6 做结构化加固：

### 9.1 继续强化 observation quality

- readiness
- freshness
- evidence gaps
- project/workspace 层面的 observation state

### 9.2 继续强化 risk -> strategy 的耦合

- repo risk 不仅要有文本 reason
- 还要有 structured findings
- 再进一步影响 AI sequencing、selection reasons、workspace aggregation

### 9.3 继续强化 portfolio intelligence

- 跨项目 attention 排序
- review batching
- workspace-level prioritization

### 9.4 继续强化 evidence 与 boundary

- 什么是 direct observation
- 什么是 inference
- 什么情况下必须切换到 shell 或项目原生验证

### 9.5 继续降低 heuristic false positives

- mock / hardcoded data detection
- cleanup/refactor candidate ordering
- toolchain guidance confidence

## 10. 对“落地规划”的正确理解

如果从今天往后看，OpenDog 的落地规划不应该理解为“再做更多命令”，而应该理解为三条并行主线：

### 主线 A：把能力做完整

围绕 `FT-03` 的每个叶子能力，把 observation、risk、strategy、verification、portfolio、cleanup、toolchain、boundary、mock-data 逐步从“可用”打磨到“稳定可依赖”。

### 主线 B：把能力做可解释

所有重要输出都要逐步从：

- 单个布尔值
- 单条 reason 文本

升级为：

- structured findings
- attention / severity / priority / confidence
- freshness / coverage / evidence quality
- recommended next action + why + what evidence

### 主线 C：把能力做可治理

任何后续扩展都必须继续遵守：

- requirement -> FT leaf 映射
- task card -> FT leaf 映射
- code / test / docs / planning 同步更新

这决定了 OpenDog 会是一个“长期可维护的 AI 工程基础设施”，而不是一次性的功能堆叠。

## 11. 建议的收敛与优化方向

如果希望在不推翻现有设计的前提下，进一步降低“过度设计感”，当前最合理的优化方向是以下四类。

### 11.1 治理层降配使用，而不是删除

建议保留：

- `FUNCTION_TREE` 三层能力锚点
- requirement 映射能力
- task card 机制

但在日常执行上，可以把部分治理约束理解为“强推荐的工程规范”，而不是每次轻微迭代都要重成本地展开完整治理动作。

### 11.2 AI 输出做分层

当前可以把输出理解为两种消费层级：

- **基础消费层**：给日常 AI 使用的轻量结论
- **高级消费层**：给需要严格决策约束时使用的结构化 evidence / risk / attention / boundary 元数据

这不要求重写系统，只要求在后续文档和默认消费路径上更明确地区分“轻量默认”与“深度结构化”。

### 11.3 对远期能力采用“先存在、后深磨”的节奏

像 mock 审查、复杂 portfolio intelligence、细粒度 data-risk，并不需要被回退，但在资源分配上应优先让位于：

- observation readiness
- repo risk -> strategy coupling
- verification evidence quality
- boundary clarity

也就是优先深挖“AI 是否能信任这些输出”，再深磨增值分析层。

### 11.4 对外定位持续收口

对外最稳定、最清晰的项目定位仍然应该是两句话：

1. 多项目文件行为观察底座
2. 面向 AI 的 MCP 决策辅助服务

其他能力都可以被描述为增值层，而不是重新定义项目本体。

## 12. 当前对项目的总体判断

站在当前状态看，OpenDog 的路线已经明确：

它不是要成为一个庞杂的平台，而是要成为一个**跨项目 AI 工程观察与决策层**。

它最有价值的地方，不在于代替目标项目实现业务逻辑，而在于提供那些目标项目通常不会自己做、但 AI 工作流又持续需要的通用能力：

- 观察
- 风险
- 证据
- 优先级
- 边界
- 清理与治理建议

从工程演进角度看，这个项目现在最正确的方向不是横向扩面，而是继续沿 `FT-03` 向下打深，把“AI 能不能真正依赖这些输出做高质量决策”这件事做扎实。

## 13. 推荐阅读顺序

如果要继续理解或推进本项目，推荐按这个顺序阅读：

1. [README](../README.md)
2. [.planning/PROJECT.md](../.planning/PROJECT.md)
3. [FUNCTION_TREE.md](../FUNCTION_TREE.md)
4. [.planning/STATE.md](../.planning/STATE.md)
5. [docs/capability-index.md](./capability-index.md)
6. [docs/ai-playbook.md](./ai-playbook.md)
7. [docs/json-contracts.md](./json-contracts.md)
8. [docs/mcp-tool-reference.md](./mcp-tool-reference.md)

---

如需继续把这份文档扩展成“面向新贡献者的 onboarding 版”或“面向 AI 的执行指引版”，建议在此基础上拆成两份：

- 一份偏产品与架构综述
- 一份偏 AI 消费顺序与执行约束
