<script setup lang="ts">
import { Calendar, ListChecks } from 'lucide-vue-next'
import Card from '@/components/ui/card.vue'
import CardContent from '@/components/ui/card-content.vue'
import Badge from '@/components/ui/badge.vue'
import { getAvatar, PRIORITY_MAP } from '@/lib/taskData'
import type { Task } from '@/api'

const props = defineProps<{
  task: Task
}>()

const emit = defineEmits<{
  click: [task: Task]
}>()

const completedSub = () => props.task.subtasks.filter((s) => s.completed).length
const avatar = () => getAvatar(props.task.assignee)
</script>

<template>
  <Card
    class="cursor-pointer hover:shadow-lg hover:-translate-y-0.5 transition-all bg-white"
    @click="emit('click', task)"
  >
    <CardContent class="p-3 space-y-2.5">
      <h4 class="font-medium text-sm leading-snug">{{ task.title }}</h4>
      <div class="flex items-center gap-2 flex-wrap">
        <Badge variant="outline" class="text-[10px] px-1.5 py-0" :class="PRIORITY_MAP[task.priority].color">
          {{ PRIORITY_MAP[task.priority].label }}
        </Badge>
        <span v-if="task.urgent" class="text-[10px] font-medium text-red-600 bg-red-50 border border-red-100 px-1.5 py-0 rounded">
          紧急
        </span>
        <span v-if="task.important" class="text-[10px] font-medium text-blue-600 bg-blue-50 border border-blue-100 px-1.5 py-0 rounded">
          重要
        </span>
      </div>
      <div class="flex items-center justify-between text-xs text-muted-foreground">
        <div class="flex items-center gap-1.5">
          <span
            v-if="task.assignee"
            class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-semibold text-white"
            :class="avatar().color"
          >
            {{ avatar().initial }}
          </span>
          <span v-else class="w-5 h-5 rounded-full bg-secondary border border-border" />
          <span>{{ task.assignee || '未分配' }}</span>
        </div>
        <span v-if="task.due_date" class="flex items-center gap-1">
          <Calendar class="w-3 h-3" />
          {{ task.due_date.slice(5) }}
        </span>
      </div>
      <div v-if="task.subtasks.length > 0" class="flex items-center gap-1 text-xs text-muted-foreground">
        <ListChecks class="w-3 h-3" />
        <span>子任务 {{ completedSub() }}/{{ task.subtasks.length }}</span>
      </div>
    </CardContent>
  </Card>
</template>
