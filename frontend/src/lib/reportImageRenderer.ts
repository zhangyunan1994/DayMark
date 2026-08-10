import type { Report } from '@/api'
import { REPORT_TYPE_LABEL } from '@/lib/reportTemplates'

const W = 800
const PAD = 40
const CW = W - 2 * PAD

type Block =
  | { type: 'h1'; text: string }
  | { type: 'h2'; text: string }
  | { type: 'h3'; text: string }
  | { type: 'p'; text: string }
  | { type: 'ul'; items: { text: string; level: number }[] }
  | { type: 'ol'; items: { text: string; level: number }[] }
  | { type: 'code'; lang: string; lines: string[] }

type Seg = { text: string; bold: boolean }

function parse(md: string): Block[] {
  const lines = md.split('\n')
  const blocks: Block[] = []
  let i = 0
  while (i < lines.length) {
    const raw = lines[i]
    const trimmed = raw.trim()
    if (!trimmed) { i++; continue }

    const hMatch = trimmed.match(/^(#{1,3})\s+(.+)/)
    if (hMatch) {
      blocks.push({ type: `h${hMatch[1].length}` as 'h1' | 'h2' | 'h3', text: hMatch[2] })
      i++; continue
    }

    if (trimmed.startsWith('```')) {
      const lang = trimmed.slice(3).trim()
      const codeLines: string[] = []
      i++
      while (i < lines.length && !lines[i].trim().startsWith('```')) {
        codeLines.push(lines[i]); i++
      }
      i++
      blocks.push({ type: 'code', lang, lines: codeLines })
      continue
    }

    const listMatch = raw.match(/^(\s*)(-|\*|\d+\.)\s+(.*)/)
    if (listMatch) {
      const ordered = /^\d+\./.test(listMatch[2])
      const items: { text: string; level: number }[] = []
      while (i < lines.length) {
        const m = lines[i].match(/^(\s*)(-|\*|\d+\.)\s+(.*)/)
        if (!m) break
        items.push({ text: m[3], level: Math.floor(m[1].length / 2) })
        i++
      }
      blocks.push({ type: ordered ? 'ol' : 'ul', items })
      continue
    }

    blocks.push({ type: 'p', text: trimmed })
    i++
  }
  return blocks
}

function font(ctx: CanvasRenderingContext2D, px: number, weight = 'normal', family = 'system-ui, -apple-system, sans-serif') {
  ctx.font = `${weight} ${px}px ${family}`
}

function parseInline(text: string): Seg[] {
  const segs: Seg[] = []
  for (const part of text.split(/(\*\*[^*]+\*\*)/)) {
    if (part.startsWith('**') && part.endsWith('**')) {
      segs.push({ text: part.slice(2, -2), bold: true })
    } else if (part) {
      segs.push({ text: part, bold: false })
    }
  }
  return segs.length ? segs : [{ text, bold: false }]
}

function wrapRich(ctx: CanvasRenderingContext2D, text: string, maxW: number, px: number): Seg[][] {
  const segs = parseInline(text)
  const lines: Seg[][] = []
  let line: Seg[] = []
  let lw = 0
  for (const seg of segs) {
    font(ctx, px, seg.bold ? 'bold' : 'normal')
    for (const ch of seg.text) {
      const cw = ctx.measureText(ch).width
      if (lw + cw > maxW && line.length > 0) {
        lines.push(line)
        line = [{ text: ch, bold: seg.bold }]
        lw = cw
      } else {
        line.push({ text: ch, bold: seg.bold })
        lw += cw
      }
    }
  }
  if (line.length) lines.push(line)
  return lines.length ? lines : [[]]
}

function drawLine(ctx: CanvasRenderingContext2D, segs: Seg[], x: number, y: number, px: number) {
  let cx = x
  for (const seg of segs) {
    font(ctx, px, seg.bold ? 'bold' : 'normal')
    ctx.fillStyle = '#333333'
    ctx.fillText(seg.text, cx, y)
    cx += ctx.measureText(seg.text).width
  }
}

function drawBlocks(ctx: CanvasRenderingContext2D, blocks: Block[], x: number, startY: number): number {
  let y = startY
  for (const block of blocks) {
    switch (block.type) {
      case 'h1': {
        font(ctx, 20, 'bold'); ctx.fillStyle = '#1a1a1a'
        for (const line of wrapRich(ctx, block.text, CW, 20)) { drawLine(ctx, line, x, y, 20); y += 30 }
        y += 18; break
      }
      case 'h2': {
        font(ctx, 18, 'bold'); ctx.fillStyle = '#1a1a1a'
        for (const line of wrapRich(ctx, block.text, CW, 18)) { drawLine(ctx, line, x, y, 18); y += 28 }
        y += 14; break
      }
      case 'h3': {
        font(ctx, 16, 'bold'); ctx.fillStyle = '#1a1a1a'
        for (const line of wrapRich(ctx, block.text, CW, 16)) { drawLine(ctx, line, x, y, 16); y += 24 }
        y += 12; break
      }
      case 'p': {
        font(ctx, 14); ctx.fillStyle = '#333333'
        for (const line of wrapRich(ctx, block.text, CW, 14)) { drawLine(ctx, line, x, y, 14); y += 22 }
        y += 10; break
      }
      case 'code': {
        const lineH = 20, padY = 10
        const codeH = block.lines.length * lineH + padY * 2
        ctx.fillStyle = '#f4f4f5'
        ctx.beginPath(); ctx.roundRect(x, y, CW, codeH, 4); ctx.fill()
        font(ctx, 13, 'normal', 'ui-monospace, monospace'); ctx.fillStyle = '#333333'
        let cy = y + padY + 14
        for (const line of block.lines) { ctx.fillText(line, x + 10, cy); cy += lineH }
        y += codeH + 12; break
      }
      case 'ul': case 'ol': {
        let idx = 1
        for (const item of block.items) {
          const indent = 20 + item.level * 20
          const bullet = block.type === 'ol' ? `${idx}.` : '•'
          font(ctx, 14); ctx.fillStyle = '#555555'
          ctx.fillText(bullet, x + item.level * 20, y)
          const mw = CW - indent
          for (const line of wrapRich(ctx, item.text, mw, 14)) {
            drawLine(ctx, line, x + indent, y, 14); y += 22
          }
          y += 4; idx++
        }
        y += 10; break
      }
    }
  }
  return y
}

export async function renderReportImage(report: Report): Promise<Blob> {
  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')!
  canvas.width = W; canvas.height = 4000
  ctx.fillStyle = '#ffffff'; ctx.fillRect(0, 0, W, 4000)
  ctx.textBaseline = 'top'

  const x = PAD
  let y = PAD

  font(ctx, 18, 'bold'); ctx.fillStyle = '#1a1a1a'
  ctx.fillText(report.title, x, y); y += 28

  font(ctx, 12); ctx.fillStyle = '#666666'
  ctx.fillText(`${REPORT_TYPE_LABEL[report.report_type]} · ${report.template_id} · ${new Date(report.created_at).toLocaleString()}`, x, y)
  y += 24

  y = drawBlocks(ctx, parse(report.content), x, y)
  y += PAD

  const out = document.createElement('canvas')
  out.width = W; out.height = Math.ceil(y)
  out.getContext('2d')!.drawImage(canvas, 0, 0)

  return new Promise<Blob>((resolve, reject) => {
    out.toBlob(b => b ? resolve(b) : reject(new Error('图片生成失败')), 'image/png')
  })
}
