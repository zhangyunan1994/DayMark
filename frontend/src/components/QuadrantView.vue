<script setup lang="ts">
import TaskCard from './TaskCard.vue'
import type { Task } from '@/api'

const QUADRANTS = [
  { id: 'q1', title: '紧急且重要', subtitle: '马上做', urgent: true, important: true, color: 'bg-red-50 border-red-200' },
  { id: 'q2', title: '紧急不重要', subtitle: '委托他人', urgent: true, important: false, color: 'bg-orange-50 border-orange-200' },
  { id: 'q3', title: '重要不紧急', subtitle: '计划做', urgent: false, important: true, color: 'bg-blue-50 border-blue-200' },
  { id: 'q4', title: '不紧急不重要', subtitle: '稍后做', urgent: false, important: false, color: 'bg-gray-50 border-gray-200' },
]

defineProps<{
  tasks: Task[]
}>()

const emit = defineEmits<{
  taskClick: [task: Task]
}>()
</script>

<template>
  <div class="grid grid-cols-2 grid-rows-2 gap-4 h-full min-h-0">
    <div v-for="q in QUADRANTS" :key="q.id" class="rounded-xl border p-3 flex flex-col gap-3" :class="q.color">
      <div class="flex items-center justify-between px-1">
        <div>
          <h3 class="font-semibold text-sm">{{ q.title }}</h3>
          <p class="text-xs text-muted-foreground">{{ q.subtitle }}</p>
        </div>
        <span class="text-xs font-medium bg-white/80 px-2 py-0.5 rounded-full border">
          {{ tasks.filter(t => t.urgent === q.urgent && t.important === q.important).length }}
        </span>
      </div>
      <div class="flex flex-col gap-2 overflow-y-auto flex-1 min-h-0">
        <TaskCard
          v-for="task in tasks.filter(t => t.urgent === q.urgent && t.important === q.important)"
          :key="task.id"
          :task="task"
          @click="emit('taskClick', $event)"
        />
      </div>
    </div>
  </div>
</template>
