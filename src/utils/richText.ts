import MarkdownIt from 'markdown-it'
import katex from 'katex'

const markdown = new MarkdownIt({
  html: false,
  breaks: false,
  linkify: false,
  typographer: false,
})

const MATH_PREFIX = '\uE000MATH'
const MATH_SUFFIX = '\uE001'

// ── Escapes repair ──

export function repairLatexEscapes(value: string): string {
  return value
    .replace(/\\\\([a-zA-Z]+)/g, '\\$1')
    .replace(/\u0008/g, '\\b')
    .replace(/\u0009/g, '\\t')
    .replace(/\u000B/g, '\\v')
    .replace(/\u000C/g, '\\f')
    .replace(/\u000D/g, '\\r')
    .replace(/\u000A(?=[A-Za-z])/g, '\\n')
    .replace(/\\\(/g, '$')
    .replace(/\\\)/g, '$')
    .replace(/\\\[/g, () => '$$')
    .replace(/\\\]/g, () => '$$')
}

export function normalizeInlineMath(value: string): string {
  return value.replace(
    /(^|[^$])\$([^$]+?)\$([^$]|$)/g,
    (_match, before, inner, after) => `${before}$${inner.trim()}$${after}`,
  )
}

// Paragraph formatting (NO \n conversion — done later, after math extraction)
function normalizeParagraphs(value: string): string {
  return value
    .replace(/\n\n/g, '\n\n###PRESERVE_BREAK###\n\n')
    .replace(/([。！？；])\n(?!\n)/g, '$1\n\n')
    .replace(/([.!?;])\s*\n(?!\n)/g, '$1\n\n')
    .replace(/(\d+\))\s*\n(?!\n)/g, '$1\n\n')
    .replace(/([\u2460-\u2473])\s*\n(?!\n)/g, '$1\n\n')
    .replace(/\n\s+([\u2460-\u2473])/g, '\n$1')
    .replace(/\n\s+(\d+\))/g, '\n$1')
    .replace(/([^\s$])(\$[^$]+\$)([^\s$])/g, '$1 $2 $3')
    .replace(/\s*###PRESERVE_BREAK###\s*/g, '\n\n')
}

export function wrapBareMath(value: string): string {
  if (/\$[^$]+\$/.test(value)) return value

  // Check for bare LaTeX commands, excluding \nA/\nB/\nC/\nD (AI newline markers)
  const latexCmds = /\\[a-zA-Z]{2,}/g
  let hasLatex = false
  let m: RegExpExecArray | null
  while ((m = latexCmds.exec(value)) !== null) {
    // Skip \n followed by a single uppercase letter — these are AI "\nA" markers
    if (/^\\n[A-Z]$/.test(m[0])) continue
    hasLatex = true
    break
  }
  if (!hasLatex) return value

  const lines = value.split('\n')
  if (lines.length === 1) return `$${value}$`
  return lines
    .map(line => {
      const trimmed = line.trim()
      return trimmed ? `$${trimmed}$` : line
    })
    .join('\n')
}

// ── KaTeX direct rendering ──

interface MathBlock {
  html: string
  display: boolean
}

function preRenderAllMath(source: string): { text: string; blocks: MathBlock[] } {
  const blocks: MathBlock[] = []

  let s = source.replace(/\$\$([^$]+?)\$\$/g, (_match, latex: string) => {
    const idx = blocks.length
    blocks[idx] = { html: renderKatex(latex.trim(), true), display: true }
    return `${MATH_PREFIX}${idx}${MATH_SUFFIX}`
  })

  s = s.replace(/(^|[^$])\$([^$]+?)\$([^$]|$)/g, (match, before, latex, after) => {
    const idx = blocks.length
    blocks[idx] = { html: renderKatex(latex.trim(), false), display: false }
    return `${before}${MATH_PREFIX}${idx}${MATH_SUFFIX}${after}`
  })

  return { text: s, blocks }
}

// Known LaTeX commands starting with \n (must survive cleanup)
const VALID_N_COMMANDS = /\\(?:neq|not|ni|nu|neg|nwarrow|nearrow|natural|normalsize|notin|nsubseteq|nsupseteq|nmid|nparallel|nsim|ncong|nleq|ngeq|nless|ngtr|nprec|nsucc|nVDash|nVdash|nvDash|nvdash|nsubseteqq|nsupseteqq|ntriangleleft|ntriangleright|ntrianglelefteq|ntrianglerighteq)\b/

function cleanLatexSource(latex: string): string {
  // Strip stray \n prefix not part of a real LaTeX command, but keep the following text.
  // \na → a, \nx → x, \neq stays as \neq
  return latex.replace(/\\(n[a-zA-Z]*)/g, (_match, rest: string) => {
    if (VALID_N_COMMANDS.test('\\' + rest)) return '\\' + rest
    // Not a known command — drop the \n prefix, keep whatever follows
    return rest.slice(1)  // 'na' → 'a', 'nx' → 'x', 'n' → ''
  })
}

function renderKatex(latex: string, displayMode: boolean): string {
  if (!latex) return ''
  try {
    return katex.renderToString(cleanLatexSource(latex), {
      displayMode,
      strict: false,
      throwOnError: false,
    })
  } catch {
    return displayMode ? `$$${latex}$$` : `$${latex}$`
  }
}

function restoreAllMath(text: string, blocks: MathBlock[]): string {
  const regex = new RegExp(`${MATH_PREFIX}(\\d+)${MATH_SUFFIX}`, 'g')
  return text.replace(regex, (_m, idx: string) => {
    const block = blocks[Number(idx)]
    if (!block) return ''
    return block.display
      ? `<p class="katex-block">${block.html}</p>`
      : block.html
  })
}

// ── Public API ──

export function renderRichText(value: string | null | undefined, fallback = ''): string {
  let source = repairLatexEscapes(value || fallback)
  if (!source) return ''

  source = normalizeInlineMath(source)
  source = wrapBareMath(source)

  // Step 1: extract and render ALL math ($...$ and $$...$$) with KaTeX directly.
  // After this, the remaining text contains zero LaTeX commands.
  const { text, blocks } = preRenderAllMath(source)

  // Step 2: now safe to convert ALL \n to real newlines (no LaTeX commands left).
  const withNewlines = text.replace(/\\n/g, '\n')

  // Step 3: normalize paragraphs and render markdown formatting.
  const formatted = normalizeParagraphs(withNewlines)

  try {
    const html = markdown.render(formatted)
    return restoreAllMath(html, blocks)
  } catch {
    return markdown.utils.escapeHtml(source).replace(/\n/g, '<br>')
  }
}
