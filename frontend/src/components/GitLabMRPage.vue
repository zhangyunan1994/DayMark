<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { GitBranch, Loader2, ExternalLink, XCircle, GitMerge, Wand2, Eye } from 'lucide-vue-next'
import MarkdownIt from 'markdown-it'
import Input from '@/components/ui/input.vue'
import Button from '@/components/ui/button.vue'
import Dialog from '@/components/ui/dialog.vue'
import { api } from '@/api'
import type { GitLabMR } from '@/api'

const md = new MarkdownIt({ html: false, linkify: true })

const mrs = ref<GitLabMR[]>([])
const loading = ref(true)
const error = ref('')
const search = ref('')
const filterState = ref<'all' | 'opened' | 'merged' | 'closed'>('all')
const rewritingIds = ref<number[]>([])
const notice = ref('')
const detailMr = ref<GitLabMR | null>(null)

const stateConfig: Record<string, { label: string; color: string; icon: any }> = {
  opened: { label: '开放', color: 'bg-green-100 text-green-700 border-green-200', icon: GitBranch },
  merged: { label: '已合并', color: 'bg-purple-100 text-purple-700 border-purple-200', icon: GitMerge },
  closed: { label: '已关闭', color: 'bg-red-100 text-red-700 border-red-200', icon: XCircle },
}

const load = async () => {
  loading.value = true
  error.value = ''
  try {
    mrs.value = await api.gitlabMrList()
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    loading.value = false
  }
}

onMounted(load)

const isRewriting = (id: number) => rewritingIds.value.includes(id)

const rewrite = async (mr: GitLabMR) => {
  if (isRewriting(mr.id)) return
  if (!window.confirm(`确定使用 LLM 重写 MR !${mr.iid} 的标题和描述吗？`)) return
  rewritingIds.value = [...rewritingIds.value, mr.id]
  notice.value = ''
  try {
    const result = await api.rewriteMr(mr.project_id, mr.iid)
    notice.value = `已重写 MR !${mr.iid}，标题：「${result.title}」`
    await load()
  } catch (e) {
    notice.value = `重写失败：${(e as Error).message}`
  } finally {
    rewritingIds.value = rewritingIds.value.filter(id => id !== mr.id)
  }
}

const filtered = computed(() => {
  let result = mrs.value
  if (filterState.value !== 'all') {
    result = result.filter(mr => mr.state === filterState.value)
  }
  if (search.value) {
    const q = search.value.toLowerCase()
    result = result.filter(mr =>
      mr.title.toLowerCase().includes(q) ||
      (mr.source_branch ?? '').toLowerCase().includes(q) ||
      (mr.target_branch ?? '').toLowerCase().includes(q) ||
      (mr.author?.username ?? '').toLowerCase().includes(q)
    )
  }
  return result
})

const formatDate = (s: string) => {
  const d = new Date(s)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return '刚刚'
  if (diffMin < 60) return `${diffMin} 分钟前`
  const diffHour = Math.floor(diffMin / 60)
  if (diffHour < 24) return `${diffHour} 小时前`
  const diffDay = Math.floor(diffHour / 24)
  if (diffDay < 30) return `${diffDay} 天前`
  return d.toLocaleDateString()
}

const stateCounts = computed(() => ({
  all: mrs.value.length,
  opened: mrs.value.filter(m => m.state === 'opened').length,
  merged: mrs.value.filter(m => m.state === 'merged').length,
  closed: mrs.value.filter(m => m.state === 'closed').length,
}))
</script>

<template>
  <div class="h-full flex flex-col">
    <header class="bg-white border-b px-6 py-4 flex items-center justify-between shrink-0">
      <div>
        <h1 class="text-xl font-bold">GitLab Merge Requests</h1>
        <p class="text-sm text-muted-foreground mt-0.5">
          {{ loading ? '加载中...' : `共 ${mrs.length} 个 MR` }}
        </p>
      </div>
      <div class="flex items-center gap-2">
        <Input
          v-model="search"
          placeholder="搜索标题、分支、作者..."
          class="h-9 w-64"
        />
      </div>
    </header>

    <div class="px-6 py-2 border-b bg-white shrink-0">
      <div class="flex gap-1 bg-muted p-0.5 rounded-md">
        <button
          v-for="(count, key) in stateCounts"
          :key="key"
          class="px-3 py-1 rounded text-xs font-medium transition-colors"
          :class="filterState === key ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'"
          @click="filterState = key as typeof filterState"
        >
          {{ { all: '全部', opened: '开放', merged: '已合并', closed: '已关闭' }[key as string] }}
          ({{ count }})
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-6">
      <div
        v-if="notice"
        class="mb-4 px-4 py-2.5 rounded-lg text-sm border"
        :class="notice.startsWith('重写失败') ? 'bg-red-50 text-red-600 border-red-200' : 'bg-green-50 text-green-700 border-green-200'"
      >
        {{ notice }}
      </div>
      <div v-if="loading" class="flex items-center justify-center h-full text-muted-foreground gap-2">
        <Loader2 class="w-4 h-4 animate-spin" />
        正在从 GitLab 加载数据...
      </div>
      <div v-else-if="error" class="flex flex-col items-center justify-center h-full text-muted-foreground gap-3">
        <XCircle class="w-10 h-10 text-red-400" />
        <p class="text-red-600">{{ error }}</p>
        <p class="text-sm">请在「设置」中配置 GitLab 连接</p>
      </div>
      <div v-else-if="filtered.length === 0" class="flex flex-col items-center justify-center h-full text-muted-foreground gap-3">
        <GitBranch class="w-10 h-10 opacity-30" />
        <p>暂无 Merge Request</p>
      </div>
      <div v-else class="space-y-2">
        <div
          v-for="mr in filtered"
          :key="mr.id"
          class="block rounded-lg border bg-white p-4 hover:shadow-md hover:border-slate-300 transition-all group"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2 mb-1">
                <span
                  class="inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded border font-medium"
                  :class="stateConfig[mr.state]?.color ?? 'bg-gray-100 text-gray-700 border-gray-200'"
                >
                  <component :is="stateConfig[mr.state]?.icon ?? GitBranch" class="w-3 h-3" />
                  {{ stateConfig[mr.state]?.label ?? mr.state }}
                </span>
                <span class="text-xs text-muted-foreground">
                  !{{ mr.iid }}
                </span>
              </div>
              <h3 class="font-medium text-sm group-hover:text-primary transition-colors">
                {{ mr.title }}
              </h3>
              <div class="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
                <span class="font-mono bg-muted px-1.5 py-0.5 rounded">
                  {{ mr.source_branch }}
                </span>
                <span>→</span>
                <span class="font-mono bg-muted px-1.5 py-0.5 rounded">
                  {{ mr.target_branch }}
                </span>
                <span class="ml-auto">
                  {{ mr.author?.name ?? mr.author?.username ?? '' }} · {{ formatDate(mr.updated_at) }}
                </span>
              </div>
            </div>
            <div class="flex items-center gap-2 shrink-0 mt-1">
              <Button
                variant="ghost"
                size="sm"
                class="h-7 px-2 text-xs gap-1"
                @click="detailMr = mr"
              >
                <Eye class="w-3 h-3" />
                详情
              </Button>
              <Button
                v-if="mr.state === 'opened'"
                variant="outline"
                size="sm"
                class="h-7 px-2 text-xs gap-1"
                :disabled="isRewriting(mr.id)"
                @click="rewrite(mr)"
              >
                <Loader2 v-if="isRewriting(mr.id)" class="w-3 h-3 animate-spin" />
                <Wand2 v-else class="w-3 h-3" />
                {{ isRewriting(mr.id) ? '重写中...' : '重写' }}
              </Button>
              <a
                :href="mr.web_url"
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center justify-center w-7 h-7 rounded-md text-muted-foreground hover:text-primary hover:bg-muted transition-colors"
                :title="`在 GitLab 中打开 !${mr.iid}`"
              >
                <ExternalLink class="w-4 h-4" />
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>

    <Dialog
      :open="!!detailMr"
      @update:open="v => { if (!v) detailMr = null }"
      class="w-full max-w-2xl"
    >
      <template v-if="detailMr">
        <div class="pr-6">
          <div class="flex items-center gap-2 mb-2">
            <span
              class="inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded border font-medium"
              :class="stateConfig[detailMr.state]?.color ?? 'bg-gray-100 text-gray-700 border-gray-200'"
            >
              <component :is="stateConfig[detailMr.state]?.icon ?? GitBranch" class="w-3 h-3" />
              {{ stateConfig[detailMr.state]?.label ?? detailMr.state }}
            </span>
            <span class="text-xs text-muted-foreground">!{{ detailMr.iid }}</span>
            <span v-if="detailMr.draft" class="text-[10px] px-1.5 py-0.5 rounded bg-gray-100 text-gray-500 border border-gray-200">Draft</span>
          </div>
          <h2 class="text-lg font-bold leading-snug">{{ detailMr.title }}</h2>
          <div class="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
            <span class="font-mono bg-muted px-1.5 py-0.5 rounded">{{ detailMr.source_branch }}</span>
            <span>→</span>
            <span class="font-mono bg-muted px-1.5 py-0.5 rounded">{{ detailMr.target_branch }}</span>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-x-4 gap-y-1.5 text-sm">
          <div class="text-muted-foreground">作者</div>
          <div>{{ detailMr.author?.name ?? detailMr.author?.username ?? '-' }}</div>
          <div class="text-muted-foreground">创建时间</div>
          <div>{{ formatDate(detailMr.created_at) }}</div>
          <div class="text-muted-foreground">更新时间</div>
          <div>{{ formatDate(detailMr.updated_at) }}</div>
          <div class="text-muted-foreground">合并时间</div>
          <div>{{ detailMr.merged_at ? formatDate(detailMr.merged_at) : '-' }}</div>
          <div class="text-muted-foreground">里程碑</div>
          <div>{{ detailMr.milestone?.title ?? '-' }}</div>
          <div class="text-muted-foreground">审核人</div>
          <div>{{ (detailMr.reviewers ?? []).map(r => r.name ?? r.username).join(', ') || '-' }}</div>
        </div>

        <div v-if="(detailMr.labels ?? []).length > 0" class="flex flex-wrap gap-1.5">
          <span
            v-for="label in detailMr.labels"
            :key="label"
            class="text-xs px-2 py-0.5 rounded-full border border-blue-200 bg-blue-50 text-blue-700"
          >
            {{ label }}
          </span>
        </div>

        <div class="border rounded-lg overflow-hidden">
          <div class="px-3 py-1.5 text-xs font-medium bg-muted/60 text-muted-foreground border-b">描述</div>
          <div
            v-if="detailMr.description"
            class="report-md p-4 overflow-y-auto max-h-[35vh] text-sm leading-relaxed"
            v-html="md.render(detailMr.description)"
          />
          <div v-else class="p-4 text-sm text-muted-foreground">无描述</div>
        </div>
      </template>
    </Dialog>
  </div>
</template>
