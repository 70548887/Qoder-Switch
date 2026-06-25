<template>
  <div class="flex gap-4 items-start p-4 bg-dark-700 rounded-lg">
    <span class="w-8 h-8 rounded-full bg-red-600 text-white flex items-center justify-center text-sm font-bold flex-shrink-0">{{ num }}</span>
    <div>
      <h3 class="font-bold text-white">{{ title }}</h3>
      <p class="text-sm text-gray-400 mt-1">{{ desc }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ num: number; title: string; desc: string }>()
</script>
