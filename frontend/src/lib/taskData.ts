import type { TaskPriority, TaskStatus } from '@/api'

export const COLUMNS: { id: TaskStatus; title: string; color: string }[] = [
  { id: 'todo', title: '待办事项', color: 'bg-slate-50 border-slate-200' },
  { id: 'inProgress', title: '进行中', color: 'bg-blue-50 border-blue-200' },
  { id: 'review', title: '审核中', color: 'bg-amber-50 border-amber-200' },
  { id: 'done', title: '已完成', color: 'bg-emerald-50 border-emerald-200' },
]

export const STATUS_MAP: Record<TaskStatus, { label: string; color: string }> = {
  todo: { label: '待办', color: 'bg-slate-100 text-slate-700 border-slate-200' },
  inProgress: { label: '进行中', color: 'bg-blue-100 text-blue-700 border-blue-200' },
  review: { label: '审核中', color: 'bg-amber-100 text-amber-700 border-amber-200' },
  done: { label: '已完成', color: 'bg-emerald-100 text-emerald-700 border-emerald-200' },
}

export const PRIORITY_OPTIONS: { value: TaskPriority; label: string }[] = [
  { value: 'high', label: '高' },
  { value: 'medium', label: '中' },
  { value: 'low', label: '低' },
]

export const PRIORITY_MAP: Record<TaskPriority, { label: string; color: string }> = {
  high: { label: '高', color: 'bg-red-100 text-red-700 border-red-200' },
  medium: { label: '中', color: 'bg-orange-100 text-orange-700 border-orange-200' },
  low: { label: '低', color: 'bg-blue-100 text-blue-700 border-blue-200' },
}

const AVATAR_COLORS = [
  'bg-blue-500',
  'bg-emerald-500',
  'bg-orange-500',
  'bg-purple-500',
  'bg-pink-500',
  'bg-teal-500',
]

export const getAvatar = (name: string) => {
  let hash = 0
  for (const ch of name) hash = (hash * 31 + ch.charCodeAt(0)) % 997
  return {
    initial: (name.trim()[0] ?? '?').toUpperCase(),
    color: AVATAR_COLORS[hash % AVATAR_COLORS.length],
  }
}
