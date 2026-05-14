# OpenDog 架构审查报告

**日期：** 2026-05-14
**方法：** [mattpocock/skills improve-codebase-architecture](https://github.com/mattpocock/skills/blob/main/skills/engineering/improve-codebase-architecture/SKILL.md)
**范围：** 约 120 个非测试 Rust 源文件（21,428 行），76 个测试相关 Rust 文件（9,526 行）；统计口径见附录
**状态：** 待审核

---

## 摘要

基于 John Ousterhout《A Philosophy of Software Design》的"深度"（depth）概念和模块接口分析，发现 5 个结构性摩擦点。最深层的共同根源是：**daemon/direct fallback 策略缺少统一 adapter**，导致调用者在 29+ 个位置重复理解 IPC 与直连路径；同时 **`serde_json::Value` 作为模块间契约**使得 15+ 文件通过字符串 key-path 耦合。修复方向应避免新增一个横跨所有业务域的宽 `ProjectService`，而应提取 fallback policy 并按业务域暴露窄接口。

---

## 发现 1：Daemon-Fallback 重复模式

**分类：** 散布式局部性（Scattered Locality）+ 浅模块（Shallow Module）
**严重度：** 高 — 影响最广，每个新功能必触

### 涉及文件（8 个文件，29 个调用点）

| 文件 | 调用点数 |
|------|----------|
| `src/mcp/project_handlers.rs` | 6 |
| `src/mcp/analysis_handlers.rs` | 5 |
| `src/cli/project_commands.rs` | 5 |
| `src/cli/guidance_commands.rs` | 4 |
| `src/mcp/verification_handlers.rs` | 3 |
| `src/mcp/guidance_handlers.rs` | 2 |
| `src/mcp/config_handlers.rs` | 2 |
| `src/mcp/risk_handlers.rs` | 2 |

### 问题描述

每个 handler/command 手动复制相同的 try-daemon / fall-through-direct 分支逻辑：

```
1. 尝试 DaemonClient::new().some_method()
2. 如果 Err(DaemonUnavailable)，回退
3. 直接调用 ProjectManager / core 模块执行相同操作
```

`DaemonClient::method()` 和 `ProjectManager::method()` 语义相同但调用路径完全不同。每个调用点必须同时了解 IPC 路径和直连路径的细节。新增功能时必须在所有 29 个模式中再添加一个实例。

### 重复模式示例

`src/mcp/project_handlers.rs` 中 `handle_register_project`（约第 16-42 行）：
- 尝试 `DaemonClient` → 如果不可用 → 锁 Mutex → 调用 `project_manager().create()`

`handle_take_snapshot`（约第 43-78 行）：
- 尝试 `DaemonClient` → 如果不可用 → 锁 Mutex → 解析 config → 调用 `snapshot::take_snapshot()`

每个函数都是一个微编排器，执行相同的分支逻辑。

### 建议方案

不要直接抽取一个覆盖 stats、snapshot、monitor、config、guidance 的宽 `ProjectService` trait；那会把 `MonitorController` 的宽接口搬到新 trait 上。建议分两层处理：

1. **Fallback adapter：** 统一封装 "daemon first / direct fallback" 策略，只在 `DaemonUnavailable` 时回退，其他 daemon 错误直接返回。
2. **窄域接口：** 按业务域暴露小接口，例如 `ProjectLifecycleService`、`SnapshotService`、`MonitorService`、`GuidanceService`、`ConfigService`。

概念示例：

```rust
// 概念性示例，非最终设计
trait ProjectLifecycleService {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo>;
    fn delete_project(&self, id: &str) -> Result<bool>;
}

struct FallbackProjectGateway<D, L> {
    daemon: D,
    local: L,
}
```

`FallbackProjectGateway` 持有 fallback policy；handler 和 command 只调用窄域接口，不再感知 IPC 与 direct 路径。`delete_project` 这类带有 "先停 monitor 再删除" 语义的操作必须在 adapter 内显式建模，不能靠简单的 trait 二选一隐藏。

### 预期收益

- 消除 ~400 行重复编排代码
- 将 daemon/direct fallback 错误语义集中到一个地方
- 新增功能只需接入对应窄域接口，而不是在每个 handler/command 中复制 fallback 分支
- 为发现 2 提供统一调用入口，但 guidance 内部依赖仍需单独整理
- 为测试提供注入点

---

## 发现 2：Guidance 三路计算重复

**分类：** 接缝间耦合（Coupling Across Seams）
**严重度：** 中 — 4 个调用点，但 guidance 是高频变更区域

### 涉及文件

- `src/guidance.rs`（126 行）— 共享 payload 组装，但 import `crate::mcp::*`
- `src/control/controller_queries.rs`（约第 174-224 行）— 重新实现项目范围逻辑
- `src/cli/guidance_commands.rs`（约第 17-53 行）— 再次重新实现
- `src/mcp/guidance_handlers.rs`（约第 62-153 行）— 又一次通过 `scoped_projects_or_error`

### 问题描述

计算 "agent guidance" 需要组装 ~15 层 JSON（stats、verification、repo risk、mock detection、toolchain、attention scoring）。这个组装逻辑在 3 个地方独立实现：

- **路径 A（MCP 直连）：** guidance_handlers → guidance.rs → mcp/project_recommendation → mcp/guidance_payload
- **路径 B（Daemon IPC）：** controller_queries 重新实现项目范围逻辑后调用 guidance.rs
- **路径 C（CLI）：** guidance_commands 再次独立实现项目列表、范围筛选

`guidance.rs` 位于 crate root 但大量 import `crate::mcp::*`，形成 cli + control + mcp + storage 的耦合网。

### 建议方案

不要假定发现 1 会自动解决本问题。Guidance 的重复有两层：

1. **调用路径重复：** CLI/MCP/control 都在处理项目列表、范围筛选、daemon fallback。
2. **模块边界反向依赖：** `src/guidance.rs` 位于 crate root，却依赖 `crate::mcp::*` 的 payload builder 和 DTO。

建议将 guidance 修复拆成两个提交序列：

1. 通过发现 1 的 fallback adapter/窄 `GuidanceService` 统一 CLI/MCP/control 的调用入口。
2. 将 guidance payload builder、DTO 和 typed result 从 `src/mcp/` 移到中性模块（例如 `src/core/guidance/` 或 `src/guidance/` 子模块），让 MCP 只负责 transport/payload 包装。

### 预期收益

- 消除 ~250 行重复的范围/筛选逻辑
- Guidance 领域变更集中在中性模块，而不是散在 CLI/MCP/control
- 消除 `guidance.rs` 对 `crate::mcp::*` 的交叉依赖

---

## 发现 3：MonitorController 上帝对象

**分类：** 浅模块（Shallow Module）+ 不可测试接口（Untestable Interface）
**严重度：** 高 — 影响整体可测试性和并发模型

### 涉及文件

- `src/control.rs`（269 行）— MonitorController 定义
- `src/control/controller_queries.rs`（258 行）— 25+ 查询方法
- `src/mcp/server_core.rs`（约第 9 行）— `Mutex<MonitorController>`

### 问题描述

MonitorController 暴露 **30+ 公开方法**，横跨所有业务域：

| 域 | 方法 |
|----|------|
| 项目生命周期 | create_project, delete_project, list_projects |
| 监控 | start_monitor, stop_monitor, stop_all, monitor_ids |
| 快照 | take_snapshot |
| 配置 | global_config, project_config_view, update_project_config, ... |
| 查询 | get_stats, get_unused_files, get_time_window_report, compare_snapshots, ... |
| 验证 | get_verification_status, record_verification_result, execute_verification |
| 数据风险 | get_data_risk_candidates, get_workspace_data_risk_overview |
| Guidance | get_agent_guidance, get_decision_brief |
| 清理 | cleanup_project_data |

**浅层包装问题：** 9 个查询方法遵循完全相同的模式：

```rust
pub fn get_stats(&self, id: &str) -> Result<...> {
    self.pm.get(id)?.ok_or_else(|| ...)?;  // 验证项目存在
    let db = self.pm.open_project_db(id)?;  // 打开数据库
    stats::get_stats(&db)?                  // 委托 core 函数
}
```

这 9 个方法可以用一个通用 helper 替代。

**并发瓶颈：** MCP handler 必须锁 `Mutex<MonitorController>` 才能执行任何操作，包括纯读取。

**不可测试：** `MonitorController::new()` 创建真实 `ProjectManager`（打开真实 SQLite 文件），无法注入内存数据库。

### 建议方案

分两步：

**第一步（低成本）：** 提取 `with_project_db()` 泛型 helper，消除 9 个查询方法的重复模式：

```rust
fn with_project_db<F, T>(&self, id: &str, f: F) -> Result<T>
where F: FnOnce(&Connection) -> Result<T> {
    self.pm.get(id)?.ok_or_else(|| ...)?;
    let db = self.pm.open_project_db(id)?;
    f(&db)
}
```

**第二步（随发现 1 协同）：** 不把 `MonitorController` 整体迁移到一个宽 `ProjectService` trait；改为把查询、监控、配置、guidance 等能力拆成窄域接口，并由 fallback adapter 组合 daemon/direct 实现。`MonitorController` 可以暂时保留为 direct implementation 的内部依赖，但不应继续作为 handler 的主要接口。

### 预期收益

- 消除 ~130 行重复的 validate-project + open-db 样板
- 解锁依赖注入，使 MCP handler 测试可以使用临时或内存实现
- 减少 Mutex 竞争面

---

## 发现 4：serde_json::Value 作为模块间契约

**分类：** 接缝间耦合 + 不可测试接口
**严重度：** 高（结构性风险）— 改动面最广，但收益也最大

### 涉及文件（src/mcp/ 下 15+ 文件，~2,000+ 行）

| 文件 | 行数 | 问题 |
|------|------|------|
| `guidance_payload.rs` | 583 | 逐 key-path 赋值构建 Value |
| `attention.rs` | 460 | 112 次字符串 key-path 访问，约 28 个唯一 key |
| `workspace_decision.rs` | 315 | 4 层嵌套 key-path 访问 |
| `project_recommendation.rs` | 500 | 8+ 个大型 `json!({...})` 块 |
| `strategy.rs` | 236 | 每个函数返回 Value |

### 问题描述

Guidance 子系统几乎完全通过 `serde_json::Value` 通信而非类型化 struct。具体影响：

1. **重构危险：** 改一个 key 名（如 `"hardcoded_candidate_count"`）需要跨几十个文件文本搜索。`attention.rs` 中有 112 次字符串 key-path 访问，约 28 个唯一 key。
2. **无编译时验证：** 拼写错误静默返回 `Value::Null` 而非编译错误。
3. **深度约定耦合：** `workspace_decision.rs` 访问 `guidance["layers"]["multi_project_portfolio"]["project_overviews"]`——与 `guidance_payload.rs` 通过 4 层 key-path 耦合，但无类型契约。
4. **测试也是字符串类型：** 断言检查 `payload["guidance"]["project_recommendations"][0]["project_id"]` 而非结构化字段访问。

### 建议方案

渐进式替换：定义 typed struct 替代 JSON key-path。

```rust
// 概念性示例：替换 guidance_payload.rs 中的 json!() 宏
struct GuidanceLayer {
    toolchain: ToolchainLayer,
    stats: StatsLayer,
    verification: VerificationLayer,
    data_risk: DataRiskLayer,
    // ...
}

struct ProjectRecommendation {
    project_id: String,
    priority_score: f64,
    category: RecommendationCategory,
    // ...
}
```

**执行策略：** 不做一次性大重构。按域逐个替换：
1. 先替换 `StatsLayer`（最独立，无跨层依赖）
2. 再替换 `VerificationLayer`
3. 最后替换嵌套最深的 `ProjectRecommendation`

每一步都可独立提交和验证。

### 预期收益

- 编译时验证所有 key-path 访问
- IDE 自动补全和重构支持
- 安全地重命名字段
- 测试断言变成结构化比较

---

## 发现 5：Control Protocol 枚举序列化冗余

**分类：** 冗余代码（Redundant Code）
**严重度：** 中 — 影响新功能开发效率

### 涉及文件

- `src/control/protocol.rs`（247 行）— 26 个 request variants + 27 个 response variants
- `src/control/request_handler.rs`（325 行）— 每个 variant 的分发逻辑
- `src/control/client/` 下 5 个文件（~443 行）— 客户端方法

### 问题描述

每个新功能必须修改 **5 个文件**：

```
1. protocol.rs — 添加 ControlRequest variant + ControlResponse variant
2. control.rs / controller_queries.rs — 添加 MonitorController 方法
3. request_handler.rs — 添加 match arm 分发
4. client/*.rs — 添加 DaemonClient 方法
5. handler/command — 调用 DaemonClient 方法
```

`UpdateGlobalConfig` 是典型例子：6 个字段在 3 个地方分别手工序列化/反序列化：
- protocol.rs 第 33-44 行（request variant 定义）
- request_handler.rs 第 43-56 行（手动解构为 ConfigPatch）
- config_ops.rs 第 43-51 行（从 ConfigPatch 构建 ControlRequest）

### 建议方案

不要把内部协议降级成 `method: String + serde_json::Value` 的泛型 envelope；这会与发现 4 的 typed-contract 目标冲突。建议保留类型化 request/response，并用 helper、derive 或代码生成减少重复样板。

概念方向：

```rust
// 概念性：每个 RPC 仍保留 typed request/response。
trait ControlRpc {
    type Request: Serialize + DeserializeOwned;
    type Response: Serialize + DeserializeOwned;
    const METHOD: &'static str;
}
```

如果最终需要 envelope，它应只存在于 wire boundary；进入 control/client/request_handler 后应立即反序列化为 typed payload。更理想的方向是用 derive 宏从 typed request/response 或 controller 方法签名生成 protocol variants、client forwarding 和 request_handler match arm。

### 预期收益

- 新功能减少 protocol/client/handler 的机械改动，但不牺牲编译期契约
- 压缩序列化和分发样板
- 避免引入更多字符串 method 与 `Value` payload 约定

---

## 优先级建议

| 优先级 | 发现 | 预计工作量 | 理由 |
|--------|------|-----------|------|
| **P0** | #3 `with_project_db()` helper | 低（0.5-1 天） | 最小安全改动，立即减少 controller 查询样板 |
| **P0** | #1 Daemon-Fallback adapter + 窄域接口 | 中（2-3 天） | 收益/成本比最高，集中错误语义和测试注入点 |
| **P1** | #2 Guidance 边界整理 | 中（2-3 天） | 不能只靠 #1 自动解决，需要拆出中性 guidance 模块 |
| **P1** | #4 Value 契约类型化 | 高（5-7 天，可分批） | 收益最大但改动面广，建议按 layer 渐进式替换 |
| **P2** | #5 Control Protocol 样板压缩 | 中（2-3 天） | 应在 typed-contract 方向明确后再做，避免引入 Value envelope |

### 建议执行路径

```
第一步：提取 with_project_db() helper（发现 3，低风险提交）
  ↓
第二步：定义 fallback adapter 的错误语义，并拆出第一个窄域接口（发现 1）
  ↓
第三步：将 project/snapshot/monitor handler 迁移到窄域接口（发现 1 + 3）
  ↓
第四步：拆出中性 guidance 模块，并通过 GuidanceService 统一 CLI/MCP/control 入口（发现 2 + 4）
  ↓
第五步：渐进式 typed struct 替换 guidance Value key-path（发现 4，按 layer 分批）
  ↓
第六步：在 typed request/response 基础上压缩 control protocol 样板（发现 5）
```

---

## 附录：关键术语

| 术语 | 定义（来自 mattpocock/skills LANGUAGE.md） |
|------|------|
| 模块（Module） | 任何具有接口和实现的东西（函数、类、包、slice） |
| 接口（Interface） | 调用者使用模块所需知道的一切：类型、不变量、错误模式、顺序、配置 |
| 深度（Depth） | 接口处的杠杆率：少量接口背后的丰富行为。深 = 高杠杆，浅 = 接口几乎和实现一样复杂 |
| 接缝（Seam） | 接口所在的位置；可以在不原地编辑的情况下改变行为的地方 |
| 删除测试 | 想象删除模块。如果复杂度消失，说明它是传递层。如果复杂度重新出现在 N 个调用者中，说明它在发挥价值 |

---

## 附录：统计口径

本文的文件/行数是静态快照，用于估算改动面，不作为精确验收指标。当前复核口径：

- 非测试 Rust 源文件：`src/` 下排除 `tests/` 目录和 `test` 命名文件，约 120 个文件、21,428 行。
- 测试相关 Rust 文件：`tests/` 目录、`src/**/tests/` 和文件名含 `test` 的 Rust 文件，约 76 个文件、9,526 行。
- control protocol 当前约 26 个 request variants、27 个 response variants；后续改动会使该数字继续变化。

---

*本报告仅供审核，不包含任何代码变更。*
