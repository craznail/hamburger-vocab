# Recall 错题同步协议文档

> 当前文档基于 `hamburger-vocab` 客户端当前实现整理，用于和 `wrong-notebook` 服务端对齐接口协议、同步状态、冲突处理和后续改造方向。

## 1. 文档目标

本文件用于固化 Recall / 小书灵当前错题同步系统的客户端协议。

目标不是描述一个理想设计，而是先准确记录当前客户端已经实现的同步模型，方便后续检查服务端 `wrong-notebook` 是否完全匹配。

当前客户端已经从“简单同步字段”升级为：

```txt
本地 op log + push/pull 同步 + cursor 增量拉取 + 服务端快照 + 冲突表
```

这套机制的目标是支持：

- 本地优先的错题编辑体验
- 服务端统一存储和多端同步
- 网络失败后的可恢复同步
- 多端编辑时的版本冲突处理
- AI 分析完成后再形成可同步错题
- 图片本地路径和远端图片信息的兼容

## 2. 当前代码位置

### 前端 API

```txt
src/api/errorItem.ts
```

负责把 Vue 页面调用转成 Tauri command。

主要方法：

```ts
getErrorNotebooks()
getErrorItems(notebookId?)
getDueErrorItems()
createErrorDraft(imageBase64, mimeType, notebookId?)
analyzeErrorDraft(id)
saveErrorItem(item)
rateErrorItem(id, quality, durationSeconds)
syncErrorItems()
getErrorSyncConflicts()
resolveErrorSyncConflictKeepLocal(localItemId)
resolveErrorSyncConflictAcceptRemote(localItemId)
```

### 前端 Store

```txt
src/stores/useErrorNotebookStore.ts
```

负责错题本、错题、登录状态、同步状态、冲突列表的统一加载。

关键点：

- `syncRemote()` 防止重复同步请求。
- `refresh(force, pullRemote)` 会在登录状态有效时自动同步。
- 同步失败时保留本地缓存展示。
- 同步失败后重新读取登录状态，避免 token 失效后 UI 仍停留在登录态。

### Tauri Command

```txt
src-tauri/src/commands/error_item.rs
src-tauri/src/commands/mobile.rs
```

`error_item.rs` 负责错题本地操作、AI 分析、同步 push/pull、冲突解决。

`mobile.rs` 负责登录、token refresh、退出登录和 auth status。

### 本地数据库 Repo

```txt
src-tauri/src/db/error_repo.rs
src-tauri/src/db/migration.rs
src-tauri/src/db/models.rs
```

`error_repo.rs` 是错题同步核心。

`migration.rs` 当前错题同步 schema version 为：

```rust
const ERROR_SYNC_SCHEMA_VERSION: &str = "4";
```

## 3. 核心同步模型

当前同步模型以本地 SQLite 为中心。

本地用户操作不会直接假设服务端成功，而是先进入本地状态或本地 op log。

整体链路：

```txt
用户创建 / 编辑 / 复习错题
  ↓
写入本地 SQLite
  ↓
生成或更新 error_sync_ops
  ↓
手动或自动触发 sync_error_items
  ↓
POST /api/mobile/sync/push
  ↓
处理 acceptedOps / conflictedOps
  ↓
GET /api/mobile/sync/pull?cursor=...
  ↓
更新本地 notebooks / errorItems / deletedIds
  ↓
保存 last_error_sync_cursor
```

## 4. 本地数据表

### 4.1 sync_state

通用 key-value 状态表。

用于保存：

```txt
server_url
access_token
refresh_token
access_expires_at
mobile_user
device_id
last_error_sync_cursor
error_sync_schema_version
```

补充说明：

- `last_error_sync_cursor` 是当前正式使用的增量同步游标。
- `last_error_sync_at` 仍可能出现在旧数据清理逻辑里，但已经不是当前 push/pull 协议的一部分。

### 4.2 error_notebooks

错题本表。

当前客户端认为错题本来自服务端同步，本地创建错题时要求 notebook 已存在。

字段：

```txt
id
remote_id
name
created_at
updated_at
```

当前实现里 `id = remote_id`。

### 4.3 error_items

错题主表。

关键字段：

```txt
id                    本地 ID
remote_id             服务端 ID，可为空
notebook_id           所属错题本 ID
question_text         题目
answer_text           标准答案
analysis              AI 解析
wrong_answer_text     错误答案
mistake_analysis      错因分析
mistake_status        错题状态
knowledge_points      知识点 JSON 字符串
user_notes            用户笔记
mastery_level         掌握等级
ef                    SM-2 ease factor
interval              复习间隔
repetitions           复习次数
next_review           下次复习日期
analysis_status       AI 分析状态
remote_version        服务端版本号
server_snapshot_json  服务端快照
deleted_at            软删除时间
```

### 4.4 error_item_images

错题图片表。

字段：

```txt
id
error_item_id
local_path
remote_key
remote_url
sha256
mime_type
created_at
updated_at
```

设计意图：

- 本地刚创建时优先使用 `local_path`。
- 服务端分析或同步成功后补充 `remote_key` / `remote_url`。
- 如果本地图片丢失，前端可回退到 `remote_url`。

### 4.5 error_sync_ops

本地同步操作日志。

字段：

```txt
op_id
entity_type
action
local_item_id
remote_item_id
base_version
payload_json
client_timestamp
status
created_at
updated_at
```

当前支持的 action：

```txt
create
update
review
delete
```

当前支持的 status：

```txt
pending
conflicted
```

服务端接受后，本地直接删除对应 op。

### 4.6 error_sync_conflicts

同步冲突表。

字段：

```txt
id
local_item_id
op_id
server_version
server_snapshot_json
error_code
created_at
updated_at
```

用于记录服务端拒绝的 op。

当前做法不是由前端直接解析服务端 `error_code`，而是 Tauri 侧先把冲突归一化后再返回前端：

```txt
VERSION_CONFLICT  -> version_conflict
VALIDATION_ERROR  -> validation_error
NOT_FOUND         -> not_found
其他              -> unknown
```

补充说明：

- 如果服务端没有明确返回 `VERSION_CONFLICT`，但带回了有效 `serverSnapshot`，客户端也会按 `version_conflict` 处理。
- 前端实际消费的字段是：

```txt
reason
hasRemoteSnapshot
```

## 5. 错题生命周期

### 5.1 创建草稿

前端调用：

```ts
createErrorDraft(imageBase64, mimeType, notebookId)
```

Tauri command：

```rust
create_error_draft
```

本地行为：

1. 解析图片 base64。
2. 计算 sha256。
3. 保存图片到 app data 目录。
4. 创建 `error_items` 记录。
5. 创建 `error_item_images` 记录。
6. `analysis_status = pending_analysis`。

此时不会生成同步 op。

原因：草稿还不是一条完整可同步错题。

### 5.2 AI 分析中

前端调用：

```ts
analyzeErrorDraft(id)
```

本地先标记：

```txt
analysis_status = analyzing
```

然后请求服务端：

```txt
POST /api/mobile/error-items/analyze
```

请求体当前结构：

```json
{
  "localId": "本地错题 ID",
  "notebookId": "错题本 ID",
  "imageBase64": "图片 base64",
  "mimeType": "image/jpeg",
  "language": "zh"
}
```

注意：当前客户端这里的 `mimeType` 暂时写死为 `image/jpeg`。后续建议改为从 `error_item_images.mime_type` 读取真实值。

### 5.3 AI 分析成功

服务端返回：

```json
{
  "questionText": "题目",
  "answerText": "答案",
  "analysis": "解析",
  "wrongAnswerText": "错误答案",
  "mistakeAnalysis": "错因分析",
  "mistakeStatus": "状态",
  "knowledgePoints": ["知识点1", "知识点2"],
  "masteryLevel": 0,
  "image": {
    "remoteKey": "远端图片 key",
    "url": "/或完整图片 URL",
    "sha256": "图片 hash",
    "contentType": "image/jpeg",
    "size": 12345
  }
}
```

本地行为：

1. 更新错题内容。
2. `analysis_status = ready`。
3. 更新图片远端信息。
4. 生成 `create` op。

### 5.4 AI 分析失败

如果服务端返回失败响应，客户端会尝试解析：

```json
{
  "code": "错误码",
  "message": "错误信息",
  "image": {
    "remoteKey": "远端图片 key",
    "url": "图片 URL",
    "sha256": "图片 hash",
    "contentType": "image/jpeg",
    "size": 12345
  }
}
```

本地行为：

1. `analysis_status = analyze_failed`。
2. 保留本地图片和可能的远端图片信息。
3. 清理该错题的 create/update sync op。
4. 清理该错题的 sync conflict。

设计意图：分析失败的错题不是完整错题，不应进入 create/update 同步队列。

### 5.5 编辑错题

前端调用：

```ts
saveErrorItem(...)
```

本地行为：

1. 更新题目、答案、解析、错因、笔记、知识点。
2. 如果 `analysis_status != ready`，清理 create/update 同步状态，不生成 op。
3. 如果已有 `remote_id`，生成或更新 `update` op。
4. 如果没有 `remote_id`，生成或更新 `create` op。

特殊规则：

如果本地已有 pending `create` op，再次编辑不会创建新的 `update` op，而是直接更新 pending `create` 的 payload。

### 5.6 复习错题

前端调用：

```ts
rateErrorItem(id, quality, durationSeconds)
```

本地行为：

1. 使用 SM-2 计算新的 `ef`、`interval`、`repetitions`、`next_review`。
2. 写入 `error_review_logs`。
3. 更新 `mastery_level`。
4. 如果该错题已有 `remote_id`，生成 `review` op。

注意：当前实现中，没有 `remote_id` 的本地 ready 错题不会单独生成 `review` op。

但本地复习仍然会：

- 更新本地 `ef` / `interval` / `repetitions` / `next_review`
- 写入 `error_review_logs`
- 刷新现有 pending `create` op 的 payload，让后续首次同步携带最新复习状态

## 6. 同步流程

### 6.1 登录和 token 刷新

登录接口：

```txt
POST /api/mobile/auth/login
```

请求体：

```json
{
  "email": "用户邮箱",
  "password": "用户密码"
}
```

响应体：

```json
{
  "accessToken": "access token",
  "refreshToken": "refresh token",
  "expiresIn": 3600,
  "user": {
    "id": "用户 ID"
  }
}
```

客户端会保存：

```txt
server_url
access_token
refresh_token
access_expires_at
mobile_user
```

如果 server url 或 user id 变化，客户端会清空远端缓存：

```txt
error_sync_conflicts
error_sync_ops
error_review_logs
error_item_images
error_items
error_notebooks
last_error_sync_cursor
```

刷新接口：

```txt
POST /api/mobile/auth/refresh
```

请求体：

```json
{
  "refreshToken": "refresh token"
}
```

响应体：

```json
{
  "accessToken": "new access token",
  "refreshToken": "new refresh token",
  "expiresIn": 3600
}
```

客户端会在 access token 过期前 30 秒主动视为过期。

如果请求返回 401，客户端会自动 refresh 后重试一次。

### 6.2 Push

客户端请求：

```txt
POST /api/mobile/sync/push
Authorization: Bearer <access_token>
```

请求体：

```json
{
  "deviceId": "本地设备 ID",
  "ops": [
    {
      "opId": "本地 op ID",
      "entityType": "error_item",
      "action": "create | update | review | delete",
      "localItemId": "本地错题 ID",
      "remoteItemId": "远端错题 ID，可为空",
      "baseVersion": 1,
      "payload": {},
      "clientTimestamp": "2026-06-27 12:00:00"
    }
  ]
}
```

服务端响应：

```json
{
  "acceptedOps": [
    {
      "opId": "本地 op ID",
      "localItemId": "本地错题 ID",
      "action": "create | update | review | delete",
      "serverSnapshot": {
        "remoteId": "远端错题 ID",
        "localId": "本地错题 ID",
        "notebookId": "错题本 ID",
        "notebookName": "错题本名称",
        "version": 2,
        "questionText": "题目",
        "answerText": "答案",
        "analysis": "解析",
        "wrongAnswerText": "错误答案",
        "mistakeAnalysis": "错因分析",
        "mistakeStatus": "状态",
        "knowledgePoints": [],
        "userNotes": "笔记",
        "masteryLevel": 0,
        "ef": 2.5,
        "interval": 1,
        "repetitions": 0,
        "nextReview": "2026-06-27",
        "image": {
          "remoteKey": "图片 key",
          "url": "/图片 URL",
          "sha256": "hash",
          "contentType": "image/jpeg",
          "size": 12345
        },
        "createdAt": "2026-06-27 10:00:00",
        "updatedAt": "2026-06-27 12:00:00"
      }
    }
  ],
  "conflictedOps": [
    {
      "opId": "本地 op ID",
      "localItemId": "本地错题 ID",
      "code": "VERSION_CONFLICT | VALIDATION_ERROR | NOT_FOUND | REMOTE_ID_REQUIRED",
      "serverVersion": 3,
      "serverSnapshot": {}
    }
  ],
  "remoteMappings": {},
  "serverCursor": 123
}
```

服务端核对补充：当前 `wrong-notebook` 的 push 响应可能额外返回 `remoteMappings` 和 `serverCursor`；`serverSnapshot` 可能包含 `notebookName` 与 `image.size`；冲突码除 `VERSION_CONFLICT` / `VALIDATION_ERROR` / `NOT_FOUND` 外，还可能出现 `REMOTE_ID_REQUIRED`。

客户端处理：

- 对 `acceptedOps`：
  - 如果 action 是 `delete`，本地软删除。
  - 如果有 `serverSnapshot`，覆盖本地工作副本。
  - 删除对应 conflict。
  - 删除对应 op。

- 对 `conflictedOps`：
  - 将本地 op 标记为 `conflicted`。
  - 写入 `error_sync_conflicts`。
  - 保留本地工作副本，不覆盖用户修改。

### 6.3 Pull

客户端请求：

```txt
GET /api/mobile/sync/pull?cursor=<last_error_sync_cursor>
Authorization: Bearer <access_token>
```

服务端响应：

```json
{
  "notebooks": [
    {
      "remoteId": "错题本 ID",
      "name": "数学错题本",
      "createdAt": "2026-06-27 10:00:00",
      "updatedAt": "2026-06-27 10:00:00"
    }
  ],
  "errorItems": [
    {
      "remoteId": "远端错题 ID",
      "localId": "本地 ID，可选",
      "notebookId": "错题本 ID",
      "version": 1,
      "questionText": "题目",
      "answerText": "答案",
      "analysis": "解析",
      "wrongAnswerText": "错误答案",
      "mistakeAnalysis": "错因分析",
      "mistakeStatus": "状态",
      "knowledgePoints": [],
      "userNotes": "笔记",
      "masteryLevel": 0,
      "ef": 2.5,
      "interval": 1,
      "repetitions": 0,
      "nextReview": "2026-06-27",
      "image": {
        "remoteKey": "图片 key",
        "url": "/图片 URL",
        "sha256": "hash",
        "contentType": "image/jpeg"
      },
      "createdAt": "2026-06-27 10:00:00",
      "updatedAt": "2026-06-27 12:00:00"
    }
  ],
  "deletedIds": ["remote-item-id"],
  "nextCursor": 123
}
```

客户端处理：

1. `notebooks`：调用 `replace_notebooks`。
2. `errorItems`：按 `remoteId` 或 `localId` 匹配本地错题。
3. 如果本地错题存在 pending/conflicted op，则只更新服务端快照，不覆盖本地工作副本。
4. 如果本地没有阻塞同步状态，则用远端 snapshot 覆盖本地工作副本。
5. `deletedIds`：如果本地没有 pending/conflicted 状态，则软删除本地错题。
6. 保存 `nextCursor` 到 `last_error_sync_cursor`。

注意：当前 `notebooks` 使用 replace 语义。服务端需要保证 `/sync/pull` 返回的是当前用户完整错题本列表。如果服务端只返回增量 notebook，客户端可能误删本地 notebook。

## 7. 冲突处理

### 7.1 保留本地版本

前端调用：

```ts
resolveErrorSyncConflictKeepLocal(localItemId)
```

注意：当前前端只会在 `reason = version_conflict` 时展示“保留本地版本 / 接受远端版本”两个操作按钮。

对 `validation_error`、`not_found`、`unknown`，详情页只展示原因提示，不直接开放这两个 resolve 操作。

本地行为：

1. 读取 conflict 中的 `server_version` 和 `server_snapshot_json`。
2. 将本地 `remote_version` 更新为服务端版本。
3. 保存服务端快照。
4. 删除 conflict。
5. 删除 conflicted op。
6. 用当前本地工作副本生成新的 `update` op。

效果：

```txt
以服务端最新版本为 baseVersion，重新提交本地内容。
```

### 7.2 接受远端版本

前端调用：

```ts
resolveErrorSyncConflictAcceptRemote(localItemId)
```

同样仅适用于当前 UI 认定的真实版本冲突场景。

本地行为：

1. 读取 conflict 中的 `server_snapshot_json`。
2. 用服务端 snapshot 覆盖本地工作副本。
3. 删除 conflict。
4. 删除 conflicted op。

效果：

```txt
丢弃本地修改，接受服务端版本。
```

## 8. 前端展示状态

前端 `ErrorItem.syncStatus` 不是数据库里的单一字段，而是由本地状态动态计算出来。

计算规则：

```txt
analysis_status != ready
  -> pending_analysis / analyzing / analyze_failed

存在 error_sync_conflicts
  -> conflict

存在 pending error_sync_ops
  -> pending_sync

否则
  -> synced
```

当前 UI 状态文案：

```txt
pending_analysis -> 待分析
pending_sync     -> 待同步
conflict         -> 有冲突
analyze_failed   -> 分析失败
synced           -> 已同步
其他             -> 本地保存
```

## 9. 当前已验证项

客户端当前验证结果：

```txt
cargo test: 39 passed
npm run build: success
```

同步相关测试覆盖了：

- 旧 schema 重置为新 schema
- v3 到 v4 migration
- 清理本地无效 sync state
- pending create payload 合并
- pull 时不覆盖本地 pending 修改
- 远端图片 URL 保存
- 从 server snapshot 回填图片信息
- pending / analyze_failed 本地草稿不进入同步
- conflict reason 和 snapshot presence 展示

## 10. 当前风险和待确认点

### 10.1 服务端协议仍需持续对齐

当前本地端已经按真实 `wrong-notebook` 接口做过联调和问题排查，但这份文档仍应持续和服务端实现保持同步。

尤其需要持续确认以下接口的返回字段和错误码没有漂移：

```txt
POST /api/mobile/auth/login
POST /api/mobile/auth/refresh
POST /api/mobile/error-items/analyze
POST /api/mobile/sync/push
GET  /api/mobile/sync/pull
```

### 10.2 analyze 接口 mimeType 暂时写死

当前客户端 `analyze_error_draft` 请求里：

```json
{
  "mimeType": "image/jpeg"
}
```

但创建草稿时已经保存真实 mime type。

建议后续改为从 `error_item_images.mime_type` 读取。

### 10.3 本地 ready 但未同步成功的错题允许复习

该策略现已确认并落地：

```txt
只要 analysis_status = ready，就允许本地复习。
同步状态只影响多端一致性，不阻断学习。
```

当前约束：

- `pending_analysis` / `analyzing` / `analyze_failed` 不进入复习。
- `deleted_at IS NOT NULL` 不进入复习。
- 本地 ready 但 `remote_id IS NULL` 的错题可以进入复习队列。
- 错题本 `due_count` 与复习队列使用同一套 ready / deleted 判断口径。
- 本地 ready 但 `remote_id IS NULL` 的错题复习后，不单独生成 `review` op，但会更新本地复习状态并刷新 pending `create` payload。

### 10.4 notebooks 的 replace 语义需要和服务端确认

当前客户端在 pull 时会 `replace_notebooks`。

这要求服务端返回全量 notebooks。

如果服务端返回增量 notebooks，客户端会误删未返回的错题本。

### 10.5 push 接口错误响应内容较少

当前客户端如果 push status 非 2xx，只返回：

```txt
同步推送失败: <status>
```

建议后续读取 response body，展示服务端错误 message，便于调试。

### 10.6 pull 接口错误响应内容较少

当前客户端如果 pull status 非 2xx，只返回：

```txt
同步拉取失败: <status>
```

### 10.7 analyze 接口耗时较长时依赖较长超时窗口

当前共享 HTTP client 默认超时是 30 秒，但 `analyze_error_draft` 已单独把 analyze 请求放宽到 180 秒。

这意味着：

```txt
登录 / push / pull 仍保持短超时
analyze 单独允许更长等待时间
```

如果后续 AI 分析链路继续变慢，需要同步评估：

- 本地体验是否需要“分析中”更明确的加载提示
- 180 秒是否仍足够
- 是否要改成长任务式异步分析协议

建议后续读取 response body，展示服务端错误 message，便于调试。

## 11. 服务端对齐清单

服务端 `wrong-notebook` 需要重点确认以下事项。

### 11.1 Auth

- [ ] `/api/mobile/auth/login` 返回 `accessToken`、`refreshToken`、`expiresIn`、`user.id`。
- [ ] `/api/mobile/auth/refresh` 返回新的 `accessToken`、`refreshToken`、`expiresIn`。
- [ ] token 过期时接口返回 401。

### 11.2 Analyze

- [ ] `/api/mobile/error-items/analyze` 支持 `localId`。
- [ ] 支持 `notebookId`。
- [ ] 支持 `imageBase64`。
- [ ] 返回 AI 分析后的结构。
- [ ] 返回图片远端信息 `image.remoteKey`、`image.url`、`image.sha256`、`image.contentType`。
- [ ] 失败时返回可解析的 `code`、`message`、`image`。

### 11.3 Push

- [ ] `/api/mobile/sync/push` 支持 `deviceId`。
- [ ] 支持 `ops[]`。
- [ ] 支持 `create`。
- [ ] 支持 `update`。
- [ ] 支持 `review`。
- [ ] 支持 `delete`。
- [ ] 正常处理后返回 `acceptedOps[]`。
- [ ] 冲突时返回 `conflictedOps[]`。
- [ ] accepted op 中返回 `serverSnapshot`。
- [ ] conflict 中返回 `serverVersion` 和 `serverSnapshot`。
- [ ] 版本冲突 code 使用 `VERSION_CONFLICT`。
- [ ] 校验失败 code 使用 `VALIDATION_ERROR`。
- [ ] 远端不存在 code 使用 `NOT_FOUND`。

### 11.4 Pull

- [ ] `/api/mobile/sync/pull` 支持 `cursor`。
- [ ] 返回 `notebooks`。
- [ ] 明确 `notebooks` 是全量还是增量。
- [ ] 返回 `errorItems`。
- [ ] 返回 `deletedIds`。
- [ ] 返回 `nextCursor` 或 `serverCursor`。
- [ ] `errorItems[].remoteId` 必填。
- [ ] `errorItems[].version` 必填。
- [ ] 图片 URL 可以是完整 URL，也可以是相对路径。

## 12. 建议下一步

建议按照以下顺序推进。

### Step 1：开放服务端仓库给 CodexPro

让 CodexPro 可以访问：

```txt
/Users/caogenyan/Documents/wrong-notebook
```

然后检查服务端接口实现是否和本文档匹配。

### Step 2：修复客户端小问题

优先修复：

- analyze 请求 mimeType 写死问题
- push/pull 错误响应不展示 body 的问题

### Step 3：本地 ready 先复习策略已完成

该策略已经确认并在客户端落地：

```txt
学习优先：只要 analysis_status = ready，就允许本地复习。
同步状态只影响多端一致性，不阻断学习。
```

当前客户端行为：

- 本地 ready 但 `remote_id IS NULL` 的错题可以进入复习队列。
- `due_count` 与复习队列使用同一套 ready / deleted / next_review 口径。
- 本地 ready 但 `remote_id IS NULL` 的错题复习后，不单独生成 `review` op。
- 本地复习会更新 SM-2 字段、写入 `error_review_logs`，并刷新 pending `create` payload。

后续待验证项：

- 服务端 create 接口是否接受已经被本地复习更新过的 SM-2 字段。
- 错题处于 conflict 状态时，如果用户继续本地复习，后续冲突解决如何合并新的 SM-2 变化。
- UI 是否持续清晰展示 conflict / pending 状态，避免用户误以为多端已经一致。

### Step 4：补充端到端测试

在客户端和服务端协议确认后，补充真实端到端场景：

- 新建错题 -> AI 分析 -> 本地复习 -> push create -> pull
- A 端编辑 -> B 端 pull
- A/B 同时编辑 -> version conflict
- conflict 状态下继续本地复习 -> 保留本地版本 -> 重新 push
- 接受远端版本 -> 本地覆盖
- 图片本地丢失 -> 使用 remote image url 展示
- token 过期 -> refresh -> retry

## 13. 当前结论

当前客户端错题同步已经具备正式产品同步系统的基础结构：

```txt
op log 驱动
本地优先
push/pull 分离
cursor 增量拉取
server snapshot
版本冲突处理
图片远端信息同步
token refresh
```

接下来最重要的不是继续加 UI，而是先把客户端和服务端协议完全对齐。

该文档可以作为客户端同步协议基线。后续服务端检查完成后，需要补充“服务端实际实现结果”和“协议差异修复记录”。
