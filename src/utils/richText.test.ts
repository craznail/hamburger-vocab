/**
 * Run with:  npx tsx src/utils/richText.test.ts
 */
import { renderRichText } from './richText'

let passed = 0
let failed = 0

function test(name: string, fn: () => void) {
  try {
    fn()
    passed++
    console.log(`  ✓ ${name}`)
  } catch (e) {
    failed++
    console.log(`  ✗ ${name}`)
    console.log(`    ${e}`)
  }
}

function assertContains(haystack: string, needle: string, label: string) {
  if (!haystack.includes(needle)) {
    throw new Error(`${label}: expected to contain "${needle}"`)
  }
}

function assertNotContains(haystack: string, needle: string, label: string) {
  if (haystack.includes(needle)) {
    throw new Error(`${label}: expected NOT to contain "${needle}"`)
  }
}

// Exclude annotation text from check (KaTeX keeps raw LaTeX source there)
function stripAnnotations(html: string): string {
  return html.replace(/<annotation[^>]*>.*?<\/annotation>/g, '')
}

console.log('\nrichText rendering tests\n')

// ═══ display math ═══
test('$$\\div$$ renders ÷', () => {
  const html = renderRichText('每条棱上有：\n\n$$\n4 \\div 1=4\n$$\n\n个小正方体。')
  assertContains(html, '<mo>÷</mo>', '÷ symbol')
  assertNotContains(stripAnnotations(html), '\\div', 'raw \\div')
})

test('$$\\times$$ renders ×', () => {
  const html = renderRichText('共 $$\n12 \\times 2=24\n$$\n 个')
  assertContains(html, '<mo>×</mo>', '× symbol')
})

test('$$\\sqrt{}$$ renders √', () => {
  const html = renderRichText('答案：\n\n$$\n\\sqrt{}\n$$')
  assertNotContains(stripAnnotations(html), '\\sqrt', 'raw \\sqrt')
})

// ═══ inline math ═══
test('inline $4$ renders', () => {
  const html = renderRichText('棱长是 $4$ 厘米')
  assertContains(html, '<mn>4</mn>', 'number 4')
})

test('inline $\\times$ renders × (bypasses markdown-it-katex)', () => {
  const html = renderRichText('错因分析：计算 $12 \\times 2$ 时出错')
  assertContains(html, '<mo>×</mo>', '× symbol in inline math')
  assertNotContains(stripAnnotations(html), '\\times', 'raw \\times in inline')
})

test('inline $\\div$ renders ÷ (bypasses markdown-it-katex)', () => {
  const html = renderRichText('算式 $4 \\div 1=4$ 正确')
  assertContains(html, '<mo>÷</mo>', '÷ symbol in inline math')
})

// ═══ $ spacing ═══
test('$ 4$ trims leading space', () => {
  const html = renderRichText('棱长是 $ 4$ 厘米')
  assertContains(html, '<mn>4</mn>', 'number 4')
})

test('$4 $ trims trailing space', () => {
  const html = renderRichText('棱长是 $4 $ 厘米')
  assertContains(html, '<mn>4</mn>', 'number 4')
})

// ═══ bare math ═══
test('bare \\times gets auto-wrapped', () => {
  const html = renderRichText('4\\times2=8')
  assertContains(html, '<mo>×</mo>', '× symbol')
})

// ═══ \n handling ═══
test('\\nA becomes newline (not protected)', () => {
  const html = renderRichText('俯视图是（）\n\n\\nA\n.选项A\n\\nB\n.选项B')
  assertNotContains(html, '\\nA', 'raw \\nA')
  assertNotContains(html, '\\nB', 'raw \\nB')
})

test('\\neq survives newline conversion', () => {
  const html = renderRichText('$x \\neq 0$')
  // Check that KaTeX rendered it (any non-raw output)
  const stripped = stripAnnotations(html)
  assertNotContains(stripped, '\\neq', 'raw \\neq')
  assertNotContains(stripped, '\\n', 'raw \\n')
  // Should have katex output
  assertContains(html, 'katex', 'katex class')
})

test('\\not survives newline conversion inside math', () => {
  const html = renderRichText('$x \\not\\in A$')
  assertNotContains(stripAnnotations(html), 'newline in place of \\not', 'broken \\not')
})

// ═══ stray \n cleanup inside math ═══
test('\\na inside math: \\n stripped but a kept', () => {
  const html = renderRichText('结果为 $\\na\\div0.5=2a$')
  assertNotContains(stripAnnotations(html), '\\na', 'raw \\na')
  assertNotContains(stripAnnotations(html), '\\n', 'raw \\n')
  // The 'a' variable should survive
  assertContains(stripAnnotations(html), '>a<', 'variable a preserved')
  assertContains(html, '<mo>÷</mo>', '÷ still renders')
})

test('\\nx inside math: \\n stripped but x kept', () => {
  const html = renderRichText('解得 $\\nx=8$')
  assertNotContains(stripAnnotations(html), '\\nx', 'raw \\nx')
  // The 'x' variable should survive
  assertContains(stripAnnotations(html), '>x<', 'variable x preserved')
})

test('\\neq inside math is preserved', () => {
  const html = renderRichText('$x \\neq 0$')
  assertNotContains(stripAnnotations(html), '\\neq', 'raw \\neq')
  assertContains(html, 'katex', 'katex output')
})

// ═══ no garbage ═══
test('no placeholder garbage in output', () => {
  const html = renderRichText('每条棱上有：\n\n$$\n4 \\div 1=4\n$$\n\n个小正方体。')
  assertNotContains(html, 'MATH', 'no MATH placeholder')
  assertNotContains(html, '\x00', 'no null bytes')
  assertNotContains(html, '�', 'no replacement chars')
})

console.log(`\n${passed} passed, ${failed} failed\n`)
process.exit(failed > 0 ? 1 : 0)
