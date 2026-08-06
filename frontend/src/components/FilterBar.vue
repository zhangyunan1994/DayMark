<script setup lang="ts">
import { ChevronDown } from 'lucide-vue-next'
import type { TaskPriority } from '@/api'
import type { QuadrantKey, TimeKey } from '@/composables/useFilters'

defineProps<{
  priority: TaskPriority | 'all'
  quadrant: QuadrantKey
  timeRange: TimeKey
  customStart: string
  customEnd: string
  resultCount: number
}>()

const emit = defineEmits<{
  'update:priority': [value: TaskPriority | 'all']
  'update:quadrant': [value: QuadrantKey]
  'update:timeRange': [value: TimeKey]
  'update:customStart': [value: string]
  'update:customEnd': [value: string]
}>()

const PRIORITY_OPTIONS: { value: TaskPriority | 'all'; label: string }[] = [
  { value: 'all', label: '全部优先级' },
  { value: 'high', label: '高优先级' },
  { value: 'medium', label: '中优先级' },
  { value: 'low', label: '低优先级' },
]

const QUADRANT_OPTIONS: { value: QuadrantKey; label: string }[] = [
  { value: 'all', label: '全部象限' },
  { value: 'q1', label: '紧急且重要' },
  { value: 'q2', label: '紧急不重要' },
  { value: 'q3', label: '重要不紧急' },
  { value: 'q4', label: '不紧急不重要' },
]

const TIME_OPTIONS: { value: TimeKey; label: string }[] = [
  { value: 'all', label: '全部时间' },
  { value: 'today', label: '今日' },
  { value: 'week', label: '本周' },
  { value: 'month', label: '本月' },
  { value: 'custom', label: '自定义' },
]
</script>

<template>
  <div class="bg-white border-b px-6 py-2.5 flex items-center gap-4 shrink-0">
    <div class="relative">
      <select
        :value="priority"
        class="appearance-none h-8 pl-3 pr-7 rounded-md border bg-background text-sm cursor-pointer hover:bg-accent"
        @change="emit('update:priority', ($event.target as HTMLSelectElement).value as TaskPriority | 'all')"
      >
        <option v-for="o in PRIORITY_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
      </select>
      <ChevronDown class="absolute right-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
    </div>

    <div class="relative">
      <select
        :value="quadrant"
        class="appearance-none h-8 pl-3 pr-7 rounded-md border bg-background text-sm cursor-pointer hover:bg-accent"
        @change="emit('update:quadrant', ($event.target as HTMLSelectElement).value as QuadrantKey)"
      >
        <option v-for="o in QUADRANT_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
      </select>
      <ChevronDown class="absolute right-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
    </div>

    <div class="flex items-center gap-1 bg-muted p-0.5 rounded-md">
      <button
        v-for="o in TIME_OPTIONS"
        :key="o.value"
        class="px-3 py-1 rounded text-xs font-medium transition-colors"
        :class="timeRange === o.value ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'"
        @click="emit('update:timeRange', o.value)"
      >
        {{ o.label }}
      </button>
    </div>

    <div v-if="timeRange === 'custom'" class="flex items-center gap-2">
      <input
        type="date"
        :value="customStart"
        class="h-8 px-2 rounded-md border bg-background text-sm"
        @change="emit('update:customStart', ($event.target as HTMLInputElement).value)"
      />
      <span class="text-muted-foreground text-xs">至</span>
      <input
        type="date"
        :value="customEnd"
        class="h-8 px-2 rounded-md border bg-background text-sm"
        @change="emit('update:customEnd', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <span class="text-xs text-muted-foreground ml-auto">
      共 {{ resultCount }} 项结果
    </span>
  </div>
</template>
