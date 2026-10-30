<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAccess } from '../../composables/useAccess';
import EvidenceCamera from './EvidenceCamera.vue';
import EvidenceThumbnail from './EvidenceThumbnail.vue';
type Photo={id:string;orderId:string;originalName:string;mime:string;sizeBytes:number;source:'camera'|'file';note:string;authorName:string;createdAt:string;deleted:boolean;revision:number};
type Upload={id:string;file:File;source:'camera'|'file';note:string};
const props=defineProps<{orderId:string;disabled?:boolean;trashed?:boolean}>();
const {can,requestAccess,user,temporary}=useAccess();
const photos=ref<Photo[]>([]),selectedId=ref(''),description=ref(''),message=ref(''),error=ref(''),cameraError=ref('');
const loading=ref(false),busy=ref(false),loaded=ref(false),imageLoading=ref(false),imageError=ref('');
const gallery=ref<HTMLDialogElement>(),files=ref<HTMLInputElement>(),camera=ref<InstanceType<typeof EvidenceCamera>>();
const opener=ref<HTMLButtonElement>();
const imageUrl=ref(''),removed=ref<Photo>(),discardNote=ref(false);
const pending=ref<Upload[]>([]);
const showRemoved=ref(false);
const selected=computed(()=>photos.value.find(p=>p.id===selectedId.value));
const position=computed(()=>photos.value.findIndex(p=>p.id===selectedId.value));
const dirty=computed(()=>!!selected.value&&description.value!==selected.value.note);
let disposed=false,imageVersion=0;
const size=(n:number)=>n<1048576?`${Math.round(n/1024)} KB`:`${(n/1048576).toFixed(1)} MB`;
const reason=(cause:unknown)=>cause instanceof Error?cause.message:String(cause);
async function allowed(operation:string){return await requestAccess(operation,'clients')&&!disposed;}
function releaseImage(){imageVersion++;if(imageUrl.value)URL.revokeObjectURL(imageUrl.value);imageUrl.value='';imageError.value='';}
async function loadImage(){
 releaseImage();const photo=selected.value;if(!photo){imageLoading.value=false;return;}
 const version=imageVersion;imageLoading.value=true;
 try{const data=await invoke<ArrayBuffer|number[]>('read_evidence',{orderId:props.orderId,id:photo.id});
  if(disposed||imageVersion!==version)return;
  imageUrl.value=URL.createObjectURL(new Blob([data instanceof ArrayBuffer?data:new Uint8Array(data)],{type:photo.mime}));
 }catch(cause){if(!disposed&&imageVersion===version)imageError.value=reason(cause);}
 finally{if(imageVersion===version)imageLoading.value=false;}
}
watch(selectedId,()=>{description.value=selected.value?.note??'';if(gallery.value?.open)void loadImage();});
watch(()=>user.value?.id,()=>{photos.value=[];loaded.value=false;releaseImage();gallery.value?.close();});
watch(()=>temporary.value?.id,()=>{if(!can('evidence.view')){photos.value=[];loaded.value=false;releaseImage();gallery.value?.close();}});
async function refresh(){
 loading.value=true;error.value='';
 try{const result=await invoke<Photo[]>('list_evidence',{orderId:props.orderId,deleted:showRemoved.value});if(disposed||!can('evidence.view'))return;
  photos.value=result;loaded.value=true;if(!result.some(p=>p.id===selectedId.value))selectedId.value=result[0]?.id??'';
  description.value=selected.value?.note??'';
 }catch(cause){if(!disposed){error.value=reason(cause);photos.value=[];selectedId.value='';loaded.value=false;releaseImage();}}
 finally{loading.value=false;}
}
async function open(){
 if(props.disabled||!await allowed('evidence.view'))return;
 discardNote.value=false;gallery.value?.showModal();await refresh();await loadImage();
}
async function toggleRemoved(){
 if(busy.value)return;if(dirty.value&&!await saveNote())return;
 if(!showRemoved.value&&!await allowed('evidence.delete'))return;
 showRemoved.value=!showRemoved.value;removed.value=undefined;await refresh();await loadImage();
}
function close(){if(busy.value)return;if(dirty.value){discardNote.value=true;return;}gallery.value?.close();releaseImage();opener.value?.focus();}
function discard(){description.value=selected.value?.note??'';discardNote.value=false;close();}
async function saveNote(){
 const photo=selected.value;if(!photo||busy.value||!await allowed('evidence.edit'))return false;
 busy.value=true;error.value='';
 try{const updated=await invoke<Photo>('update_evidence',{orderId:props.orderId,id:photo.id,note:description.value,deleted:false,expectedRevision:photo.revision});
  photos.value=photos.value.map(p=>p.id===updated.id?updated:p);message.value='Descripción guardada.';return true;
 }catch(cause){error.value=reason(cause);return false;}finally{busy.value=false;}
}
async function select(photo:Photo){if(busy.value)return;if(dirty.value&&!await saveNote())return;selectedId.value=photo.id;}
async function navigate(delta:number){const photo=photos.value[position.value+delta];if(photo)await select(photo);}
async function pick(){
 if(props.disabled||props.trashed||busy.value||!await allowed('evidence.add'))return;
 if(dirty.value&&!await saveNote())return;
 if(showRemoved.value){showRemoved.value=false;await refresh();}
 if(!gallery.value?.open){await open();if(!gallery.value?.open)return;}
 files.value?.click();
}
async function openCamera(){
 if(props.disabled||props.trashed||busy.value||!await allowed('evidence.add'))return;
 if(dirty.value&&!await saveNote())return;
 if(showRemoved.value){showRemoved.value=false;await refresh();}
 if(!gallery.value?.open){await open();if(!gallery.value?.open)return;}
 cameraError.value='';await camera.value?.open();
}
async function checkPhoto(file:File){
 if(!['image/jpeg','image/png','image/webp'].includes(file.type))throw new Error('Selecciona una foto JPG, PNG o WebP.');
 if(!file.size||file.size>20*1024*1024)throw new Error('La foto debe ocupar como máximo 20 MB.');
 const url=URL.createObjectURL(file);try{const image=new Image();image.src=url;await image.decode();if(!image.naturalWidth)throw new Error();}
 catch{throw new Error('No se pudo abrir la imagen. Prueba con otra foto.');}finally{URL.revokeObjectURL(url);}
}
function newUpload(file:File,source:'camera'|'file',note:string):Upload{return{id:crypto.randomUUID().replace(/-/g,''),file,source,note};}
async function upload(item:Upload){
 await checkPhoto(item.file);
 const bytes=Array.from(new Uint8Array(await item.file.arrayBuffer()));
 return await invoke<Photo>('add_evidence',{input:{id:item.id,orderId:props.orderId,originalName:item.file.name,mime:item.file.type,source:item.source,note:item.note,bytes}});
}
function include(photo:Photo){if(disposed)return;if(!photo.deleted){photos.value=[...photos.value.filter(p=>p.id!==photo.id),photo];selectedId.value=photo.id;loaded.value=true;}message.value='Foto guardada en este equipo.';}
async function importFiles(event:Event){
 const input=event.target as HTMLInputElement;const chosen=Array.from(input.files??[]);input.value='';
 if(!chosen.length||busy.value)return;
 if(dirty.value&&!await saveNote())return;
 busy.value=true;error.value='';message.value='Guardando fotografías…';
 const errors:string[]=[];
 for(const file of chosen){
  const item=newUpload(file,'file','');
  try{include(await upload(item));}
  catch(cause){pending.value.push(item);errors.push(`${file.name}: ${reason(cause)}`);}
 }
 busy.value=false;message.value=errors.length?'Hay fotos pendientes de guardar. Puedes reintentarlas.':'Fotografías guardadas en este equipo.';
 error.value=errors.join('\n');await loadImage();
}
async function retry(){
 if(busy.value||!await allowed('evidence.add'))return;busy.value=true;error.value='';const errors:string[]=[];
 for(const item of [...pending.value]){try{include(await upload(item));pending.value=pending.value.filter(p=>p.id!==item.id);}catch(cause){errors.push(`${item.file.name}: ${reason(cause)}`);}}
 busy.value=false;error.value=errors.join('\n');await loadImage();
}
let cameraUpload:Upload|undefined;
async function capture(file:File,note:string){
 if(busy.value)return;if(dirty.value&&!await saveNote())return;
 busy.value=true;cameraError.value='';
 if(cameraUpload?.file!==file)cameraUpload=newUpload(file,'camera',note);
 try{include(await upload(cameraUpload));cameraUpload=undefined;camera.value?.saved();await loadImage();}
 catch(cause){cameraError.value=reason(cause);}finally{busy.value=false;}
}
async function remove(restore=false){
 const photo=restore?(showRemoved.value?selected.value:removed.value):selected.value;if(!photo||busy.value||!await allowed('evidence.delete'))return;
 if(!restore&&dirty.value&&!await saveNote())return;
 const current=restore?photo:selected.value!;
 busy.value=true;error.value='';
 try{const updated=await invoke<Photo>('update_evidence',{orderId:props.orderId,id:current.id,note:current.note,deleted:!restore,expectedRevision:current.revision});
  if(restore){if(showRemoved.value){photos.value=photos.value.filter(p=>p.id!==updated.id);selectedId.value=photos.value[0]?.id??'';}else{photos.value.push(updated);selectedId.value=updated.id;}removed.value=undefined;message.value='Foto restaurada.';}
  else{removed.value=updated;photos.value=photos.value.filter(p=>p.id!==updated.id);selectedId.value=photos.value[0]?.id??'';message.value='Foto retirada. El archivo y su historial se conservan.';}
  await loadImage();
 }catch(cause){error.value=reason(cause);}finally{busy.value=false;}
}
onBeforeUnmount(()=>{disposed=true;releaseImage();gallery.value?.close();});
</script>
<template>
 <section class="ev-entry" aria-label="Evidencias de la orden">
  <div class="ev-entry-title"><h3>Evidencias</h3><span v-if="loaded">{{ photos.length }} {{ photos.length===1?'foto':'fotos' }}</span></div>
  <p v-if="disabled" class="help">Guarda o termina de editar la orden para gestionar sus fotografías.</p>
  <p v-else class="help">Fotos de recepción, fallas y reparación, guardadas en este equipo.</p>
  <button ref="opener" class="btn btn-s" :disabled="disabled" @click="open">Abrir galería</button>
  <div v-if="!trashed" class="ev-actions"><button class="btn btn-p" :disabled="disabled" @click="openCamera">Tomar foto</button><button class="btn btn-s" :disabled="disabled" @click="pick">Seleccionar archivos</button></div>
  <input ref="files" type="file" accept="image/jpeg,image/png,image/webp" multiple hidden @change="importFiles">
  <Teleport to="body">
  <dialog ref="gallery" class="evidence-gallery" aria-labelledby="evidence-gallery-title" @cancel.prevent="close">
   <header class="ev-head"><div><h2 id="evidence-gallery-title">Evidencias de la orden</h2><p class="help">{{ orderId }} · {{ trashed?'Orden en papelera':'Almacenamiento en este equipo' }}</p></div><button class="ev-close" aria-label="Cerrar galería" :disabled="busy" @click="close">×</button></header>
   <div v-if="discardNote" class="ev-discard" role="alert"><h3>Hay una descripción sin guardar</h3><div class="ev-actions"><button class="btn btn-s" @click="discardNote=false">Seguir editando</button><button class="btn btn-s" @click="discard">Descartar cambios</button><button class="btn btn-p" @click="saveNote().then(saved=>{if(saved){discardNote=false;close();}})">Guardar y cerrar</button></div></div>
   <template v-else>
   <div class="ev-toolbar"><div v-if="!trashed" class="ev-actions"><button class="btn btn-p" :disabled="busy||loading" @click="openCamera">Tomar foto</button><button class="btn btn-s" :disabled="busy||loading" @click="pick">Seleccionar archivos</button></div><div class="ev-actions"><button class="btn btn-s" :disabled="busy||loading" @click="toggleRemoved">{{ showRemoved?'Volver a fotos activas':'Ver fotos retiradas' }}</button><button class="btn btn-s" :disabled="busy||loading||dirty" @click="refresh().then(loadImage)">Actualizar galería</button></div><span class="help">JPG, PNG o WebP · Hasta 20 MB por foto</span></div>
   <p v-if="error" class="ev-error ev-pad" role="alert">{{ error }}</p>
   <div v-if="pending.length" class="ev-pad ev-actions"><button class="btn btn-s" :disabled="busy" @click="retry">Reintentar {{ pending.length }} {{ pending.length===1?'foto pendiente':'fotos pendientes' }}</button><button class="btn btn-s" :disabled="busy" @click="pending=[];error='';message='Solicitudes pendientes descartadas. Las fotos que ya se hubieran guardado siguen en la galería.'">Descartar pendientes</button></div>
   <div class="ev-grid"><div class="ev-viewer">
    <div class="ev-stage" tabindex="0" aria-label="Vista de evidencia. Usa flechas izquierda y derecha para cambiar de foto." @keydown.left.prevent="navigate(-1)" @keydown.right.prevent="navigate(1)">
     <p v-if="loading||imageLoading" role="status">Cargando fotografías…</p>
     <div v-else-if="imageError" class="ev-empty"><h3>No se pudo abrir esta foto</h3><p>{{ imageError }}</p><button class="btn btn-s" @click="loadImage">Volver a consultar</button></div>
     <img v-else-if="imageUrl" :src="imageUrl" :alt="selected?.note||`Evidencia de ${orderId}`" @error="imageError='El archivo no se pudo mostrar como imagen. Revisa el respaldo.'">
     <div v-else class="ev-empty"><h3>{{ error?'Evidencias no disponibles':showRemoved?'No hay fotos retiradas':'Aún no hay fotos' }}</h3><p>{{ error?'Vuelve a consultar la galería.':showRemoved?'Las fotos retiradas se conservan aquí para poder restaurarlas.':'Toma una foto con la cámara web o selecciona las que ya pasaste del celular a tu computadora.' }}</p><button v-if="error" class="btn btn-s" @click="refresh().then(loadImage)">Volver a consultar</button></div>
    </div>
    <div class="ev-navigation"><span class="ev-filename">{{ selected?.originalName||'Sin fotografía seleccionada' }}</span><div class="ev-actions"><button class="ev-nav" aria-label="Foto anterior" :disabled="busy||position<=0" @click="navigate(-1)">‹</button><span class="help">{{ selected?`${position+1} de ${photos.length}`:'0 fotos' }}</span><button class="ev-nav" aria-label="Foto siguiente" :disabled="busy||position>=photos.length-1" @click="navigate(1)">›</button></div></div>
    <div class="ev-thumbs" role="group" aria-label="Seleccionar fotografía"><button v-for="(photo,i) in photos" :key="photo.id" :aria-current="selectedId===photo.id?'true':undefined" :aria-label="`Ver foto ${i+1}: ${photo.note||photo.originalName}`" :disabled="busy" @click="select(photo)"><EvidenceThumbnail :order-id="orderId" :id="photo.id" :mime="photo.mime"/><span>{{ i+1 }} · {{ photo.originalName }}</span></button></div>
   </div><aside class="ev-detail"><h3>Detalle de la foto</h3><dl v-if="selected"><dt>Origen</dt><dd>{{ selected.source==='camera'?'Cámara web':'Archivo de la computadora' }}</dd><dt>Agregada por</dt><dd>{{ selected.authorName }}</dd><dt>Fecha</dt><dd>{{ new Date(selected.createdAt).toLocaleString('es-MX') }}</dd><dt>Tamaño</dt><dd>{{ size(selected.sizeBytes) }}</dd></dl>
    <label for="evidence-description">Descripción (opcional)</label><textarea id="evidence-description" v-model="description" rows="4" maxlength="500" :disabled="!selected||busy||trashed||showRemoved" placeholder="Describe lo que muestra esta fotografía."></textarea>
    <div class="ev-actions ev-detail-actions"><template v-if="!showRemoved"><button class="btn btn-p" :disabled="busy||!dirty||trashed" @click="saveNote">Guardar descripción</button><button class="btn btn-s" :disabled="busy||!selected||trashed" @click="remove(false)">Retirar esta foto</button><button v-if="removed&&!trashed" class="btn btn-s" :disabled="busy" @click="remove(true)">Deshacer retiro</button></template><button v-else class="btn btn-p" :disabled="busy||!selected||trashed" @click="remove(true)">Restaurar esta foto</button></div>
   </aside></div>
   <footer class="ev-foot"><p class="help" role="status">{{ busy?'Guardando en este equipo…':message||'Las fotos se conservan al cerrar y volver a abrir HRL.' }}</p><button class="btn btn-s" :disabled="busy" @click="close">Cerrar</button></footer>
   </template>
  </dialog>
  <EvidenceCamera ref="camera" :order-id="orderId" :busy="busy" :error="cameraError" @capture="capture" @closed="cameraUpload=undefined;cameraError=''" @files="pick" />
  </Teleport>
 </section>
</template>
<style scoped>
.ev-entry{display:grid;gap:12px}.ev-entry-title{display:flex;justify-content:space-between;gap:12px;align-items:center}.ev-entry-title h3{font:600 17px var(--disp);margin:0}.ev-entry-title span{font-size:13px;color:var(--muted)}.ev-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.ev-entry>.ev-actions{display:grid;grid-template-columns:1fr}.evidence-gallery{width:min(1050px,calc(100% - 32px));max-height:calc(100dvh - 32px);padding:0;border:1px solid #d5dde4;border-radius:16px;background:white;color:var(--ink);font:16px/1.5 var(--body);box-shadow:0 24px 64px #0f243040}.evidence-gallery::backdrop{background:#0f2430a6}.ev-head{display:flex;align-items:center;justify-content:space-between;gap:20px;padding:20px 24px;border-bottom:1px solid var(--line)}.ev-head h2{font:700 25px/1.2 var(--disp);margin:0 0 5px}.ev-close,.ev-nav{display:inline-grid;place-items:center;min-width:44px;min-height:44px;border:1px solid #73838f;border-radius:8px;background:white;color:var(--ink);font-size:26px;cursor:pointer}.ev-toolbar{padding:18px 24px;display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap}.ev-grid{display:grid;grid-template-columns:minmax(0,1fr) 270px}.ev-viewer{min-width:0;padding:0 24px 24px}.ev-stage{height:clamp(230px,36vh,390px);background:#0f2430;border-radius:10px;display:flex;align-items:center;justify-content:center;color:white;overflow:hidden}.ev-stage img{width:100%;height:100%;object-fit:contain}.ev-empty{max-width:410px;text-align:center;padding:24px}.ev-empty h3{font:600 21px var(--disp);margin:0 0 12px}.ev-empty p{font-size:14px;color:#c5d6e2;line-height:1.5;margin:0 0 14px}.ev-navigation{display:flex;justify-content:space-between;align-items:center;gap:12px;margin-top:14px}.ev-filename{font-size:14px;font-weight:600;overflow-wrap:anywhere;min-width:0}.ev-navigation>.ev-actions{flex:none}.ev-thumbs{display:flex;gap:10px;overflow-x:auto;padding:6px 3px 12px;margin-top:12px}.ev-thumbs button{display:flex;flex-direction:column;justify-content:center;gap:4px;flex:none;width:112px;min-height:64px;padding:8px;border:1px solid #73838f;border-radius:8px;background:var(--tint);font:14px var(--body);color:var(--ink);cursor:pointer}.ev-thumbs button span:last-child{max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ev-thumbs [aria-current=true]{outline:2px solid var(--acc);outline-offset:1px;border-color:var(--acc)}.ev-detail{min-width:0;padding:0 24px 24px 0}.ev-detail h3{font:600 19px var(--disp);margin:0 0 16px}.ev-detail dl{margin:0 0 20px}.ev-detail dt{font-size:13px;color:var(--muted);margin-top:10px}.ev-detail dd{font-size:14px;font-weight:600;margin:2px 0 0;overflow-wrap:anywhere}.ev-detail label{font-size:14px;font-weight:600}.ev-detail textarea{width:100%;margin-top:6px;padding:10px 12px;border:1px solid #73838f;border-radius:8px;font:16px var(--body);color:var(--ink);background:white;resize:vertical}.ev-detail-actions{margin-top:12px;display:grid;grid-template-columns:1fr}.ev-foot{border-top:1px solid var(--line);padding:16px 24px;background:var(--tint);display:flex;align-items:center;justify-content:space-between;gap:14px}.ev-pad{padding:0 24px 16px}.ev-discard{padding:24px;display:grid;gap:18px}.ev-error{color:#a32828;font-size:14px;white-space:pre-line}.evidence-gallery :focus-visible{outline:3px solid var(--acc);outline-offset:3px}.evidence-gallery button:disabled{opacity:.5;cursor:not-allowed}@media(max-width:720px){.ev-grid{grid-template-columns:1fr}.ev-detail{padding:20px 24px;border-top:1px solid var(--line)}.ev-stage{height:280px}.ev-navigation{flex-wrap:wrap}}@media(max-width:480px){.ev-head,.ev-toolbar{padding:16px}.ev-viewer{padding:0 16px 16px}.ev-detail{padding:16px}.ev-foot{padding:16px;flex-wrap:wrap}.ev-toolbar>.ev-actions{width:100%}.ev-toolbar .btn{flex:1}.ev-stage{height:240px}}
</style>
