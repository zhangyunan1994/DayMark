<script setup lang="ts">
import { provide, computed } from 'vue'

const props = withDefaults(defineProps<{
  defaultValue?: string
  modelValue?: string
}>(), {
  defaultValue: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const activeTab = computed({
  get: () => props.modelValue ?? props.defaultValue,
  set: (v) => emit('update:modelValue', v),
})

provide('tabs-active', activeTab)
provide('tabs-set', (v: string) => emit('update:modelValue', v))
</script>

<template>
  <div>
    <slot />
  </div>
</template>
