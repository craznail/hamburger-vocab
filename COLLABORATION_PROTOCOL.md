# Recall 三方协作协议

> 参与方：用户（产品 Owner）、ChatGPT（产品与架构负责人）、本地 Codex（代码实现负责人）。

## 1. 协作目标

让 Recall / 小书灵项目在长期开发中保持方向稳定、上下文可继承、代码实现可验证。

项目北极星：

```txt
把任何知识变成长期记忆
```

所有产品、架构、UI、代码实现都必须服务于这个目标。

## 2. 三方职责

### 2.1 用户

用户是最终决策者。

职责：

- 确认产品方向
- 拍板关键取舍
- 决定是否接受代码改动
- 提供真实使用反馈

### 2.2 ChatGPT

ChatGPT 是产品方向、体验设计和架构边界负责人。

职责：

- 维护产品北极星和长期原则
- 写 PRD、同步协议、决策记录、任务计划
- 判断功能是否符合 Learning First / Knowledge First / Growth is Feedback
- 给本地 Codex 写实现任务
- Review Codex 的实现是否偏离产品意图
- 发现代码实现和产品文档之间的不一致

ChatGPT 不应只从代码便利性角度做产品决策。

### 2.3 本地 Codex

本地 Codex 是代码开发负责人。

职责：

- 阅读 `.ai-bridge/current-plan.md`
- 根据任务计划修改代码
- 补充测试
- 运行构建和测试
- 在 `.ai-bridge/codex-report.md` 写实现报告
- 标明改了哪些文件、为什么改、测试结果是什么

Codex 不应自行改变产品方向。

## 3. 共同事实源

聊天记录不是唯一记忆。仓库文档才是长期事实源。

核心文档：

```txt
RECALL_PROGRESS.md              项目进度和阶段
RECALL_PRODUCT_PRINCIPLES.md    产品原则和边界
ERROR_SYNC_PROTOCOL.md          错题同步协议
DECISIONS.md                    关键决策记录
COLLABORATION_PROTOCOL.md       三方协作规则
.ai-bridge/current-plan.md      当前给 Codex 的任务
.ai-bridge/codex-report.md      Codex 执行报告
.ai-bridge/review-notes.md      ChatGPT Review 记录
```

## 4. 标准工作流

每个重要任务按以下流程推进：

```txt
1. 用户提出目标或问题
2. ChatGPT 判断产品方向和技术边界
3. ChatGPT 更新 DECISIONS.md / 相关文档
4. ChatGPT 写 .ai-bridge/current-plan.md
5. 本地 Codex 根据 current-plan 实现
6. 本地 Codex 写 .ai-bridge/codex-report.md
7. ChatGPT Review 代码和报告
8. 用户最终确认是否接受
```

## 5. 任务交接规则

### 5.1 ChatGPT 写给 Codex 的计划必须包含

- 背景
- 产品决策
- 实现范围
- 不允许做什么
- 验收标准
- 需要运行的测试
- 需要更新的文档

### 5.2 Codex 的报告必须包含

- 修改文件列表
- 核心实现说明
- 测试命令和结果
- 未解决问题
- 需要产品确认的问题

## 6. 产品护栏

Recall 不是：

- 单纯背单词工具
- 儿童化教育 App
- 后台管理系统
- 错题本管理后台
- 只追求功能堆叠的学习工具

Recall 是：

```txt
记忆驱动学习系统
```

核心原则：

- Learning First：学习不能被同步、设置、管理流程阻断。
- Knowledge First：知识内容是中心，UI 和功能围绕知识长期保留与召回。
- Growth is Feedback：成长感来自长期可见的记忆积累，而不是短期游戏化刺激。

## 7. 代码改动边界

Codex 可以主动处理：

- Bug 修复
- 类型错误
- 测试失败
- 构建失败
- 小范围重构
- 与任务明确相关的实现

Codex 不应未经确认主动处理：

- 产品定位变化
- 关键数据模型大改
- 同步协议字段大改
- UI 风格方向大改
- 删除长期文档
- 引入重大依赖

## 8. 文档改动规则

重要改动必须同步更新文档。

如果代码和文档冲突：

```txt
先标记冲突，再由 ChatGPT 和用户判断，不要默默覆盖。
```

如果需要删除文档：

```txt
优先归档，不直接删除。
```

建议归档路径：

```txt
docs/archive/
```

## 9. 决策记录规则

所有影响后续实现的决定都写入 `DECISIONS.md`。

格式：

```md
## YYYY-MM-DD：决策标题

### 背景

### 选项

### 决定

### 影响

### 状态
```

## 10. 当前协作启动状态

本协议建立后，第一阶段目标是：

```txt
先把协作机制稳定下来，再继续推进错题同步和 Recall 产品主线。
```

当前优先事项：

1. 确认 `PRD.md` 删除是否应该接受。
2. 修正 `ERROR_SYNC_PROTOCOL.md` 中不够准确的表述。
3. 建立 `DECISIONS.md` 作为共同决策记录。
4. 建立 `.ai-bridge` 作为 ChatGPT 与本地 Codex 的交接区。
