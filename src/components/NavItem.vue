<template>
  <button
    class="w-full py-3 px-2 text-center text-xs rounded transition-colors"
    :class="active ? 'bg-accent text-white' : 'text-gray-400 hover:text-white hover:bg-dark-700'"
    @click="$emit('click')"
  >
    <div class="text-lg mb-1">{{ icon }}</div>
    <div>{{ label }}</div>
  </button>
</template>

<script setup lang="ts">
defineProps<{ icon: string; label: string; active: boolean }>()
defineEmits(['click'])
</script>
