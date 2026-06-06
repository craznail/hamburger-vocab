
const MIN_EF = 1.3
const MAX_EF = 2.5

function todayStr() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

function addDays(dateStr, days) {
  const d = new Date(dateStr)
  d.setDate(d.getDate() + days)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

/**
 * Compute next review schedule based on SM-2 algorithm
 *
 * @param {number} quality - 0 (forgot), 3 (hazy), 5 (mastered)
 * @param {object} card - current card state { ef, interval, repetitions }
 * @returns {{ ef: number, interval: number, repetitions: number, nextReview: string }}
 */
export function computeNextReview(quality, card) {
  let { ef, interval, repetitions } = card

  if (quality === 0) {
    // Forgot: reset
    repetitions = 0
    interval = 1
    ef = Math.max(MIN_EF, ef - 0.2)
  } else if (quality === 3) {
    // Hazy: reset repetitions, keep short interval
    repetitions = 0
    interval = 1
    ef = Math.max(MIN_EF, ef - 0.15)
  } else if (quality === 5) {
    // Mastered: increase interval
    repetitions += 1
    if (repetitions === 1) {
      interval = 1
    } else if (repetitions === 2) {
      interval = 6
    } else {
      interval = Math.ceil(interval * ef)
    }
    ef = Math.min(MAX_EF, ef + 0.1)
  }

  const nextReview = addDays(todayStr(), interval)

  return { ef, interval, repetitions, nextReview }
}
