<script setup lang="ts">
import { computed } from 'vue'
import KanbanBoard from '@/components/KanbanBoard.vue'
import QuadrantView from '@/components/QuadrantView.vue'
import ListView from '@/components/ListView.vue'
import GanttView from '@/components/GanttView.vue'
import FilterBar from '@/components/FilterBar.vue'
import Tabs from '@/components/ui/tabs.vue'
import TabsList from '@/components/ui/tabs-list.vue'
import TabsTrigger from '@/components/ui/tabs-trigger.vue'
import { COLUMNS } from '@/lib/taskData'
import type { Task } from '@/api'
import type { QuadrantKey, TimeKey } from '@/composables/useFilters'

type TaskViewMode = 'board' | 'quadrant' | 'list' | 'gantt'

const props = defineProps<{
  viewMode: TaskViewMode
  tasks: Task[]
  isLoading: boolean
  priority: 'low' | 'medium' | 'high' | 'all'
  quadrant: QuadrantKey
  timeRange: TimeKey
  customStart: string
  customEnd: string
  resultCount: number
}>()

const emit = defineEmits<{
  'update:viewMode': [value: TaskViewMode]
  'update:priority': [value: 'low' | 'medium' | 'high' | 'all']
  'update:quadrant': [value: QuadrantKey]
  'update:timeRange': [value: TimeKey]
  'update:customStart': [value: string]
  'update:customEnd': [value: string]
  taskClick: [task: Task]
}>()

const viewModeValue = computed({
  get: () => props.viewMode,
  set: (v: string) => emit('update:viewMode', v as TaskViewMode),
})

const doneCount = computed(() => props.tasks.filter(t => t.status === 'done').length)
const inProgressCount = computed(() => props.tasks.filter(t => t.status === 'inProgress').length)
</script>

<template>
  <header class="bg-white border-b px-6 py-4 flex items-center gap-8 shrink-0">
    <div class="shrink-0">
      <h1 class="text-xl font-bold">项目任务看板</h1>
      <p class="text-sm text-muted-foreground mt-0.5">点击卡片即可编辑并调整状态</p>
    </div>
    <div class="flex items-center gap-6 ml-auto">
      <div class="flex items-center gap-2 text-sm">
        <span class="text-muted-foreground">总任务</span>
        <span class="font-semibold">{{ resultCount }}</span>
      </div>
      <div class="flex items-center gap-2 text-sm">
        <span class="text-muted-foreground">进行中</span>
        <span class="font-semibold text-blue-600">{{ inProgressCount }}</span>
      </div>
      <div class="flex items-center gap-2 text-sm">
        <span class="text-muted-foreground">已完成</span>
        <span class="font-semibold text-emerald-600">{{ doneCount }}</span>
      </div>
    </div>
    <div class="flex items-center gap-3">
      <Tabs v-model="viewModeValue">
        <TabsList>
          <TabsTrigger value="board">看板视图</TabsTrigger>
          <TabsTrigger value="quadrant">四象限视图</TabsTrigger>
          <TabsTrigger value="list">列表视图</TabsTrigger>
          <TabsTrigger value="gantt">甘特图</TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  </header>
  <FilterBar
    :priority="priority"
    :quadrant="quadrant"
    :time-range="timeRange"
    :custom-start="customStart"
    :custom-end="customEnd"
    :result-count="resultCount"
    @update:priority="emit('update:priority', $event)"
    @update:quadrant="emit('update:quadrant', $event)"
    @update:time-range="emit('update:timeRange', $event)"
    @update:custom-start="emit('update:customStart', $event)"
    @update:custom-end="emit('update:customEnd', $event)"
  />
  <div class="flex-1 overflow-x-auto overflow-y-hidden p-6">
    <div v-if="isLoading" class="flex items-center justify-center h-full text-muted-foreground">
      正在从数据库加载任务数据，请稍候...
    </div>
    <KanbanBoard
      v-else-if="viewMode === 'board'"
      :columns="COLUMNS"
      :tasks="tasks"
      @task-click="emit('taskClick', $event)"
    />
    <QuadrantView
      v-else-if="viewMode === 'quadrant'"
      :tasks="tasks"
      @task-click="emit('taskClick', $event)"
    />
    <ListView
      v-else-if="viewMode === 'list'"
      :tasks="tasks"
      @task-click="emit('taskClick', $event)"
    />
    <GanttView
      v-else
      :tasks="tasks"
      @task-click="emit('taskClick', $event)"
    />
  </div>
</template>
