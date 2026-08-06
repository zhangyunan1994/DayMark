import { ref, computed } from 'vue'
import { api } from '@/api'
import type { GitLabConfig, Settings } from '@/api'

const DEFAULT_GITLAB: GitLabConfig = {
  base_url: 'https://gitlab.com',
  username: '',
  token: '',
}

export function useServerSettings() {
  const settings = ref<Settings | null>(null)
  const loading = ref(false)
  const error = ref('')

  const load = async () => {
    loading.value = true
    error.value = ''
    try {
      settings.value = await api.getSettings()
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  const save = async (next: Settings) => {
    error.value = ''
    try {
      const saved = await api.saveSettings(next)
      settings.value = saved
      return saved
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  const gitlab = computed(() => settings.value?.gitlab ?? DEFAULT_GITLAB)

  return { settings, gitlab, loading, error, load, save }
}
