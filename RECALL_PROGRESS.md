# Recall 项目进程记录（RECALL_PROGRESS.md）

## 🧭 项目北极星

把任何知识变成长期记忆（Turn knowledge into long-term memory）

---

## 🧠 当前系统阶段

### V1：学习工具阶段
- 单词卡片系统
- 基础复习
- 听写训练
- 错题录入（初版）

---

### V2：学习系统阶段
- TodayPage（今日学习入口）
- Stats（学习统计）
- 错题系统独立模块
- 多端同步基础完成

---

### V3：学习流系统阶段
- Session 学习流（queue驱动）
- 单词 / 错题 / 听写统一进入学习流
- results 学习行为记录
- next 自动推进
- 初版学习闭环形成

---

### V4：学习智能调度阶段（当前）
- Session 动态排序（weight + recent performance）
- 错题优先级提升
- fatigue（学习疲劳）控制
- memory strength（记忆强度）引入
- TodayPage 单任务推荐逻辑

---

### V4.2：记忆驱动系统（进行中）
- 错题插队机制（error injection）
- 记忆衰减雏形（recency-based decay）
- session results 记录学习质量
- 学习行为开始可分析化

---

## 🧠 核心模块结构

### Learning Layer（执行层）
- WordReview
- ErrorReview
- Dictation
- FreePractice
- SessionPage（统一执行入口）

---

### Intelligence Layer（调度层）
- useLearningSession.ts
- 动态权重系统
- 错题优先级系统
- fatigue model

---

### Memory Layer（记忆层）
- cards / error_items
- review_logs / error_review_logs
- mastery_level
- repetitions
- next_review

---

### Feedback Layer（反馈层）
- StatsPage
- streak system
- accuracy
- study time
- weekly activity

---

## 🚀 当前正在推进的方向

### Learning Intelligence 完善
- memory decay model
- error injection system
- adaptive queue generator
- session-level optimization

### 2026-06-28 业务主线复核
- 客户端错题复习闭环已落地：待复习队列、错题复习页、评分、SM-2 更新、review log、TodayPage 入口。
- 客户端错题同步闭环已落地：op log、push/pull、cursor、server snapshot、冲突记录、token refresh。
- 本地 ready 但未同步成功的错题已允许先进入复习，不再被 `remote_id` 阻断。
- 本轮验证结果：`cargo test` 39 passed，`npm run build` success。
- 服务端 `wrong-notebook` 仍需接入当前 CodexPro allowed roots 后复验集成测试。

---

## 🧭 下一步计划

### V5：Memory Prediction Engine（预测层）
- 预测遗忘内容
- 提前召回机制
- 学习负载控制
- 个性化学习节奏

---

## 🧠 产品状态总结

当前系统演化路径：

工具 → 学习系统 → 学习流 → 调度系统 → 记忆系统（进行中）

最终目标：

> Memory-driven Learning OS（记忆驱动学习系统）