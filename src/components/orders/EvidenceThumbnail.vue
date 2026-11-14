<script setup lang="ts">

import { useEvidenceThumbnail } from '../../composables/useEvidenceThumbnail';
const props=defineProps<{orderId:string;id:string;mime:string}>();
const {url}=useEvidenceThumbnail(props);

</script>
<template><span ref="element" class="evidence-thumb-image" aria-hidden="true"><img v-if="url" :src="url" alt=""><span v-else>Foto</span></span></template>
<style scoped>.evidence-thumb-image{display:grid;place-items:center;width:100%;height:56px;overflow:hidden;background:#e7ecf0;border-radius:4px;font-size:12px;color:#55656f}.evidence-thumb-image img{width:100%;height:100%;object-fit:cover}</style>
