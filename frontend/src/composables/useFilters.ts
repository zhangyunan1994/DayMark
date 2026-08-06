import { ref, computed } from 'vue'
import type { Task, TaskPriority } from '@/api'

export type QuadrantKey = 'all' | 'q1' | 'q2' | 'q3' | 'q4'
export type TimeKey = 'all' | 'today' | 'week' | 'month' | 'custom'

function filterByPriority(task: Task, priority: TaskPriority | 'all') {
  return priority === 'all' || task.priority === priority
}

function filterByQuadrant(task: Task, quadrant: QuadrantKey) {
  if (quadrant === 'all') return true
  const map: Record<string, { urgent: boolean; important: boolean }> = {
    q1: { urgent: true, important: true },
    q2: { urgent: true, important: false },
    q3: { urgent: false, important: true },
    q4: { urgent: false, important: false },
  }
  const q = map[quadrant]
  return task.urgent === q.urgent && task.important === q.important
}

function filterByTime(task: Task, time: TimeKey, customStart: string, customEnd: string) {
  if (time === 'all') return true
  const created = new Date(task.created_at)
  const now = new Date()

  if (time === 'today') {
    const start = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    return created >= start
  }
  if (time === 'week') {
    const day = now.getDay()
    const diff = now.getDate() - day + (day === 0 ? -6 : 1)
    const start = new Date(now.getFullYear(), now.getMonth(), diff)
    return created >= start
  }
  if (time === 'month') {
    const start = new Date(now.getFullYear(), now.getMonth(), 1)
    return created >= start
  }
  if (time === 'custom' && customStart && customEnd) {
    const start = new Date(customStart + 'T00:00:00')
    const end = new Date(customEnd + 'T23:59:59')
    return created >= start && created <= end
  }
  return true
}

function pad(n: number) {
  return String(n).padStart(2, '0')
}

function isoDate(d: Date) {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

export function useFilters(tasks: ReturnType<typeof import('vue').ref<Task[]>> | undefined) {
  const priority = ref<TaskPriority | 'all'>('all')
  const quadrant = ref<QuadrantKey>('all')
  const timeRange = ref<TimeKey>('all')
  const customStart = ref(isoDate(new Date()))
  const customEnd = ref(isoDate(new Date()))

  const filtered = computed(() =>
    (tasks?.value ?? []).filter(
      (t) =>
        filterByPriority(t, priority.value) &&
        filterByQuadrant(t, quadrant.value) &&
        filterByTime(t, timeRange.value, customStart.value, customEnd.value)
    )
  )

  return {
    filtered,
    priority,
    quadrant,
    timeRange,
    customStart,
    customEnd,
  }
}
