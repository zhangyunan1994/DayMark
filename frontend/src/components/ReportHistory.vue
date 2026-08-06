<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MarkdownIt from 'markdown-it'
import { Check, Copy, Loader2, Trash2 } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import { api } from '@/api'
import type { Report } from '@/api'
import { REPORT_TYPE_LABEL } from '@/lib/reportTemplates'

const md = new MarkdownIt()

const reports = ref<Report[]>([])
const loading = ref(true)
const expandedId = ref<number | null>(null)
const copiedId = ref<number | null>(null)

const load = async () => {
  loading.value = true
  try {
    reports.value = await api.listReports()
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

onMounted(load)

const copy = async (report: Report) => {
  await navigator.clipboard.writeText(report.content)
  copiedId.value = report.id
  setTimeout(() => (copiedId.value = null), 2000)
}

const remove = async (id: number) => {
  if (!confirm('确定删除该报告？')) return
  await api.deleteReport(id)
  await load()
}
</script>

<template>
  <div class="h-full flex flex-col">
    <header class="bg-white border-b px-6 py-4 flex items-center justify-between shrink-0">
      <div>
        <h1 class="text-xl font-bold">报告记录</h1>
        <p class="text-sm text-muted-foreground mt-0.5">
          已生成的报告共 {{ reports.length }} 份
        </p>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="loading" class="flex items-center justify-center h-full text-muted-foreground gap-2">
        <Loader2 class="w-4 h-4 animate-spin" />
        加载中...
      </div>
      <div v-else-if="reports.length === 0" class="flex flex-col items-center justify-center h-full text-muted-foreground gap-3">
        <FileText class="w-10 h-10 opacity-30" />
        <p>暂无报告记录</p>
      </div>
      <div v-else class="space-y-3">
        <div v-for="r in reports" :key="r.id" class="rounded-lg border bg-white overflow-hidden">
          <div
            class="px-4 py-3 flex items-center justify-between cursor-pointer hover:bg-accent/40 transition-colors"
            @click="expandedId = expandedId === r.id ? null : r.id"
          >
            <div class="min-w-0">
              <div class="font-medium text-sm">{{ r.title }}</div>
              <div class="text-xs text-muted-foreground mt-0.5">
                <span class="inline-block px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground mr-2">
                  {{ REPORT_TYPE_LABEL[r.report_type] }}
                </span>
                {{ r.template_id }} · {{ new Date(r.created_at).toLocaleString() }}
              </div>
            </div>
            <div class="flex items-center gap-1 shrink-0 ml-4">
              <Button variant="ghost" size="sm" class="gap-1" @click.stop="copy(r)">
                <Check v-if="copiedId === r.id" class="w-3.5 h-3.5 text-emerald-600" />
                <Copy v-else class="w-3.5 h-3.5" />
                {{ copiedId === r.id ? '已复制' : '复制' }}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                class="text-destructive hover:text-destructive"
                @click.stop="remove(r.id)"
              >
                <Trash2 class="w-4 h-4" />
              </Button>
            </div>
          </div>

          <div v-if="expandedId === r.id" class="border-t px-4 py-4 bg-muted/20">
            <div class="report-md max-h-[60vh] overflow-y-auto text-sm leading-relaxed" v-html="md.render(r.content)" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
