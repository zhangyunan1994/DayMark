<script setup lang="ts">
import { computed, inject } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  value: string
  class?: string
}>()

const cls = computed(() =>
  cn(
    'inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50',
    props.class
  )
)

const activeTab = inject<ReturnType<typeof computed>>('tabs-active')!
const setTab = inject<(v: string) => void>('tabs-set')!

const isActive = computed(() => activeTab.value === props.value)
</script>

<template>
  <button
    :class="[
      cls,
      isActive ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground'
    ]"
    role="tab"
    :aria-selected="isActive"
    @click="setTab(value)"
  >
    <slot />
  </button>
</template>
