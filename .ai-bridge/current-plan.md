# 归档 PRD 并允许本地 ready 错题先复习

Updated: 2026-06-28T03:13:45.925Z
Workspace: /Users/caogenyan/Documents/hamburger
Target agent: Codex (codex)

## Plan

# 归档 PRD 并允许本地 ready 错题先复习

## 背景

用户已正式确认两个决策：

```txt
1. PRD.md 不删除，改为归档。
2. 本地 ready 错题允许未同步先复习。
```

这两个决策已经写入 `DECISIONS.md`。

本次任务分为两部分：

1. 处理旧 `PRD.md` 的归档，避免历史产品上下文丢失。
2. 修改错题复习队列逻辑，让本地 AI 分析完成但尚未同步成功的错题也能进入复习。

## 产品原则

必须遵守：

```txt
Learning First：学习不能被同步状态阻断。
Knowledge First：知识内容和复习行为优先。
Growth is Feedback：复习结果要保留为长期成长反馈。
```

## Part 1：归档 PRD.md

### 当前问题

工作区当前有：

```txt
D PRD.md
```

说明旧 `PRD.md` 被删除。

用户已确认：不要直接删除，改为归档。

### 实现要求

请恢复旧 `PRD.md` 内容，并归档到：

```txt
docs/archive/PRD_vocab_v0.2.md
```

推荐做法：

```txt
1. 先恢复 PRD.md 的旧内容。
2. 创建 docs/archive/ 目录。
3. 将 PRD.md 移动/归档为 docs/archive/PRD_vocab_v0.2.md。
```

目标是避免历史内容丢失。

最终状态可以是 git 识别为 rename，也可以是删除旧路径 + 新增归档文件，但必须保证归档文件内容完整。

### 不要做

- 不要直接丢弃旧 PRD 内容。
- 不要现在新写完整 `RECALL_PRD.md`，这会由 ChatGPT 后续单独规划。
- 不要把旧 PRD 混入新的 Recall 产品方向。

## Part 2：允许本地 ready 错题进入复习队列

### 当前代码现状

当前代码中，错题待复习查询和错题本 due_count 都要求：

```sql
AND e.remote_id IS NOT NULL
```

已知位置：

```txt
src-tauri/src/db/error_repo.rs:584
src-tauri/src/db/error_repo.rs:755
```

这导致：

```txt
本地 AI 分析完成，但 create op 尚未被服务端接受的错题，不会进入复习队列。
```

### 产品决策

采用本地优先策略：

```txt
只要 analysis_status = ready，就允许本地复习。
同步状态只影响多端一致性，不阻断学习。
```

### 实现要求

请修改：

```txt
src-tauri/src/db/error_repo.rs
```

要求：

1. `get_due_error_items` 不再要求 `remote_id IS NOT NULL`。
2. 错题本 `due_count` 统计不再要求 `remote_id IS NOT NULL`。
3. 仍然必须排除：
   - `pending_analysis`
   - `analyzing`
   - `analyze_failed`
   - `deleted_at IS NOT NULL`
4. 只有 `analysis_status = 'ready'` 的错题可以进入复习队列。
5. 已同步错题原有行为不应被破坏。
6. 无 `remote_id` 的 ready 错题被复习时，应允许更新本地 SM-2 字段和 review log。
7. 如果当前 `rate_error_item` 对无 `remote_id` 的错题不生成 `review` sync op，可以暂时保持该行为。
8. 不要为了同步方便重新阻断本地学习。

## 测试要求

请补充或更新 Rust repo 层测试，至少覆盖：

1. 本地 ready 但 `remote_id IS NULL` 的错题会出现在 `get_due_error_items`。
2. 本地 `pending_analysis` 的错题不会进入复习队列。
3. 本地 `analyzing` 的错题不会进入复习队列。
4. 本地 `analyze_failed` 的错题不会进入复习队列。
5. `deleted_at IS NOT NULL` 的错题不会进入复习队列。
6. 错题本 `due_count` 包含本地 ready 但未同步的到期错题。
7. 无 `remote_id` 的 ready 错题复习后，本地 SM-2 字段和 review log 正常更新。
8. 无 `remote_id` 的 ready 错题复习后，不应生成错误的 `review` sync op；如果当前策略是不生成 review op，请用测试锁住。

## 需要运行的验证

请运行：

```txt
cargo test
```

如果前端类型可能受影响，再运行：

```txt
npm run build
```

## 文档更新要求

请更新：

```txt
ERROR_SYNC_PROTOCOL.md
```

把之前的“本地 ready 但未同步成功的错题是否允许复习”从待确认改为已确认策略：

```txt
本地 ready 后即可进入复习，同步状态不阻断学习。
```

如有必要，也可以在 `RECALL_PROGRESS.md` 中补一条当前进展，但不要大改产品方向。

## 报告要求

完成后请填写：

```txt
.ai-bridge/codex-report.md
```

报告必须包含：

- 修改文件列表
- PRD 归档结果
- 错题复习逻辑修改点
- 新增/修改的测试
- 实际运行的验证命令和结果
- 是否存在未解决问题
- 是否需要 ChatGPT Review 的重点

同时更新：

```txt
.ai-bridge/agent-status.md
```

## 不允许做什么

- 不要修改服务端仓库。
- 不要引入新依赖。
- 不要大改同步协议。
- 不要重写错题模块。
- 不要新建完整 `RECALL_PRD.md`。
- 不要删除长期文档。
- 不要把 pending/analyzing/analyze_failed 错题放入复习队列。

## 验收标准

本次任务完成后应满足：

```txt
1. 旧 PRD 内容已归档到 docs/archive/PRD_vocab_v0.2.md。
2. 本地 ready 且到期的错题，即使 remote_id 为空，也能进入复习队列。
3. pending/analyzing/analyze_failed/deleted 错题不会进入复习队列。
4. due_count 与 get_due_error_items 逻辑一致。
5. 无 remote_id 的本地复习可以更新本地复习状态。
6. cargo test 通过。
7. ERROR_SYNC_PROTOCOL.md 已同步更新。
8. Codex 报告已填写。
```

## Implementation contract

- Work from this plan in small, reviewable steps.
- Keep edits scoped to the requested task and existing project conventions.
- Run focused verification before handing work back.
- Update .ai-bridge/agent-status.md with files touched, checks run, results, blockers, and review notes.
- Save the final review diff to .ai-bridge/implementation-diff.patch when practical.
- Append notable execution events to .ai-bridge/execution-log.jsonl when the implementation agent supports logging.
