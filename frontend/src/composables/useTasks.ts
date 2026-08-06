import { ref, onMounted } from 'vue'
import { api } from '@/api'
import type { Task, TaskInput } from '@/api'

export const emptyForm: TaskInput = {
  title: '',
  description: '',
  status: 'todo',
  priority: 'medium',
  assignee: '',
  due_date: null,
  urgent: false,
  important: false,
  subtasks: [],
}

export function useTasks() {
  const tasks = ref<Task[]>([])
  const isLoading = ref(true)

  const load = async () => {
    isLoading.value = true
    try {
      tasks.value = await api.list()
    } finally {
      isLoading.value = false
    }
  }

  const save = async (task: Partial<TaskInput> & { id?: number }) => {
    if (task.id) {
      await api.update(task.id, task)
    } else {
      await api.create(task as TaskInput)
    }
    await load()
  }

  const remove = async (id: number) => {
    await api.remove(id)
    await load()
  }

  onMounted(load)

  return { tasks, isLoading, save, remove }
}
