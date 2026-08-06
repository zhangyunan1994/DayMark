<script setup lang="ts">
import { Calendar, User } from 'lucide-vue-next'
import Badge from '@/components/ui/badge.vue'
import { STATUS_MAP, PRIORITY_MAP } from '@/lib/taskData'
import type { Task } from '@/api'

defineProps<{
  tasks: Task[]
}>()

const emit = defineEmits<{
  taskClick: [task: Task]
}>()
</script>

<template>
  <div class="bg-white rounded-xl border overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b bg-muted/50 text-muted-foreground text-left">
            <th class="px-4 py-3 font-medium w-auto">任务标题</th>
            <th class="px-3 py-3 font-medium w-30">状态</th>
            <th class="px-3 py-3 font-medium w-30">优先级</th>
            <th class="px-3 py-3 font-medium w-38">负责人</th>
            <th class="px-3 py-3 font-medium w-48">截止日期</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="task in tasks"
            :key="task.id"
            class="border-b cursor-pointer hover:bg-muted/50"
            @click="emit('taskClick', task)"
          >
            <td class="px-3 py-3 font-medium">
              <div class="flex items-center gap-2">
                <span v-if="task.urgent" class="text-[10px] text-red-600 bg-red-50 border border-red-100 px-1 rounded">紧急</span>
                <span v-if="task.important" class="text-[10px] text-blue-600 bg-blue-50 border border-blue-100 px-1 rounded">重要</span>
                {{ task.title }}
              </div>
            </td>
            <td class="px-3 py-3">
              <Badge variant="outline" class="text-[10px]" :class="STATUS_MAP[task.status].color">
                {{ STATUS_MAP[task.status].label }}
              </Badge>
            </td>
            <td class="px-3 py-3">
              <Badge variant="outline" class="text-[10px]" :class="PRIORITY_MAP[task.priority].color">
                {{ PRIORITY_MAP[task.priority].label }}
              </Badge>
            </td>
            <td class="px-3 py-3">
              <div class="flex items-center gap-1.5">
                <User class="w-3.5 h-3.5 text-muted-foreground" />
                <span>{{ task.assignee || '-' }}</span>
              </div>
            </td>
            <td class="px-3 py-3">
              <div class="flex items-center gap-1.5 text-muted-foreground">
                <Calendar class="w-3.5 h-3.5" />
                {{ task.due_date ? task.due_date : '-' }}
              </div>
            </td>
          </tr>
          <tr v-if="tasks.length === 0">
            <td colspan="5" class="px-4 py-10 text-center text-muted-foreground">暂无任务</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
