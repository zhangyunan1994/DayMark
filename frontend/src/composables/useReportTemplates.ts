import { ref } from 'vue'
import { api } from '@/api'
import type { ReportTemplate, ReportTemplateInput } from '@/api'

const templates = ref<ReportTemplate[]>([])
const loaded = ref(false)

export function useReportTemplates() {
  const load = async () => {
    try {
      templates.value = await api.listReportTemplates()
    } catch {
      templates.value = []
    } finally {
      loaded.value = true
    }
  }

  const create = async (input: ReportTemplateInput) => {
    const template = await api.createReportTemplate(input)
    templates.value = [...templates.value, template]
    return template
  }

  const update = async (id: string, input: ReportTemplateInput) => {
    const template = await api.updateReportTemplate(id, input)
    templates.value = templates.value.map((t) => (t.id === id ? template : t))
    return template
  }

  const remove = async (id: string) => {
    await api.deleteReportTemplate(id)
    templates.value = templates.value.filter((t) => t.id !== id)
  }

  return { templates, loaded, load, create, update, remove }
}
