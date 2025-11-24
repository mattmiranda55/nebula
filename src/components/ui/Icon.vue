<template>
  <i :class="classes" aria-hidden="true"></i>
</template>

<script setup lang="ts">
import { computed, withDefaults, defineProps } from 'vue'

const props = withDefaults(defineProps<{
  name: string
  size?: string
  className?: string
  spin?: boolean
}>(), {
  size: '',
  className: '',
  spin: false
})

const classes = computed(() => {
  const c: string[] = ['pi', `pi-${props.name}`]
  if (props.spin) c.push('pi-spin')
  if (props.size) c.push(props.size || '')
  if (props.className) c.push(props.className || '')
  return c.join(' ').trim()
})
</script>
<!--

EXAMPLE USAGE:
<Icon name="database" size="text-xl" spin="true" />

-->