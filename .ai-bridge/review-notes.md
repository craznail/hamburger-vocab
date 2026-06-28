# ChatGPT Review Notes

> Review 时间：2026-06-28  
> Review 对象：本地 Codex 对 `.ai-bridge/current-plan.md` 的执行结果  
> 任务标题：建立 ai-bridge 自动化协作方案 B

## Review 状态

```txt
有条件通过，不建议立即长期启动 watcher；建议先做一轮安全加固。
```

## 1. Review 对象

本次 Review 对象：

```txt
.ai-bridge/codex-report.md
.ai-bridge/task-state.json
.ai-bridge/README.md
scripts/ai-bridge-runner.sh
scripts/ai-bridge-watch.sh
package.json
当前工作区状态
```

Codex 报告中写明已运行：

```txt
bash -n scripts/ai-bridge-runner.sh
bash -n scripts/ai-bridge-watch.sh
AI_BRIDGE_DRY_RUN=1 bash scripts/ai-bridge-runner.sh
```

我这边 CodexPro 的 safe bash allowlist 不允许直接运行 `bash -n`，所以语法验证只能采信 Codex 报告；我已做静态 review。

## 2. 已完成内容

通过项：

- 已新增 `.ai-bridge/task-state.json`。
- 已新增 `scripts/ai-bridge-runner.sh`。
- 已新增 `scripts/ai-bridge-watch.sh`。
- 已扩写 `.ai-bridge/README.md`。
- 已在 `package.json` 增加：
  - `ai-bridge:run`
  - `ai-bridge:watch`
- runner 有状态门禁：只有 `ready_for_codex` 才执行。
- runner 有 lock：`.ai-bridge/.runner.lock`。
- runner 不自动 commit / push。
- watcher 支持 `fswatch`，也支持轮询 fallback。
- README 写清楚了角色、状态机、使用方式和安全边界。

## 3. 状态机检查

当前状态机符合方案 B 初版目标：

```txt
idle
ready_for_codex
running
codex_done
waiting_for_chatgpt_review
review_done
failed
```

当前 `.ai-bridge/task-state.json` 是：

```txt
status = idle
requiresReview = true
autoCommit = false
```

这是安全的初始状态。

## 4. 关键风险

### 风险 1：runner 只检查 report 非空，可能误用旧报告

当前 runner 在 Codex 执行后只检查：

```bash
[[ -s "$REPORT_FILE" ]]
```

这意味着如果 `.ai-bridge/codex-report.md` 已经存在旧报告，而 Codex 这次没有正确更新报告，runner 仍可能把任务推进到：

```txt
waiting_for_chatgpt_review
```

这会造成“看起来执行完成，其实 review 的是旧报告”。

建议修复：

```txt
执行前记录 codex-report.md 的 mtime / sha256 / size。
执行后必须确认 report 有更新。
或要求 report 中包含当前 taskId / current-plan 标题。
```

这是我认为进入长期 watcher 前必须修的点。

### 风险 2：无 Codex 命令时可能反复触发

当前 runner 如果找不到可用 Codex 命令，会：

```txt
保持 status = ready_for_codex
写 lastError
退出 0
```

在 watcher 轮询模式下，状态文件被更新后，mtime 改变，可能导致 watcher 再次触发 runner，形成重复提示。

建议修复：

```txt
无可用 Codex 命令时，将 status 改为 failed。
或新增 manual_required 状态。
```

考虑我们当前状态机没有 `manual_required`，建议先用 `failed`。

### 风险 3：默认检测到 codex 就真实执行，需要用户明确知道

Codex 这版 runner 如果检测到本机存在 `codex` 命令，会默认执行：

```txt
codex exec -C <repo> -a never -s workspace-write ...
```

这符合“方案 B 自动触发”的方向，但比最保守的 dry-run/manual 模式更进一步。

我认为可以保留，但 README 需要更醒目地说明：

```txt
一旦 watcher 运行，且 task-state.status = ready_for_codex，就会真实触发 Codex。
```

或者增加开关：

```txt
AI_BRIDGE_ENABLE_DEFAULT_CODEX=1
```

只有显式打开时才使用默认 codex 命令；否则必须配置 `AI_BRIDGE_CODEX_CMD`。

我倾向加这个开关，让方案 B 更安全。

## 5. 是否建议接受

```txt
建议接受为方案 B v0.1，但不要长期启动 watcher。
```

当前版本已经可用于手动 runner / dry-run 演练，但进入“开着 watcher 自动跑”的阶段前，需要先完成 v0.2 安全加固。

## 6. 建议下一轮小修

下一轮请 Codex 做一个小任务：

```txt
ai-bridge v0.2 安全加固
```

必须修：

1. 防止旧 `codex-report.md` 被误判为本次报告。
2. 无 Codex 命令时不要保持 `ready_for_codex` 造成 watcher 重复触发。
3. 明确默认真实执行策略，建议加 `AI_BRIDGE_ENABLE_DEFAULT_CODEX=1` 开关。
4. README 补充“开启 watcher 后的真实执行风险”。
5. 增加 dry-run / no-command / stale-report 的验证说明。

## 7. 当前使用建议

目前可以安全使用：

```bash
AI_BRIDGE_DRY_RUN=1 npm run ai-bridge:run
```

暂时不建议长期运行：

```bash
npm run ai-bridge:watch
```

等 v0.2 安全加固完成后，再正式启用 watcher。
