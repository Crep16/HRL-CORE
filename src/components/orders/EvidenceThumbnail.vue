<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
const props=defineProps<{orderId:string;id:string;mime:string}>();
const element=ref<HTMLElement>(),url=ref('');
let observer:IntersectionObserver|undefined,disposed=false;
async function load(){
 observer?.disconnect();
 let original='';
 try{
  const data=await invoke<ArrayBuffer|number[]>('read_evidence',{orderId:props.orderId,id:props.id});
  if(disposed)return;
  original=URL.createObjectURL(new Blob([data instanceof ArrayBuffer?data:new Uint8Array(data)],{type:props.mime}));
  const image=new Image();image.src=original;await image.decode();if(disposed)return;
  const canvas=document.createElement('canvas');const scale=Math.min(1,192/Math.max(image.naturalWidth,image.naturalHeight));
  canvas.width=Math.max(1,Math.round(image.naturalWidth*scale));canvas.height=Math.max(1,Math.round(image.naturalHeight*scale));
  canvas.getContext('2d')!.drawImage(image,0,0,canvas.width,canvas.height);
  const blob=await new Promise<Blob|null>(resolve=>canvas.toBlob(resolve,'image/jpeg',.75));
  if(blob&&!disposed)url.value=URL.createObjectURL(blob);
 }catch{/* The filename on the parent button remains available when a thumbnail cannot load. */}
 finally{if(original)URL.revokeObjectURL(original);}
}
onMounted(()=>{observer=new IntersectionObserver(entries=>{if(entries.some(entry=>entry.isIntersecting))void load();});if(element.value)observer.observe(element.value);});
onBeforeUnmount(()=>{disposed=true;observer?.disconnect();if(url.value)URL.revokeObjectURL(url.value);});
</script>
<template><span ref="element" class="evidence-thumb-image" aria-hidden="true"><img v-if="url" :src="url" alt=""><span v-else>Foto</span></span></template>
<style scoped>.evidence-thumb-image{display:grid;place-items:center;width:100%;height:56px;overflow:hidden;background:#e7ecf0;border-radius:4px;font-size:12px;color:#55656f}.evidence-thumb-image img{width:100%;height:100%;object-fit:cover}</style>
