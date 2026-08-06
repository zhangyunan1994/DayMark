<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: string
}>()

const cls = computed(() =>
  cn('text-lg font-semibold leading-none tracking-tight', props.class)
)
</script>

<template>
  <h2 :class="cls">
    <slot />
  </h2>
</template>
