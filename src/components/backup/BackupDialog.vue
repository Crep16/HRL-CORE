<script setup lang="ts">
import {nextTick,ref,watch} from 'vue';
import {storeToRefs} from 'pinia';
import {useBackupStore} from '../../stores/useBackupStore';
const model=useBackupStore();
const {open,busy,unlocked,restoreMode,restored,masterPassword,backupPassword,error,notice,status}=storeToRefs(model);
const dialog=ref<HTMLDialogElement>();
let opener:HTMLElement|null=null;
watch(open,async value=>{await nextTick();if(value){opener=document.querySelector<HTMLElement>('.header-tools summary')??document.activeElement as HTMLElement;dialog.value?.showModal();dialog.value?.querySelector<HTMLInputElement>('input')?.focus();}else{dialog.value?.close();opener?.focus();}});
watch(unlocked,async value=>{if(value){await nextTick();document.getElementById('backup-create')?.focus();}});
watch(restoreMode,async value=>{if(value){await nextTick();document.getElementById('backup-restore-password')?.focus();}});
watch(restored,async value=>{if(value){await nextTick();document.getElementById('backup-restart')?.focus();}});
</script>
<template>
 <dialog ref="dialog" class="backup-dialog" aria-labelledby="backup-title" :aria-busy="busy" @cancel.prevent="model.close">
  <header><h2 id="backup-title">Respaldos</h2><button type="button" class="btn btn-s" :disabled="busy||restored" @click="model.close">Cerrar</button></header>
  <p v-if="busy" role="status">Preparando o verificando el respaldo. Espera a que termine…</p>
  <p v-if="error" role="alert" class="backup-error">{{error}}</p><p v-if="notice" role="status">{{notice}}</p>
  <button v-if="restored" id="backup-restart" class="btn btn-p" :disabled="busy" @click="model.restart">Reiniciar HRL CORE</button>
  <template v-else>
   <form v-if="!unlocked&&!restoreMode" @submit.prevent="model.unlock">
    <p>Disponible para acceso Completo. Escribe la clave maestra para gestionar las copias.</p>
    <label for="backup-master">Clave maestra del taller</label><input id="backup-master" v-model="masterPassword" type="password" autocomplete="off" required maxlength="1024" :disabled="busy">
    <button class="btn btn-p" :disabled="busy">Abrir Respaldos</button>
   </form>
   <section v-if="unlocked&&status">
    <p>Los archivos incluyen órdenes, dinero, auditoría, fotografías, logo y las claves protegidas. Se abren con la clave maestra vigente al crear cada copia.</p>
    <p><strong>Carpeta:</strong> {{status.folder}}</p>
    <div class="backup-actions"><button id="backup-create" class="btn btn-p" :disabled="busy" @click="model.create">Crear respaldo ahora</button><button class="btn btn-s" :disabled="busy" @click="model.openFolder">Abrir carpeta Respaldos</button></div>
    <p>Copia tus respaldos a un USB o disco externo. Una copia que permanece solo en esta computadora no cubre su pérdida total.</p>
    <label class="backup-toggle"><input type="checkbox" :checked="status.automatic" :disabled="busy" @change="model.configure(($event.target as HTMLInputElement).checked)">Crear un respaldo al cerrar HRL CORE</label>
    <p>Si cambias la clave maestra, vuelve a activar esta opción. Las copias anteriores conservan su clave original.</p>
    <p v-if="status.warning" role="status" class="backup-error">{{status.warning}}</p>
    <details><summary>Copias guardadas ({{status.files.length}})</summary><ul><li v-for="file in status.files" :key="file.name">{{file.name}} · {{Math.ceil(file.bytes/1024)}} KB</li></ul><p v-if="!status.files.length">Aún no hay respaldos. Crea el primero o activa el respaldo al cerrar.</p></details>
    <button class="btn btn-s" :disabled="busy" @click="restoreMode=!restoreMode">Restaurar un respaldo</button>
   </section>
   <form v-if="restoreMode" @submit.prevent="model.restore">
    <h3>Restaurar respaldo</h3><p>Se reemplazará la información de esta instalación después de validar la copia. La instalación anterior se conservará por separado. Después tendrás que reiniciar e iniciar sesión con una cuenta del respaldo.</p>
    <label for="backup-restore-password">Clave maestra con la que se creó el respaldo</label><input id="backup-restore-password" v-model="backupPassword" type="password" autocomplete="off" required maxlength="1024" :disabled="busy">
    <button class="btn btn-p" :disabled="busy">Seleccionar archivo y restaurar</button>
   </form>
  </template>
 </dialog>
</template>
<style scoped>
.backup-dialog p{margin:0}
.backup-dialog{width:min(700px,calc(100vw - 32px));max-height:90dvh;overflow:auto;border:1px solid #7c8d99;border-radius:12px;padding:24px;color:#12222d;background:white;font:16px/1.5 var(--body,Arial,sans-serif)}.backup-dialog::backdrop{background:#12222d80}.backup-dialog header,.backup-actions{display:flex;gap:12px;justify-content:space-between;align-items:center;flex-wrap:wrap}.backup-dialog h2{margin:0;font-size:24px}.backup-dialog p,.backup-dialog li{overflow-wrap:anywhere}.backup-dialog form,.backup-dialog section{display:grid;gap:12px;margin-top:16px}.backup-dialog input[type=password]{width:100%;min-height:44px;padding:10px;border:1px solid #7c8d99;border-radius:6px;font:inherit}.backup-dialog button,.backup-dialog summary{min-height:44px}.backup-toggle{display:flex;gap:10px;align-items:center;min-height:44px}.backup-toggle input{width:20px;height:20px}.backup-error{color:#91251d;background:#fff0ee;padding:12px}.backup-dialog :focus-visible{outline:3px solid #1b72ad;outline-offset:3px}
</style>
