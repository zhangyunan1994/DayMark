<script setup lang="ts">
import { computed, inject } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  value: string
  class?: string
}>()

const cls = computed(() =>
  cn(
    'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
    props.class
  )
)

const activeTab = inject<ReturnType<typeof computed>>('tabs-active')!
const isActive = computed(() => activeTab.value === props.value)
</script>

<template>
  <div v-if="isActive" :class="cls" role="tabpanel">
    <slot />
  </div>
</template>
