<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api, type UserMessage, type AnalyticsSummary } from '@/api'

const dateStart = ref('')
const dateEnd = ref('')
const directory = ref('')
const messages = ref<UserMessage[]>([])
const summary = ref<AnalyticsSummary | null>(null)
const isLoading = ref(false)
const activeTab = ref<'summary' | 'messages'>('summary')

const loadData = async () => {
  isLoading.value = true
  try {
    const [msgs, sum] = await Promise.all([
      api.opencodeMessages({
        date_start: dateStart.value || undefined,
        date_end: dateEnd.value || undefined,
        directory: directory.value || undefined,
        limit: 500,
      }),
      api.opencodeSummary({
        date_start: dateStart.value || undefined,
        date_end: dateEnd.value || undefined,
      }),
    ])
    messages.value = msgs
    summary.value = sum
  } catch (e) {
    console.error('Failed to load opencode data:', e)
  } finally {
    isLoading.value = false
  }
}

const setQuickDate = (days: number) => {
  const end = new Date()
  const start = new Date()
  start.setDate(start.getDate() - days)
  dateStart.value = start.toISOString().split('T')[0]
  dateEnd.value = end.toISOString().split('T')[0]
  loadData()
}

const clearFilters = () => {
  dateStart.value = ''
  dateEnd.value = ''
  directory.value = ''
  loadData()
}

const truncateText = (text: string, maxLen = 200) => {
  if (text.length <= maxLen) return text
  return text.substring(0, maxLen) + '...'
}

const formatDate = (dateStr: string) => {
  return dateStr.replace('T', ' ').substring(0, 19)
}

const formatDirectory = (dir: string) => {
  const parts = dir.split('/')
  return parts[parts.length - 1] || dir
}

onMounted(() => {
  loadData()
})
</script>

<template>
  <div class="flex-1 overflow-auto p-6">
    <div class="max-w-7xl mx-auto">
      <div class="mb-6">
        <h1 class="text-2xl font-bold text-gray-900 mb-2">OpenCode 用户记录分析</h1>
        <p class="text-gray-500">分析 opencode 使用情况，查看用户交互记录</p>
      </div>

      <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4 mb-6">
        <div class="flex flex-wrap gap-4 items-end">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">开始日期</label>
            <input
              v-model="dateStart"
              type="date"
              class="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">结束日期</label>
            <input
              v-model="dateEnd"
              type="date"
              class="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">目录筛选</label>
            <input
              v-model="directory"
              type="text"
              placeholder="输入目录关键词"
              class="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 w-48"
            />
          </div>
          <button
            @click="loadData"
            :disabled="isLoading"
            class="bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
          >
            {{ isLoading ? '加载中...' : '查询' }}
          </button>
          <button
            @click="clearFilters"
            class="bg-gray-100 text-gray-700 px-4 py-2 rounded-md text-sm font-medium hover:bg-gray-200"
          >
            清除筛选
          </button>
        </div>
        <div class="flex gap-2 mt-3">
          <button @click="setQuickDate(7)" class="text-xs text-blue-600 hover:underline">近7天</button>
          <button @click="setQuickDate(30)" class="text-xs text-blue-600 hover:underline">近30天</button>
          <button @click="setQuickDate(90)" class="text-xs text-blue-600 hover:underline">近90天</button>
          <button @click="setQuickDate(365)" class="text-xs text-blue-600 hover:underline">近一年</button>
        </div>
      </div>

      <div v-if="summary" class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
          <div class="text-sm text-gray-500">总消息数</div>
          <div class="text-2xl font-bold text-gray-900">{{ summary.total_messages }}</div>
        </div>
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
          <div class="text-sm text-gray-500">会话数</div>
          <div class="text-2xl font-bold text-gray-900">{{ summary.total_sessions }}</div>
        </div>
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
          <div class="text-sm text-gray-500">项目数</div>
          <div class="text-2xl font-bold text-gray-900">{{ summary.total_directories }}</div>
        </div>
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
          <div class="text-sm text-gray-500">时间范围</div>
          <div class="text-sm font-medium text-gray-900">
            <template v-if="summary.date_range">
              {{ summary.date_range.earliest }} ~ {{ summary.date_range.latest }}
            </template>
            <span v-else class="text-gray-400">无数据</span>
          </div>
        </div>
      </div>

      <div class="bg-white rounded-lg shadow-sm border border-gray-200">
        <div class="border-b border-gray-200">
          <nav class="flex">
            <button
              @click="activeTab = 'summary'"
              :class="[
                'px-4 py-3 text-sm font-medium border-b-2',
                activeTab === 'summary'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              ]"
            >
              每日统计
            </button>
            <button
              @click="activeTab = 'messages'"
              :class="[
                'px-4 py-3 text-sm font-medium border-b-2',
                activeTab === 'messages'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              ]"
            >
              消息记录 ({{ messages.length }})
            </button>
          </nav>
        </div>

        <div v-if="activeTab === 'summary'" class="p-4">
          <div v-if="summary && summary.daily_summaries.length > 0">
            <div class="overflow-x-auto">
              <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                  <tr>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">日期</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">消息数</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">会话数</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">项目目录</th>
                  </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                  <tr v-for="day in summary.daily_summaries" :key="day.date" class="hover:bg-gray-50">
                    <td class="px-4 py-3 text-sm font-medium text-gray-900">{{ day.date }}</td>
                    <td class="px-4 py-3 text-sm text-gray-900">{{ day.message_count }}</td>
                    <td class="px-4 py-3 text-sm text-gray-900">{{ day.session_count }}</td>
                    <td class="px-4 py-3 text-sm text-gray-500">
                      <div class="flex flex-wrap gap-1">
                        <span
                          v-for="dir in day.directories"
                          :key="dir"
                          class="inline-block bg-gray-100 rounded px-2 py-0.5 text-xs"
                          :title="dir"
                        >
                          {{ formatDirectory(dir) }}
                        </span>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
          <div v-else class="text-center py-8 text-gray-500">
            暂无数据
          </div>
        </div>

        <div v-if="activeTab === 'messages'" class="p-4">
          <div v-if="messages.length > 0" class="space-y-3">
            <div
              v-for="(msg, idx) in messages"
              :key="idx"
              class="border border-gray-200 rounded-lg p-4 hover:bg-gray-50"
            >
              <div class="flex items-start justify-between mb-2">
                <div class="flex items-center gap-2">
                  <span class="text-xs text-gray-400">{{ formatDate(msg.time_created) }}</span>
                  <span class="text-xs bg-blue-100 text-blue-700 rounded px-2 py-0.5" :title="msg.directory">
                    {{ formatDirectory(msg.directory) }}
                  </span>
                </div>
                <span class="text-xs text-gray-400">会话: {{ msg.session_id.substring(0, 8) }}...</span>
              </div>
              <div class="text-sm font-medium text-gray-900 mb-1">{{ msg.title }}</div>
              <div class="text-sm text-gray-600 whitespace-pre-wrap">{{ truncateText(msg.user_text) }}</div>
            </div>
          </div>
          <div v-else class="text-center py-8 text-gray-500">
            暂无消息记录
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
