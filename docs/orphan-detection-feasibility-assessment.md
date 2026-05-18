# OpenDog MCP 孤儿功能发现能力 — 可行性评估

> 评估日期：2026-05-18
> 触发仓库：mystocks_spec（用于场景驱动设计）
> 评估对象：为 OpenDog MCP 新增"发现无效功能、孤儿文件、死路由、死测试"能力

---

## 一、背景

用户在 mystocks_spec 仓库中实际运行了 OpenDog MCP，并基于该仓库的真实场景（孤儿 API 模块、占位包、死测试、契约漂移）提出了一个完整的方案设计，包含 8 项核心能力、4 个 MCP Tool 设计和一套"证据分层 + 否决项"的判定算法。

本评估回答两个核心问题：

1. 方案在技术上是否可行？
2. 是否需要 OpenDog MCP 专门实现一套多信号证据链？

---

## 二、用户方案设计摘要

### 2.1 目标场景（来自 mystocks_spec 实际分析）

| 场景 | 示例 | 关键信号 |
|---|---|---|
| 孤儿 API 模块 | `web/backend/app/api/efinance.py` | 未注册到 `app.include_router`，不在 `app.routes`，不在 `/openapi.json` |
| 占位包 / 骨架包 | `mystocks_api/` | 多数文件为空 `__init__.py`，无路由注册，无部署入口 |
| 死测试 | `test_efinance_file.py`（测试死目标）vs `test_efinance_adapter.py`（测试活跃 adapter） | 必须区分 `tests_adapter` 和 `tests_unregistered_api_file` |
| 契约漂移 | API 文件存在但未贡献 OpenAPI path | 运行时真相是路由 + contract，不是文件存在性 |

### 2.2 提出的 8 项核心能力

1. **Import Graph Indexer** — Python AST/LibCST 建模块导入图
2. **FastAPI Route Auditor** — 安全导入 app，枚举 `app.routes`
3. **OpenAPI Contract Diff** — 导出前后 `/openapi.json` 对比
4. **Pytest Test Mapper** — `pytest --collect-only` + AST 分类测试意图
5. **Entrypoint Scanner** — 扫描 PM2/Docker/CI/shell 脚本找 `uvicorn module:app`
6. **Frontend Consumer Scanner** — 扫描 `web/frontend/src/api/**` 和生成的类型
7. **Spec/Docs/Ownership Gate** — 读取 `architecture/STANDARDS.md`、OpenSpec、OWNERSHIP
8. **Telemetry Adapter** — nginx access log / OpenTelemetry 流量证据

### 2.3 提出的判定算法

```
safe_to_remove:
  - 无生产 import incoming edge
  - 无 runtime route
  - 无 OpenAPI path/schema 贡献
  - 无部署/脚本入口
  - 无前端/API client 消费者
  - 测试只覆盖该死目标
  - 无动态导入风险
  - 不违反 OpenSpec/ownership gate

review_required:
  - 只有文本引用或文档引用
  - 有动态导入风险
  - 有历史兼容层迹象
  - 有测试但无法确定意图

blocked:
  - 出现在 runtime app.routes
  - 出现在 /openapi.json
  - 被主 app/router registry 注册
  - 被前端活跃 client 调用
  - 被部署脚本直接启动
  - 被 OpenSpec 当前 capability 明确要求
```

---

## 三、设计层面评估：优秀

用户设计的核心洞察是正确的：

1. **"文件是否被引用" ≠ "功能是否进入运行时系统"** — 对 API 而言，运行时真相是路由注册 + OpenAPI contract + 部署入口，不是文件存在性。
2. **静态扫描永远不能单独证明"安全删除"** — Python 有 `importlib`、字符串拼接导入、插件注册、条件导入；外部系统可能直接调用旧 API 路径。
3. **测试意图区分至关重要** — 不能因为名字里有 `efinance` 就误删 adapter 测试。
4. **正确输出是候选分级，不是删除命令** — `remove_candidate / review_required / blocked` 的三分类比"可安全删除"的二元判断更诚实。

这与 mystocks_spec 的 `architecture/STANDARDS.md:87`（清理删除必须先做"代码路径 + 功能树"双重判定）完全一致。设计层面没有问题。

---

## 四、实现层面评估：不能在 OpenDog Rust 中全部原生实现

### 4.1 OpenDog 现有能力边界

OpenDog 运行时的三层产品：

| 层 | 做什么 | 技术方式 |
|---|---|---|
| 观测内核 (FT-01) | 文件访问追踪、快照基线、变更检测 | `/proc` 扫描 + inotify，语言无关 |
| 服务交付 (FT-02) | CLI/MCP/daemon 入口 | Rust 原生 |
| AI 决策辅助 (FT-03) | 仓库风险、验证证据、数据风险、优先级排序 | 部分调用 `git` 子进程，部分内置 |

关键发现：**OpenDog 已有调用外部工具的成熟模式**：
- `repo_risk` 模块通过 `std::process::Command` 调用 `git status`、`git diff`、`git rev-list`
- `verification` 模块通过 `sh -lc` 执行任意命令并记录结果

这为"编排外部扫描器"提供了现成的架构基础。

### 4.2 各能力与 OpenDog 现有架构的距离

| # | 能力 | 所需技术 | OpenDog Rust 能否实现 | 原因 |
|---|---|---|---|---|
| 1 | Import Graph Indexer | Python AST/LibCST | **否** | 动态导入（`importlib`、`__import__`、字符串路径）在静态 Rust 中无法解析 |
| 2 | FastAPI Route Auditor | Python 运行时 `app.routes` | **否** | 装饰器注册、条件路由、中间件注入只能在运行时确定 |
| 3 | OpenAPI Contract Diff | 启动 FastAPI → 导出 JSON → diff | **否** | 需要目标项目可运行，依赖其 venv 环境 |
| 4 | Pytest Test Mapper | `pytest --collect-only` + AST | **否** | 参数化测试、fixture、conftest 插件动态生成测试节点 |
| 5 | Entrypoint Scanner | 正则扫描 Dockerfile/PM2/CI/shell | **是** | 纯文本模式匹配，语言无关 |
| 6 | Frontend Consumer Scanner | 正则匹配 URL path（轻量）/ TypeScript AST（深度） | **部分** | 80% 场景可通过正则覆盖；AST 级别需 TypeScript 工具 |
| 7 | Spec/Docs/Ownership Gate | 文本解析 + 规则引擎 | **是** | 文件内容读取 + 关键词/模式匹配，语言无关 |
| 8 | Telemetry Adapter | nginx log / OpenTelemetry | **否** | 完全是外部运行环境数据 |

### 4.3 为什么不能在 Rust 中做 Python/TypeScript 语义分析

用户在方案中自己列出的边界情况已经说明了问题：

- `importlib.import_module(...)` — 参数是运行时字符串，静态分析无法追踪
- `__import__(...)` — 同上
- 字符串拼接导入 — `mod = __import__("mystocks_" + suffix)`
- 插件注册 — 通过 `entry_points` 或 `pkg_resources` 动态发现
- 条件导入 — `if platform == "linux": import ...`
- 外部消费者 — 前端、其他服务、cron 脚本不会出现在 Python import graph 里

如果强行在 Rust 中实现静态分析，结果将是"大部分时候对，但在关键边界情况出错"——这恰恰是用户希望避免的。

---

## 五、推荐架构：OpenDog 做框架，外部脚本做采集

### 5.1 总览

```
OpenDog MCP（Rust）
│
├── 新增：证据聚合框架
│   ├── orphan_detection 模块
│   ├── scan_orphans 工具        → 编排外部扫描器，聚合信号
│   ├── audit_module 工具        → 对单个模块做多维度判定
│   ├── verify_deletion_plan 工具 → 证据链 + 门禁引擎
│   ├── 置信度计算引擎
│   └── 门禁引擎（veto gates）
│
├── 新增：通用扫描器（Rust 原生）
│   ├── entrypoint_scanner     → 正则扫描 Docker/PM2/CI/shell
│   ├── doc_ownership_gate     → 读取 STANDARDS.md / OpenSpec / OWNERSHIP
│   └── frontend_url_scanner   → 正则匹配 URL path 字符串
│
├── 新增：外部扫描器编排层
│   ├── Python scanner runner  → 调用 external/python/ 脚本
│   ├── TypeScript scanner     → 调用 external/typescript/ 脚本
│   └── scanner output parser  → 解析标准化 JSON 输出
│
└── 复用：现有能力
    ├── 文件快照（文件存在性基线）
    ├── 文件分类（source/infrastructure/backup）
    ├── 监控数据（访问次数）
    └── 验证证据（test/lint/build 结果）
```

### 5.2 分工原则

| 层级 | 放在哪里 | 理由 |
|---|---|---|
| 语言无关的文本扫描 | OpenDog Rust | 正则 + 文本匹配，无语言依赖 |
| 语言特定的语义分析 | 外部脚本（Python/TS） | 需要目标语言的运行时和生态工具 |
| 证据聚合 + 门禁判定 | OpenDog Rust | 纯逻辑，跨语言通用，需要持久化 |
| 运行时流量数据 | 外部系统（nginx/OTEL） | 不属于仓库内分析 |

### 5.3 与现有模式的对应

这个分工和 OpenDog 现有 `verification` 模块的模式完全一致：

| 现有模式 | 新增模式 |
|---|---|
| `run_verification_command` 调用 `pytest` | `scan_orphans` 调用 `python_import_graph.py` |
| 记录 exit_code、stdout_tail、stderr_tail | 解析 JSON 输出（import edges、routes、test types） |
| `verification_status_layer` 判断 `safe_for_cleanup` | `orphan_classification` 判断 `remove_candidate / blocked` |

OpenDog 不自己写测试框架，它只负责执行命令、记录结果、评估门禁。orphan detection 应该是一样的模式。

---

## 六、对 8 项能力的具体实现建议

| # | 能力 | 实现位置 | 实现方式 |
|---|---|---|---|
| 1 | Import Graph Indexer | 外部 Python 脚本 | `import_graph.py --root <path>` → JSON |
| 2 | FastAPI Route Auditor | 外部 Python 脚本 | `fastapi_audit.py --app web.backend.app.main:app` → JSON |
| 3 | OpenAPI Contract Diff | 外部 Python 脚本 | `openapi_diff.py --app ... --candidates ...` → JSON |
| 4 | Pytest Test Mapper | 外部 Python 脚本 | `pytest_mapper.py --targets ...` → JSON |
| 5 | Entrypoint Scanner | OpenDog Rust | 内置正则扫描 |
| 6 | Frontend Consumer Scanner | OpenDog Rust（基础）+ 外部 TS（深度） | 正则优先；需要 AST 时走外部 TS 脚本 |
| 7 | Spec/Docs/Ownership Gate | OpenDog Rust | 内置文本解析 + 规则引擎 |
| 8 | Telemetry Adapter | 外部（可选） | 独立系统，不在 Phase 1 范围 |

### 6.1 外部脚本的接口规范

所有外部扫描器统一输出格式：

```json
{
  "scanner": "import_graph",
  "version": "1.0.0",
  "root": "/path/to/project",
  "elapsed_ms": 2340,
  "result": {
    "modules": {
      "web.backend.app.api.efinance": {
        "file": "web/backend/app/api/efinance.py",
        "incoming_refs": [],
        "outgoing_refs": ["fastapi.APIRouter"],
        "dynamic_import_risk": false
      }
    },
    "errors": [],
    "warnings": ["No __init__.py in mystocks_api/"]
  }
}
```

### 6.2 置信度计算

```
confidence = base_confidence × signal_density × freshness_factor

base_confidence:
  - 0.95 if all applicable scanners passed
  - 0.85 if one scanner failed or was skipped
  - 0.70 if multiple scanners failed
  - 0.50 if only entrypoint scanner passed

signal_density:
  - min(1.0, num_passing_signals / num_expected_signals)

freshness_factor:
  - 1.00 if scan < 1 hour old
  - 0.85 if scan < 24 hours old
  - 0.60 if scan < 7 days old
  - 0.30 otherwise
```

---

## 七、分阶段落地路径

### Phase 1：OpenDog 侧最小可行框架（~2-3 周）

- [ ] 新增 `src/mcp/orphan_detection/` 模块
- [ ] 实现通用 `entrypoint_scanner`（Rust 原生，正则扫描）
- [ ] 实现通用 `doc_ownership_gate`（Rust 原生，读取 STANDARDS.md / OpenSpec）
- [ ] 实现 `scan_orphans` MCP Tool（接收外部 JSON 输入 + 内部扫描结果 → 聚合分类）
- [ ] 实现 `verify_deletion_plan` MCP Tool（证据链 + 门禁引擎）
- [ ] 实现 `remove_candidate / review_required / blocked` 三分类逻辑
- [ ] 用 mystocks_spec 的已知场景做端到端验证

### Phase 2：外部扫描器（~3-4 周）

- [ ] `opendog-python-scanner` 包
  - [ ] `import_graph.py` — 基于 `ast` / `libcst` 建导入图
  - [ ] `fastapi_audit.py` — 运行时枚举 `app.routes`
  - [ ] `pytest_mapper.py` — `pytest --collect-only` + 测试分类
  - [ ] `openapi_diff.py` — 导出 + 对比
- [ ] `opendog-typescript-scanner` 包（可选，Phase 2 后期）
  - [ ] `frontend_consumers.ts` — 扫描 API client 调用
- [ ] OpenDog MCP 增加 `--scanner-path` 参数指定外部扫描器位置
- [ ] 自动发现已安装的扫描器

### Phase 3：深度集成与治理（~2-3 周）

- [ ] 证据持久化到 SQLite（跨时间对比）
- [ ] 项目级扫描器配置（在 `opendog config` 中指定语言和扫描器映射）
- [ ] 自动检测项目语言并选择对应扫描器
- [ ] 与 OpenSpec workflow 集成：扫描结果可自动创建 change/proposal
- [ ] Telemetry adapter（可选，作为独立插件）

---

## 八、MCP Tool 设计调整建议

用户提出的 4 个 Tool 设计基本正确，以下为微调建议：

### 8.1 `scan_orphans`

```
原始参数: { root, targets?, languages, entrypoints, include_tests }
调整建议:
  - languages → scanners: ["import_graph", "fastapi_audit", "pytest_mapper"]
    理由: 让用户明确指定用哪些扫描器，OpenDog 不关心是什么语言
  - 增加 max_depth 控制递归深度
  - 增加 min_confidence 过滤低置信度结果
```

### 8.2 `audit_fastapi` → 泛化为 `audit_module`

```
调整建议: 名称改为 audit_module
  - 增加 kind: "fastapi_router" | "python_module" | "typescript_component" | "django_view"
    理由: 同一种工具覆盖不同场景，不绑定单一框架
```

### 8.3 `map_tests`

```
原始参数: { pytest_args, targets }
调整建议:
  - 增加分类标准: ["adapter", "endpoint", "file_existence", "module", "unknown"]
  - 返回中区分 recommended_action: "keep" | "remove_with_target" | "review"
```

### 8.4 `verify_deletion_plan`

```
原始参数: { targets, keep, gates }
调整建议:
  - gates 增加 "project_specific" 类型，允许仓库的 STANDARDS.md 或 OpenSpec
    定义额外的门禁规则
  - 增加 dry_run 模式，仅输出判定不执行任何变更
  - 返回中增加 migration_notes（需要先迁移什么才能安全删除）
```

---

## 九、关键风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| 动态导入漏检 | 标记为 safe 的模块实际被条件导入 | 动态导入风险单独标记 `dynamic_import_risk: true`，不进入 `safe_to_remove` |
| 外部消费者不可见 | API 路径被外部系统调用但仓库内无 trace | 增加 `blocked_by_external_consumer_unknown` 分类，要求 telemetry 证据 |
| 扫描器环境不一致 | 外部 Python 脚本使用了不同的 venv | OpenDog 不管理 venv，由用户通过 `--scanner-python` 指定解释器路径 |
| 测试意图误判 | adapter 测试被分类为死测试 | `pytest_mapper` 分析测试文件的 import 目标（adapter vs API route），不是只看文件名 |
| 仓库治理规则冲突 | OpenDog 判定与项目 STANDARDS.md 冲突 | `doc_ownership_gate` 以项目自身治理文件为最高优先级 |

---

## 十、结论

**设计层面**：用户的方案是正确的。"证据分层 + 否决项"模型、"文件存在性 ≠ 功能进入运行时"的洞察、测试意图区分、三分类输出——这些都是真实工程判断。

**实现层面**：不能在 OpenDog Rust 中全部原生实现。语言特定的语义分析（Python AST、FastAPI runtime、pytest、TypeScript AST）需要外部运行时。推荐的架构是：

- **OpenDog 做证据聚合 + 门禁引擎 + 通用扫描器**（entrypoint、doc gate、URL pattern）
- **外部脚本做语言特定分析**（import graph、FastAPI routes、pytest、OpenAPI diff、frontend consumers）
- **OpenDog 通过子进程编排外部脚本**（复用已有的 `git` / `sh -lc` 调用模式）

**优先级**：Phase 1（OpenDog 框架 + 通用扫描器）应优先启动。这是语言无关的基础设施，可以在一个仓库上快速验证整个链路。Phase 2（外部扫描器）可以先用手工脚本验证，再标准化为独立包。

**关于"是否需要一套多信号证据链"**：需要。证据链框架本身就是核心价值，且应该实现在 OpenDog 内。这是语言无关的纯逻辑层——信号融合、置信度计算、门禁判定——正是 OpenDog 作为"决策辅助系统"应做的事。
