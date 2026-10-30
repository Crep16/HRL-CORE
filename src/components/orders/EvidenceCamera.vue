<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref } from 'vue';
const props=defineProps<{ orderId:string; busy:boolean; error:string }>();
const emit=defineEmits<{ capture:[file:File,note:string]; closed:[]; files:[] }>();
const dialog=ref<HTMLDialogElement>();
const video=ref<HTMLVideoElement>();
const devices=ref<MediaDeviceInfo[]>([]);
const deviceId=ref('');
const phase=ref<'off'|'loading'|'live'|'review'>('off');
const cameraError=ref('');
const photo=ref<File>();
const preview=ref('');
const note=ref('');
const discard=ref(false);
const taking=ref(false);
const enableButton=ref<HTMLButtonElement>();
const addButton=ref<HTMLButtonElement>();
let stream:MediaStream|null=null,version=0,disposed=false;
function stop(){stream?.getTracks().forEach(track=>track.stop());stream=null;if(video.value)video.value.srcObject=null;}
function release(){if(preview.value)URL.revokeObjectURL(preview.value);preview.value='';photo.value=undefined;}
async function open(){discard.value=false;cameraError.value='';note.value='';phase.value='off';dialog.value?.showModal();await nextTick();enableButton.value?.focus();}
function close(force=false){
 if(props.busy||taking.value)return;
 if(photo.value&&!force){discard.value=true;return;}
 version++;stop();release();dialog.value?.close();emit('closed');
}
function saved(){version++;stop();release();dialog.value?.close();emit('closed');}
async function enable(){
 const current=++version;stop();cameraError.value='';phase.value='loading';
 try{
  if(!navigator.mediaDevices?.getUserMedia)throw Object.assign(new Error(),{name:'Unavailable'});
  const opened=await navigator.mediaDevices.getUserMedia({video:deviceId.value?{deviceId:{exact:deviceId.value}}:{width:{ideal:1920},height:{ideal:1080}},audio:false});
  if(disposed||current!==version||!dialog.value?.open){opened.getTracks().forEach(track=>track.stop());return;}
  stream=opened;phase.value='live';await nextTick();
  if(video.value){video.value.srcObject=opened;await video.value.play();}
  const found=await navigator.mediaDevices.enumerateDevices();
  if(disposed||current!==version)return;
  devices.value=found.filter(device=>device.kind==='videoinput');
  deviceId.value=opened.getVideoTracks()[0]?.getSettings().deviceId??'';
 }catch(cause){
  if(disposed||current!==version)return;
  stop();phase.value='off';
  const messages:Record<string,string>={NotAllowedError:'No se permitió usar la cámara. Habilita el permiso o selecciona archivos.',NotFoundError:'No encontramos una cámara. Conecta una o selecciona archivos.',NotReadableError:'La cámara está ocupada o no responde. Cierra otras aplicaciones que la estén usando.',Unavailable:'La cámara no está disponible en esta ventana. Puedes seleccionar fotos guardadas en tu computadora.'};
  cameraError.value=messages[(cause as Error).name]??'No se pudo encender la cámara. Intenta de nuevo o selecciona archivos.';
 }
}
async function take(){
 if(!video.value?.videoWidth||taking.value)return;
 taking.value=true;const current=version;
 try{
  const canvas=document.createElement('canvas');canvas.width=video.value.videoWidth;canvas.height=video.value.videoHeight;
  canvas.getContext('2d')!.drawImage(video.value,0,0);
  const blob=await new Promise<Blob|null>(resolve=>canvas.toBlob(resolve,'image/jpeg',.92));
  if(!blob)throw new Error();if(disposed||current!==version)return;
  photo.value=new File([blob],`Captura-${Date.now()}.jpg`,{type:'image/jpeg'});
  preview.value=URL.createObjectURL(blob);stop();phase.value='review';await nextTick();addButton.value?.focus();
 }catch{cameraError.value='No se pudo tomar la foto. Intenta de nuevo.';}
 finally{taking.value=false;}
}
function retake(){release();note.value='';void enable();}
onBeforeUnmount(()=>{disposed=true;version++;stop();release();dialog.value?.close();});
defineExpose({open,saved});
</script>
<template>
 <dialog ref="dialog" class="evidence-camera" aria-labelledby="evidence-camera-title" @cancel.prevent="close()">
  <header><div><h2 id="evidence-camera-title">Tomar foto</h2><p class="help">Evidencia para {{ orderId }}</p></div><button class="ev-close" type="button" aria-label="Cerrar cámara" :disabled="busy||taking" @click="close()">×</button></header>
  <div class="ev-camera-body">
   <div v-if="discard" class="ev-discard" role="alert"><strong>¿Descartar esta captura?</strong><p>La foto aún no se ha agregado a la orden.</p><div class="ev-actions"><button class="btn btn-s" @click="discard=false">Seguir revisando</button><button class="btn btn-s" @click="close(true)">Descartar foto</button></div></div>
   <template v-else>
   <div v-if="phase==='live'&&devices.length>1" class="ev-field"><label for="evidence-device">Cámara</label><select id="evidence-device" v-model="deviceId" :disabled="taking" @change="enable"><option v-for="(device,i) in devices" :key="device.deviceId" :value="device.deviceId">{{ device.label||`Cámara ${i+1}` }}</option></select></div>
   <p role="status" class="help">{{ phase==='review'?'Revisa la foto antes de agregarla':phase==='live'?'Vista en vivo · Solo fotografía':phase==='loading'?'Esperando a la cámara…':'La cámara está apagada' }}</p>
   <div class="ev-stage">
    <video v-show="phase==='live'" ref="video" autoplay playsinline muted aria-label="Vista en vivo de la cámara"></video>
    <img v-if="phase==='review'" :src="preview" alt="Captura pendiente de guardar">
    <div v-if="phase==='off'||phase==='loading'" class="ev-empty"><h3>Enciende la cámara para comenzar</h3><p>Se solicitará acceso a la cámara. No se usa el micrófono.</p><button ref="enableButton" class="btn btn-p" :disabled="phase==='loading'" @click="enable">{{ phase==='loading'?'Encendiendo…':'Encender cámara' }}</button></div>
   </div>
   <p v-if="cameraError||error" class="ev-error" role="alert">{{ cameraError||error }}</p>
   <button v-if="cameraError" class="btn btn-s" @click="close(true);emit('files')">Seleccionar archivos en su lugar</button>
   <div class="ev-actions ev-center"><button v-if="phase==='live'" class="btn btn-p" :disabled="taking" @click="take">{{ taking?'Capturando…':'Tomar foto' }}</button><button v-if="phase==='review'" class="btn btn-s" :disabled="busy" @click="retake">Volver a tomar</button></div>
   <div v-if="phase==='review'" class="ev-field"><label for="capture-description">Descripción (opcional)</label><textarea id="capture-description" v-model="note" maxlength="500" rows="2" :disabled="busy||!!error" placeholder="¿Qué muestra esta foto?"></textarea><p v-if="error" class="help">El reintento conserva la foto y descripción enviadas para evitar duplicados.</p></div>
   </template>
  </div>
  <footer v-if="!discard"><p class="help">La cámara se apaga al capturar o cerrar.</p><div class="ev-actions"><button class="btn btn-s" :disabled="busy||taking" @click="close()">Cancelar</button><button ref="addButton" class="btn btn-p" :disabled="phase!=='review'||busy" @click="photo&&emit('capture',photo,note)">{{ busy?'Guardando…':error?'Reintentar guardado':'Agregar a la orden' }}</button></div></footer>
 </dialog>
</template>
<style scoped>
.evidence-camera{width:min(690px,calc(100% - 32px));max-height:calc(100dvh - 32px);padding:0;border:1px solid #d5dde4;border-radius:16px;background:white;color:var(--ink);box-shadow:0 24px 64px #0f243040}.evidence-camera::backdrop{background:#0f2430a6}.evidence-camera header,.evidence-camera footer{padding:18px 22px;display:flex;justify-content:space-between;align-items:center;gap:16px;flex-wrap:wrap}.evidence-camera header{border-bottom:1px solid var(--line)}.evidence-camera h2{font:700 25px/1.2 var(--disp);margin:0 0 5px}.evidence-camera footer{background:var(--tint);border-top:1px solid var(--line)}.ev-camera-body{padding:22px;display:grid;gap:14px}.ev-close{border:1px solid #73838f;background:white;border-radius:8px;width:44px;height:44px;font-size:26px;cursor:pointer}.ev-stage{aspect-ratio:16/10;min-height:200px;background:#0f2430;border-radius:10px;display:flex;align-items:center;justify-content:center;overflow:hidden}.ev-stage video,.ev-stage img{width:100%;height:100%;object-fit:contain;min-width:0}.ev-empty{padding:24px;text-align:center;color:white;max-width:430px}.ev-empty h3{font:600 20px var(--disp);margin:0 0 10px}.ev-empty p{color:#c5d6e2;font-size:14px;line-height:1.5;margin-bottom:18px}.ev-actions{display:flex;gap:10px;flex-wrap:wrap}.ev-center{justify-content:center}.ev-field{display:grid;gap:7px}.ev-field label{font-size:14px;font-weight:600}.ev-field textarea,.ev-field select{border:1px solid #73838f;border-radius:8px;padding:10px 12px;background:white;color:var(--ink);font:16px var(--body);width:100%;min-height:44px}.ev-error{color:#a32828;line-height:1.5;font-size:14px}.ev-discard{display:grid;gap:14px}.evidence-camera :focus-visible{outline:3px solid var(--acc);outline-offset:3px}@media(max-width:480px){.ev-stage{aspect-ratio:4/3}.ev-camera-body{padding:16px}.evidence-camera footer .ev-actions{width:100%}.evidence-camera footer .btn{flex:1}}
</style>
