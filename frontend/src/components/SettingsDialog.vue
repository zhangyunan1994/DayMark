<script setup lang="ts">
import { ref, watch } from 'vue'
import Dialog from '@/components/ui/dialog.vue'
import DialogHeader from '@/components/ui/dialog-header.vue'
import DialogTitle from '@/components/ui/dialog-title.vue'
import DialogFooter from '@/components/ui/dialog-footer.vue'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Tabs from '@/components/ui/tabs.vue'
import TabsList from '@/components/ui/tabs-list.vue'
import TabsTrigger from '@/components/ui/tabs-trigger.vue'
import TabsContent from '@/components/ui/tabs-content.vue'
import { api } from '@/api'
import type { LLMConfig, Settings } from '@/api'

const props = defineProps<{
  open: boolean
  settings: Settings | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  save: [settings: Settings]
}>()

const PRESETS: { provider_name: string; api_base_url: string; model: string }[] = [
  { provider_name: 'DeepSeek', api_base_url: 'https://api.deepseek.com', model: 'deepseek-chat' },
  { provider_name: 'OpenAI', api_base_url: 'https://api.openai.com/v1', model: 'gpt-4o' },
  { provider_name: '通义千问', api_base_url: 'https://dashscope.aliyuncs.com/compatible-mode/v1', model: 'qwen-plus' },
  { provider_name: 'Moonshot', api_base_url: 'https://api.moonshot.cn/v1', model: 'moonshot-v1-8k' },
]

const emptyLlm: LLMConfig = {
  provider_name: '',
  api_base_url: '',
  api_key: '',
  model: '',
  temperature: 0.7,
  max_tokens: 2000,
  is_active: false,
}

const maskKey = (key: string) => (key.length > 8 ? `${key.slice(0, 8)}••••••••` : '••••••••')

const draft = ref<Settings | null>(null)
const editing = ref<LLMConfig | null>(null)
const showForm = ref(false)
const form = ref<LLMConfig>({ ...emptyLlm })
const message = ref<{ ok: boolean; text: string } | null>(null)
const testing = ref(false)
const activeTab = ref('llm')
const gitlabDraft = ref<{ base_url: string; username: string; token: string }>({
  base_url: 'https://gitlab.com',
  username: '',
  token: '',
})

watch(
  () => props.open,
  (val) => {
    if (val) {
      const next = props.settings ? JSON.parse(JSON.stringify(props.settings)) : null
      draft.value = next
      gitlabDraft.value = {
        base_url: next?.gitlab?.base_url || 'https://gitlab.com',
        username: next?.gitlab?.username || '',
        token: next?.gitlab?.token || '',
      }
      showForm.value = false
      editing.value = null
      message.value = null
    }
  }
)

const persist = async (next: Settings) => {
  await emit('save', next)
  draft.value = next
}

const testLlm = async (config: LLMConfig) => {
  if (!config.api_base_url || !config.model) return
  testing.value = true
  message.value = null
  try {
    const result = await api.testLlm({ ...config, is_active: false })
    message.value = { ok: result.ok, text: result.message }
  } catch (e) {
    message.value = { ok: false, text: (e as Error).message }
  } finally {
    testing.value = false
  }
}

const testGitlab = async () => {
  if (!gitlabDraft.value.base_url || !gitlabDraft.value.token) return
  testing.value = true
  message.value = null
  try {
    const result = await api.testGitlab(gitlabDraft.value)
    message.value = { ok: result.ok, text: result.message }
  } catch (e) {
    message.value = { ok: false, text: (e as Error).message }
  } finally {
    testing.value = false
  }
}

const saveGitlab = async () => {
  if (!draft.value) return
  await persist({ ...draft.value, gitlab: { ...draft.value.gitlab, ...gitlabDraft.value } })
  message.value = { ok: true, text: 'GitLab 配置已保存' }
}

const openAdd = (preset?: { provider_name: string; api_base_url: string; model: string }) => {
  editing.value = null
  form.value = preset
    ? { ...emptyLlm, ...preset, is_active: draft.value?.llm_configs.length === 0 }
    : { ...emptyLlm, is_active: draft.value?.llm_configs.length === 0 }
  showForm.value = true
  message.value = null
}

const openEdit = (config: LLMConfig) => {
  editing.value = config
  form.value = { ...config }
  showForm.value = true
  message.value = null
}

const saveForm = async () => {
  if (!form.value.provider_name.trim() || !form.value.api_base_url.trim() || !form.value.model.trim()) {
    message.value = { ok: false, text: '名称、URL、模型为必填项' }
    return
  }
  if (!draft.value) return
  const llmConfigs = [...draft.value.llm_configs]
  if (editing.value) {
    const idx = llmConfigs.findIndex((c) => c.id === editing.value!.id)
    if (idx >= 0) llmConfigs[idx] = { ...form.value, id: editing.value.id }
  } else {
    llmConfigs.push({ ...form.value, id: crypto.randomUUID() })
  }
  await persist({ ...draft.value, llm_configs: llmConfigs })
  showForm.value = false
  message.value = null
}

const setActive = async (id: string) => {
  if (!draft.value) return
  await persist({
    ...draft.value,
    llm_configs: draft.value.llm_configs.map((c) => ({ ...c, is_active: c.id === id })),
  })
}

const removeConfig = async (id: string) => {
  if (!draft.value) return
  await persist({ ...draft.value, llm_configs: draft.value.llm_configs.filter((c) => c.id !== id) })
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)" class="sm:max-w-2xl max-h-[85vh] overflow-y-auto">
    <DialogHeader>
      <DialogTitle>设置</DialogTitle>
    </DialogHeader>
    <Tabs v-model="activeTab" default-value="llm">
      <TabsList class="mb-4">
        <TabsTrigger value="llm">LLM 配置</TabsTrigger>
        <TabsTrigger value="gitlab">GitLab 配置</TabsTrigger>
      </TabsList>

      <TabsContent value="llm" class="space-y-4">
        <div class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">配置 OpenAI API 兼容厂商，用于 AI 生成报告</p>
          <Button size="sm" @click="openAdd()">添加配置</Button>
        </div>

        <div class="flex flex-wrap gap-2">
          <Button v-for="p in PRESETS" :key="p.provider_name" variant="outline" size="sm" @click="openAdd(p)">
            {{ p.provider_name }}
          </Button>
        </div>

        <div v-if="showForm" class="space-y-3 rounded-lg border p-4">
          <div class="grid grid-cols-2 gap-3">
            <div>
              <Label>名称</Label>
              <Input v-model="form.provider_name" placeholder="例如：DeepSeek" />
            </div>
            <div>
              <Label>模型</Label>
              <Input v-model="form.model" placeholder="deepseek-chat" />
            </div>
            <div class="col-span-2">
              <Label>API 地址</Label>
              <Input v-model="form.api_base_url" placeholder="https://api.example.com/v1" />
            </div>
            <div class="col-span-2">
              <Label>API Key</Label>
              <Input type="password" v-model="form.api_key" placeholder="sk-..." />
            </div>
            <div>
              <Label>温度</Label>
              <Input
                type="number"
                :model-value="String(form.temperature)"
                @update:model-value="form.temperature = Number($event)"
                min="0"
                max="2"
                step="0.1"
              />
            </div>
            <div>
              <Label>Max Tokens</Label>
              <Input
                type="number"
                :model-value="String(form.max_tokens)"
                @update:model-value="form.max_tokens = Number($event)"
                min="100"
              />
            </div>
          </div>
          <div class="flex justify-end gap-2">
            <Button variant="outline" size="sm" :disabled="testing" @click="testLlm(form)">
              {{ testing ? '测试中...' : '测试连接' }}
            </Button>
            <Button variant="ghost" size="sm" @click="showForm = false">取消</Button>
            <Button size="sm" @click="saveForm">
              {{ editing ? '保存修改' : '保存' }}
            </Button>
          </div>
        </div>

        <p v-if="draft && draft.llm_configs.length === 0" class="text-sm text-muted-foreground text-center py-6">
          暂无 LLM 配置，请添加一个
        </p>
        <div v-else class="space-y-2">
          <div
            v-for="c in draft?.llm_configs"
            :key="c.id"
            class="rounded-lg border p-3 flex items-center justify-between gap-4"
            :class="c.is_active ? 'border-primary bg-accent/50' : ''"
          >
            <div class="min-w-0">
              <div class="flex items-center gap-2">
                <span class="font-medium text-sm">{{ c.provider_name }}</span>
                <span v-if="c.is_active" class="text-[10px] px-2 py-0.5 rounded-full bg-primary text-primary-foreground">
                  当前使用
                </span>
              </div>
              <div class="text-xs text-muted-foreground mt-0.5 space-y-0.5">
                <div class="truncate">API：{{ c.api_base_url }}</div>
                <div>模型：{{ c.model }} · Key：{{ maskKey(c.api_key) }}</div>
              </div>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <Button v-if="!c.is_active" variant="ghost" size="sm" @click="c.id && setActive(c.id)">设为当前</Button>
              <Button variant="ghost" size="sm" @click="openEdit(c)">编辑</Button>
              <Button
                variant="ghost"
                size="sm"
                class="text-destructive hover:text-destructive"
                @click="c.id && removeConfig(c.id)"
              >
                删除
              </Button>
            </div>
          </div>
        </div>
      </TabsContent>

      <TabsContent value="gitlab" class="space-y-4">
        <div class="space-y-3">
          <div>
            <Label>实例地址</Label>
            <Input v-model="gitlabDraft.base_url" placeholder="https://gitlab.com" />
          </div>
          <div>
            <Label>用户名</Label>
            <Input v-model="gitlabDraft.username" placeholder="你的 GitLab 用户名" />
          </div>
          <div>
            <Label>Personal Access Token</Label>
            <Input type="password" v-model="gitlabDraft.token" placeholder="glpat-..." />
            <p class="text-xs text-muted-foreground mt-1">
              需要 read_api 权限。在 GitLab → Settings → Access Tokens 中创建
            </p>
          </div>
          <div class="flex justify-end gap-2">
            <Button variant="outline" size="sm" :disabled="testing" @click="testGitlab">
              {{ testing ? '测试中...' : '测试连接' }}
            </Button>
            <Button size="sm" @click="saveGitlab">保存 GitLab 配置</Button>
          </div>
        </div>
      </TabsContent>
    </Tabs>

    <div
      v-if="message"
      class="text-sm px-3 py-2 rounded-md border"
      :class="message.ok
        ? 'text-emerald-700 border-emerald-200 bg-emerald-50'
        : 'text-red-700 border-red-200 bg-red-50'"
    >
      {{ message.text }}
    </div>

    <DialogFooter>
      <Button @click="emit('update:open', false)">完成</Button>
    </DialogFooter>
  </Dialog>
</template>
