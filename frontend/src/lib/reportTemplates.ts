export type ReportType = 'daily' | 'weekly' | 'monthly' | 'opencode'

export interface ReportTemplate {
  id: string
  report_type: ReportType
  name: string
  description: string
  sections: string[]
}

export const REPORT_TEMPLATES: ReportTemplate[] = [
  // Daily
  { id: 'result-oriented', report_type: 'daily', name: '成果导向日报', description: '以业务成果和价值输出为核心，弱化过程记录', sections: ['今日核心成果', '关键指标变化', '推进中的重点事项'] },
  { id: 'three-sentence', report_type: 'daily', name: '三句话日报', description: '用最短篇幅总结一天工作', sections: ['完成了什么', '遇到了什么问题', '明天计划'] },
  { id: 'top3', report_type: 'daily', name: 'TOP3 日报', description: '只保留最重要的三件事', sections: ['最重要的事 1', '最重要的事 2', '最重要的事 3'] },
  // Weekly
  { id: 'weekly-progress', report_type: 'weekly', name: '周进展报告', description: '本周工作进展总结，适合大多数场景', sections: ['本周完成', '进行中的工作', '遇到的问题', '下周计划'] },
  { id: 'weekly-achievements', report_type: 'weekly', name: '周成果报告', description: '聚焦本周关键成果和价值输出', sections: ['关键成果', '数据指标变化', '下周重点'] },
  { id: 'weekly-brief', report_type: 'weekly', name: '简洁周报', description: '三段式简洁周报，快速完成', sections: ['本周做了什么', '遇到了什么问题', '下周要做什么'] },
  // Monthly
  { id: 'monthly-review', report_type: 'monthly', name: '月度总结', description: '全面的月度工作回顾', sections: ['本月完成', '关键成果', '数据指标', '问题与改进', '下月计划'] },
  { id: 'monthly-okr', report_type: 'monthly', name: 'OKR 月报', description: '基于 OKR 的月度进展汇报', sections: ['目标完成情况', '关键结果进展', '下月 OKR'] },
  { id: 'monthly-brief', report_type: 'monthly', name: '简洁月报', description: '快速完成月度汇报', sections: ['本月做了什么', '主要成果', '下月计划'] },
]

export const REPORT_TYPE_LABEL: Record<ReportType, string> = {
  daily: '日报',
  weekly: '周报',
  monthly: '月报',
  opencode: 'opencode',
}

const pad = (n: number) => String(n).padStart(2, '0')

export const dateRangeForType = (type: ReportType) => {
  const today = new Date()
  const iso = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
  if (type === 'daily') return { start: iso(today), end: iso(today) }
  if (type === 'weekly') {
    const day = today.getDay()
    const diff = today.getDate() - day + (day === 0 ? -6 : 1)
    const monday = new Date(today)
    monday.setDate(diff)
    const sunday = new Date(monday)
    sunday.setDate(monday.getDate() + 6)
    return { start: iso(monday), end: iso(sunday) }
  }
  const first = new Date(today.getFullYear(), today.getMonth(), 1)
  const last = new Date(today.getFullYear(), today.getMonth() + 1, 0)
  return { start: iso(first), end: iso(last) }
}