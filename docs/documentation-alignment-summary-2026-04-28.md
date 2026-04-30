# OPENDOG 文档对齐与减重总结

> 记录时间：2026-04-28  
> 目标：把 OPENDOG 当前文档体系收口到一致的产品定位、阅读路径、治理边界和 AI 使用语义上

## 1. 本轮目标

本轮文档治理不是新增产品能力，而是解决以下问题：

- 文档口径存在轻微漂移
- README 同时承载“当前入口”和“历史归档”，过重
- 多份文档对同一定位与边界反复解释
- AI 消费路径、治理路径、契约路径之间职责不够清晰
- “shell / git / test / lint / build 是外部真理源”这件事在不同文档里说法不统一

## 2. 核心结论

当前文档体系已经统一到以下判断：

- OpenDog 没有方向漂移，仍然是多项目 AI 观察与决策支持系统
- 能力面偏宽，但边界清晰且受控
- 运行时应理解为三层能力，加一个治理覆盖层
- 当前正确方向不是横向继续扩面，而是沿 `FT-03` 继续纵向打深
- `git`、`tests`、`lint`、`build` 是外部真理源；OPENDOG 输出属于决策辅助证据

## 3. 关键调整

### 3.1 新增单点定位入口

新增：

- [positioning.md](./positioning.md)

作用：

- 用一页说明项目是什么、不是什么、当前设计判断、运行时结构、治理覆盖层定位
- 作为其他消费文档的统一上游入口

### 3.2 README 减重

README 从混合型文档收口为入口型文档：

- 标题改成“项目概览（当前实现 + 历史方案导航）”
- 保留当前实现、能力结构、工具入口、AI 最短使用顺序
- 删除大段历史原始方案正文
- 改为跳转到独立历史归档

结果：

- `README.md` 从 `318` 行降到 `133` 行

### 3.3 历史方案独立归档

新增：

- [historical-original-plan.md](./historical-original-plan.md)

作用：

- 保留最初方案与早期设计语境
- 不再污染当前入口阅读体验
- 明确提示该文档不是当前行为真相来源

### 3.4 核心消费文档职责拆分

当前分工如下：

- [README.md](../README.md)：总入口
- [positioning.md](./positioning.md)：产品定位与边界
- [capability-index.md](./capability-index.md)：能力到命令/文档映射
- [ai-playbook.md](./ai-playbook.md)：AI 使用顺序、shell 切换、安全规则
- [mcp-tool-reference.md](./mcp-tool-reference.md)：MCP 请求/响应使用说明
- [json-contracts.md](./json-contracts.md)：机器消费字段、schema、稳定性与错误契约
- [.planning/FUNCTION_TREE.md](../.planning/FUNCTION_TREE.md)：能力治理锚点

### 3.5 统一导航

以下核心文档都新增了统一的 `Quick Navigation`：

- [positioning.md](./positioning.md)
- [capability-index.md](./capability-index.md)
- [ai-playbook.md](./ai-playbook.md)
- [mcp-tool-reference.md](./mcp-tool-reference.md)
- [json-contracts.md](./json-contracts.md)

这样读者在任意一页都能快速跳到：

- 产品定位
- 能力映射
- AI 工作流
- MCP 用法
- JSON 契约

### 3.6 文档去重

本轮去掉了几类重复：

- 多份文档重复讲完整产品定位
- `ai-playbook` 与 `json-contracts` 大段重复字段说明
- `json-contracts` 与 `mcp-tool-reference` 重复逐工具 MCP 服务字段
- README 与 AI 手册/历史方案之间的大段重合

比较明显的结果：

- `ai-playbook.md` 从 `497` 行降到 `394` 行
- `json-contracts.md` 从 `813` 行降到 `735` 行

### 3.7 统一 authority wording

以下文档已经统一到同一套边界表述：

- [README.md](../README.md)
- [positioning.md](./positioning.md)
- [capability-index.md](./capability-index.md)
- [ai-playbook.md](./ai-playbook.md)
- [mcp-tool-reference.md](./mcp-tool-reference.md)
- [json-contracts.md](./json-contracts.md)
- [opendog-project-panorama-and-rollout-plan.md](./opendog-project-panorama-and-rollout-plan.md)

统一规则：

- `git`、`tests`、`lint`、`build` 是外部真理源
- OPENDOG 输出是决策辅助证据
- 需要确认时，切换到 `shell` 或项目原生验证

## 4. 规划文档同步

以下规划主文档也已对齐：

- [.planning/PROJECT.md](../.planning/PROJECT.md)
- [.planning/STATE.md](../.planning/STATE.md)
- [.planning/FUNCTION_TREE.md](../.planning/FUNCTION_TREE.md)
- [opendog-project-panorama-and-rollout-plan.md](./opendog-project-panorama-and-rollout-plan.md)

同步后的统一判断是：

- 没有方向漂移
- 开口偏大但合理可控
- 有局部轻度过度设计，但尚未伤及主架构
- 治理层应按比例使用，而不是删除
- 当前最该做的是继续沿 `FT-03` 纵向打深

## 5. 当前文档体系的推荐阅读顺序

对于首次理解项目的人：

1. [positioning.md](./positioning.md)
2. [README.md](../README.md)
3. [capability-index.md](./capability-index.md)

对于 AI 或自动化消费者：

1. [ai-playbook.md](./ai-playbook.md)
2. [mcp-tool-reference.md](./mcp-tool-reference.md)
3. [json-contracts.md](./json-contracts.md)

对于规划与演进：

1. [.planning/PROJECT.md](../.planning/PROJECT.md)
2. [.planning/STATE.md](../.planning/STATE.md)
3. [.planning/FUNCTION_TREE.md](../.planning/FUNCTION_TREE.md)

对于历史追溯：

1. [historical-original-plan.md](./historical-original-plan.md)
2. [opendog-project-panorama-and-rollout-plan.md](./opendog-project-panorama-and-rollout-plan.md)

## 6. 本轮产出状态

本轮主要属于文档治理与信息架构收口，不涉及功能新增。

规划校验结果保持正常：

- `python3 scripts/validate_planning_governance.py`
- `python3 scripts/validate_task_cards.py`

## 7. 后续建议

这条线可以暂时收口。后续更值得投入的方向不是继续打磨文档措辞，而是回到产品和代码本身：

- 继续沿 `FT-03` 做 evidence / strategy / boundary 的可信度提升
- 继续强化 daemon / MCP / local control plane 的运行一致性
- 在新增能力前，先判断是否真的属于现有能力树的纵深增强
