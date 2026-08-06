<script setup lang="ts">
import { ref } from 'vue'
import { Plus, Trash2 } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import Checkbox from '@/components/ui/checkbox.vue'
import Input from '@/components/ui/input.vue'
import type { Subtask } from '@/api'

const props = defineProps<{
  subtasks: Subtask[]
}>()

const emit = defineEmits<{
  update: [value: Subtask[]]
}>()

const input = ref('')

const toggle = (sid: string) => {
  emit(
    'update',
    props.subtasks.map((s) => (s.id === sid ? { ...s, completed: !s.completed } : s))
  )
}

const add = () => {
  if (!input.value.trim()) return
  emit('update', [
    ...props.subtasks,
    { id: Date.now().toString(), title: input.value.trim(), completed: false },
  ])
  input.value = ''
}

const remove = (sid: string) => emit('update', props.subtasks.filter((s) => s.id !== sid))

const completedCount = () => props.subtasks.filter((s) => s.completed).length
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center justify-between">
      <span class="text-sm font-medium">子任务</span>
      <span class="text-xs text-muted-foreground">
        {{ completedCount() }}/{{ subtasks.length }}
      </span>
    </div>
    <div class="flex gap-2">
      <Input
        v-model="input"
        placeholder="输入子任务按回车添加"
        class="h-8 text-sm"
        @keydown.enter="add"
      />
      <Button type="button" size="sm" variant="outline" class="h-8 px-2" @click="add">
        <Plus class="w-4 h-4" />
      </Button>
    </div>
    <div class="space-y-1 max-h-40 overflow-y-auto">
      <div v-for="s in subtasks" :key="s.id" class="flex items-center gap-2 group">
        <Checkbox :model-value="s.completed" @update:model-value="toggle(s.id)" />
        <span
          class="flex-1 text-sm"
          :class="s.completed ? 'line-through text-muted-foreground' : ''"
        >
          {{ s.title }}
        </span>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
          @click="remove(s.id)"
        >
          <Trash2 class="w-3 h-3 text-destructive" />
        </Button>
      </div>
    </div>
  </div>
</template>
