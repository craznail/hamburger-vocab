# Recall 决策记录

> 本文件记录 Recall / 小书灵项目的重要产品、架构和协作决策。后续 ChatGPT 与本地 Codex 都应以本文档为共同事实源。

## 2026-06-28：建立三方协作机制

### 背景

用户希望 ChatGPT 和本地 Codex 一起参与 Recall 项目开发。

分工方向：

- ChatGPT 偏产品、体验、架构边界、文档和决策。
- 本地 Codex 偏代码实现、测试、重构和修 bug。
- 用户是最终决策者。

### 决定

建立三方协作机制：

```txt
用户：产品 Owner / 最终决策者
ChatGPT：产品与架构负责人
本地 Codex：代码实现负责人
```

长期协作文档：

```txt
COLLABORATION_PROTOCOL.md
RECALL_PRODUCT_PRINCIPLES.md
DECISIONS.md
ERROR_SYNC_PROTOCOL.md
RECALL_PROGRESS.md
.ai-bridge/current-plan.md
.ai-bridge/codex-report.md
.ai-bridge/review-notes.md
```

### 影响

后续所有重要代码改动前，先明确产品决策和任务计划。

Codex 不应自行改变产品方向。

ChatGPT 不应只从代码便利性角度做产品决策。

### 状态

已建立。

---

## 2026-06-28：确认 Recall 产品北极星

### 背景

项目早期从单词学习、错题系统逐步演化为更通用的记忆驱动学习系统。

用户明确不希望产品变成儿童教育 App，也不希望变成后台管理系统。

### 决定

Recall 的北极星是：

```txt
把任何知识变成长期记忆
```

产品定位：

```txt
记忆驱动学习系统
```

核心原则：

- Learning First
- Knowledge First
- Growth is Feedback

### 影响

任何功能、UI、同步策略、数据模型都必须服务于长期记忆目标。

### 状态

已确认。

---

## 2026-06-28：文档优先于聊天上下文

### 背景

项目聊天上下文越来越长，切换窗口后容易丢失上下文。

### 决定

仓库 Markdown 文档作为长期事实源，聊天只作为临时讨论区。

关键事实和决策必须落到文档中。

### 影响

后续需要保持：

- `DECISIONS.md` 记录决策
- `RECALL_PROGRESS.md` 记录进度
- `ERROR_SYNC_PROTOCOL.md` 记录同步协议
- `.ai-bridge/current-plan.md` 记录给 Codex 的任务

### 状态

已确认。

---

## 2026-06-28：错题同步协议文档作为客户端协议基线

### 背景

当前客户端错题同步已经完成重写，具备：

```txt
op log 驱动
push/pull 分离
cursor 增量拉取
server snapshot
冲突处理
图片远端信息同步
token refresh
```

### 决定

`ERROR_SYNC_PROTOCOL.md` 作为当前客户端错题同步协议基线。

但服务端 `wrong-notebook` 源码尚未在当前 CodexPro 工作区中复核。

### 影响

后续服务端接入 CodexPro 后，需要根据该文档逐项核对：

```txt
POST /api/mobile/auth/login
POST /api/mobile/auth/refresh
POST /api/mobile/error-items/analyze
POST /api/mobile/sync/push
GET  /api/mobile/sync/pull
```

### 状态

客户端基线已建立；服务端源码复核待进行。

---

## 2026-06-28：本地 ready 错题允许未同步先复习

### 背景

当前客户端 `get_due_error_items` 要求：

```sql
AND e.remote_id IS NOT NULL
```

这意味着本地 AI 分析成功但尚未同步成功的错题不会进入复习队列。

这和 Learning First 原则冲突。

### 选项

#### 方案 A：服务端确认后才能复习

优点：

- 服务端数据一致性更强。
- review op 更简单。

缺点：

- 网络失败或服务端失败会阻断学习。
- 不符合本地优先体验。

#### 方案 B：本地 ready 后即可复习

优点：

- 学习不中断。
- 更符合 Learning First。
- 本地优先体验更好。

缺点：

- 需要明确本地 review 如何和后续 create 同步合并。
- 测试场景更多。

### 决定

采用方案 B：

```txt
只要 analysis_status = ready，就允许本地复习。
同步状态只影响多端一致性，不阻断学习。
```

具体实现约束：

- `pending_analysis` / `analyzing` / `analyze_failed` 不进入复习。
- 本地 ready 但无 `remote_id` 的错题可以进入复习队列。
- 本地 ready 但无 `remote_id` 的错题应能更新本地 SM-2 复习状态。
- 如果当前实现对无 `remote_id` 的错题不生成 `review` op，可以暂时保持；后续由 create payload 或后续同步策略统一处理。
- 不允许为了同步方便阻断本地学习。

### 影响

需要修改：

```txt
src-tauri/src/db/error_repo.rs
```

重点包括：

- `get_due_error_items`
- 错题本 `due_count` 统计
- 相关 Rust repo 层测试

### 状态

已由用户确认，等待 Codex 实现。

---

## 2026-06-28：PRD.md 不直接删除，改为归档

### 背景

当前工作区显示：

```txt
D PRD.md
```

说明旧 `PRD.md` 被删除。

该文档虽然内容偏旧，仍然保留了早期单词卡片学习系统的产品上下文。

### 决定

`PRD.md` 不直接删除，改为归档。

推荐路径：

```txt
docs/archive/PRD_vocab_v0.2.md
```

后续新建新的 Recall 产品文档：

```txt
RECALL_PRD.md
```

### 影响

Codex 需要先恢复旧 `PRD.md` 内容，再移动到归档目录，避免历史产品上下文丢失。

仓库状态应从：

```txt
D PRD.md
```

变为类似：

```txt
R PRD.md -> docs/archive/PRD_vocab_v0.2.md
```

或等价的内容保留状态。

### 状态

已由用户确认，等待 Codex 处理。

---

## 2026-06-28：文档删除规则

### 背景

长期项目容易因为重构或清理误删历史上下文。

### 决定

任何长期文档不应直接删除。

如果过时：

```txt
先归档，再新建替代文档。
```

### 影响

Codex 在未得到明确任务前，不应删除：

```txt
PRD.md
RECALL_PROGRESS.md
ERROR_SYNC_PROTOCOL.md
DECISIONS.md
COLLABORATION_PROTOCOL.md
RECALL_PRODUCT_PRINCIPLES.md
```

### 状态

已建立。
