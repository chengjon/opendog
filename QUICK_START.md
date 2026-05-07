# OPENDOG Quick Start

本文是基于当前仓库实现状态整理的上手指南，目标是回答三个问题：

1. OPENDOG 现在已经实现了什么。
2. 作为 CLI / daemon / MCP 工具，应该怎么用。
3. 日常使用时有哪些边界和容易踩坑的地方。

如果你只想先跑起来，优先看“最短路径”；如果你想理解每一步为什么这么跑，再看“推荐上手路径”。

## 0. 最短路径

如果你只想先把 OPENDOG 跑起来，先抄这组命令：

- 把 `demo` 换成你的项目 ID
- 把 `/abs/path/to/project` 换成项目的真实绝对路径

终端 1：

```bash
cargo run -- daemon
```

终端 2：

```bash
cargo run -- create --id demo --path /abs/path/to/project
cargo run -- snapshot --id demo
cargo run -- start --id demo
cargo run -- list
cargo run -- decision-brief --project demo
cargo run -- stop --id demo
```

如果你想拿机器可读结果，把倒数第二条换成下面这条，最后仍然执行 `stop`：

```bash
cargo run -- decision-brief --project demo --json
```

如果这次只是烟测，不准备长期保留这个项目记录，可以在最后额外执行：

```bash
cargo run -- delete --id demo
```

这只会删除 OPENDOG 记录和它自己的数据库，不会删除你的源码目录。

## 1. OPENDOG 是什么

OPENDOG 不是单纯的文件监控器，它当前是一个面向 AI 工作流的多项目观测与决策辅助系统：

- 观察层：为多个项目建立快照、持续监控文件活动、记录统计证据
- 服务层：通过 CLI、daemon、本地控制面和 MCP 对外提供统一能力
- 决策层：给 AI 或操作者提供 guidance、decision brief、verification evidence、data-risk review 等辅助信息

当前实现已经覆盖这些能力：

- 多项目注册与隔离
- SQLite 持久化
- 全量快照
- `/proc` + `inotify` 混合监控
- 文件访问统计、未访问文件识别
- 时间窗报告、快照对比、使用趋势
- test/lint/build 验证记录与执行
- mock / hardcoded pseudo-data 风险检测
- workspace 级项目优先级和 AI 指导
- 可移植导出（JSON / CSV）
- OPENDOG 自身留存证据的清理和存储维护
- MCP stdio 服务

一个重要边界：

- `git`、测试、lint、build 仍然是外部真理源；OPENDOG 输出是“证据和决策辅助”，不是最终事实裁决器

## 2. 运行环境与数据目录

### 运行环境

当前实现明显偏向 Linux / 类 Unix 环境，原因包括：

- 监控依赖 `/proc`
- daemon 本地控制面使用 Unix socket
- 文件变化检测依赖 `inotify`

如果你在 WSL 下使用：

- WSL2 明显比 WSL1 更合适
- 项目目录如果位于 `/mnt/...` 的 Windows 挂载盘上，监控可靠性会下降

### 项目路径要求

- `create --path` 必须是绝对路径
- 路径必须已经存在，且是目录
- `id` 只能包含字母、数字、`-`、`_`
- `id` 最长 64 个字符

### 数据目录

默认数据写入 `~/.opendog/`：

```text
~/.opendog/
├── config.json
└── data/
    ├── daemon.pid
    ├── daemon.sock
    ├── registry.db
    └── projects/
        └── <project_id>.db
```

说明：

- `config.json` 只有在你首次写入全局配置时才会创建
- 运行时还可能出现 `*.db-wal` 和 `*.db-shm` 之类的 SQLite sidecar 文件

## 3. 构建方式

本文命令示例默认使用开发态：

```bash
cargo run -- <subcommand> ...
```

如果你已经构建 release，也可以改用：

```bash
cargo build --release
./target/release/opendog <subcommand> ...
```

补充：

- 很多查询、报告、配置、验证相关命令都支持 `--json`，如果你要把输出接给脚本或 AI，优先考虑加上这个参数
- 最常用的几个机器可读入口通常是：`decision-brief --json`、`agent-guidance --json`、`report window --json`、`verification --json`

## 4. 推荐上手路径

这是最实用、也最接近日常使用的路径：先启动 daemon，再从另一个终端用 CLI 访问它。

本指南已经按下面这条主路径做过一次烟测：

```text
daemon -> create -> snapshot -> start -> list -> decision-brief -> stop
```

烟测范围说明：

- 已实际走通的是 CLI + daemon 主链路
- MCP 部分在本指南里主要是入口说明，不覆盖具体 AI client 的接线与握手细节

### 终端 1：启动 daemon

```bash
cargo run -- daemon
```

daemon 启动后会：

- 加载已注册项目
- 尝试恢复并启动后台 monitor
- 在本地控制面上接受 CLI / MCP 请求

### 终端 2：注册项目并建立基线

```bash
cargo run -- create --id demo --path /abs/path/to/project
# 如果仓库里有很多元目录，或你的活动主要来自 cargo/git/bash/sh：
# cargo run -- config set-project --id demo --ignore-pattern .git --ignore-pattern node_modules --ignore-pattern dist --ignore-pattern target --ignore-pattern __pycache__ --ignore-pattern .cache --ignore-pattern build --ignore-pattern .next --ignore-pattern .nuxt --ignore-pattern vendor --ignore-pattern .venv --ignore-pattern venv --ignore-pattern .tox --ignore-pattern .mypy_cache --ignore-pattern .pytest_cache --ignore-pattern .gradle --ignore-pattern .idea --ignore-pattern .vscode --ignore-pattern '*.pyc' --ignore-pattern .DS_Store --ignore-pattern .worktrees --ignore-pattern .planning --ignore-pattern .omc --ignore-pattern .gitnexus --process claude --process codex --process node --process python --process python3 --process gpt --process glm --process cargo --process git --process bash --process sh
cargo run -- snapshot --id demo
cargo run -- start --id demo
```

此时 `start` 会优先请求 daemon 启动后台监控，因此命令会立即返回。

### 读取基础观察结果

```bash
cargo run -- list
cargo run -- stats --id demo
cargo run -- unused --id demo
```

如果刚刚 `start` 完就看 `stats`，结果可能仍然很平，甚至还是 0 accessed。先让项目产生一段真实编辑、测试或构建活动，再看统计更可靠。

### 进入 AI 决策辅助入口

```bash
cargo run -- decision-brief --project demo
cargo run -- agent-guidance --project demo --top 3
```

### 停止后台监控

```bash
cargo run -- stop --id demo
```

## 5. 不启动 daemon 时怎么用

你也可以直接本地运行，不经过 daemon：

```bash
cargo run -- create --id demo --path /abs/path/to/project
cargo run -- snapshot --id demo
cargo run -- start --id demo
```

但这里有一个关键区别：

- 没有 daemon 时，`start` 会以前台模式运行，并阻塞当前终端，直到你按 `Ctrl+C`
- `stop` 只用于停止 daemon-managed monitor，不能停止这个前台 `start`

因此：

- 临时试验可以直接 `start`
- 日常持续运行更推荐 `daemon + start/stop`

## 6. 常见命令按场景使用

### 6.1 项目注册与监控

```bash
cargo run -- create --id demo --path /abs/path/to/project
cargo run -- snapshot --id demo
cargo run -- start --id demo
cargo run -- stop --id demo
cargo run -- list
cargo run -- delete --id demo
```

适用场景：

- `create`：把项目注册到 OPENDOG
- `snapshot`：建立或刷新文件基线
- `start`：开始持续观测
- `stop`：停止 daemon 托管的后台监控
- `list`：查看已注册项目
- `delete`：删除项目及其 OPENDOG 数据

### 6.2 统计与报告

基础统计：

```bash
cargo run -- stats --id demo
cargo run -- unused --id demo
```

时间窗报告：

```bash
cargo run -- report window --id demo --window 24h --limit 10
cargo run -- report window --id demo --window 7d --limit 20
```

快照对比：

```bash
cargo run -- report compare --id demo
cargo run -- report compare --id demo --base-run-id 10 --head-run-id 12 --limit 50
```

`report compare` 至少需要两次 `snapshot`。首次使用时，先在变更前后各做一次 snapshot，再比较。

使用趋势：

```bash
cargo run -- report trend --id demo --window 7d --limit 10
cargo run -- report trend --id demo --window 30d --limit 20
```

当前支持的时间窗：

- `24h`
- `7d`
- `30d`

### 6.3 AI 指导、决策骨架与数据风险

如果你只想先问一句“我现在应该做什么”，优先从这两个入口开始：

```bash
cargo run -- decision-brief --project demo
cargo run -- agent-guidance --project demo --top 5
```

它们的区别：

- `decision-brief`：更像给 AI 的稳定决策包，适合作为统一入口
- `agent-guidance`：更像“下一步建议清单”

项目级数据风险：

```bash
cargo run -- data-risk --id demo
cargo run -- data-risk --id demo --candidate-type hardcoded --min-review-priority high --limit 20
```

workspace 级项目优先级：

```bash
cargo run -- workspace-data-risk
cargo run -- workspace-data-risk --candidate-type all --min-review-priority medium --project-limit 10
```

当前支持的筛选值：

- `candidate-type`: `all` / `mock` / `hardcoded`
- `min-review-priority`: `low` / `medium` / `high`

如果仓库本身包含很多元目录或工作树痕迹，先调整项目级 ignore patterns，再解读 `unused` 和 `data-risk`。

### 6.4 验证记录与执行

查询最近验证结果：

```bash
cargo run -- verification --id demo
```

直接执行并记录：

```bash
cargo run -- run-verification --id demo --kind test --command "cargo test"
cargo run -- run-verification --id demo --kind lint --command "cargo clippy --all-targets --all-features -- -D warnings"
cargo run -- run-verification --id demo --kind build --command "cargo build --release"
```

如果验证命令是在外部跑的，也可以手动录入：

```bash
cargo run -- record-verification --id demo --kind test --status passed --command "cargo test" --exit-code 0 --summary "all tests passed"
```

当前 `kind` 只接受：

- `test`
- `lint`
- `build`

推荐顺序：

1. `decision-brief` 或 `agent-guidance`
2. `verification`
3. `data-risk`
4. 再决定是否做 cleanup / refactor / 大范围修改

说明：

- `run-verification` 和 `record-verification` 的主要作用是记录验证证据
- 验证证据会出现在 `verification` / `decision-brief` / `agent-guidance` 里
- 它不保证一定让 `stats` 立刻出现可见 activity；两者是相关但不等价的证据流

### 6.5 导出

导出统计证据：

```bash
cargo run -- export --id demo --format json --view stats --output /tmp/demo-stats.json
cargo run -- export --id demo --format csv --view unused --output /tmp/demo-unused.csv
cargo run -- export --id demo --format csv --view core --min-access-count 5 --output /tmp/demo-core.csv
```

当前支持：

- `format`: `json` / `csv`
- `view`: `stats` / `unused` / `core`

其中：

- `stats`：全量统计视图
- `unused`：未访问文件视图
- `core`：按访问次数筛出的核心文件视图

### 6.6 配置

查看全局默认配置：

```bash
cargo run -- config show
```

查看某个项目的有效配置：

```bash
cargo run -- config show --id demo
```

设置全局默认忽略项和进程白名单：

```bash
cargo run -- config set-global --ignore-pattern .git --ignore-pattern node_modules --ignore-pattern dist --ignore-pattern target --ignore-pattern __pycache__ --ignore-pattern .cache --ignore-pattern build --ignore-pattern .next --ignore-pattern .nuxt --ignore-pattern vendor --ignore-pattern .venv --ignore-pattern venv --ignore-pattern .tox --ignore-pattern .mypy_cache --ignore-pattern .pytest_cache --ignore-pattern .gradle --ignore-pattern .idea --ignore-pattern .vscode --ignore-pattern '*.pyc' --ignore-pattern .DS_Store --ignore-pattern coverage --ignore-pattern .turbo --process claude --process codex --process node --process python --process python3 --process gpt --process glm --process cargo --process pytest
```

设置项目级覆写：

```bash
cargo run -- config set-project --id demo --ignore-pattern .git --ignore-pattern node_modules --ignore-pattern dist --ignore-pattern target --ignore-pattern __pycache__ --ignore-pattern .cache --ignore-pattern build --ignore-pattern .next --ignore-pattern .nuxt --ignore-pattern vendor --ignore-pattern .venv --ignore-pattern venv --ignore-pattern .tox --ignore-pattern .mypy_cache --ignore-pattern .pytest_cache --ignore-pattern .gradle --ignore-pattern .idea --ignore-pattern .vscode --ignore-pattern '*.pyc' --ignore-pattern .DS_Store --ignore-pattern generated --ignore-pattern fixtures/tmp --process claude --process codex --process node --process python --process python3 --process gpt --process glm --process cargo --process rust-analyzer
```

如果 daemon 正在管理该项目，修改后可显式 reload：

```bash
cargo run -- config reload --id demo
```

说明：

- 全局默认配置保存在 `~/.opendog/config.json`
- 项目级配置保存在注册表和项目记录里
- 默认忽略项已经包含 `.git`、`node_modules`、`dist`、`target`、`__pycache__` 等常见目录
- 默认进程白名单已经包含 `claude`、`codex`、`node`、`python`、`python3`、`gpt`、`glm`
- `config set-global ...` 会重写全局默认列表，不是对默认值做追加
- `config set-project --ignore-pattern ...` 也会覆盖该项目的 ignore 列表，不是在默认列表后面自动追加
- 默认进程白名单并不包含 `cargo`、`git`、`bash`、`sh`
- `config set-project --process ...` 会覆盖该项目的 `process whitelist`，不是在默认列表后面自动追加
- 如果你只是想给单个项目补充额外 ignore 或 process，优先使用 `config set-project`
- 如果你既想保留默认 ignore，又想增加 `.worktrees`、`.planning` 这类目录，要把默认项和新增项一起写出来
- 如果你既想保留默认项，又想增加 `cargo`、`git`、`bash`、`sh`，要把这些默认项和新增项一起写出来
- 如果你想撤销项目级 ignore 覆盖并重新继承全局默认，可以使用 `cargo run -- config set-project --id demo --inherit-ignore-patterns`
- 如果你想撤销项目级覆盖并重新继承全局默认，可以使用 `cargo run -- config set-project --id demo --inherit-process-whitelist`
- 如果你想看当前完整的有效 ignore patterns 和 process whitelist，直接运行 `cargo run -- config show --id demo`

### 6.7 清理 OPENDOG 留存数据

`cleanup-data` 清理的是 OPENDOG 自己保留的证据，不是你的源码文件。

先 dry-run：

```bash
cargo run -- cleanup-data --id demo --scope activity --older-than-days 14 --dry-run
```

再执行真实清理：

```bash
cargo run -- cleanup-data --id demo --scope all --older-than-days 30 --keep-snapshot-runs 10
```

只有在大量清理后才建议考虑 `vacuum`：

```bash
cargo run -- cleanup-data --id demo --scope all --older-than-days 30 --keep-snapshot-runs 10 --vacuum
```

当前支持的 scope：

- `activity`
- `snapshots`
- `verification`
- `all`

## 7. MCP 用法

如果你是把 OPENDOG 接到 AI 客户端里，用的是 stdio MCP 入口：

```bash
cargo run -- mcp
```

MCP 侧的主入口建议是：

- `get_guidance`

常用工具分组如下：

- 基础控制：`create_project`、`take_snapshot`、`start_monitor`、`stop_monitor`、`list_projects`、`delete_project`
- 观察与报告：`get_stats`、`get_unused_files`、`get_time_window_report`、`compare_snapshots`、`get_usage_trends`
- 配置查询：`get_global_config`、`get_project_config`
- AI 辅助：`get_guidance`
- 验证：`get_verification_status`、`record_verification_result`、`run_verification_command`
- 数据风险：`get_data_risk_candidates`、`get_workspace_data_risk_overview`

如果你已经启动了 daemon，MCP 也会优先复用 daemon 持有的状态。

## 8. 推荐的实际工作流

### 工作流 A：刚接入一个新项目

```bash
cargo run -- create --id demo --path /abs/path/to/project
cargo run -- snapshot --id demo
cargo run -- start --id demo
cargo run -- decision-brief --project demo
```

### 工作流 B：想知道最近哪里活跃

```bash
cargo run -- report window --id demo --window 24h
cargo run -- report trend --id demo --window 7d
cargo run -- snapshot --id demo
cargo run -- report compare --id demo
```

`report compare` 前至少要有两次 snapshot；否则会报错。

### 工作流 C：准备做清理或重构

```bash
cargo run -- verification --id demo
cargo run -- data-risk --id demo --min-review-priority medium
cargo run -- unused --id demo
```

然后再回到 shell / 项目原生命令确认：

```bash
git status
git diff
cargo test
```

### 工作流 D：多个项目之间先决定看哪个

```bash
cargo run -- workspace-data-risk --project-limit 10
cargo run -- agent-guidance --top 5
```

## 9. 重要注意事项

- `start` 在没有 daemon 时会阻塞当前终端；这是预期行为
- `stop` 只会停止 daemon-managed monitor
- `cleanup-data` 只清理 OPENDOG 数据，不会删除项目源码
- `unused` 只能说明“未被 OPENDOG 观察到访问”，不能直接等价于“可以安全删除”
- `data-risk` 只是候选识别，不是精准分类器
- `stats`、`unused`、`trend` 都依赖真实活动和观察窗口；不要把刚 snapshot 完的结果当成结论
- 瞬时打开又关闭的访问可能被轮询式观测漏掉，长一点的真实活动更容易被记录
- 很多面向查询和集成的 CLI 命令支持 `--json`，需要机器可读输出时优先使用
- 对大范围修改、清理、重构，必须结合 `git`、测试、lint、build 进行确认

## 10. 常见排障

### `stats` 一直是 `0 accessed`

按下面顺序排查：

1. 先确认 monitor 是否真的在运行：

```bash
cargo run -- list
```

2. 再确认项目的有效 `process whitelist`：

```bash
cargo run -- config show --id demo
```

3. 如果你的主要活动来自 `cargo`、`git`、`bash`、`sh`，先补充项目级配置：

```bash
cargo run -- config set-project --id demo --process claude --process codex --process node --process python --process python3 --process gpt --process glm --process cargo --process git --process bash --process sh
```

4. 如果 daemon 正在管理 monitor，必要时显式 reload：

```bash
cargo run -- config reload --id demo
```

5. 再做一段更持续的真实活动，例如一次稍长的 build 或 test，然后重新看：

```bash
cargo run -- run-verification --id demo --kind build --command "cargo build"
cargo run -- stats --id demo
```

判断原则：

- 刚做完 snapshot、还没有真实活动时，`0 accessed` 是正常现象
- 很短的瞬时访问可能会被轮询式观测漏掉
- `run-verification` 成功只能证明验证证据已记录，不等于 `stats` 一定立刻变成非零
- 如果你看到 `decision-brief` 提示先生成 activity，再看 stats，优先按它的建议走

## 11. 进一步阅读

- `README.md`：当前实现概览与 CLI/MCP 总览
- `docs/positioning.md`：项目定位与边界
- `docs/ai-playbook.md`：AI 使用顺序和 shell handoff 规则
- `docs/mcp-tool-reference.md`：MCP 工具请求/响应说明
- `docs/json-contracts.md`：JSON 契约说明
- `docs/capability-index.md`：能力到入口的快速映射
