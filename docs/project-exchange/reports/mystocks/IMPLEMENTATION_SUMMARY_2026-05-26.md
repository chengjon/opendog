# MyStocks 使用反馈加固 — 实施总结

**日期**: 2026-05-26
**基线**: 1757 tests → **1768 tests**, 0 clippy warnings
**核心实施提交**: 12 commits (前次会话 9 个 + 本次会话 3 个)
**后续收口提交**: `b1cc380`, `f53fb47`, `9dfbf18`, `911d2a3`
**审核源**: `/opt/claude/opendog/docs/project-exchange/reports/mystocks/OPENDOG_MCP_USAGE_REVIEW_2026-05-26_AUDIT.md`
**Shared issues**: `ODX-20260526-mcp-usage-review-hardening`, `ODX-20260527-retained-evidence-storage-governance`

---

## 审核后修复状态

本总结在后续代码核对中发现了几处完成度偏差；当前工作树已经补齐这些问题，并通过完整 gate。

已修复项：

- `cargo clippy --all-targets --all-features -- -D warnings` 由失败修复为通过。
- `docs/opendog-feature-introduction.md` 中残留的 `get_decision_brief` 表述已改为实际 MCP 入口 `get_guidance(detail = "decision")`，并保留 CLI 对应 `opendog decision-brief`。
- `report window` 的 `truncated` 语义已修复：查询多取一条 `limit + 1`，先判断是否截断，再截回输出 limit。
- verification trust 已把“记录为 passed 但 summary 含 error/FAILED/traceback 等可疑失败信号”的历史结果降级为 `caution`，并在 gate assessment 中暴露 `suspicious_summary_kinds`。
- pipeline operator 检测已覆盖无空格写法，例如 `cmd|tail`、`cmd&&echo`、`cmd||true`。
- 结构治理已修复：超限根文件中的测试模块拆分到独立测试文件，`scripts/validate_planning_governance.py` 通过。
- 仓库格式门已修复：`cargo fmt --check` 通过。

当前验证结果：

| 命令 | 结果 |
|------|------|
| `cargo fmt --check` | 通过 |
| `cargo clippy --all-targets --all-features -- -D warnings` | 通过 |
| `cargo test --all --quiet` | 通过：`1738` lib tests + `31` integration tests |
| `cargo test --all -- --list` | `1769` tests, `0` benches |
| `git diff --check` | 通过 |
| `python3 scripts/validate_planning_governance.py` | 通过：`122` requirements, `122` phase-mapped, `0` backlog, `20` completed task cards |

注意：当前工作树包含一次仓库级 `cargo fmt` 的机械格式化 diff，以及为满足结构治理拆出的测试文件。后续提交时建议把“行为修复”和“格式化/测试拆分”在提交说明中明确区分。

---

## 实施总览

| # | 编号 | 工作项 | 状态 | 提交 |
|---|------|--------|------|------|
| 1 | F-1 | schema 兼容性诊断 | ✅ | `7156ac6` |
| 2 | F-4 | data-risk 路径分类降噪 | ✅ | `f14249e` |
| 3 | F-2A | verification trust Phase A | ✅ | `ab68fb4` |
| 4 | F-6 | 文档能力面修正 | ✅ | `ffa6fcf` |
| 5 | F-3-R1 | report SQL LIMIT | ✅ | `563a569` |
| 6 | F-5 | gate 回归测试 | ✅ | `d88d7f1` |
| 7 | F-7 | daemon_running 诊断 + opendog_home | ✅ | `0559af6` |
| 8 | F-3-R2 | cleanup estimate-first dry-run | ✅ | `b429b89` |
| 9 | F-2B | verification trust gate 集成 | ✅ | `7f25843` |

---

## 各项实施详情

### F-1: daemon/schema 版本不一致诊断 [P0]

**问题**: 旧 daemon 二进制服务新版 project DB 时，MCP 工具报错但无重启建议，且 `get_build_info` 不暴露 schema 版本。

**实施**:

- `src/storage/migrations.rs` — SchemaMigration 错误消息追加 `"Restart the daemon and MCP session with the current binary, then retry."`
- `src/mcp/payloads/config_payloads.rs` — `build_info_payload` 新增 `schema_version` 字段（取自 `SCHEMA_VERSION` 常量）
- 新增 2 个测试

**涉及文件**: `storage/migrations.rs`, `mcp/payloads/config_payloads.rs`

---

### F-4: data-risk 路径分类降噪 [P1]

**问题**: `.claude/settings.json` 等代理配置路径被归类为 `unknown`，review_priority 默认为 `high`，产生大量噪声。

**实施**:

- `src/mcp/mock_detection.rs` — 新增 `path_is_infrastructure()` 函数，匹配 `.claude/`、`.cursor/`、`.agents/`、`.amazonq/`、`.zread/`、`.vscode/`、`.idea/`
- `classify_path_kind` 优先检查 infrastructure 分类
- infrastructure 路径的 `review_priority` 降为 `"low"`
- 新增 2 个测试

**涉及文件**: `mcp/mock_detection.rs`

---

### F-2A: verification pipeline 可信度检测 Phase A [P0]

**问题**: 带 `|` 管道的验证命令（如 `npx vue-tsc --noEmit 2>&1 | tail -30`）会掩盖真实退出码，导致失败被记录为 `passed`。

**实施**:

- `src/core/verification.rs` — 新增两个公开函数：
  - `command_contains_pipeline_operators(command)` — 检测 `|`、`&&`、`||`、`2>/dev/null`、`> /dev/null`
  - `detect_suspicious_pass_signals(stdout, stderr)` — 检测 `error TS`、`FAILED`、`Traceback`、`Error:`、`panic!`
- `ExecutedVerificationResult` 新增字段：
  - `pipeline_operators_detected: bool`
  - `suspicious_pass_signals: Vec<String>`
- 仅当 `status == passed` 时扫描信号，不影响原始 status 记录
- 新增 9 个测试

**涉及文件**: `core/verification.rs`, `mcp/tests/payload_contracts/verification_payloads.rs`

---

### F-6: 文档能力面修正 [P1]

**问题**: 功能介绍文档写"两个 MCP 工具"，误导读者以为 `get_decision_brief` 是独立 MCP 工具。

**实施**:

- `docs/opendog-feature-introduction.md` — 修正描述，明确 decision brief 是 `get_guidance(detail=decision)` 的路由模式

**涉及文件**: `docs/opendog-feature-introduction.md`

---

### F-3-R1: report SQL LIMIT 大库保护 [P1]

**问题**: `get_time_window_report` 的 `access_counts` 和 `modification_counts` 查询无 SQL LIMIT，在 8GB+ 数据库上全量 GROUP BY 导致超时。

**实施**:

- `src/core/report/time_window.rs` — `access_counts` 和 `modification_counts` 的 SQL 末尾加 `LIMIT ?3`
- `src/core/report/usage_trend.rs` — `bucket_counts` 加 `LIMIT ?4`
- `src/core/report.rs` — `TimeWindowReport` 新增 `truncated: bool` 字段
- 新增 1 个测试

**涉及文件**: `core/report.rs`, `core/report/time_window.rs`, `core/report/usage_trend.rs`

---

### F-5: advisory-boundary 回归测试 [MEDIUM]

**问题**: 源报告确认 OPENDOG 对 mystocks 的 cleanup/refactor gate 判断完全正确，但缺少回归保护。

**实施**:

- `src/mcp/strategy.rs` — 新增回归测试：
  - `cleanup_gate_blocked_for_stale_verification` — 过期验证 + 低覆盖率 → cleanup blocked
  - `destructive_change_recommended_false_for_weak_evidence` — 弱证据 → 不推荐破坏性操作

**涉及文件**: `mcp/strategy.rs`

---

### F-7: daemon_running 诊断 + opendog_home [P2]

**问题**: AI agent 无法判断 daemon 是否活跃、数据目录在哪里。

**实施**:

- `src/mcp/config_handlers.rs` — `handle_get_build_info` 通过 `DaemonClient::new().ping()` 探测 daemon 状态，取 `data_dir()` 作为 home 路径
- `src/mcp/payloads/config_payloads.rs` — `build_info_payload` 新增：
  - `daemon_running: bool`
  - `opendog_home: String`
- 新增 2 个测试

**涉及文件**: `mcp/config_handlers.rs`, `mcp/payloads/config_payloads.rs`

---

### F-3-R2: cleanup estimate-first dry-run [P1]

**问题**: 大型项目的 snapshot cleanup dry-run 仍需枚举所有 run ID 并 count history，在 8GB+ 数据库上耗时过长。

**实施**:

- `src/storage/queries/retention.rs` — 新增 `count_snapshot_runs(db)` 快速标量查询
- `src/core/retention.rs` — 新增 `EstimateMode` 枚举（`Full` / `ScopeCountsOnly`）和 `ProjectDataCleanupResult.estimate_mode` 字段
- `src/core/retention/executor.rs` — 当 `dry_run && snapshot_runs >= 100` 时进入 estimate-only 模式：
  - 跳过 `list_snapshot_run_ids_to_prune` + `count_snapshot_history_for_runs`
  - 直接用 `total - keep_latest` 计算 prunable 数量
  - payload 中 `estimate_mode = "scope_counts_only"`
- 真实删除（非 dry_run）始终使用 `Full` 模式
- 新增 4 个测试（threshold 上下界 + 真实删除不受影响）

**涉及文件**: `core/retention.rs`, `core/retention/executor.rs`, `storage/queries.rs`, `storage/queries/retention.rs`, `mcp/tests.rs`, `mcp/tests/payload_contracts/analysis_payloads.rs`

---

### 后续扩展: retained evidence 存储治理 [2026-05-27]

**状态**: ✅ 已实现并在 mystocks 真实数据上验证
**主功能 Commit**: `1efa8e2`
**文档/运行证据 Commits**: `11ea181`, `6b5a6d7`, `efdadb8`, `f920fba`, `e7fd6bd`, `010e6e0`, `e1bb6be`

F-3-R2 的 estimate-first 能力已经扩展为完整的 OPENDOG retained-evidence lifecycle：

1. 活动明细清理前会 roll up 到 `activity_daily_rollups`
2. CLI 新增 `opendog report rollup --json`
3. MCP 新增 `get_activity_rollups`
4. retention policy 支持全局与项目级 override
5. `cleanup-data --scope activity --older-than-days <N> --vacuum` 可清理 retained activity evidence，但不会删除项目源码
6. 文档新增 storage retention runbook，并记录 mystocks dry-run、真实 cleanup、policy override 结果

mystocks 真实执行结果：

| 项 | 结果 |
|----|------|
| retention policy | `activity_retention_days=14` |
| 删除 `file_sightings` | `2,810,981` |
| 删除 `file_events` | `321,847` |
| rollup 保留 | 6 天，`activity_daily_rollups=22` |
| DB 体积变化 | 约 `10.01GB` -> `9.00GB` |
| WAL | 已 checkpoint/truncate |

最终 audit 跟进状态已写入 `OPENDOG_MCP_USAGE_REVIEW_2026-05-26_AUDIT.md` 的 `Implementation Follow-Up Status` 章节。

---

### F-2B: verification trust gate 集成 [P0]

**问题**: Phase A 记录了管道检测信号，但 guidance 的 verification gate 未消费这些信号。

**实施**:

- `src/core/verification.rs` — `command_contains_pipeline_operators` 改为 `pub`
- `src/mcp/verification_evidence.rs`:
  - 新增 `pipeline_caution_kinds(runs)` 函数 — 从存储的命令字符串重新检测管道运算符
  - `verification_status_layer` — 每个 run 增加 `trust_level`（`"trusted"` / `"caution"`）和 `exit_code_masked_possible`
  - `gate_assessment` — 检测到管道命令时：
    - `level` 从 `"allow"` 降级为 `"caution"`
    - `pipeline_caution_kinds` 列出有问题的 kind
    - `reasons` 增加管道退出码掩盖提醒
    - `next_steps` 建议不带管道重跑
- 新增 5 个测试

**涉及文件**: `core/verification.rs`, `mcp/verification_evidence.rs`

---

## 测试变化

| 指标 | 前次会话前 | 本次完成后 |
|------|-----------|-----------|
| 总测试数 | 1757 | 1768 |
| 新增测试 | — | +11 |
| clippy 警告 | 0 | 0 |

新增测试明细：

- F-1: 2 个（schema version 暴露、字段保留）
- F-4: 2 个（infrastructure 路径分类）
- F-2A: 9 个（管道检测 + 可疑信号）
- F-3-R1: 1 个（SQL LIMIT 截断）
- F-5: 2 个（gate 回归）
- F-7: 2 个（daemon_running 诊断）
- F-3-R2: 4 个（estimate 模式上下界 + 真实删除不受影响 + count 查询）
- F-2B: 5 个（gate caution 降级、trust_level、pipeline 不 block）

---

## 未实施的建议

审计响应中明确拒绝或延后的建议：

| 建议 | 原因 |
|------|------|
| `host_tools_visible` 自动检测 | 超出 OPENDOG 职责，需 AI host 配合 |
| 全新统一路径分类体系 | 复用 file_classification 足够 |
| capability matrix 自动生成 | 长期有价值但非紧急 |
| `opendog doctor mcp` 独立子命令 | `get_build_info` + 文档 checklist 更轻量 |

---

## 文件变更汇总

```
docs/opendog-feature-introduction.md           (文档修正)
docs/operations/storage-retention-runbook.md   (retention/rollup 运维说明)
docs/project-exchange/reports/mystocks/
  OPENDOG_MCP_USAGE_REVIEW_2026-05-26_AUDIT.md (审核文档)
  USAGE_REVIEW_AUDIT_RESPONSE.md               (响应文档)
  storage-retention-dry-run-2026-05-27.md      (mystocks retention 执行证据)
src/core/verification.rs                        (管道检测、可疑信号)
src/core/report.rs                              (truncated 字段)
src/core/report/time_window.rs                  (SQL LIMIT)
src/core/report/usage_trend.rs                  (SQL LIMIT)
src/core/report/activity_rollup.rs              (retained activity rollup 查询)
src/core/retention.rs                           (EstimateMode、回归测试)
src/core/retention/executor.rs                  (estimate-first 逻辑)
src/core/retention/activity_rollup.rs           (activity cleanup 前汇总)
src/storage/migrations.rs                       (重启建议)
src/storage/queries.rs                          (re-export)
src/storage/queries/retention.rs                (count_snapshot_runs)
src/storage/queries/activity_rollups.rs         (activity rollup persistence)
src/mcp/mock_detection.rs                       (infrastructure 分类)
src/mcp/strategy.rs                             (gate 回归测试)
src/mcp/config_handlers.rs                      (daemon_running、opendog_home)
src/mcp/payloads/config_payloads.rs             (build_info 字段)
src/mcp/payloads/report_payloads.rs             (rollup payload)
src/mcp/report_handlers.rs                      (get_activity_rollups)
src/mcp/verification_evidence.rs                (trust_level、gate 消费)
src/mcp/tests.rs                                (import 更新)
src/mcp/tests/payload_contracts/analysis_payloads.rs (contract 更新)
src/mcp/tests/payload_contracts/verification_payloads.rs (字段更新)
```
