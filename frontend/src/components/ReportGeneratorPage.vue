<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import MarkdownIt from 'markdown-it'
import { Check, Copy, FileText, Loader2, Plus, Pencil, Trash2, Eye } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Textarea from '@/components/ui/textarea.vue'
import Dialog from '@/components/ui/dialog.vue'
import { api } from '@/api'
import type { Report, ReportTemplate } from '@/api'
import { useReportTemplates } from '@/composables/useReportTemplates'
import {
  REPORT_TEMPLATES,
  REPORT_TYPE_LABEL,
  dateRangeForType,
} from '@/lib/reportTemplates'
import type { ReportType } from '@/lib/reportTemplates'

defineProps<{
  hasLlm: boolean
  hasGitlab: boolean
}>()

const md = new MarkdownIt()

const reportType = ref<ReportType>('weekly')
const dateRange = ref(dateRangeForType('weekly'))
const templateId = ref('weekly-progress')
const generating = ref(false)
const error = ref('')
const result = ref<Report | null>(null)
const copied = ref(false)

interface TemplateItem {
  id: string
  report_type: ReportType
  name: string
  description: string
  sections: string[]
  builtin: boolean
}

const builtinTemplates: TemplateItem[] = REPORT_TEMPLATES.map((t) => ({ ...t, builtin: true }))

const { templates: customTemplates, load: loadCustomTemplates } = useReportTemplates()
onMounted(loadCustomTemplates)

const templates = computed<TemplateItem[]>(() => {
  const customs: TemplateItem[] = customTemplates.value.map((t) => ({ ...t, builtin: false }))
  return [...builtinTemplates, ...customs].filter((t) => t.report_type === reportType.value)
})
const template = computed(() => templates.value.find((t) => t.id === templateId.value) ?? templates.value[0])

watch(reportType, (type) => {
  dateRange.value = dateRangeForType(type)
  const t = builtinTemplates.find((t) => t.report_type === type)
  if (t) templateId.value = t.id
})

const switchType = (type: ReportType) => {
  reportType.value = type
  dateRange.value = dateRangeForType(type)
  templateId.value = builtinTemplates.find((t) => t.report_type === type)?.id ?? ''
}

const generate = async () => {
  if (!template.value) return
  generating.value = true
  error.value = ''
  try {
    const report = await api.generateReport({
      report_type: reportType.value,
      template_name: template.value.name,
      template_sections: template.value.sections,
      date_start: dateRange.value.start,
      date_end: dateRange.value.end,
      include_gitlab: true,
    })
    result.value = report
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    generating.value = false
  }
}

const copy = async () => {
  if (!result.value) return
  await navigator.clipboard.writeText(result.value.content)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}

const viewTemplate = ref<TemplateItem | null>(null)

const editorOpen = ref(false)
const editingTemplate = ref<TemplateItem | null>(null)
const saving = ref(false)
const formError = ref('')
const form = ref({
  name: '',
  report_type: 'weekly' as ReportType,
  description: '',
  sectionsText: '',
})

const openCreate = () => {
  editingTemplate.value = null
  form.value = {
    name: '',
    report_type: reportType.value,
    description: '',
    sectionsText: '',
  }
  formError.value = ''
  editorOpen.value = true
}

const openEdit = (t: TemplateItem) => {
  editingTemplate.value = t
  form.value = {
    name: t.name,
    report_type: t.report_type,
    description: t.description,
    sectionsText: t.sections.join('\n'),
  }
  formError.value = ''
  editorOpen.value = true
}

const save = async () => {
  if (!form.value.name.trim()) {
    formError.value = '请输入模版名称'
    return
  }
  const input = {
    name: form.value.name.trim(),
    report_type: form.value.report_type,
    description: form.value.description.trim(),
    sections: form.value.sectionsText.split('\n').map((s) => s.trim()).filter(Boolean),
  }
  saving.value = true
  formError.value = ''
  try {
    let saved: ReportTemplate
    if (editingTemplate.value) {
      saved = await api.updateReportTemplate(editingTemplate.value.id, input)
    } else {
      saved = await api.createReportTemplate(input)
    }
    if (editingTemplate.value && templateId.value === editingTemplate.value.id) {
      templateId.value = saved.id
    }
    editorOpen.value = false
    loadCustomTemplates()
  } catch (e) {
    formError.value = (e as Error).message
  } finally {
    saving.value = false
  }
}

const removeTemplate = async (t: TemplateItem) => {
  if (!window.confirm(`确定删除模版「${t.name}」吗？`)) return
  try {
    await api.deleteReportTemplate(t.id)
    if (templateId.value === t.id) templateId.value = ''
    loadCustomTemplates()
  } catch (e) {
    alert(`删除失败：${(e as Error).message}`)
  }
}
</script>

<template>
  <div class="h-full flex flex-col">
    <header class="bg-white border-b px-6 py-4 flex items-center justify-between shrink-0">
      <div>
        <h1 class="text-xl font-bold">报告模版</h1>
        <p class="text-sm text-muted-foreground mt-0.5">
          {{ hasLlm ? '选择模版后 AI 将自动根据工作记录生成报告' : '配置 LLM 后可使用 AI 生成能力' }}
        </p>
      </div>
      <div class="flex items-center gap-2">
        <Button variant="outline" class="gap-2" @click="openCreate">
          <Plus class="w-4 h-4" />
          新增模版
        </Button>
        <Button :disabled="generating || !template" class="gap-2" @click="generate">
          <Loader2 v-if="generating" class="w-4 h-4 animate-spin" />
          <FileText v-else class="w-4 h-4" />
          {{ generating ? 'AI 生成中...' : '生成报告' }}
        </Button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div class="space-y-6">
        <div
          v-if="!hasLlm"
          class="text-sm px-3 py-2 rounded-md border border-amber-200 bg-amber-50 text-amber-700"
        >
          未配置 LLM，将使用模版生成框架内容。可在「设置」中配置 LLM 后获得 AI 生成能力。
        </div>

        <div class="flex flex-wrap items-end gap-4">
          <div>
            <Label>报告类型</Label>
            <div class="flex gap-1 mt-1.5 bg-muted p-1 rounded-md">
              <button
                v-for="t in (['daily', 'weekly', 'monthly'] as ReportType[])"
                :key="t"
                class="px-4 py-1.5 rounded text-sm font-medium transition-colors"
                :class="reportType === t ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'"
                @click="switchType(t)"
              >
                {{ REPORT_TYPE_LABEL[t] }}
              </button>
            </div>
          </div>
          <div class="flex items-end gap-2">
            <div>
              <Label>开始日期</Label>
              <Input
                type="date"
                class="mt-1.5 h-9 w-40"
                :model-value="dateRange.start"
                @update:model-value="dateRange.start = $event"
              />
            </div>
            <span class="text-muted-foreground pb-2">至</span>
            <div>
              <Label>结束日期</Label>
              <Input
                type="date"
                class="mt-1.5 h-9 w-40"
                :model-value="dateRange.end"
                @update:model-value="dateRange.end = $event"
              />
            </div>
          </div>
        </div>

        <div>
          <Label>选择模版</Label>
          <div class="grid grid-cols-3 gap-3 mt-1.5">
            <div
              v-for="t in templates"
              :key="t.id"
              class="text-left p-4 rounded-lg border transition-all cursor-pointer"
              :class="template?.id === t.id ? 'border-primary bg-accent/60' : 'hover:border-slate-300'"
              @click="templateId = t.id"
            >
              <div class="flex items-center justify-between">
                <span class="text-sm font-semibold">{{ t.name }}</span>
                <Check v-if="template?.id === t.id" class="w-4 h-4 text-primary" />
              </div>
              <p class="text-xs text-muted-foreground mt-1.5 leading-relaxed">
                {{ t.description }}
              </p>
              <div class="flex flex-wrap gap-1 mt-3">
                <span
                  v-for="s in t.sections"
                  :key="s"
                  class="text-[10px] px-1.5 py-0.5 rounded bg-background border text-muted-foreground"
                >
                  {{ s }}
                </span>
              </div>
              <div class="flex items-center gap-1 mt-3" @click.stop>
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-7 px-2 text-xs gap-1"
                  @click="viewTemplate = t"
                >
                  <Eye class="w-3 h-3" />
                  查看
                </Button>
                <template v-if="!t.builtin">
                  <Button
                    variant="ghost"
                    size="sm"
                    class="h-7 px-2 text-xs gap-1"
                    @click="openEdit(t)"
                  >
                    <Pencil class="w-3 h-3" />
                    编辑
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    class="h-7 px-2 text-xs gap-1 text-red-600 hover:text-red-600 hover:bg-red-50"
                    @click="removeTemplate(t)"
                  >
                    <Trash2 class="w-3 h-3" />
                    删除
                  </Button>
                </template>
              </div>
            </div>
          </div>
        </div>

        <div v-if="error" class="text-sm text-red-600 whitespace-pre-wrap">{{ error }}</div>

        <div v-if="result" class="rounded-lg border">
          <div class="flex items-center justify-between px-4 py-2.5 border-b bg-muted/40">
            <span class="text-sm font-medium">{{ result.title }}</span>
            <div class="flex items-center gap-2">
              <Button variant="outline" size="sm" class="gap-1.5" @click="copy">
                <Check v-if="copied" class="w-3.5 h-3.5" />
                <Copy v-else class="w-3.5 h-3.5" />
                {{ copied ? '已复制' : '复制' }}
              </Button>
            </div>
          </div>
          <div class="report-md p-5 overflow-y-auto max-h-[50vh] text-sm leading-relaxed" v-html="md.render(result.content)" />
        </div>
      </div>
    </div>

    <Dialog :open="!!viewTemplate" @update:open="v => { if (!v) viewTemplate = null }">
      <template v-if="viewTemplate">
        <div class="pr-6">
          <div class="flex items-center gap-2 mb-1">
            <span class="text-[10px] px-1.5 py-0.5 rounded border bg-blue-50 text-blue-700 border-blue-200">
              {{ REPORT_TYPE_LABEL[viewTemplate.report_type] }}
            </span>
            <span v-if="viewTemplate.builtin" class="text-[10px] px-1.5 py-0.5 rounded border bg-gray-100 text-gray-600 border-gray-200">内置</span>
            <span v-else class="text-[10px] px-1.5 py-0.5 rounded border bg-purple-50 text-purple-700 border-purple-200">自定义</span>
          </div>
          <h2 class="text-lg font-bold">{{ viewTemplate.name }}</h2>
          <p class="text-sm text-muted-foreground mt-1">{{ viewTemplate.description || '无描述' }}</p>
        </div>
        <div>
          <div class="text-sm font-medium mb-2">板块结构</div>
          <ol class="space-y-1.5">
            <li
              v-for="(s, i) in viewTemplate.sections"
              :key="i"
              class="flex items-center gap-2 text-sm"
            >
              <span class="w-5 h-5 shrink-0 inline-flex items-center justify-center rounded-full bg-muted text-xs font-medium">
                {{ i + 1 }}
              </span>
              {{ s }}
            </li>
          </ol>
        </div>
      </template>
    </Dialog>

    <Dialog :open="editorOpen" @update:open="v => { if (!v) editorOpen = false }">
      <div class="pr-6 space-y-4">
        <h2 class="text-lg font-bold">{{ editingTemplate ? '编辑模版' : '新增模版' }}</h2>
        <div>
          <Label>模版名称</Label>
          <Input v-model="form.name" class="mt-1.5" placeholder="例如：研发周报" />
        </div>
        <div>
          <Label>报告类型</Label>
          <div class="flex gap-1 mt-1.5 bg-muted p-1 rounded-md">
            <button
              v-for="t in (['daily', 'weekly', 'monthly'] as ReportType[])"
              :key="t"
              class="px-4 py-1.5 rounded text-sm font-medium transition-colors"
              :class="form.report_type === t ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'"
              @click="form.report_type = t"
            >
              {{ REPORT_TYPE_LABEL[t] }}
            </button>
          </div>
        </div>
        <div>
          <Label>描述</Label>
          <Input v-model="form.description" class="mt-1.5" placeholder="模版简介" />
        </div>
        <div>
          <Label>板块结构（每行一个板块）</Label>
          <Textarea
            v-model="form.sectionsText"
            class="mt-1.5"
            :rows="5"
            placeholder="例如：&#10;本周完成&#10;进行中的工作&#10;遇到的问题&#10;下周计划"
          />
        </div>
        <div v-if="formError" class="text-sm text-red-600">{{ formError }}</div>
        <div class="flex justify-end gap-2 pt-2">
          <Button variant="outline" @click="editorOpen = false">取消</Button>
          <Button :disabled="saving" class="gap-2" @click="save">
            <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />
            {{ editingTemplate ? '保存' : '创建' }}
          </Button>
        </div>
      </div>
    </Dialog>
  </div>
</template>
