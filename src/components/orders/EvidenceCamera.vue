<script setup lang="ts">
import { useEvidenceCamera } from '../../composables/useEvidenceCamera';
const props = defineProps<{ orderId: string; busy: boolean; error: string }>();
const emit = defineEmits<{ capture: [file: File, note: string]; closed: []; files: [] }>();
const camera = useEvidenceCamera(props, () => emit('closed'));
const {devices,deviceId,phase,cameraError,photo,preview,note,discard,taking,open,close,saved,enable,take,retake} = camera;
defineExpose({open,saved});
</script>
<template>
 <dialog :ref="camera.dialog" class="evidence-camera" aria-labelledby="evidence-camera-title" @cancel.prevent="close()">
  <header><div><h2 id="evidence-camera-title">Tomar foto</h2><p class="help">Evidencia para {{ orderId }}</p></div><button class="ev-close" type="button" aria-label="Cerrar cámara" :disabled="busy||taking" @click="close()">×</button></header>
  <div class="ev-camera-body">
   <div v-if="discard" class="ev-discard" role="alert"><strong>¿Descartar esta captura?</strong><p>La foto aún no se ha agregado a la orden.</p><div class="ev-actions"><button class="btn btn-s" @click="discard=false">Seguir revisando</button><button class="btn btn-s" @click="close(true)">Descartar foto</button></div></div>
   <template v-else>
   <div v-if="phase==='live'&&devices.length>1" class="ev-field"><label for="evidence-device">Cámara</label><select id="evidence-device" v-model="deviceId" :disabled="taking" @change="enable"><option v-for="(device,i) in devices" :key="device.deviceId" :value="device.deviceId">{{ device.label||`Cámara ${i+1}` }}</option></select></div>
   <p role="status" class="help">{{ phase==='review'?'Revisa la foto antes de agregarla':phase==='live'?'Vista en vivo · Solo fotografía':phase==='loading'?'Esperando a la cámara…':'La cámara está apagada' }}</p>
   <div class="ev-stage">
    <video v-show="phase==='live'" :ref="camera.video" autoplay playsinline muted aria-label="Vista en vivo de la cámara"></video>
    <img v-if="phase==='review'" :src="preview" alt="Captura pendiente de guardar">
    <div v-if="phase==='off'||phase==='loading'" class="ev-empty"><h3>Enciende la cámara para comenzar</h3><p>Se solicitará acceso a la cámara. No se usa el micrófono.</p><button :ref="camera.enableButton" class="btn btn-p" :disabled="phase==='loading'" @click="enable">{{ phase==='loading'?'Encendiendo…':'Encender cámara' }}</button></div>
   </div>
   <p v-if="cameraError||error" class="ev-error" role="alert">{{ cameraError||error }}</p>
   <button v-if="cameraError" class="btn btn-s" @click="close(true);emit('files')">Seleccionar archivos en su lugar</button>
   <div class="ev-actions ev-center"><button v-if="phase==='live'" class="btn btn-p" :disabled="taking" @click="take">{{ taking?'Capturando…':'Tomar foto' }}</button><button v-if="phase==='review'" class="btn btn-s" :disabled="busy" @click="retake">Volver a tomar</button></div>
   <div v-if="phase==='review'" class="ev-field"><label for="capture-description">Descripción (opcional)</label><textarea id="capture-description" v-model="note" maxlength="500" rows="2" :disabled="busy||!!error" placeholder="¿Qué muestra esta foto?"></textarea><p v-if="error" class="help">El reintento conserva la foto y descripción enviadas para evitar duplicados.</p></div>
   </template>
  </div>
  <footer v-if="!discard"><p class="help">La cámara se apaga al capturar o cerrar.</p><div class="ev-actions"><button class="btn btn-s" :disabled="busy||taking" @click="close()">Cancelar</button><button :ref="camera.addButton" class="btn btn-p" :disabled="phase!=='review'||busy" @click="photo&&emit('capture',photo,note)">{{ busy?'Guardando…':error?'Reintentar guardado':'Agregar a la orden' }}</button></div></footer>
 </dialog>
</template>
<style scoped>
.evidence-camera{width:min(690px,calc(100% - 32px));max-height:calc(100dvh - 32px);padding:0;border:1px solid #d5dde4;border-radius:16px;background:white;color:var(--ink);box-shadow:0 24px 64px #0f243040}.evidence-camera::backdrop{background:#0f2430a6}.evidence-camera header,.evidence-camera footer{padding:18px 22px;display:flex;justify-content:space-between;align-items:center;gap:16px;flex-wrap:wrap}.evidence-camera header{border-bottom:1px solid var(--line)}.evidence-camera h2{font:700 25px/1.2 var(--disp);margin:0 0 5px}.evidence-camera footer{background:var(--tint);border-top:1px solid var(--line)}.ev-camera-body{padding:22px;display:grid;gap:14px}.ev-close{border:1px solid #73838f;background:white;border-radius:8px;width:44px;height:44px;font-size:26px;cursor:pointer}.ev-stage{aspect-ratio:16/10;min-height:200px;background:#0f2430;border-radius:10px;display:flex;align-items:center;justify-content:center;overflow:hidden}.ev-stage video,.ev-stage img{width:100%;height:100%;object-fit:contain;min-width:0}.ev-empty{padding:24px;text-align:center;color:white;max-width:430px}.ev-empty h3{font:600 20px var(--disp);margin:0 0 10px}.ev-empty p{color:#c5d6e2;font-size:14px;line-height:1.5;margin-bottom:18px}.ev-actions{display:flex;gap:10px;flex-wrap:wrap}.ev-center{justify-content:center}.ev-field{display:grid;gap:7px}.ev-field label{font-size:14px;font-weight:600}.ev-field textarea,.ev-field select{border:1px solid #73838f;border-radius:8px;padding:10px 12px;background:white;color:var(--ink);font:16px var(--body);width:100%;min-height:44px}.ev-error{color:#a32828;line-height:1.5;font-size:14px}.ev-discard{display:grid;gap:14px}.evidence-camera :focus-visible{outline:3px solid var(--acc);outline-offset:3px}@media(max-width:480px){.ev-stage{aspect-ratio:4/3}.ev-camera-body{padding:16px}.evidence-camera footer .ev-actions{width:100%}.evidence-camera footer .btn{flex:1}}
</style>
