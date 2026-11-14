<script setup lang="ts">
defineProps<{ name: string }>();
const paths: Record<string, string> = {
  search: 'M18 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0ZM16 16l5 5',
  add: 'M12 5v14M5 12h14',
  edit: 'M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4z',
  save: 'M4 3h13l4 4v14H3V3h1ZM7 3v6h10V3M7 21v-8h10v8',
  clients: 'M5 3h14v18H5zM2 7h4M2 12h4M2 17h4M9 17v-1a3 3 0 0 1 6 0v1M14 9a2 2 0 1 1-4 0 2 2 0 0 1 4 0Z',
  logbook: 'M4 3h16v18H4zM8 3v18M11 8h6M11 12h6M11 16h4',
  company: 'M4 21V3h12v18M16 10h4v11M2 21h20M8 7h4M8 11h4M8 15h4M9 21v-3h2v3',
  users: 'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M22 21v-2a4 4 0 0 0-3-3.87M15 3.13a4 4 0 0 1 0 7.75M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z',
  user: 'M20 21v-2a7 7 0 0 0-14 0v2M17 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z',
  repair: 'M14.5 6.5a5 5 0 0 0-6.3 6.3L3 18a2.12 2.12 0 0 0 3 3l5.2-5.2a5 5 0 0 0 6.3-6.3l-3 3-3-3 3-3Z',
  shield: 'M12 3 4 6v6c0 5 8 9 8 9s8-4 8-9V6l-8-3ZM8 12l3 3 5-6',
  status: 'M9 6h12M9 12h12M9 18h12M3 6l1 1 2-2M3 12l1 1 2-2M3 18l1 1 2-2',
  trash: 'M3 6h18M8 6V3h8v3M5 6l1 15h12l1-15M10 10v7M14 10v7',
  restore: 'M3 10a9 9 0 1 1 1.5 9M3 4v6h6M12 7v5l3 2',
};
</script>

<template>
  <svg class="i app-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false"><path :d="paths[name]" /></svg>
</template>
