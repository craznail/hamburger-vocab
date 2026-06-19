<script setup>
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ChevronRight, Flame, Settings, ShieldCheck } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'
import BottomNav from '../components/BottomNav.vue'

const router = useRouter()
const route = useRoute()
const store = useAppStore()
const errorStore = useErrorNotebookStore()
const profileHeroArt = new URL('../assets/hero/profile-hero-bg.png', import.meta.url).href
const studyStatsIcon = new URL('../assets/icons/profile-study-stats.svg', import.meta.url).href
const achievementIcon = new URL('../assets/icons/profile-achievement.svg', import.meta.url).href
const syncIcon = new URL('../assets/icons/profile-sync.svg', import.meta.url).href
const settingsIcon = new URL('../assets/icons/profile-settings.svg', import.meta.url).href
const ttsIcon = new URL('../assets/icons/profile-tts.svg', import.meta.url).href
const aboutIcon = new URL('../assets/icons/profile-about.svg', import.meta.url).href

const useDevMockData = computed(() => import.meta.env.DEV && route.query.real !== '1')

onMounted(() => {
  if (useDevMockData.value) return
  store.refreshAll()
  void errorStore.ensureFresh()
})

const profileSummary = computed(() => useDevMockData.value ? {
  deckCount: 4,
  masteredCount: 128,
  correctedCount: 36,
  streakDays: 7,
  displayName: '学习者',
  avatarInitial: 'R',
} : {
  deckCount: store.decks.length,
  masteredCount: Number(store.learningStats?.masteredCards || 0),
  correctedCount: Math.max(0, errorStore.dueCount || 0),
  streakDays: Number(store.learningStats?.streakDays || 0),
  displayName: '学习者',
  avatarInitial: 'R',
})

const studyRows = [
  { label: '学习统计', hint: '查看学习时长、掌握分布等数据', icon: studyStatsIcon, route: 'Stats' },
  { label: '学习成就', hint: '回顾里程碑，保持学习动力', icon: achievementIcon, route: 'Achievements' },
]

const settingRows = [
  { label: '同步与服务端', hint: '手动同步数据，管理服务', icon: syncIcon, route: 'Settings' },
  { label: '学习设置', hint: '卡片显示、复习顺序等', icon: settingsIcon, route: 'Settings' },
  { label: 'TTS 发音设置', hint: '语速、音色与发音引擎', icon: ttsIcon, route: 'Settings' },
  { label: '关于应用', hint: '版本信息与使用说明', icon: aboutIcon, route: 'Settings' },
]
</script>

<template>
  <div class="profile-page app-page flex min-h-screen flex-col">
    <div class="profile-shell">
      <header class="profile-topbar">
        <div class="min-w-0">
          <h1 class="profile-title">我的</h1>
          <p class="profile-subtitle">本地学习档案</p>
        </div>
        <button class="profile-settings-button" type="button" aria-label="打开设置" @click="router.push({ name: 'Settings' })">
          <Settings class="h-5.5 w-5.5" />
        </button>
      </header>

      <section class="profile-hero-card" :style="{ '--hero-image': `url(${profileHeroArt})` }">
        <div class="profile-hero-main">
          <div class="profile-avatar-shell">
            <div class="profile-avatar">{{ profileSummary.avatarInitial }}</div>
          </div>

          <div class="profile-hero-copy">
            <h2 class="profile-name">{{ profileSummary.displayName }}</h2>
            <div class="profile-streak-pill">
              <Flame class="h-4.5 w-4.5" />
              已连续学习 {{ profileSummary.streakDays }} 天
            </div>
          </div>
        </div>

        <div class="profile-stat-strip">
          <div class="profile-stat-item">
            <strong>{{ profileSummary.deckCount }}</strong>
            <span>知识库</span>
          </div>
          <div class="profile-stat-item">
            <strong>{{ profileSummary.masteredCount }}</strong>
            <span>已掌握</span>
          </div>
          <div class="profile-stat-item">
            <strong>{{ profileSummary.correctedCount }}</strong>
            <span>错题订正</span>
          </div>
        </div>
      </section>

      <section class="profile-section-card">
        <h2 class="profile-section-title">学习记录</h2>

        <div class="profile-list-card">
          <button
            v-for="row in studyRows"
            :key="row.label"
            class="profile-list-item"
            type="button"
            @click="router.push({ name: row.route })"
          >
            <img class="profile-list-icon" :src="row.icon" alt="" aria-hidden="true" />
            <span class="profile-list-copy">
              <span class="profile-list-title">{{ row.label }}</span>
              <span class="profile-list-hint">{{ row.hint }}</span>
            </span>
            <ChevronRight class="profile-list-chevron h-5 w-5" />
          </button>
        </div>
      </section>

      <section class="profile-section-card profile-section-card-settings">
        <h2 class="profile-section-title">数据与设置</h2>

        <div class="profile-list-card">
          <button
            v-for="row in settingRows"
            :key="row.label"
            class="profile-list-item"
            type="button"
            @click="router.push({ name: row.route })"
          >
            <img class="profile-list-icon" :src="row.icon" alt="" aria-hidden="true" />
            <span class="profile-list-copy">
              <span class="profile-list-title">{{ row.label }}</span>
              <span class="profile-list-hint">{{ row.hint }}</span>
            </span>
            <ChevronRight class="profile-list-chevron h-5 w-5" />
          </button>
        </div>
      </section>

      <p class="profile-device-note">
        <ShieldCheck class="h-4.5 w-4.5" />
        数据保存在当前设备，可按需同步到服务端
      </p>
    </div>

    <BottomNav />
  </div>
</template>

<style scoped>
.profile-page {
  background:
    linear-gradient(rgba(89, 122, 194, 0.012) 1px, transparent 1px),
    linear-gradient(90deg, rgba(89, 122, 194, 0.012) 1px, transparent 1px),
    radial-gradient(circle at 22% 0%, rgba(110, 154, 245, 0.075), transparent 32%),
    linear-gradient(180deg, #fffefe 0%, #f7faff 42%, #eef4fc 100%);
  background-size: 28px 28px, 28px 28px, auto, auto;
}

.profile-shell {
  flex: 1;
  padding: calc(var(--safe-area-top) + 0.96rem) 1rem 6.92rem;
}

.profile-topbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.profile-title {
  margin: 0;
  color: #152b62;
  font-size: 1.74rem;
  line-height: 1.02;
  font-weight: 950;
}

.profile-subtitle {
  margin: 0.48rem 0 0;
  color: #6178a9;
  font-size: 0.96rem;
  line-height: 1.4;
  font-weight: 650;
}

.profile-settings-button {
  display: grid;
  width: 3.06rem;
  height: 3.06rem;
  flex: 0 0 auto;
  place-items: center;
  border: 1px solid rgba(217, 226, 243, 0.92);
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.94);
  color: #24407d;
  box-shadow:
    0 12px 22px rgba(82, 106, 159, 0.045),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.profile-hero-card,
.profile-section-card {
  position: relative;
  overflow: hidden;
  border: 1px solid rgba(216, 227, 245, 0.92);
  border-radius: 1.62rem;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.97) 0%, rgba(247, 250, 255, 0.97) 100%);
  box-shadow:
    0 18px 34px rgba(73, 100, 158, 0.055),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.profile-hero-card {
  margin-top: 1.18rem;
  padding: 1.2rem 1.06rem 1.06rem;
}

.profile-hero-card::before {
  content: "";
  position: absolute;
  inset: 0;
  background-image:
    radial-gradient(circle at right 38%, rgba(201, 220, 255, 0.11), transparent 30%),
    linear-gradient(90deg, rgba(255, 255, 255, 0.98) 0%, rgba(255, 255, 255, 0.88) 34%, rgba(248, 251, 255, 0.36) 56%, rgba(248, 251, 255, 0.7) 100%),
    var(--hero-image);
  background-position: right center, left top, right -0.86rem top 0.12rem;
  background-repeat: no-repeat;
  background-size: auto, auto, 95% auto;
  pointer-events: none;
}

.profile-hero-card > * {
  position: relative;
  z-index: 1;
}

.profile-hero-main {
  display: flex;
  align-items: center;
  gap: 1.02rem;
  min-height: 11.9rem;
}

.profile-avatar-shell {
  display: grid;
  width: 7.52rem;
  height: 7.52rem;
  flex: 0 0 auto;
  place-items: center;
  border-radius: 999px;
  background: rgba(245, 249, 255, 0.96);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.96),
    0 10px 22px rgba(93, 117, 174, 0.06);
}

.profile-avatar {
  display: grid;
  width: 6.44rem;
  height: 6.44rem;
  place-items: center;
  border-radius: 999px;
  background: linear-gradient(180deg, #8eb9ff 0%, #5c96ff 100%);
  color: white;
  font-size: 3.7rem;
  line-height: 1;
  font-weight: 500;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.32),
    0 14px 24px rgba(65, 117, 237, 0.14);
}

.profile-hero-copy {
  min-width: 0;
  flex: 1;
  padding-top: 0.18rem;
}

.profile-name {
  margin: 0;
  color: #152b62;
  font-size: 2.06rem;
  line-height: 1.06;
  font-weight: 950;
}

.profile-streak-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
  min-height: 2.18rem;
  margin-top: 0.98rem;
  padding: 0 0.94rem;
  border: 1px solid rgba(199, 239, 222, 0.95);
  border-radius: 999px;
  background: linear-gradient(180deg, rgba(242, 255, 249, 0.98) 0%, rgba(232, 252, 242, 0.98) 100%);
  color: #26be86;
  font-size: 0.84rem;
  font-weight: 850;
}

.profile-stat-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-top: 0.74rem;
  border: 1px solid rgba(223, 231, 246, 0.92);
  border-radius: 1.44rem;
  background: rgba(255, 255, 255, 0.92);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.profile-stat-item {
  display: grid;
  justify-items: center;
  gap: 0.38rem;
  padding: 1.38rem 0.55rem 1.14rem;
}

.profile-stat-item + .profile-stat-item {
  border-left: 1px solid rgba(223, 231, 247, 0.92);
}

.profile-stat-item strong {
  color: #152b62;
  font-size: 2.34rem;
  line-height: 1;
  font-weight: 950;
}

.profile-stat-item span {
  color: #637aa9;
  font-size: 0.92rem;
  font-weight: 700;
}

.profile-section-card {
  margin-top: 1.22rem;
  padding: 1.22rem 1.06rem 1.08rem;
}

.profile-section-card-settings {
  margin-top: 1.12rem;
}

.profile-section-title {
  margin: 0 0 1rem;
  color: #152b62;
  font-size: 1.26rem;
  line-height: 1.1;
  font-weight: 950;
}

.profile-list-card {
  overflow: hidden;
  border: 1px solid rgba(221, 230, 247, 0.94);
  border-radius: 1.34rem;
  background: rgba(255, 255, 255, 0.9);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.profile-list-item {
  display: flex;
  align-items: center;
  gap: 0.98rem;
  width: 100%;
  padding: 1.18rem 1.04rem;
  border: 0;
  border-bottom: 1px solid rgba(226, 234, 248, 0.94);
  background: transparent;
  text-align: left;
}

.profile-list-item:last-child {
  border-bottom: 0;
}

.profile-list-icon {
  width: 3.64rem;
  height: 3.64rem;
  flex: 0 0 auto;
}

.profile-list-copy {
  display: grid;
  min-width: 0;
  flex: 1;
}

.profile-list-title {
  color: #152b62;
  font-size: 1.02rem;
  font-weight: 900;
}

.profile-list-hint {
  margin-top: 0.34rem;
  color: #7184ad;
  font-size: 0.88rem;
  line-height: 1.42;
  font-weight: 650;
}

.profile-list-chevron {
  color: #9aaacc;
  flex: 0 0 auto;
}

.profile-device-note {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.48rem;
  margin: 1.12rem 0 0.24rem;
  color: #7f91b8;
  font-size: 0.82rem;
  line-height: 1.5;
  font-weight: 650;
  text-align: center;
}
</style>
