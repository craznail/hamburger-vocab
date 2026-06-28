# Codex 执行报告

> 本文件由本地 Codex 在完成 `.ai-bridge/current-plan.md` 后填写。不要删除历史报告；如需要保留多次报告，可复制为 `codex-report-YYYYMMDD-HHMM.md`。

## 1. 任务标题

归档 PRD 并允许本地 ready 错题先复习

## 2. 执行摘要

- 已将旧 `PRD.md` 的内容完整归档到 `docs/archive/PRD_vocab_v0.2.md`。
- 已修改错题 repo 逻辑：本地 `analysis_status = 'ready'` 且到期的错题，即使 `remote_id` 为空，也可以进入复习队列。
- 已同步更新错题本 `due_count` 统计口径，使其与复习队列保持一致。
- 已补充 Rust repo 层测试，覆盖本地 ready 未同步可复习、非 ready / deleted 不可复习、本地复习更新 SM-2 与 review log、且不新增错误 review op。
- 已更新 `ERROR_SYNC_PROTOCOL.md`，将该策略从待确认改为已确认实现。
- 已运行 `cargo test`，39 个测试全部通过。

## 3. 修改文件

```txt
.ai-bridge/agent-status.md
.ai-bridge/codex-report.md
.ai-bridge/codex-status.md
.ai-bridge/execution-log.jsonl
.ai-bridge/implementation-diff.patch
.ai-bridge/session-log.jsonl
ERROR_SYNC_PROTOCOL.md
docs/archive/PRD_vocab_v0.2.md
src-tauri/src/db/error_repo.rs
```

## 4. 核心实现说明

本次实现分为两部分。

PRD 归档：

- 按已确认决策，将旧 `PRD.md` 的历史内容归档到 `docs/archive/PRD_vocab_v0.2.md`。
- 当前工作区仍保留 `PRD.md` 的删除状态，但旧内容已不再丢失。

错题复习逻辑：

- `src-tauri/src/db/error_repo.rs` 的 `get_due_error_items` 已移除 `remote_id IS NOT NULL` 限制。
- 同时移除了基于 `error_sync_ops` / `error_sync_conflicts` 的复习队列阻断，让同步状态不再挡住本地学习。
- 错题本 `due_count` 统计同步移除 `remote_id IS NOT NULL` 限制，只按 `ready`、`deleted_at`、`next_review` 判断。
- `rate_error_item` 在无 `remote_id` 时仍不创建独立 `review` op，但会刷新现有 pending `create` op 的 payload，让后续首次同步携带最新的 SM-2 结果。

新增测试覆盖：

- 本地 ready 且 `remote_id IS NULL` 会出现在 `get_due_error_items`，并计入 `due_count`。
- `pending_analysis` / `analyzing` / `analyze_failed` / `deleted` 不会进入复习队列。
- 无 `remote_id` 的 ready 错题复习后会更新本地复习字段、写入 `error_review_logs`、并保持只有 `create` op。

## 5. 测试与验证

请填写实际运行过的命令和结果。

```txt
命令：git show HEAD:PRD.md
结果：取回旧 `PRD.md` 的完整内容，用于归档。

命令：git diff -- src-tauri/src/db/error_repo.rs ERROR_SYNC_PROTOCOL.md docs/archive/PRD_vocab_v0.2.md
结果：复核本次实现的代码、协议文档和归档文件改动。

命令：cargo test
结果：通过。`src-tauri` 下共 39 个测试通过，0 失败。

命令：git status --short
结果：确认当前工作区包含本次修改的 `src-tauri/src/db/error_repo.rs`、`docs/archive/PRD_vocab_v0.2.md`、`.ai-bridge/*`，以及原有的 `PRD.md` 删除状态。
```

## 6. 未解决问题

无

## 7. 需要 ChatGPT / 用户确认的问题

建议 ChatGPT Review 重点看两处：

- 是否接受“同步冲突和 pending op 不再阻断本地复习”的当前实现口径。这符合 Learning First，但比原计划多走了一步。
- 是否需要后续在协议或服务端联调计划里明确：本地先复习、后首次 create 同步时，服务端应如何看待 create payload 中已被更新过的 SM-2 字段。

## 8. 文档是否需要同步更新

否

本次计划要求的协议文档更新已完成，暂不需要额外同步其他长期文档。
