# 错题复习页面 UI 美化改造

Updated: 2026-06-28T09:03:20.110Z
Workspace: /Users/caogenyan/Documents/hamburger
Target agent: Codex (codex)

## Plan

# 错题复习页面 UI 美化改造

## 背景

用户提供了当前错题列表页面截图，并希望把现有 `错题复习` 页面 UI 改得更好看。

当前方向：

- 错题列表页已经形成较成熟的视觉语言：白底、浅蓝渐变、圆角卡片、柔和描边、轻量阴影、蓝色主按钮、底部导航、整体偏精致学习 App。
- 错题复习页应该和错题列表页属于同一产品家族。
- 这次只做 UI/交互呈现层优化，不改复习算法、不改同步协议、不改 Tauri command、不改数据库。

用户已确认要让本地 Codex 执行界面修改。

## 目标

重构 `src/pages/ErrorReviewPage.vue` 的视觉层，让它接近下面这个设计方向：

```txt
页面名称：错题复习
设计关键词：轻盈、专注、精致、学习感、同错题列表页视觉一致
主色：#1f66ff / #2f7cff 系蓝色
背景：白色 + 浅蓝径向渐变
卡片：大圆角、细蓝描边、柔和阴影
内容：题目卡片更突出，答案解析更清晰，评分按钮更像底部浮层
```

参考设计描述：

1. 顶部 Header
   - 左侧返回按钮，圆角浅色按钮。
   - 中间标题：`错题复习`。
   - 标题下方小字：`把错因重新学会，推进到长期记忆`。
   - 右侧进度胶囊：`1 / 3`，当前数字蓝色，带一点进度条视觉。
   - Header 要避开安全区，适合 iPhone/Tauri mobile。

2. 顶部题目 Hero Card
   - 大圆角卡片，浅蓝/白渐变背景。
   - 细蓝色 border，轻阴影。
   - 左侧或左上展示题目图片缩略图，如果有 `imageSrc`。
   - 顶部显示两个小 pill：
     - `今日待复习`
     - `Lv. {{ current.masteryLevel }}`
   - 题干用较大字号、深蓝色、加粗。
   - 知识点 tag 用浅蓝小胶囊。
   - 未展开答案时显示主按钮：`展开答案`。
   - 右侧可以用 CSS 做淡淡的立方体/几何装饰，不需要新增图片资源；也可以用现有 lucide icon / CSS gradient 实现。

3. 答案与解析区
   - 展开答案后显示。
   - 使用分段 tab/segmented control：
     - `答案与解析`
     - `错因与笔记`
   - 当前激活 tab 仍沿用现有 `activeTab` 逻辑。
   - `答案与解析` 内容中：
     - `标准答案` section：显示 `current.answerText`，fallback `暂无答案`。
     - `解析` section：显示 `current.analysis`，fallback `暂无解析`。
   - `错因与笔记` 内容中：
     - `错答记录`
     - `错因分析`
     - `笔记`
   - 每个 section 使用小图标 + 标题 + RichText，层级清晰。
   - 如果 `错因与笔记` 暂无内容，要给一个优雅空状态，例如：`这道题还没有记录错因，复习后可以补充。`

4. 底部评分浮层
   - 答案展开后，底部固定显示三个评分按钮：
     - `忘了` / `基本没想起来`
     - `模糊` / `思路还不稳定`
     - `掌握` / `能独立做出`
   - 三个按钮使用浅红 / 浅橙 / 浅绿，但要柔和，不要刺眼。
   - 按钮更大、更圆润、更像移动端底部操作区。
   - 注意底部安全区 `var(--safe-area-bottom)`。

5. 完成态
   - 保留完成态，但美化为和整体一致的圆角卡片。
   - 文案可以保持：`本轮错题复习完成`。
   - 按钮：`返回错题本`。

## 当前代码位置

主要修改：

```txt
src/pages/ErrorReviewPage.vue
```

可参考但尽量不要修改：

```txt
src/pages/ErrorNotebookPage.vue
src/pages/TodayPage.vue
src/style.css
```

## 重要约束

这次只做 UI，不要改业务逻辑。

不要修改：

```txt
src-tauri/src/db/error_repo.rs
src-tauri/src/commands/error_item.rs
src/api/errorItem.ts
ERROR_SYNC_PROTOCOL.md
DECISIONS.md
RECALL_PROGRESS.md
```

除非构建失败且必须小修类型问题，否则不要碰业务代码。

必须保留现有行为：

- `onMounted` 仍调用 `errorApi.getDueErrorItems()`。
- 没有待复习错题时进入完成态。
- `rate(0)`、`rate(3)`、`rate(5)` 对应三个评分。
- `errorApi.rateErrorItem(current.value.id, quality, seconds)` 调用不能变。
- 图片优先使用 `localImagePath`，失败后 fallback 到 `remoteImageUrl`。
- `RichText` 继续用于题干、答案、解析、错因、笔记。
- 不要引入新依赖。
- 不要新增图片资源。

## 建议实现方式

### Step 1：整理 template 层级

把页面结构调整为：

```txt
.app-page.review-page
  header.review-topbar
  main.review-shell
    section.review-done-card              // done
    template current
      section.review-hero-card            // 题目卡
      section.review-answer-card          // revealed 后显示
      section.review-note-preview 或 tab 内错因内容
  footer.review-rating-bar                // revealed 后显示评分
```

可以保留现有 class，也可以重命名，但不要让样式混乱。

### Step 2：Header 改造

当前 header 是：

```txt
.error-header
```

建议改成更接近错题列表页风格：

- 返回按钮左浮。
- 中间 title/subtitle。
- 右侧 progress pill。
- `progressLabel` 可以继续用，但最好拆成数字视觉：当前/总数。

如果实现拆数字比较麻烦，可以仍显示 `{{ progressLabel }}`，但样式要像胶囊。

### Step 3：题目 Hero Card

题目卡里保留：

- 图片
- nextReview / masteryLevel 信息
- 题干
- knowledge tags
- 展开答案按钮

视觉上改为：

- 题目卡整体背景：

```css
background:
  radial-gradient(circle at 82% 18%, rgba(47, 124, 255, 0.12), transparent 30%),
  linear-gradient(135deg, rgba(255,255,255,.98), rgba(238,245,255,.96));
```

- 右侧装饰可以用 pseudo-element：

```css
.review-hero-card::after { ... }
```

做淡蓝方块/网格感即可。

### Step 4：答案区卡片化

现有答案区域已经有 tab 和 content section，但视觉略普通。

请改为：

- `.review-tabs-panel` 外层圆角大卡。
- `.review-tabs` 像 segmented control。
- active tab 蓝色下划线/蓝色文字。
- `.review-content-card` 内部白色，section 分割线更轻。

### Step 5：底部评分按钮

当前底部按钮已经可用，但需要更高级：

- footer 固定底部。
- 背景白色半透明 + blur。
- 三个按钮等宽。
- 每个按钮有 icon / title / hint。
- 可继续使用纯 CSS 画表情，也可以只用文字和颜色。
- 不要引入新 icon 依赖；已有 lucide 可继续用。

### Step 6：响应式与移动端安全区

注意：

- 页面底部 padding 要给评分区留空间。
- `min-height: 100vh`。
- 使用已有 CSS 变量：`--safe-area-top`、`--safe-area-bottom`。
- 不要让内容被底部评分区遮挡。

## 推荐文案

Header subtitle：

```txt
把错因重新学会，推进到长期记忆
```

Hero pill：

```txt
今日待复习
Lv. {{ current.masteryLevel }}
```

展开按钮：

```txt
展开答案
```

评分按钮：

```txt
忘了
基本没想起来

模糊
思路还不稳定

掌握
能独立做出
```

完成态：

```txt
本轮错题复习完成
这次一共完成 {{ items.length }} 道题。回到错题本继续整理一两道，记忆会更稳。
```

## 验收标准

必须满足：

1. `npm run build` 通过。
2. 不修改业务逻辑和后端 Rust 代码。
3. 页面仍能：加载到期错题、展开答案、切换 tab、评分进入下一题、完成后展示完成态。
4. UI 风格和错题列表页一致：浅蓝、白底、圆角、轻阴影、精致。
5. 移动端底部评分区不遮挡内容。
6. 图片 fallback 行为仍保留。
7. 无新增依赖。

## 需要运行的验证

请至少运行：

```bash
npm run build
```

如果改动涉及 TypeScript 逻辑，也建议运行：

```bash
npm run build
```

当前项目没有单独 typecheck script，所以 build 是必要验证。

## 报告要求

完成后更新：

```txt
.ai-bridge/codex-report.md
.ai-bridge/agent-status.md
.ai-bridge/implementation-diff.patch
```

报告中写清楚：

1. 修改了哪些文件。
2. UI 主要改了哪些部分。
3. 是否改了业务逻辑；如果没有，明确写“未改业务逻辑”。
4. `npm run build` 结果。
5. 是否存在未解决问题。

## 额外提醒

当前 `.ai-bridge` 和自动化脚本本身不是本轮重点，不要继续优化协作脚本。

本轮只聚焦：

```txt
错题复习页面 UI 更好看
```

## Implementation contract

- Work from this plan in small, reviewable steps.
- Keep edits scoped to the requested task and existing project conventions.
- Run focused verification before handing work back.
- Update .ai-bridge/agent-status.md with files touched, checks run, results, blockers, and review notes.
- Save the final review diff to .ai-bridge/implementation-diff.patch when practical.
- Append notable execution events to .ai-bridge/execution-log.jsonl when the implementation agent supports logging.
