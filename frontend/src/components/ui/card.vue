<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: string
}>()

const cls = computed(() =>
  cn('rounded-xl border bg-card text-card-foreground shadow', props.class)
)
</script>

<template>
  <div :class="cls">
    <slot />
  </div>
</template>
