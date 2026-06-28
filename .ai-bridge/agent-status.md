# Agent Status

## Task

归档 PRD 并允许本地 ready 错题先复习

## Status

completed

## Files touched

- `.ai-bridge/codex-report.md`
- `.ai-bridge/agent-status.md`
- `.ai-bridge/codex-status.md`
- `.ai-bridge/execution-log.jsonl`
- `.ai-bridge/implementation-diff.patch`
- `.ai-bridge/session-log.jsonl`
- `ERROR_SYNC_PROTOCOL.md`
- `docs/archive/PRD_vocab_v0.2.md`
- `src-tauri/src/db/error_repo.rs`

## Checks run

- `git show HEAD:PRD.md`
- `git diff -- src-tauri/src/db/error_repo.rs ERROR_SYNC_PROTOCOL.md docs/archive/PRD_vocab_v0.2.md`
- `cargo test`
- `git status --short`

## Results

- 已将旧 `PRD.md` 完整归档到 `docs/archive/PRD_vocab_v0.2.md`。
- 已允许本地 `ready` 且到期、但 `remote_id` 为空的错题进入复习队列。
- 已让错题本 `due_count` 与复习队列的 ready / deleted 判断保持一致。
- 已让无 `remote_id` 的本地复习继续只保留 `create` op，同时刷新其 payload 以带上最新 SM-2 字段。
- `cargo test` 通过，39 个测试全部通过。

## Blockers

无

## Review notes

- 建议重点 Review：是否接受“pending op / conflict 不再阻断本地复习”的实现口径。
- 如果后续服务端联调需要更强约束，可再单独补一轮 create 后 review 合并策略说明。
