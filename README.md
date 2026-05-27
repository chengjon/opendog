OPENDOG — Multi-Project Observation & AI Decision-Support System

> OPENDOG tracks which files AI tools access, identifies unused/stale files vs actively-used core files, and exposes reusable operator/AI entry surfaces through daemon, CLI, and MCP for repo risk, verification evidence, retained-evidence lifecycle, and suspicious mock or hardcoded data review.
>
> 当前状态：**全部能力已交付**。FUNCTION_TREE.md 中全部 27 个叶子节点（FT-01 观测内核 8 个、FT-02 服务交付 4 个、FT-03 AI 决策辅助 15 个）均已标记为 `shipped`。通过 MCP 27 个工具、2 个只读 Resource、23 个 CLI 命令、daemon + systemd 持久运行四种入口对外提供能力。

## 阅读导航

- 项目定位：[docs/positioning.md](/opt/claude/opendog/docs/positioning.md)
- 快速上手：[QUICKSTART.md](/opt/claude/opendog/QUICKSTART.md)
- MCP 工具完整参考：[docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md)
- AI 行动手册：[docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)
- 能力/入口速查：[docs/capability-index.md](/opt/claude/opendog/docs/capability-index.md)
- CLI/MCP JSON 契约：[docs/json-contracts.md](/opt/claude/opendog/docs/json-contracts.md)

## 当前实现概览

**三层能力，全部已交付：**

| 层级 | 能力 | 入口 |
|------|------|------|
| 观测内核 (FT-01) | 多项目隔离、SQLite 持久化、全量快照、`/proc` + `inotify` 混合监控、统计分析、导出、对比报告、留存证据生命周期 | MCP + CLI + daemon |
| 服务交付与运行协调 (FT-02) | CLI 操作入口、MCP AI 入口、daemon systemd 持久运行、本地控制面协调 | CLI + MCP + daemon |
| AI 决策辅助 (FT-03) | 工作区观察、仓库风险、执行策略、验证证据、多项目优先级、清理/重构审查、工具链指导、约束/边界、MOCK / 硬编码数据审查、治理状态观察 | MCP + CLI |

**技术要点：**

- `opendog mcp` 自动确保 daemon-backed 运行路径可用，监控状态可跨 MCP 会话稳定复用
- 所有 MCP 工具均使用 daemon-first 模式：优先通过 Unix socket IPC 走 daemon 控制面，daemon 不可用时回退到直接 DB 访问
- 其他 MCP Host 接入时，建议设置固定的 `OPENDOG_HOME`，确保跨会话复用同一份 daemon / registry / DB
- `/proc/<pid>/fd` 扫描只归因文件级 fd（排除目录 fd），单次扫描周期内按 `(pid, fd)` 去重
- 源码 / AI 基础设施 / 备份文件软分类，避免 `.claude/`、`.agents/` 等工具目录噪声淹没源文件判断
- `git`、测试、lint、build 仍然是外部真理源；OPENDOG 输出属于决策辅助证据，遇到需要确认的场景应切换到 shell 或项目原生验证
- 治理工件（FUNCTION_TREE、task-cards、REQUIREMENTS 映射）已全部落地，主要用于能力演进和变更校验，不影响日常使用

## 当前能力结构

从能力视角看，OPENDOG 是 3 层结构：

1. **观测内核 (FT-01)**
   多项目隔离、快照基线、运行时监控与归因、统计证据、导出、对比报告、留存证据生命周期
2. **服务交付与运行协调 (FT-02)**
   CLI 操作入口、MCP AI 入口、daemon 运行时、本地控制面协调
3. **AI 决策辅助层 (FT-03)**
   工作区观察、仓库风险、执行策略、验证证据、多项目优先级、清理/重构审查、工具链指导、约束/边界、MOCK / 硬编码数据审查、治理状态观察

另有一个治理覆盖层，用于约束能力归属、需求映射和任务执行边界，但它不属于运行时主链路。

能力治理锚点见 [FUNCTION_TREE.md](/opt/claude/opendog/FUNCTION_TREE.md)。

## 当前 MCP 工具

MCP surface 共 **27 个工具** + **2 个只读 Resource**，全部采用 daemon-first 模式。下面按能力簇分组。

### 项目生命周期（4 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `register_project` | 注册项目根目录 | `id`（项目 ID）、`path`（绝对路径） |
| `delete_project` | 删除项目及全部数据 | `id` |
| `list_projects` | 列出所有已注册项目 | 无 |
| `take_snapshot` | 触发全量文件扫描，记录文件路径/大小/元数据 | `id` |

### 观测与监控（3 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `start_monitor` | 启动 /proc 扫描 + inotify 变更检测 | `id` |
| `stop_monitor` | 停止监控 | `id` |
| `get_stats` | 查询文件使用统计（访问次数、持续时间、修改次数等） | `id`、`limit`（默认 50）、`path_classification`（all/source/infrastructure/backup/project） |

### 统计分析（5 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `get_unused_files` | 列出从未被访问的文件 | `id`、`limit`、`path_classification` |
| `get_time_window_report` | 时间窗口内的文件活跃统计 | `id`、`window`（24h/7d/30d）、`limit` |
| `compare_snapshots` | 对比两次快照差异 | `id`、`base_run_id`、`head_run_id`（省略则对比最近两次）、`limit` |
| `get_usage_trends` | 分桶使用趋势 | `id`、`window`（24h/7d/30d）、`limit` |
| `get_activity_rollups` | 查询保留策略压缩后的每日活动汇总 | `id`、`window`（24h/7d/30d）、`limit` |

### 配置查询（3 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `get_global_config` | 获取全局默认配置（ignore 模式、进程白名单） | 无 |
| `get_project_config` | 获取项目解析后的生效配置 | `id` |
| `get_build_info` | 获取 binary 版本、git hash、构建时间、是否需要重建 | 无 |

### AI 决策辅助（2 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `get_guidance` | 统一 AI 决策辅助入口，含工作区观察、风险、策略、约束等全部决策层 | `project_id`（可选，默认工作区范围）、`top`（默认 5）、`detail`（summary/decision） |
| `get_workspace_data_risk_overview` | 跨项目聚合数据风险信号，优先级排序帮助 AI 决定先关注哪个项目 | `candidate_type`（all/mock/hardcoded）、`min_review_priority`、`project_limit` |

### 验证证据（3 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `get_verification_status` | 获取项目最新的 test/lint/build 验证证据及安全门判断 | `id` |
| `record_verification_result` | 记录一条验证结果作为证据 | `id`、`kind`（test/lint/build）、`status`、`command`、`exit_code`、`summary` |
| `run_verification_command` | 在项目根目录执行验证命令并自动记录结果 | `id`、`kind`、`command` |

### 数据风险（1 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `get_data_risk_candidates` | 检测项目中的 mock/fixture/demo 数据和可疑硬编码业务数据 | `id`、`candidate_type`（all/mock/hardcoded）、`min_review_priority`、`limit` |

### 治理状态观察（4 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `create_governance_lane` | 创建治理工作泳道，用于跟踪 AI 会话意图和边界 | `id`、`lane_id`、`title`、`description` |
| `upsert_governance_node` | 在泳道内创建或更新治理节点 | `id`、`node_id`、`lane_id`、`state`、`summary` 等 |
| `get_governance_state` | 读取项目治理泳道和节点，含快照/验证/数据风险交叉观察提示 | `id`、`lane_id`、`node_id`、`active_only` |
| `close_governance_lane` | 关闭、推迟或删除治理泳道 | `id`、`lane_id`、`action`（complete/defer/delete） |

### 孤立文件检测（2 个）

| 工具 | 说明 | 关键参数 |
|------|------|----------|
| `scan_orphans` | 扫描项目中的孤立文件/模块/路由候选，综合内部和外部扫描器证据 | `id`、`subjects`、`include_internal_scanners`、`required_scanners`、`limit` |
| `verify_deletion_plan` | 验证删除目标是否有足够证据支持安全删除 | `id`、`targets`、`external_reports`、`required_project_verification_commands` |

### 只读 Resources（2 个）

| URI | 说明 |
|-----|------|
| `opendog://projects` | 已注册项目列表（同 `list_projects`） |
| `opendog://project/{id}/verification` | 单项目验证证据（同 `get_verification_status`） |

### MCP 使用提示

- `get_guidance` 是 AI 决策辅助的主入口：`detail=summary` 返回完整 guidance 负载，`detail=decision` 返回稳定决策骨架
- `opendog mcp` 会自动确保并复用 daemon 的本地控制面状态，监控状态不会因 MCP 会话断开而丢失
- 若希望跨宿主/跨会话稳定复用同一份状态，配置固定的 `OPENDOG_HOME=/absolute/path/to/opendog-state`
- 完整参数细节和返回字段见 [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md)

## 当前 CLI 命令

- 当前 CLI 顶层入口共 `23` 个命令，下面按能力簇分组列出
- 基础：`opendog register|snapshot|start|stop|stats|unused|list|delete|daemon|mcp|export`
- 比较报告：`opendog report window|compare|trend|rollup [--json]`
- 数据清理：`opendog cleanup-data --id <ID> --scope <activity|snapshots|verification|all> [--dry-run] [--vacuum] [--json]`
- 配置：`opendog config show|set-project|set-global|reload [--json]`，可用 `--retention-policy-json` 调整保留策略
- 维护：`opendog self-update status|build --source /opt/claude/opendog [--json]`
- 指导：`opendog agent-guidance [--project <ID>] [--top <N>] [--json]`
- 决策骨架：`opendog decision-brief [--project <ID>] [--top <N>] [--json]`
- 验证：`opendog record-verification|verification|run-verification [--json]`
- 数据风险：`opendog data-risk [--json]` / `opendog workspace-data-risk [--json]`
- 治理：`opendog governance create-lane|upsert-node|show|close-lane [--json]`

## 推荐的 AI 使用顺序

1. **确保项目已注册并有基线**：`register_project` → `take_snapshot` → `start_monitor`
2. **获取决策辅助**：`get_guidance`（先 `detail=summary` 了解全局，再 `detail=decision` 拿稳定决策骨架）
3. **判断文件活跃度**：`get_stats`（带 `path_classification=source` 过滤源码）、`get_unused_files`、`get_time_window_report`
4. **判断能否安全修改**：`get_verification_status` → `get_data_risk_candidates`
5. **多项目优先级**：`get_workspace_data_risk_overview` 决定先关注哪个项目
6. **存储维护**：如 guidance 标记存储维护候选，用 CLI `opendog cleanup-data --dry-run` 预览；清理后用 `opendog report rollup --json` 查看保留下来的每日活动汇总
7. **治理跟踪**：`create_governance_lane` → `upsert_governance_node` 记录工作意图和边界

更完整的行动顺序、shell 切换时机和安全边界见 [docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)。

## Quick Start for AI

快速上手见 [QUICKSTART.md](/opt/claude/opendog/QUICKSTART.md)。

## 技术背景

- `inotify` 不提供 PID，不能从事件里直接得到"是谁访问了文件"
- 实际实现采用 `/proc/<pid>/fd` 周期扫描作为主归因来源，`inotify` 主要用于变化检测
- OPENDOG 的定位不只是监控后端，而是给 AI 提供可复用的观察层、证据层、约束层

完整历史方案归档见 [docs/historical-original-plan.md](/opt/claude/opendog/docs/historical-original-plan.md)。

## 文档导航

- 项目定位：[docs/positioning.md](/opt/claude/opendog/docs/positioning.md)
- AI 行动手册：[docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)
- MCP 工具完整参考：[docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md)
- CLI/MCP JSON 契约：[docs/json-contracts.md](/opt/claude/opendog/docs/json-contracts.md)
- 能力治理锚点：[FUNCTION_TREE.md](/opt/claude/opendog/FUNCTION_TREE.md)
- 项目上下文：[.planning/PROJECT.md](/opt/claude/opendog/.planning/PROJECT.md)
