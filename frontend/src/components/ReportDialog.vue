<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import MarkdownIt from 'markdown-it'
import { Check, Copy, Loader2, Trash2 } from 'lucide-vue-next'
import Dialog from '@/components/ui/dialog.vue'
import DialogHeader from '@/components/ui/dialog-header.vue'
import DialogTitle from '@/components/ui/dialog-title.vue'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Tabs from '@/components/ui/tabs.vue'
import TabsList from '@/components/ui/tabs-list.vue'
import TabsTrigger from '@/components/ui/tabs-trigger.vue'
import TabsContent from '@/components/ui/tabs-content.vue'
import { api } from '@/api'
import type { Report } from '@/api'
import {
  REPORT_TEMPLATES,
  REPORT_TYPE_LABEL,
  dateRangeForType,
} from '@/lib/reportTemplates'
import type { ReportType } from '@/lib/reportTemplates'

const props = defineProps<{
  open: boolean
  hasLlm: boolean
  hasGitlab: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const md = new MarkdownIt()

const reportType = ref<ReportType>('weekly')
const dateRange = ref(dateRangeForType('weekly'))
const templateId = ref('weekly-progress')
const generating = ref(false)
const error = ref('')
const result = ref<Report | null>(null)
const copied = ref(false)
const history = ref<Report[]>([])
const activeTab = ref('generate')

const templates = computed(() => REPORT_TEMPLATES.filter((t) => t.report_type === reportType.value))
const template = computed(() => templates.value.find((t) => t.id === templateId.value) ?? templates.value[0])

watch(
  () => props.open,
  (val) => {
    if (val) {
      result.value = null
      error.value = ''
      loadHistory()
    }
  }
)

watch(reportType, (type) => {
  dateRange.value = dateRangeForType(type)
  const t = REPORT_TEMPLATES.find((t) => t.report_type === type)
  if (t) templateId.value = t.id
})

const loadHistory = async () => {
  try {
    history.value = await api.listReports()
  } catch (e) {
    error.value = (e as Error).message
  }
}

const generate = async () => {
  if (!template.value) return
  generating.value = true
  error.value = ''
  try {
    const report = await api.generateReport({
      report_type: reportType.value,
      template_name: template.value.name,
      date_start: dateRange.value.start,
      date_end: dateRange.value.end,
      include_gitlab: true,
    })
    result.value = report
    await loadHistory()
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

const remove = async (id: number) => {
  await api.deleteReport(id)
  await loadHistory()
}

const showReport = (report: Report) => {
  result.value = report
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)" class="sm:max-w-4xl max-h-[85vh] overflow-hidden flex flex-col p-0">
    <DialogHeader class="px-6 pt-6 pb-0">
      <DialogTitle>生成报告</DialogTitle>
    </DialogHeader>

    <Tabs v-model="activeTab" default-value="generate" class="flex flex-col min-h-0 flex-1">
      <div class="px-6 py-3 border-b">
        <TabsList>
          <TabsTrigger value="generate">生成报告</TabsTrigger>
          <TabsTrigger value="history">历史报告 ({{ history.length }})</TabsTrigger>
        </TabsList>
      </div>

      <TabsContent value="generate" class="min-h-0 flex-1 overflow-y-auto p-6 space-y-5 mt-0">
        <div
          v-if="!hasLlm"
          class="text-sm px-3 py-2 rounded-md border border-amber-200 bg-amber-50 text-amber-700"
        >
          未配置 LLM，将使用模板生成框架内容。可在侧边栏「设置」中配置 LLM 后获得 AI 生成能力。
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
                @click="reportType = t"
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
          <Label>选择模板</Label>
          <div class="grid grid-cols-3 gap-2 mt-1.5">
            <button
              v-for="t in templates"
              :key="t.id"
              class="text-left p-3 rounded-lg border transition-all"
              :class="template?.id === t.id ? 'border-primary bg-accent/60' : 'hover:border-slate-300'"
              @click="templateId = t.id"
            >
              <div class="flex items-center justify-between">
                <span class="text-sm font-semibold">{{ t.name }}</span>
                <Check v-if="template?.id === t.id" class="w-3.5 h-3.5 text-primary" />
              </div>
              <p class="text-xs text-muted-foreground mt-1 leading-relaxed">
                {{ t.description }}
              </p>
              <div class="flex flex-wrap gap-1 mt-2">
                <span
                  v-for="s in t.sections.slice(0, 3)"
                  :key="s"
                  class="text-[10px] px-1.5 py-0.5 rounded bg-background border text-muted-foreground"
                >
                  {{ s }}
                </span>
              </div>
            </button>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <Button :disabled="generating || !template" class="gap-2" @click="generate">
            <Loader2 v-if="generating" class="w-4 h-4 animate-spin" />
            {{ generating ? 'AI 生成中...' : hasLlm ? `AI 生成${REPORT_TYPE_LABEL[reportType]}` : `生成${REPORT_TYPE_LABEL[reportType]}` }}
          </Button>
          <span v-if="!hasGitlab" class="text-xs text-muted-foreground">
            未配置 GitLab，报告将不包含合并请求数据
          </span>
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
              <Button variant="ghost" size="sm" @click="result = null">关闭</Button>
            </div>
          </div>
          <div class="report-md p-5 overflow-y-auto max-h-[45vh] text-sm leading-relaxed" v-html="md.render(result.content)" />
        </div>
      </TabsContent>

      <TabsContent value="history" class="min-h-0 flex-1 overflow-y-auto p-6 mt-0">
        <p v-if="history.length === 0" class="text-center text-muted-foreground py-10">暂无历史报告</p>
        <div v-else class="space-y-2">
          <div
            v-for="r in history"
            :key="r.id"
            class="rounded-lg border p-3 flex items-center justify-between gap-4 hover:bg-accent/40 cursor-pointer"
            @click="showReport(r)"
          >
            <div class="min-w-0">
              <div class="font-medium text-sm truncate">{{ r.title }}</div>
              <div class="text-xs text-muted-foreground mt-0.5">
                {{ REPORT_TYPE_LABEL[r.report_type] }} · {{ r.template_id }} · {{ new Date(r.created_at).toLocaleString() }}
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              class="shrink-0 text-destructive hover:text-destructive"
              @click.stop="remove(r.id)"
            >
              <Trash2 class="w-4 h-4" />
            </Button>
          </div>
        </div>
      </TabsContent>
    </Tabs>
  </Dialog>
</template>
