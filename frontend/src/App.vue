<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import Sidebar from '@/components/Sidebar.vue'
import TaskBoardPage from '@/components/TaskBoardPage.vue'
import TaskEditor from '@/components/TaskEditor.vue'
import SettingsPage from '@/components/SettingsPage.vue'
import ReportHistory from '@/components/ReportHistory.vue'
import ReportGeneratorPage from '@/components/ReportGeneratorPage.vue'
import GitLabMRPage from '@/components/GitLabMRPage.vue'
import OpenCodeAnalytics from '@/components/OpenCodeAnalytics.vue'
import { useTasks } from '@/composables/useTasks'
import { useServerSettings } from '@/composables/useServerSettings'
import { useFilters } from '@/composables/useFilters'
import type { Task } from '@/api'

type ViewMode = 'board' | 'quadrant' | 'list' | 'gantt' | 'reports' | 'templates' | 'settings' | 'mr' | 'opencode'
type TaskViewMode = 'board' | 'quadrant' | 'list' | 'gantt'

const TASK_VIEW_MODES: TaskViewMode[] = ['board', 'quadrant', 'list', 'gantt']

const { tasks, save, remove, isLoading } = useTasks()
const { settings, load: loadSettings } = useServerSettings()

const editingTask = ref<Task | null>(null)
const isEditorOpen = ref(false)
const viewMode = ref<ViewMode>('list')

const { filtered, priority, quadrant, timeRange, customStart, customEnd } = useFilters(tasks)

const isTaskView = computed(() => TASK_VIEW_MODES.includes(viewMode.value as TaskViewMode))
const taskViewMode = computed<TaskViewMode>({
  get: () => viewMode.value as TaskViewMode,
  set: (v) => { viewMode.value = v },
})

const hasActiveLlm = computed(() => {
  const configs = settings.value?.llm_configs ?? []
  return configs.some((c) => c.is_active) || configs.length > 0
})

const hasGitlab = computed(() => !!settings.value?.gitlab?.token)

onMounted(() => {
  loadSettings()
})

const openNew = () => {
  editingTask.value = null
  isEditorOpen.value = true
}

const openEdit = (task: Task) => {
  editingTask.value = task
  isEditorOpen.value = true
}

const toggleView = (mode: ViewMode) => {
  viewMode.value = mode
}
</script>

<template>
  <div class="h-screen flex bg-gray-50 overflow-hidden">
    <Sidebar
      @new-task="openNew"
      @open-settings="toggleView('settings')"
      @open-reports="toggleView('reports')"
      @open-board="viewMode = 'list'"
      @open-templates="toggleView('templates')"
      @open-mr="toggleView('mr')"
      @open-opencode="toggleView('opencode')"
    />
    <div class="flex-1 flex flex-col overflow-hidden">
      <TaskBoardPage
        v-if="isTaskView"
        v-model:view-mode="taskViewMode"
        :tasks="filtered"
        :is-loading="isLoading"
        v-model:priority="priority"
        v-model:quadrant="quadrant"
        v-model:time-range="timeRange"
        v-model:custom-start="customStart"
        v-model:custom-end="customEnd"
        :result-count="filtered.length"
        @task-click="openEdit"
      />
      <template v-else>
        <ReportHistory v-if="viewMode === 'reports'" />
        <SettingsPage v-else-if="viewMode === 'settings'" />
        <GitLabMRPage v-else-if="viewMode === 'mr'" />
        <OpenCodeAnalytics v-else-if="viewMode === 'opencode'" />
        <ReportGeneratorPage
          v-else-if="viewMode === 'templates'"
          :has-llm="hasActiveLlm"
          :has-gitlab="hasGitlab"
        />
      </template>
    </div>
    <TaskEditor
      :open="isEditorOpen"
      :task="editingTask"
      @update:open="isEditorOpen = $event"
      @save="save"
      @delete="remove"
    />
  </div>
</template>
