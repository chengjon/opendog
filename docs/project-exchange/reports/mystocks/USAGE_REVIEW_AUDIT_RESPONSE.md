# MyStocks 使用反馈审核响应

**审核源**: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md`
**辅助审核**: `/opt/claude/opendog/docs/project-exchange/reports/mystocks/OPENDOG_MCP_USAGE_REVIEW_2026-05-26_AUDIT.md`（Codex）
**审核者**: OPENDOG maintainer（Claude Code + GLM-5.1）
**日期**: 2026-05-26
**测试基线**: 1740 tests, 0 clippy warnings
**Shared issues**: `ODX-20260526-mcp-usage-review-hardening`, `ODX-20260527-retained-evidence-storage-governance`

## 总体评价

源报告质量高、定位准确。核心判断——"OPENDOG 定位为观察仪表盘和行动路由器，不替代 git/test/lint"——完全正确。所有 7 个技术发现均经代码验证为真实问题（6 个完全准确，1 个部分准确）。Codex 审计的 triage 排序整体合理，我对部分优先级有调整。

以下逐项给出审核结论和实施建议。

---

## F-1: daemon/schema 版本不一致 [源 P0]

**源报告位置**: 5.2 节 (line 115-135), 6 节 P0 (line 252-267)
**Codex 审计**: HIGH, 建议 P0
**代码验证**: ACCURATE

**现状**:

`src/storage/migrations.rs:31-37` 当 `current_version > SCHEMA_VERSION` 时返回：
```rust
"{} database schema version {} is newer than supported version {}"
```
无重启建议。`get_build_info` (`src/mcp/payloads/config_payloads.rs:12-52`) 不暴露 schema version 字段。

**审核结论**: 接受。改动小、收益高。

**实施计划**:

1. **错误消息丰富** — 在 `SchemaMigration` 错误中追加建议文本：
   - "Restart the daemon and MCP session with the current binary, then retry."
   - 不改错误类型，只丰富 message。

2. **build_info 字段扩展** — 在 `build_info_payload` 中增加：
   - `schema_version`: 当前 binary 支持的 schema 版本（`SCHEMA_VERSION` 常量）
   - `daemon_status`: 通过控制面探测 daemon 是否活跃（复用 `DaemonClient::new().list_projects()` 的连通性）

3. **MCP handler 层** — 当 handler 捕获 `SchemaMigration` 错误时，附加 `daemon_restart_required: true` 到错误 payload。

**涉及文件**: `storage/migrations.rs`, `mcp/payloads/config_payloads.rs`, `mcp/mod.rs`（handler 错误映射）
**预估工作量**: 小

---

## F-2: verification pipeline 可信度标记 [源 P0]

**源报告位置**: 5.4 节 (line 164-184), 6 节 P0 (line 268-278)
**Codex 审计**: HIGH, 建议 P0
**代码验证**: ACCURATE

**现状**:

`src/core/verification.rs:131-153` 使用 `sh -lc` 执行用户命令，直接将 `output.status.success()` 映射为 `"passed"` / `"failed"`。无管道检测、无输出内容分析。

**审核结论**: 接受，但分两阶段实施。Phase A 成本极低，Phase B 需要设计。

**实施计划**:

**Phase A — 记录时检测（不改历史 schema）**:

在 `RecordVerificationInput` 和 `ExecutedVerificationResult` 中增加可选字段：
- `pipeline_operators_detected: bool` — 命令字符串包含 `|`、`||`、`&&`、`;`、`> /dev/null`、`2>/dev/null` 时为 true
- `exit_code_masked_possible: bool` — 当 `pipeline_operators_detected && status == "passed"` 时为 true
- `suspicious_pass_signals: Vec<String>` — 当 status=passed 但 stdout/stderr 包含 `error TS`、`FAILED`、`Traceback`、`Error:`、`panic` 等模式时，记录匹配到的信号

这些字段写入现有 JSON summary 或新增列。不改变 `status` 字段本身的值——保持原始记录不变。

**Phase B — 消费侧集成**:

在 `get_verification_status` 和 guidance 的 verification gate 中，当 `exit_code_masked_possible` 为 true 时：
- verification trust level 从 `trusted` 降级为 `caution`
- guidance 的 recommended next action 建议用不含管道的方式重新运行

**涉及文件**: `core/verification.rs`, `storage/queries.rs`（新增列）, `storage/schema.rs`（migration）, `mcp/verification_evidence.rs`, `mcp/strategy.rs`（gate 逻辑）
**预估工作量**: Phase A 小，Phase B 中

---

## F-3: 大库 report/cleanup 超时 [源 P1]

**源报告位置**: 5.3 节 (line 137-163), 6 节 P1 (line 280-291)
**Codex 审计**: HIGH, 建议拆为 report protection + cleanup estimate-first 两个 P1
**代码验证**: ACCURATE

**现状**:

`src/core/report/time_window.rs:110-136` 的 `access_counts` 和 `modification_counts` 查询无 SQL `LIMIT`，全部行返回后才在 Rust 层 truncate（line 95: `files.truncate(limit.max(1))`）。在 8GB 数据库上全量 GROUP BY 导致超时。

cleanup dry-run（`src/core/retention/executor.rs`）先调用 `collect_storage_metrics`（轻量 PRAGMA），但对 snapshot scope 仍执行 `list_snapshot_run_ids_to_prune` + `count_snapshot_history_for_runs` 全量计数。

**审核结论**: 接受。拆为两个独立改进项。

**实施计划**:

**R-1: report SQL LIMIT**:

在 `access_counts` 和 `modification_counts` 的 SQL 末尾加 `LIMIT ?`，参数用调用方传入的 limit 值（已有）。同时：
- payload 中增加 `truncated: bool` 标记（当返回行数 == limit 时为 true）
- summary 的 COUNT 查询不受影响（已是标量结果），但可以在查询前检查 `PRAGMA page_count` 做快速预判

**R-2: cleanup estimate-first**:

cleanup dry-run 对 snapshot scope 增加快速路径：
- 先用 `SELECT COUNT(*) FROM snapshot_runs` 获取总数（标量，快）
- 如果总数 < threshold（如 100），正常执行详细 dry-run
- 如果总数 >= threshold，返回 estimate-only 模式，只给 scope 级别数量，不展开文件级明细
- payload 中加 `estimate_mode: "full" | "scope_counts_only"`

**涉及文件**: `core/report/time_window.rs`, `core/retention/executor.rs`, `storage/queries.rs`
**预估工作量**: R-1 小，R-2 中

---

## F-4: data-risk 路径分类噪声 [源 P1]

**源报告位置**: 5.5 节 (line 186-211), 6 节 P1 (line 292-307)
**Codex 审计**: MEDIUM, 建议统一路径分类
**代码验证**: ACCURATE

**现状**:

`src/mcp/mock_detection.rs:117-129` 的 `classify_path_kind` 只识别 4 种路径类型：`generated_artifact`、`test_only`、`runtime_shared`、`documentation`，其余为 `unknown`。`.claude/` 路径落入 `unknown`。

同时，`src/core/file_classification.rs` 已有更完善的分类器，把 `.claude`、`.cursor`、`.agents` 归为 `infrastructure`。但 data-risk 没有复用这个分类器。

mock candidate 的 review_priority 逻辑（`mock_detection.rs:408-414`）：`is_test_only` → medium, `generated_artifact` → low, 其余 → **high**。`.claude/settings.json` 落入 `unknown` → high，产生噪声。

**审核结论**: 接受。最小改动是让 data-risk 复用 file_classification，不需要建全新分类体系。

**实施计划**:

1. 在 `mock_detection.rs` 中引入 `file_classification::classify_file_path` 的结果
2. 当路径被归类为 `infrastructure`（含 `.claude/`、`.cursor/`、`.vscode/` 等）时：
   - `path_classification` 设为 `"infrastructure"`（而非 `unknown`）
   - `review_priority` 设为 `"low"`
3. 保持 `classify_path_kind` 对 source/test/docs 的判断不变，只增加 infrastructure 逃逸

**涉及文件**: `mcp/mock_detection.rs`（主要）, `core/file_classification.rs`（可能导出更多子类别）
**预估工作量**: 小

---

## F-5: unused/gate 行为验证 [源 MEDIUM]

**源报告位置**: 5.6 节 (line 213-249)
**Codex 审计**: MEDIUM, 建议增加回归测试
**代码验证**: ACCURATE — 当前 gate 逻辑正确

**现状**:

源报告确认 OPENDOG 对 mystocks 的决策完全合理：
- cleanup gate: blocked
- refactor gate: blocked
- destructive_change_recommended: false
- recommended_next_action: take_snapshot

这是正面验证，说明当前 advisory boundary 工作正常。

**审核结论**: 不需要代码改动。增加回归测试以确保未来不退化。

**实施计划**:

在 guidance 或 strategy 测试中增加 fixture：
- dirty worktree + stale verification + low activity coverage → cleanup gate blocked
- storage_maintenance flagged → deletion requires human confirmation
- 验证 payload 中不包含 `destructive_change_recommended: true`

**涉及文件**: `mcp/strategy.rs` 测试模块（现有）
**预估工作量**: 小

---

## F-6: CLI/MCP/文档能力面不一致 [源 P1]

**源报告位置**: 3 节 (line 67-78), 6 节 P1 (line 309-323)
**Codex 审计**: MEDIUM
**代码验证**: ACCURATE

**具体不一致**:

| 能力 | CLI | MCP | 文档 |
|------|-----|-----|------|
| decision brief | `opendog decision-brief` | `get_guidance(detail=decision)` | 功能介绍写"两个 MCP 工具"，误导 |
| verify_deletion_plan | 无 | `verify_deletion_plan` | 无 CLI 说明 |
| cleanup-data | 有 | 无（operator-only） | 正确 |

**审核结论**: 这是文档问题，不需要代码改动。`get_decision_brief` 不暴露为独立 MCP 工具是正确的架构选择（避免能力碎片化），只需要文档澄清。

**实施计划**:

1. 在 `docs/mcp-tool-reference.md` 中明确 `get_decision_brief` 是 `get_guidance(detail=decision)` 的内部路由，不是独立工具
2. 在 `docs/opendog-feature-introduction.md` 中修正"两个 MCP 工具"的描述
3. 在 `CLAUDE.md` 的 MCP Tools 列表中补注 verify_deletion_plan 为 MCP-only
4. 考虑生成静态 capability matrix（可从 tool_inventory + CLI clap 定义自动生成）

**涉及文件**: 纯文档
**预估工作量**: 小

---

## F-7: MCP host 接入诊断 [源 P2]

**源报告位置**: 5.1 节 (line 100-113), 6 节 P2 (line 324-338)
**Codex 审计**: LOW
**代码验证**: 准确但部分超出 OPENDOG 职责

**审核结论**: 部分接受。OPENDOG 能诊断 server/daemon/config 状态，但无法知道 AI host 是否暴露了 tools。`host_tools_visible` 应作为用户侧 checklist 文档，不在 OPENDOG 代码中实现。

**实施计划**:

1. 在 `get_build_info` 中加入 `daemon_running: bool`（通过 DaemonClient 探测）
2. 在文档中增加"接入检查清单"section，列出用户应验证的步骤
3. 不实现 `host_tools_visible` 自动检测

**涉及文件**: `mcp/payloads/config_payloads.rs`, 文档
**预估工作量**: 小

---

## 实施优先级排序

综合源报告、Codex 审计、代码验证和改动成本后的最终排序：

| 优先级 | 编号 | 工作项 | 源优先级 | 改动量 | 收益 |
|--------|------|--------|----------|--------|------|
| **1** | F-1 | schema 兼容性诊断 | P0 | 小 | 高 — 直接消除 MCP 不可用的困惑 |
| **2** | F-4 | data-risk 路径分类降噪 | P1 | 小 | 高 — 消除高频噪声误报 |
| **3** | F-2A | verification trust Phase A | P0 | 小 | 中 — 记录可疑验证信号 |
| **4** | F-6 | 文档能力面修正 | P1 | 小 | 中 — 消除接口混淆 |
| **5** | F-3-R1 | report SQL LIMIT | P1 | 小 | 高 — 大库查询保护 |
| **6** | F-5 | gate 回归测试 | MEDIUM | 小 | 中 — 防止 advisory boundary 退化 |
| **7** | F-7 | daemon_running 诊断 + 接入文档 | P2 | 小 | 低 |
| **8** | F-3-R2 | cleanup estimate-first | P1 | 中 | 中 — 大库 dry-run 保护 |
| **9** | F-2B | verification trust Phase B | P0 | 中 | 中 — gate 集成 |

排序原则：小改动高收益优先（F-1, F-4），然后补 Phase A 记录层（F-2A），再做需要 schema migration 或多文件协调的中等改动（F-3-R2, F-2B）。

---

## 实施后状态更新 [2026-05-27]

本响应中的 9 个工作项已经全部完成代码或文档层面的可接受实现，并在 `OPENDOG_MCP_USAGE_REVIEW_2026-05-26_AUDIT.md` 的 `Implementation Follow-Up Status` 章节逐项登记；最终跟进登记提交为 `e1bb6be`。

| 编号 | 状态 | 证据 |
|------|------|------|
| F-1 | 已实现 | `7156ac6` |
| F-4 | 已实现 | `f14249e` |
| F-2A | 已实现 | `ab68fb4` |
| F-6 | 已实现并后续同步 retention/rollup 文档 | `ffa6fcf`, `11ea181`, `6b5a6d7` |
| F-3-R1 | 已实现 | `563a569` |
| F-5 | 已实现 | `d88d7f1` |
| F-7 | 已按 OPENDOG 可观测边界实现 | `0559af6` |
| F-3-R2 | 已实现并扩展为 retained evidence storage governance | `b429b89`, `1efa8e2` |
| F-2B | 已实现 | `7f25843` |

F-3-R2 后续扩展的 retained evidence 治理已经落地：

- CLI: `opendog report rollup --json`
- MCP: `get_activity_rollups`
- 存储: `activity_daily_rollups`
- 运维: `docs/storage-retention-runbook.md`
- mystocks 执行证据: `storage-retention-dry-run-2026-05-27.md`

mystocks 项目已应用 14 天 activity retention policy，并完成一次真实 cleanup + vacuum + WAL checkpoint。该操作删除 OPENDOG retained activity evidence 中的旧明细行，并保留 daily rollup 汇总；不删除 mystocks 项目源码文件。

---

## 未接受的建议

| 建议 | 原因 |
|------|------|
| `host_tools_visible` 自动检测 | 超出 OPENDOG 职责，需 AI host 配合 |
| 全新统一路径分类体系 | 过度工程，复用 file_classification 足够 |
| capability matrix 自动生成 | 长期有价值但非紧急，可后置 |
| `opendog doctor mcp` 独立子命令 | 过重，`get_build_info` + 文档 checklist 更轻量 |

---

## 源报告正面反馈确认

源报告对以下方面给出了正面评价，经审核确认属实：

1. **summary-first 改善** — `get_stats` 从超大 payload 降到 30KB，`get_guidance(detail=decision)` 从 serialization error 修复到 72KB 可正常返回。
2. **advisory boundary 正确** — dirty worktree + stale evidence + low coverage 场景下，gate 正确 blocked，不推荐破坏性操作。
3. **MCP 可用性** — 26 工具 + 4 resources，initialize/tools/list/resources/list 全部正常。
4. **定位清晰** — 源报告准确理解 OPENDOG 不替代 git/test/lint，定位为观察+路由。

这些是产品定位和实现质量的正面信号，不需要改动，但应作为 regression baseline 保护。
