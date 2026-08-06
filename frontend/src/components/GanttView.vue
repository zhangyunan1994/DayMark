<script setup lang="ts">
import { computed } from 'vue'
import { addDays, differenceInDays, eachDayOfInterval, format, isValid, parseISO } from 'date-fns'
import type { Task, TaskStatus } from '@/api'

const STATUS_COLORS: Record<TaskStatus, string> = {
  todo: 'bg-slate-400',
  inProgress: 'bg-blue-500',
  review: 'bg-amber-500',
  done: 'bg-emerald-500',
}

const props = defineProps<{
  tasks: Task[]
}>()

const emit = defineEmits<{
  taskClick: [task: Task]
}>()

const parseDueDate = (s: string) => parseISO(s.replace(' ', 'T'))

const getTaskRange = (task: Task) => {
  const end = task.due_date ? parseDueDate(task.due_date) : null
  let start = task.created_at ? new Date(task.created_at) : null
  if (!end || !isValid(end)) return null
  if (!start || !isValid(start) || start > end) start = addDays(end, -5)
  return { start, end }
}

const timeline = computed(() => {
  const ranges = props.tasks.map(getTaskRange).filter((r): r is { start: Date; end: Date } => !!r)
  if (ranges.length === 0) return [] as Date[]
  const minDate = new Date(Math.min(...ranges.map((r) => r.start.getTime())))
  const maxDate = new Date(Math.max(...ranges.map((r) => r.end.getTime())))
  const start = addDays(minDate, -1)
  const end = addDays(maxDate, 1)
  return eachDayOfInterval({ start, end })
})

const dayCount = computed(() => timeline.value.length)

const getBarStyle = (task: Task) => {
  const range = getTaskRange(task)
  if (!range || timeline.value.length === 0) return null
  const offsetDays = differenceInDays(range.start, timeline.value[0])
  const durationDays = differenceInDays(range.end, range.start) + 1
  return {
    left: `${(offsetDays / dayCount.value) * 100}%`,
    width: `${Math.max((durationDays / dayCount.value) * 100, 2)}%`,
  }
}
</script>

<template>
  <div v-if="timeline.length === 0" class="flex items-center justify-center h-full text-muted-foreground">
    暂无有效的日期数据用于展示甘特图
  </div>
  <div v-else class="bg-white rounded-xl border overflow-hidden flex flex-col h-full">
    <div class="overflow-auto flex-1">
      <div class="min-w-[800px] p-4">
        <div class="flex mb-2">
          <div class="w-48 shrink-0 text-xs font-semibold text-muted-foreground px-2">任务</div>
          <div class="flex-1 flex">
            <div
              v-for="(date, i) in timeline"
              :key="i"
              class="flex-1 text-[10px] text-center text-muted-foreground border-l first:border-l-0"
            >
              {{ format(date, 'MM/dd') }}
            </div>
          </div>
        </div>
        <div class="space-y-2">
          <div
            v-for="task in tasks"
            :key="task.id"
            class="flex items-center group cursor-pointer hover:bg-muted/30 rounded"
            @click="emit('taskClick', task)"
          >
            <div class="w-48 shrink-0 px-2">
              <div class="text-sm font-medium truncate">{{ task.title }}</div>
              <div class="text-[10px] text-muted-foreground">{{ task.assignee }}</div>
            </div>
            <div class="flex-1 relative h-6">
              <div class="absolute inset-0 flex pointer-events-none">
                <div
                  v-for="(_, i) in timeline"
                  :key="i"
                  class="flex-1 border-l border-dashed border-border/40 first:border-l-0"
                />
              </div>
              <div
                v-if="getBarStyle(task)"
                class="absolute top-1 h-4 rounded-full opacity-80 group-hover:opacity-100 transition-opacity"
                :class="STATUS_COLORS[task.status]"
                :style="getBarStyle(task)!"
                :title="task.due_date ? `${format(parseDueDate(task.due_date), 'yyyy-MM-dd HH:mm')} 截止` : ''"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
