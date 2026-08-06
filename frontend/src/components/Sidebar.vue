<script setup lang="ts">
import { Layout, Plus, Settings, ListTodo, NotebookPen, History, GitBranch } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'

defineEmits<{
  newTask: []
  openSettings: []
  openReports: []
  openBoard: []
  openTemplates: []
  openMr: []
}>()
</script>

<template>
  <div class="w-52 h-full bg-white border-r flex flex-col shrink-0">
    <div class="p-5 border-b">
      <div class="flex items-center gap-2 mb-4">
        <Layout class="w-5 h-5 text-primary" />
        <h2 class="font-bold text-lg">任务管理中心</h2>
      </div>
      <Button class="w-full gap-2" @click="$emit('newTask')">
        <Plus class="w-4 h-4" /> 新建任务
      </Button>
    </div>
    
    <div class="p-4 space-y-2 flex-1">
      <button
        class="w-full flex items-center gap-2 py-2 px-4 rounded-md bg-secondary text-foreground font-medium text-sm transition-colors hover:bg-secondary/80"
        @click="$emit('openBoard')"
      >
        <ListTodo class="w-4 h-4 shrink-0" />
        待办任务
      </button>
      <button
        class="w-full flex items-center gap-2 py-2 px-4 rounded-md bg-secondary text-foreground font-medium text-sm transition-colors hover:bg-secondary/80"
        @click="$emit('openMr')"
      >
        <GitBranch class="w-4 h-4 shrink-0" />
        GitLab MR
      </button>
      <button
        class="w-full flex items-center gap-2 py-2 px-4 rounded-md bg-secondary text-foreground font-medium text-sm transition-colors hover:bg-secondary/80"
        @click="$emit('openTemplates')"
      >
        <NotebookPen class="w-4 h-4 shrink-0" />
        报告模版
      </button>
      <button
        class="w-full flex items-center gap-2 py-2 px-4 rounded-md bg-secondary text-foreground font-medium text-sm transition-colors hover:bg-secondary/80"
        @click="$emit('openReports')"
      >
        <History class="w-4 h-4 shrink-0" />
        报告记录
      </button>

    </div>

    <div class="p-3 border-t">
      <button
        class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
        @click="$emit('openSettings')"
      >
        <Settings class="w-4 h-4" />
        设置
      </button>
    </div>
  </div>
</template>
