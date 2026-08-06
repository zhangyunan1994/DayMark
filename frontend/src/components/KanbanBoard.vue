<script setup lang="ts">
import TaskCard from './TaskCard.vue'
import type { Task, TaskStatus } from '@/api'

defineProps<{
  columns: { id: TaskStatus; title: string; color: string }[]
  tasks: Task[]
}>()

const emit = defineEmits<{
  taskClick: [task: Task]
}>()
</script>

<template>
  <div class="flex gap-4 h-full min-w-max">
    <div v-for="col in columns" :key="col.id" class="w-72 rounded-xl border p-3 flex flex-col gap-3" :class="col.color">
      <div class="flex items-center justify-between px-1">
        <h3 class="font-semibold text-sm">{{ col.title }}</h3>
        <span class="text-xs font-medium bg-white/80 px-2 py-0.5 rounded-full border">
          {{ tasks.filter(t => t.status === col.id).length }}
        </span>
      </div>
      <div class="flex flex-col gap-2 overflow-y-auto flex-1 min-h-0">
        <TaskCard
          v-for="task in tasks.filter(t => t.status === col.id)"
          :key="task.id"
          :task="task"
          @click="emit('taskClick', $event)"
        />
      </div>
    </div>
  </div>
</template>
