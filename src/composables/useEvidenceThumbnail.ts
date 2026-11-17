import { onBeforeUnmount, onMounted, ref } from 'vue';
import { useWorkshopDataStore } from '../stores/useWorkshopDataStore.ts';
import { useTemplateRef } from 'vue';
export function useEvidenceThumbnail(props:{orderId:string;id:string;mime:string}){
const element=useTemplateRef<HTMLElement>('element'),url=ref('');
let observer:IntersectionObserver|undefined,disposed=false;
async function load(){
 observer?.disconnect();
 if(!props.mime.startsWith('image/'))return;
 let original='';
 try{
  const data=await useWorkshopDataStore().readPhoto(props.orderId,props.id);
  if(disposed)return;
  original=URL.createObjectURL(new Blob([data instanceof ArrayBuffer?data:new Uint8Array(data)],{type:props.mime}));
  const image=new Image();image.src=original;await image.decode();if(disposed)return;
  const canvas=document.createElement('canvas');const scale=Math.min(1,192/Math.max(image.naturalWidth,image.naturalHeight));
  canvas.width=Math.max(1,Math.round(image.naturalWidth*scale));canvas.height=Math.max(1,Math.round(image.naturalHeight*scale));
  canvas.getContext('2d')!.drawImage(image,0,0,canvas.width,canvas.height);
  const blob=await new Promise<Blob|null>(resolve=>canvas.toBlob(resolve,'image/jpeg',.75));
  if(blob&&!disposed)url.value=URL.createObjectURL(blob);
 }catch{/* El botón conserva el nombre cuando no carga la miniatura. */}
 finally{if(original)URL.revokeObjectURL(original);}
}
onMounted(()=>{observer=new IntersectionObserver(entries=>{if(entries.some(entry=>entry.isIntersecting))void load();});if(element.value)observer.observe(element.value);});
onBeforeUnmount(()=>{disposed=true;observer?.disconnect();if(url.value)URL.revokeObjectURL(url.value);});
return {url};
}
