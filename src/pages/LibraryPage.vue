<script setup>
import { computed, nextTick, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { BookOpen, Headphones, Import, Search, Sigma } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'

const router = useRouter()
const store = useAppStore()

const searchTerm = ref('')
const activeFilter = ref('all')
const searchInput = ref(null)
const libraryHeroArt = new URL('../assets/hero/library-hero-bg.png', import.meta.url).href
const libraryStatNewIcon = new URL('../assets/icons/library-stat-new.svg', import.meta.url).href
const libraryStatReviewIcon = new URL('../assets/icons/library-stat-review.svg', import.meta.url).href

const filters = [
  { key: 'all', label: '全部' },
  { key: 'learning', label: '学习中' },
  { key: 'mastered', label: '已掌握' },
]

const deckThemes = [
  'library-deck-card-blue',
  'library-deck-card-green',
  'library-deck-card-orange',
  'library-deck-card-cyan',
]

onMounted(() => {
  store.refreshAll()
})

const totals = computed(() => {
  const totalCards = store.decks.reduce((sum, deck) => sum + getDeckTotal(deck), 0)
  const mastered = store.decks.reduce((sum, deck) => sum + getDeckMastered(deck), 0)
  const todayNew = toCount(store.learningStats?.newCards)
  const dueCards = toCount(store.learningStats?.dueCards)

  return {
    decks: store.decks.length,
    totalCards,
    mastered,
    todayNew,
    dueCards,
  }
})

const filteredDecks = computed(() => {
  const keyword = searchTerm.value.trim().toLowerCase()

  return store.sortedDecks.filter((deck) => {
    const ratio = getRatio(deck)
    const total = getDeckTotal(deck)

    const matchesFilter =
      activeFilter.value === 'all' ||
      (activeFilter.value === 'learning' && total > 0 && ratio < 100) ||
      (activeFilter.value === 'mastered' && total > 0 && ratio >= 100)

    const matchesKeyword =
      !keyword ||
      deck.name.toLowerCase().includes(keyword)

    return matchesFilter && matchesKeyword
  })
})

function toCount(value) {
  if (typeof value === 'bigint') return Number(value)
  if (typeof value === 'number') return value
  return 0
}

function getDeckTotal(deck) {
  return toCount(deck.wordCount || deck.word_count || deck.total || 0)
}

function getDeckMastered(deck) {
  return toCount(deck.masteredCount || deck.mastered_count || deck.mastered || 0)
}

function getDeckDue(deck) {
  return toCount(deck.dueCount || deck.due_count || deck.due || 0)
}

function getRatio(deck) {
  const total = getDeckTotal(deck)
  if (!total) return 0
  return Math.round((getDeckMastered(deck) / total) * 100)
}

function getTheme(index) {
  return deckThemes[index % deckThemes.length]
}

function getCoverLines(name) {
  const plain = name.replace(/\s+/g, '')
  if (plain.length <= 4) return [plain]
  if (plain.length <= 8) return [plain.slice(0, 4), plain.slice(4)]
  return [plain.slice(0, 4), plain.slice(4, 8)]
}

function getDeckTag(deck) {
  const name = deck.name

  if (name.includes('听写') || name.includes('听力')) {
    return { label: '听写', unit: '词' }
  }
  if (name.includes('错题') || name.includes('数学')) {
    return { label: '错题关联', unit: '条' }
  }
  if (name.includes('英语') || name.includes('高考') || name.includes('核心词')) {
    return { label: '英语', unit: '词' }
  }

  return { label: '词库', unit: '词' }
}

function getDeckCoverIcon(deck) {
  const name = deck.name

  if (name.includes('听写') || name.includes('听力')) return Headphones
  if (name.includes('错题') || name.includes('数学')) return Sigma
  return BookOpen
}

function getProgressStyle(index) {
  if (index % deckThemes.length === 1) return { background: 'linear-gradient(90deg, #35c48c 0%, #6ad5a9 100%)' }
  if (index % deckThemes.length === 2) return { background: 'linear-gradient(90deg, #ff9f43 0%, #ffc163 100%)' }
  if (index % deckThemes.length === 3) return { background: 'linear-gradient(90deg, #40bddf 0%, #7fdff1 100%)' }
  return {}
}

function openDeck(deckId) {
  router.push({ name: 'DeckDetail', params: { id: deckId } })
}

function goImport() {
  router.push({ name: 'Import' })
}

async function focusSearch() {
  await nextTick()
  searchInput.value?.focus()
}
</script>

<template>
  <div class="app-page library-page flex min-h-screen flex-col">
    <main class="library-shell">
      <header class="library-topbar">
        <div class="min-w-0">
          <h1 class="library-title">我的知识库</h1>
          <p class="library-subtitle">把导入的词库整理成可复习的路径</p>
        </div>

        <div class="library-header-actions">
          <button class="library-header-action" title="导入" @click="goImport">
            <Import class="h-6 w-6" />
            <span>导入</span>
          </button>
          <button class="library-header-action" title="搜索" @click="focusSearch">
            <Search class="h-6 w-6" />
            <span>搜索</span>
          </button>
        </div>
      </header>

      <section class="library-hero library-hero-surface" :style="{ '--hero-image': `url(${libraryHeroArt})` }">
        <div class="library-hero-grid">
          <div class="library-hero-copy">
            <div class="library-hero-kicker">
              <BookOpen class="h-4.5 w-4.5" />
              知识库总数
            </div>

            <div class="library-hero-number-row">
              <span class="library-hero-number">{{ totals.decks }}</span>
              <span class="library-hero-unit">个词库</span>
            </div>

            <div class="library-hero-stats">
              <div class="library-hero-stat">
                <div class="library-hero-stat-label">
                  <img class="library-hero-stat-icon" :src="libraryStatNewIcon" alt="" aria-hidden="true" />
                  今日新增
                </div>
                <div class="library-hero-stat-value">
                  {{ totals.todayNew }}
                  <span>词</span>
                </div>
              </div>

              <div class="library-hero-stat">
                <div class="library-hero-stat-label">
                  <img class="library-hero-stat-icon" :src="libraryStatReviewIcon" alt="" aria-hidden="true" />
                  待复习
                </div>
                <div class="library-hero-stat-value">
                  {{ totals.dueCards }}
                  <span>词</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class="library-controls">
        <label class="library-searchbar">
          <Search class="h-5 w-5 shrink-0 text-[#8a9bbc]" />
          <input
            ref="searchInput"
            v-model="searchTerm"
            type="text"
            class="library-search-input"
            placeholder="搜索词库或单词"
          />
        </label>

        <div class="library-filterbar">
          <button
            v-for="filter in filters"
            :key="filter.key"
            class="library-filter-button"
            :class="{ 'library-filter-button-active': activeFilter === filter.key }"
            @click="activeFilter = filter.key"
          >
            {{ filter.label }}
          </button>
        </div>
      </section>

      <section class="library-list">
        <article
          v-for="(deck, index) in filteredDecks"
          :key="deck.id"
          class="library-deck-card"
          role="button"
          tabindex="0"
          @click="openDeck(deck.id)"
          @keydown.enter.prevent="openDeck(deck.id)"
          @keydown.space.prevent="openDeck(deck.id)"
        >
          <div class="library-deck-cover" :class="getTheme(index)">
            <span class="library-deck-cover-bookmark" />
            <span class="library-deck-cover-ring" v-if="index % deckThemes.length === 2" />
            <component :is="getDeckCoverIcon(deck)" class="library-deck-cover-icon" />
            <div class="library-deck-cover-text">
              <span v-for="line in getCoverLines(deck.name)" :key="line">{{ line }}</span>
            </div>
          </div>

          <div class="library-deck-body">
            <div class="library-deck-head">
              <div class="min-w-0">
                <h2 class="library-deck-title">{{ deck.name }}</h2>
                <div class="library-deck-meta">
                  <span class="library-deck-tag">{{ getDeckTag(deck).label }}</span>
                  <span>{{ getDeckTotal(deck).toLocaleString() }} {{ getDeckTag(deck).unit }}</span>
                </div>
              </div>

              <div class="library-deck-side">
                <span class="library-deck-menu" aria-hidden="true">
                  <i />
                  <i />
                  <i />
                </span>
                <div class="library-deck-percent">
                  {{ getRatio(deck) }}%
                  <span class="library-deck-chevron">›</span>
                </div>
              </div>
            </div>

            <div class="library-deck-progress">
              <div class="library-deck-progress-track">
                <div
                  class="library-deck-progress-fill"
                  :style="{ width: `${getRatio(deck)}%`, ...getProgressStyle(index) }"
                />
              </div>
            </div>

            <p class="library-deck-footnote">
              已掌握 {{ getDeckMastered(deck).toLocaleString() }} {{ getDeckTag(deck).unit }}
            </p>
          </div>
        </article>

        <div v-if="filteredDecks.length === 0" class="library-empty-card">
          <BookOpen class="mx-auto mb-3 h-11 w-11 text-blue-200" />
          <p class="library-empty-title">{{ store.decks.length === 0 ? '还没有知识库' : '没有匹配结果' }}</p>
          <p class="library-empty-copy">
            {{ store.decks.length === 0 ? '导入文本文件后，就能把知识点整理成可复习的卡片。' : '试试更短的关键词，或者切换到其他筛选状态。' }}
          </p>
          <button
            v-if="store.decks.length === 0"
            class="library-empty-action"
            @click="goImport"
          >
            导入文件
          </button>
        </div>
      </section>
    </main>

    <BottomNav />
  </div>
</template>

<style scoped>
.library-page {
  background:
    linear-gradient(rgba(89, 122, 194, 0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(89, 122, 194, 0.015) 1px, transparent 1px),
    radial-gradient(circle at 22% 0%, rgba(110, 154, 245, 0.075), transparent 32%),
    linear-gradient(180deg, #fffefe 0%, #f7faff 42%, #eef4fc 100%);
  background-size: 28px 28px, 28px 28px, auto, auto;
}

.library-shell {
  flex: 1;
  padding: calc(var(--safe-area-top) + 0.9rem) 1rem 6.95rem;
}

.library-topbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.9rem;
  margin-bottom: 0.92rem;
  padding: 0.15rem 0.1rem 0;
}

.library-title {
  margin: 0;
  color: #1a326c;
  font-size: 1.64rem;
  line-height: 1;
  font-weight: 950;
}

.library-subtitle {
  margin: 0.36rem 0 0;
  color: #657ba9;
  font-size: 0.82rem;
  line-height: 1.38;
  font-weight: 650;
}

.library-header-actions {
  display: flex;
  align-items: flex-start;
  gap: 0.76rem;
  padding-top: 0.14rem;
}

.library-header-action {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.22rem;
  border: 0;
  background: transparent;
  color: #294785;
  font-size: 0.74rem;
  font-weight: 750;
}

.library-hero,
.library-filterbar,
.library-deck-card,
.library-empty-card {
  border: 1px solid rgba(219, 228, 244, 0.94);
  border-radius: 1.18rem;
  background:
    radial-gradient(circle at 78% 12%, rgba(93, 133, 225, 0.06), transparent 24%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.97) 0%, rgba(251, 253, 255, 0.94) 100%);
  box-shadow:
    0 10px 22px rgba(75, 104, 164, 0.032),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.library-hero {
  position: relative;
  overflow: hidden;
  padding: 0.92rem 0.96rem 0.84rem;
}

.library-hero-surface {
  background:
    radial-gradient(circle at 74% 46%, rgba(108, 149, 255, 0.14), transparent 28%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.985) 0%, rgba(248, 251, 255, 0.95) 100%);
}

.library-hero-surface::before {
  content: "";
  position: absolute;
  inset: 0;
  background-image: var(--hero-image);
  background-position: right -0.15rem center;
  background-repeat: no-repeat;
  background-size: auto 108%;
  pointer-events: none;
}

.library-hero-surface::after {
  content: "";
  position: absolute;
  inset: 0;
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.98) 0%, rgba(255, 255, 255, 0.9) 34%, rgba(255, 255, 255, 0.58) 56%, rgba(255, 255, 255, 0.08) 80%);
  pointer-events: none;
}

.library-hero-surface > * {
  position: relative;
  z-index: 1;
}

.library-hero-grid {
  min-height: 9.96rem;
}

.library-hero-copy {
  max-width: 12rem;
}

.library-hero-kicker {
  display: inline-flex;
  align-items: center;
  gap: 0.48rem;
  color: #203f81;
  font-size: 0.8rem;
  font-weight: 800;
}

.library-hero-number-row {
  display: flex;
  align-items: flex-end;
  gap: 0.42rem;
  margin-top: 0.58rem;
}

.library-hero-number {
  color: #245dff;
  font-size: 4.22rem;
  line-height: 0.9;
  font-weight: 950;
  letter-spacing: -0.04em;
}

.library-hero-unit {
  padding-bottom: 0.42rem;
  color: #495f95;
  font-size: 0.84rem;
  font-weight: 700;
}

.library-hero-stats {
  display: grid;
  grid-template-columns: repeat(2, max-content);
  gap: 0;
  margin-top: 0.7rem;
  width: fit-content;
}

.library-hero-stat {
  padding: 0 0.16rem 0 0;
}

.library-hero-stat + .library-hero-stat {
  border-left: 1px solid rgba(228, 234, 246, 0.95);
  padding-left: 0.72rem;
  margin-left: 0.44rem;
}

.library-hero-stat-label {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  white-space: nowrap;
  color: #445b91;
  font-size: 0.73rem;
  font-weight: 700;
}

.library-hero-stat-icon {
  width: 1.36rem;
  height: 1.36rem;
  flex: 0 0 auto;
}

.library-hero-stat-value {
  display: flex;
  align-items: flex-end;
  gap: 0.18rem;
  margin-top: 0.3rem;
  color: #1d3369;
  font-size: 1.18rem;
  line-height: 0.95;
  font-weight: 900;
  white-space: nowrap;
}

.library-hero-stat-value span {
  color: #6d80aa;
  font-size: 0.7rem;
  font-weight: 700;
  transform: translateY(-0.04rem);
}

.library-hero-illustration {
  position: relative;
  height: 11rem;
  display: none;
}

.library-hero-image {
  position: absolute;
  inset: 0.35rem 0 0.15rem 0.15rem;
  background-position: center;
  background-repeat: no-repeat;
  background-size: contain;
}

.library-hero-gridlines {
  position: absolute;
  inset: 1.25rem 0.35rem 1.2rem 0.2rem;
  background:
    linear-gradient(rgba(119, 153, 230, 0.12) 1px, transparent 1px),
    linear-gradient(90deg, rgba(119, 153, 230, 0.12) 1px, transparent 1px);
  background-size: 0.8rem 0.8rem;
  transform: perspective(240px) rotateX(67deg) rotateZ(-31deg);
  transform-origin: center;
  opacity: 0.48;
}

.library-controls {
  margin-top: 0.8rem;
}

.library-searchbar {
  display: flex;
  align-items: center;
  gap: 0.62rem;
  min-height: 2.5rem;
  border: 1px solid rgba(216, 226, 244, 0.92);
  border-radius: 999px;
  background: linear-gradient(180deg, rgba(252, 253, 255, 0.98), rgba(246, 250, 255, 0.96));
  padding: 0 0.95rem;
  box-shadow:
    0 8px 16px rgba(75, 104, 164, 0.022),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.library-search-input {
  width: 100%;
  border: 0;
  background: transparent;
  color: #274277;
  font-size: 0.88rem;
  outline: none;
}

.library-search-input::placeholder {
  color: #8fa0c2;
}

.library-filterbar {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0;
  margin-top: 0.6rem;
  padding: 0.12rem;
  background: linear-gradient(180deg, rgba(252, 253, 255, 0.98), rgba(246, 250, 255, 0.96));
}

.library-filter-button {
  min-height: 2.08rem;
  border: 0;
  border-left: 1px solid rgba(231, 236, 247, 0.94);
  background: transparent;
  color: #425987;
  font-size: 0.86rem;
  font-weight: 800;
}

.library-filter-button:first-child {
  border-left: 0;
}

.library-filter-button-active {
  border-left: 0;
  border-radius: 0.98rem;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(251, 253, 255, 0.94));
  color: #245dff;
  box-shadow:
    0 6px 16px rgba(80, 110, 176, 0.05),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.library-filter-button-active + .library-filter-button {
  border-left-color: transparent;
}

.library-list {
  display: grid;
  gap: 0.68rem;
  margin-top: 0.76rem;
}

.library-deck-card {
  display: flex;
  align-items: center;
  gap: 0.84rem;
  padding: 0.8rem 0.84rem 0.76rem;
  text-align: left;
  cursor: pointer;
  transition: transform 180ms ease, box-shadow 180ms ease, border-color 180ms ease;
}

.library-deck-card:active {
  transform: translateY(1px) scale(0.997);
}

.library-deck-cover {
  position: relative;
  display: flex;
  width: 4.92rem;
  height: 6.26rem;
  flex: 0 0 auto;
  align-items: flex-end;
  overflow: hidden;
  border-radius: 1rem;
  border: 1px solid rgba(255, 255, 255, 0.64);
  padding: 0.76rem 0.68rem;
  color: white;
  box-shadow:
    0 10px 22px rgba(95, 126, 194, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.26);
}

.library-deck-cover::before {
  content: "";
  position: absolute;
  inset: 0.12rem;
  border-radius: 0.88rem;
  border: 1px solid rgba(255, 255, 255, 0.32);
}

.library-deck-cover::after {
  content: "";
  position: absolute;
  left: 0.58rem;
  top: 0.6rem;
  width: 1.22rem;
  height: 0.3rem;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.32);
  transform: rotate(-24deg);
}

.library-deck-card-blue {
  background: linear-gradient(160deg, #4f88ff 0%, #2d63f0 100%);
}

.library-deck-card-green {
  background: linear-gradient(160deg, #66d4ba 0%, #33b493 100%);
}

.library-deck-card-orange {
  background: linear-gradient(160deg, #ffba4f 0%, #f49b25 100%);
}

.library-deck-card-cyan {
  background: linear-gradient(160deg, #54a9ff 0%, #2885da 100%);
}

.library-deck-cover-bookmark {
  position: absolute;
  right: 0.52rem;
  top: 0;
  width: 0.74rem;
  height: 1.18rem;
  background: rgba(255, 242, 194, 0.88);
  clip-path: polygon(0 0, 100% 0, 100% 82%, 50% 100%, 0 82%);
}

.library-deck-cover-ring {
  position: absolute;
  left: -0.2rem;
  top: 0.76rem;
  width: 0.38rem;
  height: 4rem;
  border-radius: 999px;
  background:
    radial-gradient(circle at center, rgba(255, 180, 76, 0.98) 0 42%, transparent 43% 100%);
  background-size: 100% 0.94rem;
  background-repeat: repeat-y;
}

.library-deck-cover-icon {
  position: absolute;
  right: 0.5rem;
  bottom: 0.46rem;
  width: 0.92rem;
  height: 0.92rem;
  color: rgba(255, 255, 255, 0.72);
}

.library-deck-cover-text {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 0.28rem;
  font-size: 0.56rem;
  line-height: 1.42;
  font-weight: 800;
  letter-spacing: 0.01em;
}

.library-deck-body {
  min-width: 0;
  flex: 1;
  padding-top: 0.05rem;
}

.library-deck-head {
  display: flex;
  justify-content: space-between;
  gap: 0.88rem;
}

.library-deck-title {
  margin: 0;
  color: #1f3367;
  font-size: 0.94rem;
  line-height: 1.2;
  font-weight: 900;
}

.library-deck-meta {
  display: flex;
  align-items: center;
  gap: 0.58rem;
  margin-top: 0.36rem;
  color: #6479a8;
  font-size: 0.72rem;
  font-weight: 650;
}

.library-deck-tag {
  display: inline-flex;
  align-items: center;
  min-height: 1.48rem;
  border: 1px solid rgba(205, 221, 255, 0.92);
  border-radius: 0.74rem;
  background: rgba(255, 255, 255, 0.94);
  padding: 0 0.48rem;
  color: #245dff;
  font-weight: 800;
}

.library-deck-side {
  display: grid;
  justify-items: end;
  gap: 0.58rem;
}

.library-deck-menu {
  display: inline-flex;
  gap: 0.22rem;
}

.library-deck-menu i {
  width: 0.22rem;
  height: 0.22rem;
  border-radius: 999px;
  background: #7b8db2;
}

.library-deck-percent {
  color: #245dff;
  font-size: 0.88rem;
  font-weight: 900;
}

.library-deck-chevron {
  margin-left: 0.12rem;
  color: #8ca0ca;
}

.library-deck-progress {
  margin-top: 0.54rem;
}

.library-deck-progress-track {
  overflow: hidden;
  height: 0.28rem;
  border-radius: 999px;
  background: #e5ebfa;
}

.library-deck-progress-fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #2f63ef 0%, #4d8bff 100%);
}

.library-deck-footnote {
  margin: 0.38rem 0 0;
  color: #667aa7;
  font-size: 0.68rem;
  font-weight: 650;
}

.library-empty-card {
  min-height: 15.9rem;
  padding: 2.6rem 1.4rem 2.3rem;
  text-align: center;
}

.library-empty-title {
  margin: 0;
  color: #435984;
  font-size: 0.98rem;
  font-weight: 800;
}

.library-empty-copy {
  margin: 0.72rem auto 0;
  max-width: 16.5rem;
  color: #8a9abc;
  font-size: 0.8rem;
  line-height: 1.75;
}

.library-empty-action {
  min-height: 3rem;
  margin-top: 1.48rem;
  border: 1px solid rgba(59, 106, 255, 0.28);
  border-radius: 1rem;
  background: linear-gradient(135deg, #3f78ff 0%, #245dff 100%);
  padding: 0 1.55rem;
  color: white;
  font-size: 0.92rem;
  font-weight: 850;
  box-shadow:
    0 10px 18px rgba(54, 94, 210, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.22);
}

@media (max-width: 720px) {
  .library-deck-card {
    padding: 1rem;
  }
}

@media (max-width: 420px) {
  .library-hero-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .library-hero-illustration {
    height: 9rem;
    margin-top: 0.3rem;
  }
}
</style>
