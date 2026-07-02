<template>
  <button
    class="w-full py-3 px-2 text-center text-xs rounded transition-colors"
    :class="active ? 'bg-blue-600/20 text-blue-400 border-l-2 border-blue-400' : 'text-gray-400 hover:text-white hover:bg-dark-700 border-l-2 border-transparent'"
    @click="$emit('click')"
  >
    <div class="flex justify-center mb-1" v-html="icon"></div>
    <div class="leading-tight">{{ label }}</div>
  </button>
</template>

<script setup lang="ts">
defineProps<{ icon: string; label: string; active: boolean }>()
defineEmits(['click'])
</script>
