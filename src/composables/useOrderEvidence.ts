import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useWorkshopDataStore } from '../stores/useWorkshopDataStore.ts';
import { useWorkshopAccess } from './useWorkshopAccess.ts';
import EvidenceCamera from '../components/orders/EvidenceCamera.vue';
import { useTemplateRef } from 'vue';
export function useOrderEvidence(props:{orderId:string;disabled?:boolean;trashed?:boolean}){
type Photo={id:string;orderId:string;originalName:string;mime:string;sizeBytes:number;source:'camera'|'file';note:string;authorName:string;createdAt:string;deleted:boolean;revision:number};
type Upload={id:string;file:File;source:'camera'|'file';note:string};
const {can,requestAccess,user,temporary}=useWorkshopAccess();
const photos=ref<Photo[]>([]),selectedId=ref(''),description=ref(''),message=ref(''),error=ref(''),cameraError=ref('');
const loading=ref(false),busy=ref(false),loaded=ref(false),imageLoading=ref(false),imageError=ref('');
const gallery=useTemplateRef<HTMLDialogElement>('gallery'),files=useTemplateRef<HTMLInputElement>('files'),camera=useTemplateRef<InstanceType<typeof EvidenceCamera>>('camera');
const opener=useTemplateRef<HTMLButtonElement>('opener');
const imageUrl=ref(''),removed=ref<Photo>(),discardNote=ref(false);
const pending=ref<Upload[]>([]);
const showRemoved=ref(false);
const selected=computed(()=>photos.value.find(p=>p.id===selectedId.value));
const position=computed(()=>photos.value.findIndex(p=>p.id===selectedId.value));
const dirty=computed(()=>!!selected.value&&description.value!==selected.value.note);
let disposed=false,imageVersion=0;
const size=(n:number)=>n<1048576?`${Math.round(n/1024)} KB`:`${(n/1048576).toFixed(1)} MB`;
const reason=(cause:unknown)=>cause && typeof cause==='object' && 'message' in cause ? String(cause.message) : String(cause);
async function allowed(operation:string){return await requestAccess(operation,'clients')&&!disposed;}
function releaseImage(){imageVersion++;if(imageUrl.value)URL.revokeObjectURL(imageUrl.value);imageUrl.value='';imageError.value='';}
async function loadImage(){
 releaseImage();const photo=selected.value;if(!photo){imageLoading.value=false;return;}
 const version=imageVersion;imageLoading.value=true;
 try{const data=await useWorkshopDataStore().readPhoto(props.orderId,photo.id);
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
 try{const result=await useWorkshopDataStore().photos(props.orderId,showRemoved.value);if(disposed||!can('evidence.view'))return;
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
 try{const updated=await useWorkshopDataStore().updatePhoto({orderId:props.orderId,id:photo.id,note:description.value,deleted:false,expectedRevision:photo.revision});
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
 return await useWorkshopDataStore().addPhoto({id:item.id,orderId:props.orderId,originalName:item.file.name,mime:item.file.type,source:item.source,note:item.note,bytes});
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
function closeCamera(){cameraUpload=undefined;cameraError.value='';}
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
 try{const updated=await useWorkshopDataStore().updatePhoto({orderId:props.orderId,id:current.id,note:current.note,deleted:!restore,expectedRevision:current.revision});
  if(restore){if(showRemoved.value){photos.value=photos.value.filter(p=>p.id!==updated.id);selectedId.value=photos.value[0]?.id??'';}else{photos.value.push(updated);selectedId.value=updated.id;}removed.value=undefined;message.value='Foto restaurada.';}
  else{removed.value=updated;photos.value=photos.value.filter(p=>p.id!==updated.id);selectedId.value=photos.value[0]?.id??'';message.value='Foto retirada. El archivo y su historial se conservan.';}
  await loadImage();
 }catch(cause){error.value=reason(cause);}finally{busy.value=false;}
}
onBeforeUnmount(()=>{disposed=true;releaseImage();gallery.value?.close();});
return {photos,selectedId,description,message,error,cameraError,loading,busy,loaded,imageLoading,imageError,gallery,files,camera,imageUrl,removed,discardNote,pending,showRemoved,selected,position,dirty,size,allowed,loadImage,refresh,open,toggleRemoved,close,discard,saveNote,select,navigate,pick,openCamera,importFiles,retry,closeCamera,capture,remove};
}
