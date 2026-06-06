
export function parseTxtContent(text) {
  const lines = text.split('\n').map(l => l.trim()).filter(l => l.length > 0)
  if (lines.length === 0) {
    return { format: 'unknown', rows: [], preview: [], errors: [] }
  }

  // Count fields per line to detect format
  const fieldCounts = lines.map(l => l.split(/\s+/).length)
  const avgFields = Math.round(fieldCounts.reduce((a, b) => a + b, 0) / fieldCounts.length)

  // Detect format based on majority
  const count1 = fieldCounts.filter(c => c === 1).length
  const count2 = fieldCounts.filter(c => c === 2).length
  const count3plus = fieldCounts.filter(c => c >= 3).length

  let format
  if (count1 >= count2 && count1 >= count3plus) {
    format = 'A'
  } else if (count2 >= count1 && count2 >= count3plus) {
    format = 'B'
  } else {
    format = 'C'
  }

  const rows = []
  const errors = []

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const parts = line.split(/\s+/)
    const lineNum = i + 1

    if (format === 'A') {
      rows.push({ word: parts[0], inflections: [], definition: '' })
    } else if (format === 'B') {
      if (parts.length < 2) {
        errors.push({ line: lineNum, text: line, msg: '缺少释义' })
        rows.push({ word: parts[0], inflections: [], definition: '' })
      } else {
        rows.push({ word: parts[0], inflections: [], definition: parts.slice(1).join(' ') })
      }
    } else if (format === 'C') {
      if (parts.length < 3) {
        errors.push({ line: lineNum, text: line, msg: '格式 C 需要至少 3 个字段：单词 词形变化 释义' })
        continue
      }
      rows.push({
        word: parts[0],
        inflections: parts.slice(1, -1),
        definition: parts[parts.length - 1]
      })
    }
  }

  // Check for mixed format — lines that don't match the detected format
  const mixedErrors = []
  lines.forEach((line, i) => {
    const parts = line.split(/\s+/)
    if (format === 'A' && parts.length !== 1) {
      mixedErrors.push({ line: i + 1, text: line, msg: `该行有 ${parts.length} 个字段，但文件格式为 A（纯单词）` })
    } else if (format === 'B' && parts.length < 2) {
      mixedErrors.push({ line: i + 1, text: line, msg: `该行只有 1 个字段，但文件格式为 B（单词+释义）` })
    }
  })

  const preview = rows.slice(0, 3)

  return {
    format,
    rows,
    preview,
    errors: [...errors, ...mixedErrors],
    totalLines: lines.length,
    validCount: rows.length
  }
}
