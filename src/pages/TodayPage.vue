<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  BookOpen,
  CalendarDays,
  ChevronRight,
  CirclePlay,
  Dices,
  GraduationCap,
  Headphones,
  NotebookPen,
  Sparkles,
} from 'lucide-vue-next'
import BottomNav from '../components/BottomNav.vue'
import { useLearningStore, type LearningSection } from '../stores/useLearningStore'

const router = useRouter()
const learning = useLearningStore()

onMounted(() => {
  void learning.refresh()
})

const focusTitle = computed(() => {
  if (learning.totalDueCount > 0) return '今天先完成这一组'
  return '今日计划已完成'
})

const focusSubtitle = computed(() => {
  if (learning.totalDueCount > 0) return '把到期知识重新拉回记忆里'
  return '可以自由练习，或整理新的知识来源'
})

function goSection(section: LearningSection) {
  if (!section.enabled) {
    if (section.type === 'error_review') {
      router.push({ name: 'ErrorNotebook' })
      return
    }
    router.push({ name: 'Library' })
    return
  }
  router.push({ name: section.routeName, query: section.query || {} })
}

function goPrimary() {
  goSection(learning.primarySection)
}

function sectionIcon(type: LearningSection['type']) {
  return {
    word_review: GraduationCap,
    dictation: Headphones,
    error_review: NotebookPen,
    free_practice: Dices,
  }[type]
}

function sectionMeta(section: LearningSection) {
  if (!section.enabled) return section.type === 'error_review' ? '去录入' : '去导入'
  if (section.count > 0) return `${section.count}${section.unit}`
  return '开始'
}
</script>

<template>
  <div class="today-page app-page flex min-h-screen flex-col">
    <main class="today-shell">
      <header class="today-topbar">
        <div class="min-w-0">
          <h1 class="today-title">今日学习</h1>
          <p class="today-subtitle">Learning First，把知识推进到长期记忆</p>
        </div>
        <button class="today-calendar-action" type="button" @click="router.push({ name: 'Stats' })">
          <CalendarDays class="h-4.5 w-4.5" />
          <span>复习日历</span>
        </button>
      </header>

      <section class="today-focus-card">
        <div class="today-focus-kicker">
          <Sparkles class="h-4 w-4" />
          Recall Queue
        </div>
        <div class="today-focus-row">
          <div>
            <h2>{{ focusTitle }}</h2>
            <p>{{ focusSubtitle }}</p>
          </div>
          <div class="today-focus-number">
            <strong>{{ learning.totalDueCount }}</strong>
            <span>待复习</span>
          </div>
        </div>
        <button class="today-primary-action" type="button" @click="goPrimary">
          <CirclePlay class="h-5 w-5" />
          {{ learning.totalDueCount > 0 ? '开始今日学习' : '自由练习' }}
        </button>
      </section>

      <section class="today-section-card">
        <div class="today-section-head">
          <h2>学习任务</h2>
          <span v-if="learning.loading">同步中...</span>
        </div>

        <div class="today-task-list">
          <button
            v-for="section in learning.sections"
            :key="section.type"
            class="today-task-card"
            :class="{ 'today-task-card-muted': !section.enabled }"
            type="button"
            @click="goSection(section)"
          >
            <span class="today-task-icon" :class="`today-task-icon-${section.type}`">
              <component :is="sectionIcon(section.type)" class="h-5 w-5" />
            </span>
            <span class="today-task-copy">
              <span class="today-task-title">{{ section.title }}</span>
              <span class="today-task-desc">{{ section.description }}</span>
            </span>
            <span class="today-task-meta">
              <span>{{ sectionMeta(section) }}</span>
              <ChevronRight class="h-4 w-4" />
            </span>
          </button>
        </div>
      </section>

      <section class="today-section-card today-knowledge-card">
        <div class="today-section-head">
          <h2>知识来源</h2>
          <button type="button" @click="router.push({ name: 'Library' })">
            全部
            <ChevronRight class="h-4 w-4" />
          </button>
        </div>
        <div class="today-source-grid">
          <button type="button" class="today-source-card" @click="router.push({ name: 'Library' })">
            <BookOpen class="h-5 w-5" />
            <span>词库</span>
          </button>
          <button type="button" class="today-source-card" @click="router.push({ name: 'ErrorNotebook' })">
            <NotebookPen class="h-5 w-5" />
            <span>错题库</span>
          </button>
        </div>
      </section>
    </main>
    <BottomNav />
  </div>
</template>

<style scoped>
.today-page {
  background:
    radial-gradient(circle at 24% 0%, rgba(86, 128, 232, 0.11), transparent 31%),
    linear-gradient(180deg, #fffefe 0%, #f7faff 42%, #eef4fc 100%);
}

.today-shell {
  flex: 1;
  padding: calc(var(--safe-area-top) + 0.96rem) 1rem 6.95rem;
}

.today-topbar,
.today-section-head,
.today-focus-row,
.today-task-card,
.today-task-meta,
.today-source-grid,
.today-source-card {
  display: flex;
  align-items: center;
}

.today-topbar {
  justify-content: space-between;
  gap: 0.9rem;
  margin-bottom: 0.94rem;
  padding: 0.15rem 0.1rem 0;
}

.today-title {
  margin: 0;
  color: #1a326c;
  font-size: 1.62rem;
  line-height: 1;
  font-weight: 950;
}

.today-subtitle {
  margin: 0.34rem 0 0;
  color: #657ba9;
  font-size: 0.84rem;
  line-height: 1.38;
  font-weight: 650;
}

.today-calendar-action,
.today-section-head button {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid rgba(214, 223, 241, 0.95);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.94);
  color: #214fd8;
  font-size: 0.82rem;
  font-weight: 800;
}

.today-calendar-action {
  min-height: 2.18rem;
  padding: 0 0.84rem;
}

.today-focus-card,
.today-section-card {
  border: 1px solid rgba(219, 228, 244, 0.94);
  border-radius: 1.75rem;
  background:
    radial-gradient(circle at 78% 12%, rgba(93, 133, 225, 0.08), transparent 24%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.98) 0%, rgba(251, 253, 255, 0.96) 100%);
  box-shadow:
    0 16px 34px rgba(75, 104, 164, 0.05),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.today-focus-card {
  padding: 1.2rem;
}

.today-focus-kicker {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
  min-height: 1.85rem;
  border-radius: 999px;
  background: rgba(232, 240, 255, 0.92);
  padding: 0 0.72rem;
  color: #2f6ff1;
  font-size: 0.74rem;
  font-weight: 900;
}

.today-focus-row {
  justify-content: space-between;
  gap: 1rem;
  margin-top: 1rem;
}

.today-focus-row h2 {
  margin: 0;
  color: #172d65;
  font-size: 1.34rem;
  font-weight: 950;
}

.today-focus-row p {
  margin: 0.35rem 0 0;
  color: #6b7ea8;
  font-size: 0.9rem;
  font-weight: 650;
}

.today-focus-number {
  display: grid;
  min-width: 5.2rem;
  min-height: 5.2rem;
  place-items: center;
  border-radius: 1.45rem;
  background: linear-gradient(135deg, #3883ff 0%, #166fff 100%);
  color: white;
}

.today-focus-number strong {
  font-size: 2rem;
  line-height: 1;
  font-weight: 950;
}

.today-focus-number span {
  margin-top: -0.45rem;
  font-size: 0.72rem;
  font-weight: 800;
  opacity: 0.84;
}

.today-primary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  width: 100%;
  min-height: 3.35rem;
  margin-top: 1.1rem;
  border: 0;
  border-radius: 1.15rem;
  background: linear-gradient(135deg, #3883ff 0%, #166fff 100%);
  color: white;
  font-size: 0.94rem;
  font-weight: 950;
}

.today-section-card {
  margin-top: 0.8rem;
  padding: 1rem;
}

.today-section-head {
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.today-section-head h2 {
  margin: 0;
  color: #1a326c;
  font-size: 1.02rem;
  font-weight: 950;
}

.today-section-head span {
  color: #8ea0c6;
  font-size: 0.78rem;
  font-weight: 800;
}

.today-section-head button {
  border: 0;
  padding: 0;
  background: transparent;
}

.today-task-list {
  display: grid;
  gap: 0.55rem;
}

.today-task-card {
  width: 100%;
  gap: 0.85rem;
  border: 1px solid rgba(226, 234, 248, 0.96);
  border-radius: 1.18rem;
  background: rgba(255, 255, 255, 0.82);
  padding: 0.88rem;
  text-align: left;
}

.today-task-card-muted {
  opacity: 0.72;
}

.today-task-icon {
  display: grid;
  width: 2.8rem;
  height: 2.8rem;
  flex: 0 0 auto;
  place-items: center;
  border-radius: 1rem;
  color: white;
}

.today-task-icon-word_review,
.today-task-icon-free_practice {
  background: linear-gradient(135deg, #3e83ff 0%, #1d6fff 100%);
}

.today-task-icon-dictation {
  background: linear-gradient(135deg, #37d6a8 0%, #20b991 100%);
}

.today-task-icon-error_review {
  background: linear-gradient(135deg, #ffb94d 0%, #ff8f4d 100%);
}

.today-task-copy {
  display: grid;
  min-width: 0;
  flex: 1;
}

.today-task-title {
  color: #1c336e;
  font-size: 0.98rem;
  font-weight: 950;
}

.today-task-desc {
  margin-top: 0.22rem;
  color: #7f91b8;
  font-size: 0.79rem;
  line-height: 1.38;
  font-weight: 650;
}

.today-task-meta {
  gap: 0.2rem;
  color: #2f6ff1;
  font-size: 0.82rem;
  font-weight: 900;
}

.today-source-grid {
  gap: 0.6rem;
}

.today-source-card {
  flex: 1;
  justify-content: center;
  gap: 0.45rem;
  min-height: 3rem;
  border: 1px solid rgba(226, 234, 248, 0.96);
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.82);
  color: #1c336e;
  font-size: 0.9rem;
  font-weight: 900;
}
</style>
