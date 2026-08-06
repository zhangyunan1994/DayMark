<script setup lang="ts">
import { ref, watch } from 'vue'
import Dialog from '@/components/ui/dialog.vue'
import DialogHeader from '@/components/ui/dialog-header.vue'
import DialogTitle from '@/components/ui/dialog-title.vue'
import DialogFooter from '@/components/ui/dialog-footer.vue'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Checkbox from '@/components/ui/checkbox.vue'
import Select from '@/components/ui/select.vue'
import Textarea from '@/components/ui/textarea.vue'
import SubtaskList from './SubtaskList.vue'
import { COLUMNS, PRIORITY_OPTIONS } from '@/lib/taskData'
import { emptyForm } from '@/composables/useTasks'
import type { Task, TaskInput } from '@/api'

const props = defineProps<{
  open: boolean
  task: Task | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  save: [task: Partial<TaskInput> & { id?: number }]
  delete: [id: number]
}>()

const form = ref<TaskInput>({ ...emptyForm })

watch(
  () => props.open,
  (val) => {
    if (val) {
      if (props.task) {
        const { id: _id, created_at: _c, updated_at: _u, ...rest } = props.task
        form.value = { ...rest }
      } else {
        form.value = { ...emptyForm }
      }
    }
  }
)

const handleSubmit = (e: Event) => {
  e.preventDefault()
  emit('save', props.task ? { id: props.task.id, ...form.value } : { ...form.value })
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogHeader>
      <DialogTitle>{{ task ? '编辑任务' : '新建任务' }}</DialogTitle>
    </DialogHeader>
    <form class="space-y-4" @submit="handleSubmit">
      <div>
        <Label>任务标题</Label>
        <Input v-model="form.title" required />
      </div>
      <div>
        <Label>描述</Label>
        <Textarea v-model="form.description" :rows="3" />
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <Label>状态</Label>
          <Select v-model="form.status">
            <option v-for="c in COLUMNS" :key="c.id" :value="c.id">{{ c.title }}</option>
          </Select>
        </div>
        <div>
          <Label>优先级</Label>
          <Select v-model="form.priority">
            <option v-for="p in PRIORITY_OPTIONS" :key="p.value" :value="p.value">{{ p.label }}</option>
          </Select>
        </div>
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <Label>负责人</Label>
          <Input v-model="form.assignee" />
        </div>
        <div>
          <Label>截止日期</Label>
          <Input
            type="datetime-local"
            :model-value="form.due_date?.replace(' ', 'T') ?? ''"
            @update:model-value="form.due_date = ($event as string).replace('T', ' ') || null"
          />
        </div>
      </div>
      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2">
          <Checkbox v-model="form.urgent" />
          <Label class="text-sm font-normal">紧急</Label>
        </div>
        <div class="flex items-center gap-2">
          <Checkbox v-model="form.important" />
          <Label class="text-sm font-normal">重要</Label>
        </div>
      </div>
      <SubtaskList :subtasks="form.subtasks" @update="form.subtasks = $event" />
      <DialogFooter class="gap-2">
        <Button
          v-if="task"
          type="button"
          variant="destructive"
          @click="emit('delete', task.id); emit('update:open', false)"
        >
          删除
        </Button>
        <Button type="submit">{{ task ? '保存' : '创建' }}</Button>
      </DialogFooter>
    </form>
  </Dialog>
</template>
