# AI Bridge

`.ai-bridge/` 是 ChatGPT、Codex 和用户之间的共享交接区。它保存当前任务、执行状态、实现报告和 review 记录，让协作不只依赖聊天上下文。

当前协作模式也以这里和仓库文档为长期事实源。即使切换到新窗口，只要先读取 `.ai-bridge/`、`COLLABORATION_PROTOCOL.md` 和 `DECISIONS.md`，就应该知道当前该怎么协作。

## 目录用途

- `current-plan.md`
  ChatGPT 或其他规划模型写给实现代理的当前任务入口。
- `task-state.json`
  自动化 runner / watcher 使用的任务状态机文件。
- `codex-report.md`
  Codex 执行后的实现报告。
- `review-notes.md`
  ChatGPT review 记录。
- `agent-status.md`
  当前实现代理的状态摘要、测试结果、阻塞和 review 提示。
- `codex-status.md`
  兼容旧流程的 Codex 状态记录。
- `implementation-diff.patch`
  当前任务的交接 diff。
- `decisions.md`
  本桥接层自身的协作/自动化决策。
- `open-questions.md`
  未解决问题。
- `execution-log.jsonl`
  通用执行事件日志。
- `session-log.jsonl`
  兼容旧流程的事件日志。

## 角色职责

- ChatGPT
  负责产品/架构决策、编写 `current-plan.md`、review `codex-report.md`，并决定是否进入 `review_done`。
- Codex
  负责按 `current-plan.md` 实现、验证、写 `codex-report.md` 和 `agent-status.md`。
- 用户
  负责最终接受权、关键取舍拍板、是否继续自动化或进入下一轮任务。

## 状态机

`task-state.json` 使用以下状态：

- `idle`
  当前没有待执行任务。
- `ready_for_codex`
  ChatGPT 已写好 `current-plan.md`，等待 Codex 执行。
- `running`
  runner 正在调用 Codex。
- `codex_done`
  Codex 已完成且报告存在。这个状态通常是 runner 的短暂过渡态。
- `waiting_for_chatgpt_review`
  Codex 已完成，等待 ChatGPT review。
- `review_done`
  ChatGPT review 已完成。
- `failed`
  runner 或执行流程失败，需要人工处理。

## task-state.json 结构

当前默认结构：

```json
{
  "taskId": "manual-current-task",
  "status": "idle",
  "planPath": ".ai-bridge/current-plan.md",
  "reportPath": ".ai-bridge/codex-report.md",
  "reviewPath": ".ai-bridge/review-notes.md",
  "agent": "codex",
  "requiresReview": true,
  "autoCommit": false,
  "lastUpdatedAt": "",
  "lastRunAt": "",
  "lastError": "",
  "lastReportFingerprint": null
}
```

其中 `lastReportFingerprint` 的结构为：

```json
{
  "exists": true,
  "size": 1234,
  "sha256": "..."
}
```

如果当前没有有效报告指纹，则为 `null`。

这个文件仍然是可读、可检查的，但推荐不再直接手改。常见状态流转优先使用：

```bash
npm run ai-bridge:state -- <command>
```

## 状态脚本

启动方式：

```bash
npm run ai-bridge:state -- <command>
```

常用命令：

- 查看当前状态：
  `npm run ai-bridge:state -- status`
- 检查桥接环境是否健康：
  `npm run ai-bridge:state -- doctor`
- 将当前任务标记为待 Codex 执行：
  `npm run ai-bridge:state -- ready --task-id ai-bridge-v0.3-state-review-helper`
- 在 ChatGPT / 用户显式确认后标记 review 完成：
  `npm run ai-bridge:state -- review-done --result accepted --summary "v0.3 accepted"`
- 手动标记失败：
  `npm run ai-bridge:state -- fail --message "reason"`
- 安全重置为空闲：
  `npm run ai-bridge:state -- reset --to idle`

状态限制：

- `ready` 默认只允许从 `idle`、`review_done`、`failed` 进入 `ready_for_codex`。
- `ready` 在 `waiting_for_chatgpt_review` 状态下必须显式加 `--force`。
- `ready` 不允许从 `running` 直接进入 `ready_for_codex`。
- `review-done` 只允许从 `waiting_for_chatgpt_review` 或 `codex_done` 执行。
- `rejected` 会把状态写成 `failed`，并记录 `lastError`。
- `reset --to idle` 默认只允许从 `failed`、`review_done`、`idle` 执行；其他状态必须显式 `--force`。

`doctor` 会检查：

- `.ai-bridge/` 和关键文件是否存在。
- `task-state.json` 是否是合法 JSON。
- 当前 `status` 是否属于允许集合。
- `autoCommit` 是否为 `false`。
- `requiresReview` 是否为 `true`。

`review-done` 只能在 ChatGPT 或用户明确确认后执行，不要把它接入自动流程。

## Runner

启动方式：

```bash
bash scripts/ai-bridge-runner.sh
```

行为：

1. 确保 `.ai-bridge/task-state.json` 存在。
2. 只有当 `status = ready_for_codex` 时才执行。
3. 执行前切到 `running`。
4. 记录 `codex-report.md` 的 fingerprint（`sha256 + size`）。
5. 按显式配置策略调用本地 Codex。
6. 成功后只有在报告存在、非空且 fingerprint 发生变化时，才先写 `codex_done`，然后推进到 `waiting_for_chatgpt_review`。
7. 失败时写 `failed` 和 `lastError`。
8. 追加日志到 `execution-log.jsonl`。

### 配置 AI_BRIDGE_CODEX_CMD

runner 优先使用环境变量 `AI_BRIDGE_CODEX_CMD`。例如：

```bash
AI_BRIDGE_CODEX_CMD='codex exec -C "/Users/caogenyan/Documents/hamburger" -a never -s workspace-write -' \
bash scripts/ai-bridge-runner.sh
```

如果没有显式配置，只有在 `AI_BRIDGE_ENABLE_DEFAULT_CODEX=1` 且本机存在 `codex` 命令时，runner 才会使用一个内置默认命令：

```bash
AI_BRIDGE_ENABLE_DEFAULT_CODEX=1 bash scripts/ai-bridge-runner.sh
```

对应默认命令为：

```txt
codex exec -C <repo> -a never -s workspace-write --output-last-message <repo>/.ai-bridge/.runner-last-message.txt -
```

如果既没有 `AI_BRIDGE_CODEX_CMD`，也没有显式开启 `AI_BRIDGE_ENABLE_DEFAULT_CODEX=1`，runner 会直接进入 `failed`，并在 `lastError` 与 `execution-log.jsonl` 中写明需要配置执行命令。

### Dry Run

可以先做一次安全演练：

```bash
AI_BRIDGE_DRY_RUN=1 AI_BRIDGE_ENABLE_DEFAULT_CODEX=1 bash scripts/ai-bridge-runner.sh
```

Dry-run 不会真正调用 Codex，也不会把状态推进到 `running`。如果没有显式配置执行命令，dry-run 同样会因为无法解析命令而失败。

## Watcher

启动方式：

```bash
bash scripts/ai-bridge-watch.sh
```

行为：

1. 监听 `.ai-bridge/current-plan.md` 和 `.ai-bridge/task-state.json`。
2. 当检测到 `status = ready_for_codex` 时调用 runner。
3. 避免重复触发，依赖 runner 的状态检查和锁目录。
4. 优先使用 `fswatch`；若没有安装，则 fallback 到轮询。

### fswatch 与轮询

- 如果系统已安装 `fswatch`，watcher 会使用事件监听。
- 如果未安装，则默认每 5 秒轮询一次。
- 可通过 `AI_BRIDGE_FORCE_POLLING=1` 强制使用轮询。
- 可通过 `AI_BRIDGE_WATCH_INTERVAL_SECONDS=10` 调整轮询间隔。

macOS 安装 `fswatch`：

```bash
brew install fswatch
```

## 安全边界

这些边界既写在这里，也尽量体现在脚本行为里：

- Codex 不自动 commit。
- Codex 不自动 push。
- Codex 不删除长期文档。
- 测试失败不能标记为完成。
- 产品决策必须写入 `DECISIONS.md`。
- ChatGPT review 仍是强制关卡。
- 用户保留最终接受权。

## 常见故障处理

- runner 没有触发
  检查 `task-state.json` 里的 `status` 是否为 `ready_for_codex`。
- runner 提示找不到 Codex
  配置 `AI_BRIDGE_CODEX_CMD`，或设置 `AI_BRIDGE_ENABLE_DEFAULT_CODEX=1` 并确认 `codex` 命令在 PATH 中。
- 任务卡在 `running`
  检查 `.ai-bridge/.runner.lock` 是否残留，确认 Codex 进程是否已退出，再人工修正状态。
- 任务变成 `failed`
  查看 `task-state.json` 的 `lastError` 和 `execution-log.jsonl`，确认是执行命令缺失、Codex 退出非零，还是报告 fingerprint 未更新。
- watcher 没反应
  先直接运行 `bash scripts/ai-bridge-runner.sh`，再检查是否安装了 `fswatch`。

## 推荐流程

1. ChatGPT 更新 `current-plan.md`，并把 `task-state.json.status` 设为 `ready_for_codex`。
   推荐使用：
   `npm run ai-bridge:state -- ready --task-id <task-id>`
2. watcher 或 runner 触发 Codex。
3. Codex 完成后，runner 自动推进到 `waiting_for_chatgpt_review`。
4. ChatGPT review。
   确认完成后再显式执行：
   `npm run ai-bridge:state -- review-done --result accepted --summary "<summary>"`
5. 用户决定是否接受并进入下一轮。

## v0.2 安全提示

- 长期启动 watcher 前，必须显式配置 `AI_BRIDGE_CODEX_CMD`，或显式设置 `AI_BRIDGE_ENABLE_DEFAULT_CODEX=1`。
- 首次使用建议先运行 dry-run：
  `AI_BRIDGE_DRY_RUN=1 AI_BRIDGE_ENABLE_DEFAULT_CODEX=1 bash scripts/ai-bridge-runner.sh`
- 启动 watcher 前建议确认 `task-state.json.status = idle`。
- 推荐优先通过 `npm run ai-bridge:state -- status`、`ready`、`review-done`、`fail`、`reset` 管理状态，而不是手改 JSON。
- watcher 运行时，只要状态进入 `ready_for_codex`，就可能触发 runner。
- runner 不会自动 commit / push。
- ChatGPT review 仍是强制关卡。
- 用户仍保留最终接受权。

## 当前协作模式

当前项目默认遵循这个模式：

- ChatGPT 负责产品、架构边界、任务计划和 review。
- Codex 负责按 `current-plan.md` 实现、验证、写报告。
- 用户负责最终拍板和接受权。

不要绕过这个模式做“自动接受”或“自动 review 完成”。
