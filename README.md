OPENDOG 项目概览（当前实现 + 历史方案导航）

> 说明：本文档最初记录的是原始设计方案。当前仓库已经不再处于“纯方案阶段”，而是有可运行实现。
> 现状以仓库代码与 `.planning/` 为准：OPENDOG 已具备观测内核、服务交付与运行协调层、AI 决策辅助层三大能力面，并通过 daemon、CLI、MCP 三种工作流入口对外提供能力。
> 当前设计判断：项目没有方向漂移；能力面相对宽，但边界清晰且受控；当前重点是继续打磨已交付的 `FT-03` 决策辅助能力，而不是继续横向开新口。

## 阅读导航

- 想先用一页看懂项目当前定位：看 [docs/positioning.md](/opt/claude/opendog/docs/positioning.md)
- 想快速了解当前实现：看 [当前实现概览](#当前实现概览)
- 想按能力/入口快速查表：看 [docs/capability-index.md](/opt/claude/opendog/docs/capability-index.md)
- 想理解当前能力分层：看 [当前能力结构](#当前能力结构)
- 想直接调用工具：看 [当前 MCP 工具](#当前-mcp-工具) 与 [当前 CLI 命令](#当前-cli-命令)
- 想知道 AI 建议的使用顺序：看 [推荐的 AI 使用顺序](#推荐的-ai-使用顺序) 与 [Quick Start for AI](#quick-start-for-ai)
- 想追溯最初设想和早期方案：看 [docs/historical-original-plan.md](/opt/claude/opendog/docs/historical-original-plan.md)

## 当前实现概览

- 已实现多项目隔离、SQLite 持久化、全量快照、`/proc` + `inotify` 混合监控、统计分析
- 已实现本地控制面，CLI / MCP 优先通过本地控制通道复用 daemon 持有的项目操作状态
- 已实现 MCP stdio 服务，当前不仅提供基础控制工具，也提供 AI 决策辅助工具
- 已实现验证结果记录与执行：测试、lint、build 可以作为 evidence 持久化
- 已实现 MOCK / hardcoded pseudo-data 检测，并可在项目级与 workspace 级汇总
- 已实现留存证据生命周期与存储维护信号：`cleanup-data`、`cleanup_project_data`、`agent-guidance` / `decision-brief` 会暴露存储维护 / `VACUUM` 候选
- 已建立 `.planning/FUNCTION_TREE.md` + `.planning/task-cards/` 的能力治理入口，后续任务卡需要先声明 `FT-*` 影响再执行
- 已为 `.planning/REQUIREMENTS.md` 建立逐段 `Maps to FT:` 映射与校验入口，避免 requirement 漂移或失去能力归属
- 已提供统一治理入口 `python3 scripts/validate_planning_governance.py`，可一次性检查 task card、requirement 映射、函数树覆盖、roadmap 统计一致性，以及关键源码/文档的结构性大小门禁
- 已交付 `CONF` 配置管理能力：支持全局默认、项目覆写、CLI/MCP 查询与修改、daemon 运行中安全 reload
- 已交付 `EXPORT` 可移植导出能力：项目统计证据可导出为稳定 JSON/CSV 工件
- 已交付 `RPT` 比较性报告能力：支持时间窗统计、快照对比、使用趋势，并覆盖 daemon 协调、CLI、MCP 三个入口

补充说明：

- 这些治理工件已经落地，但主要用于能力演进、任务归属和变更校验，不是要求每次日常使用 CLI/MCP 时都先走一遍重流程
- 当前更推荐把它理解成“运行时 3 层能力 + 一个治理覆盖层”，而不是四个完全同权的产品层
- `git`、测试、lint、build 仍然是外部真理源；OPENDOG 输出属于决策辅助证据，遇到需要确认的场景应切换到 shell 或项目原生验证
- 如果只想先确认“这个项目现在到底怎么定位”，优先读 [docs/positioning.md](/opt/claude/opendog/docs/positioning.md)

## 当前能力结构

从能力视角看，当前 OPENDOG 已经不是单一监控后端，而是 3 层结构：

1. 观测内核
   多项目隔离、快照基线、运行时监控与归因、统计证据、导出、对比报告、留存证据生命周期
2. 服务交付与运行协调
   CLI 操作侧入口、MCP AI 侧入口、daemon 运行时、本地控制面协调
3. AI 决策辅助层
   工作区观察、仓库风险、执行策略、验证证据、多项目优先级、清理/重构审查、工具链指导、约束/边界、MOCK / 硬编码数据审查

另有一个治理覆盖层，用于约束能力归属、需求映射和任务执行边界，但它不属于运行时主链路。

能力治理锚点见 [.planning/FUNCTION_TREE.md](/opt/claude/opendog/.planning/FUNCTION_TREE.md)。

## 当前 MCP 工具

- 当前 MCP surface 共 `25` 个工具，下面按能力簇分组列出
- 基础控制：`create_project`、`take_snapshot`、`start_monitor`、`stop_monitor`、`get_stats`、`get_unused_files`、`list_projects`、`delete_project`
- 比较报告：`get_time_window_report`、`compare_snapshots`、`get_usage_trends`
- 数据清理：`cleanup_project_data`
- 配置管理：`get_global_config`、`get_project_config`、`update_global_config`、`update_project_config`、`reload_project_config`
- 导出：`export_project_evidence`
- AI 辅助：`get_agent_guidance`、`get_decision_brief`、`get_verification_status`、`record_verification_result`、`run_verification_command`
- 数据风险：`get_data_risk_candidates`、`get_workspace_data_risk_overview`

MCP 作用域提示：

- `get_agent_guidance` 支持可选 `project_id` 与 `top`
- `get_decision_brief` 支持可选 `project_id` 与 `top`
- 如果 daemon 已经运行，这两个 MCP 工具会优先复用 daemon 的本地控制面状态

## 当前 CLI 命令

- 当前 CLI 顶层入口共 `21` 个命令，下面按能力簇分组列出
- 基础：`opendog create|snapshot|start|stop|stats|unused|list|delete|daemon|mcp|export`
- 比较报告：`opendog report window|compare|trend [--json]`
- 数据清理：`opendog cleanup-data --id <ID> --scope <activity|snapshots|verification|all> [--dry-run] [--vacuum] [--json]`
- 配置：`opendog config show|set-project|set-global|reload [--json]`
- 指导：`opendog agent-guidance [--project <ID>] [--top <N>] [--json]`
- 决策骨架：`opendog decision-brief [--project <ID>] [--top <N>] [--json]`
- 验证：`opendog record-verification|verification|run-verification [--json]`
- 数据风险：`opendog data-risk [--json]` / `opendog workspace-data-risk [--json]`

## 推荐的 AI 使用顺序

- 如果还没有基线：先 `take_snapshot`
- 如果没有持续观测：先 `start_monitor`
- 如果要先拿一份统一的 AI 决策骨架：先看 `get_decision_brief`
- 如果要判断项目当前是否适合修改：先看 `get_agent_guidance`
- 如果要判断最近哪些文件真的变热、变冷、或发生结构变化：先看 `get_time_window_report`、`compare_snapshots`、`get_usage_trends`
- 如果要控制多项目长期沉淀下来的 OPENDOG 观测数据：用 `cleanup_project_data` 或 `opendog cleanup-data` 先 dry-run 再清；只有在大批量清理后才考虑 `vacuum`
- 如果 `agent-guidance` / `decision-brief` 已经把某个项目标成存储维护候选，优先看它注入的 `cleanup_project_data` / `cleanup-data` 入口模板
- 如果要判断是否能安全清理或重构：先看 `get_verification_status`，再看 `get_data_risk_candidates`
- 如果你同时维护多个项目：先看 `get_workspace_data_risk_overview`，再决定先进入哪个项目
- 更完整的行动顺序、shell 切换时机和安全边界，见 [docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)

## Quick Start for AI

按下面顺序调用，通常不会错：

1. 建项目或确认项目存在：`create_project` / `opendog create`
2. 建立基线：`take_snapshot` / `opendog snapshot`
3. 开启持续观测：`start_monitor` / `opendog start`
4. 先拿统一判断入口：`get_decision_brief` 或 `get_agent_guidance` / `opendog decision-brief` 或 `opendog agent-guidance`
5. 再按需要进入 report、verification、data-risk、workspace-data-risk 等具体路径

常用判断入口：

- 想知道“先看哪个项目”：`get_workspace_data_risk_overview` / `opendog workspace-data-risk`
- 想先拿统一决策骨架：`get_decision_brief` / `opendog decision-brief`
- 想知道“现在适不适合改”：`get_agent_guidance` / `opendog agent-guidance`
- 想知道“最近变了什么”：`compare_snapshots`、`get_time_window_report`、`get_usage_trends`
- 想知道“能不能安全清理/重构”：先 `get_verification_status`，再 `get_data_risk_candidates`

详细调用顺序见 [docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)。
能力到 MCP / CLI / JSON contract 的单页映射见 [docs/capability-index.md](/opt/claude/opendog/docs/capability-index.md)。
CLI JSON 契约见 [docs/json-contracts.md](/opt/claude/opendog/docs/json-contracts.md)。
MCP 请求参数与返回重点见 [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md)。

## 历史方案与关键修正

- `inotify` 不提供 PID，不能从事件里直接得到“是谁访问了文件”
- 实际实现采用 `/proc/<pid>/fd` 周期扫描作为主 attribution 来源，`inotify` 主要用于变化检测
- OPENDOG 现在的定位不只是“监控后端”，还包括“给 AI 提供可复用的观察层、证据层、约束层，以及留存证据生命周期 / 本地控制面协调能力”

完整历史方案归档见 [docs/historical-original-plan.md](/opt/claude/opendog/docs/historical-original-plan.md)。

如果你要了解当前怎么用、当前实现了什么、当前能力如何分层，优先阅读：

- [docs/positioning.md](/opt/claude/opendog/docs/positioning.md)
- [docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)
- [docs/json-contracts.md](/opt/claude/opendog/docs/json-contracts.md)
- [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md)
- [.planning/PROJECT.md](/opt/claude/opendog/.planning/PROJECT.md)
- [.planning/FUNCTION_TREE.md](/opt/claude/opendog/.planning/FUNCTION_TREE.md)
