<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import type { Clause, CompanySettings } from '../../types/company';
const settings = defineModel<CompanySettings>({ required: true });
const emit = defineEmits<{ editing: [value: boolean] }>();
const editor = ref<Clause>();
const isNew = ref(false);
const error = ref('');
const removed = ref<{clause: Clause; index: number}>();
watch(() => settings.value, () => {removed.value=undefined;});
function edit(clause?: Clause) {
  editor.value = clause ? {...clause} : {id:crypto.randomUUID(), title:'', text:'', entry:true, exit:false};
  isNew.value = !clause; error.value = ''; emit('editing', true);
  nextTick(() => document.getElementById('clause-title')?.focus());
}
function cancel() { editor.value = undefined; error.value = ''; emit('editing', false); }
function apply() {
  if (!editor.value) return;
  const clause = {...editor.value, title:editor.value.title.trim(), text:editor.value.text.trim()};
  if (!clause.title || !clause.text) { error.value = 'Completa el título y el texto de la cláusula.'; return; }
  if (isNew.value) settings.value.clauses.push(clause);
  else { const index = settings.value.clauses.findIndex(c => c.id === clause.id); if (index >= 0) settings.value.clauses[index] = clause; }
  cancel();
}
function move(index: number, offset: number) {
  const destination = index + offset;
  if (destination < 0 || destination >= settings.value.clauses.length) return;
  const clause = settings.value.clauses.splice(index, 1)[0]!;
  settings.value.clauses.splice(destination, 0, clause);
  nextTick(() => {
    const action=document.getElementById(`clause-${clause.id}-${offset < 0 ? 'up' : 'down'}`) as HTMLButtonElement | null;
    if(action && !action.disabled) action.focus();
    else document.getElementById(`clause-${clause.id}`)?.focus();
  });
}
function remove(index: number) {
  removed.value = {clause:{...settings.value.clauses[index]!},index};
  settings.value.clauses.splice(index,1);
}
function undo() {
  if (!removed.value) return;
  if (!settings.value.clauses.some(c=>c.id===removed.value!.clause.id)) settings.value.clauses.splice(Math.min(removed.value.index, settings.value.clauses.length), 0, removed.value.clause);
  removed.value = undefined;
}
</script>

<template>
  <div class="company-clause-controls">
    <div class="company-field"><label for="clause-position">Dónde se imprimen</label><select id="clause-position" v-model="settings.clausePosition"><option value="foot">Al pie de la nota</option><option value="back">En el reverso</option></select></div>
    <label class="company-check"><input v-model="settings.numberClauses" type="checkbox" /> Numerar cláusulas</label>
  </div>
  <p class="company-help">Textos de ejemplo del prototipo. Revísalos antes de utilizarlos con clientes.</p>
  <div class="company-section-toolbar"><p>{{ settings.clauses.length }} cláusulas · {{ settings.clauses.filter(c=>c.entry||c.exit).length }} activas</p><button class="btn btn-s" type="button" :disabled="!!editor" @click="edit()">Agregar cláusula</button></div>
  <p v-if="!settings.clauses.length" class="company-empty">No hay cláusulas. Agrega una para incluir condiciones en las notas.</p>
  <ol class="company-clause-list">
    <li v-for="(clause, index) in settings.clauses" :key="clause.id" :class="{selected:editor?.id===clause.id}">
      <span class="clause-number" aria-hidden="true">{{ index + 1 }}</span>
      <button :id="`clause-${clause.id}`" class="clause-select" type="button" :disabled="!!editor" @click="edit(clause)"><strong>{{ clause.title }}</strong><span>{{ [clause.entry ? 'Entrada' : '',clause.exit ? 'Salida' : ''].filter(Boolean).join(' · ') || 'No se imprime' }}</span></button>
      <div class="clause-row-actions">
        <button :id="`clause-${clause.id}-up`" class="company-icon-button" type="button" :disabled="index===0 || !!editor" :aria-label="`Subir ${clause.title}`" @click="move(index,-1)">↑</button>
        <button :id="`clause-${clause.id}-down`" class="company-icon-button" type="button" :disabled="index===settings.clauses.length-1 || !!editor" :aria-label="`Bajar ${clause.title}`" @click="move(index,1)">↓</button>
        <button class="company-icon-button" type="button" :disabled="!!editor" :aria-label="`Quitar ${clause.title}`" @click="remove(index)">×</button>
      </div>
    </li>
  </ol>
  <div v-if="removed" class="company-feedback" role="status">Se quitó «{{ removed.clause.title }}». <button class="lnk" type="button" @click="undo">Deshacer</button></div>
  <section v-if="editor" class="company-clause-editor" aria-labelledby="clause-editor-title">
    <h3 id="clause-editor-title">{{ isNew ? 'Nueva cláusula' : 'Editar cláusula' }}</h3>
    <p v-if="error" class="company-error" role="alert">{{ error }}</p>
    <div class="company-field"><label for="clause-title">Título</label><input id="clause-title" v-model="editor.title" maxlength="80" :aria-invalid="!!error && !editor.title.trim()" /></div>
    <div class="company-field"><label for="clause-text">Texto que se imprime</label><textarea id="clause-text" v-model="editor.text" rows="4" maxlength="400" :aria-invalid="!!error && !editor.text.trim()" aria-describedby="clause-count"></textarea><span id="clause-count" class="company-count">{{ editor.text.length }} de 400</span></div>
    <fieldset class="company-inline-options"><legend>Sale en</legend><label class="company-check"><input v-model="editor.entry" type="checkbox" /> Entrada</label><label class="company-check"><input v-model="editor.exit" type="checkbox" /> Salida</label></fieldset>
    <p class="company-help">Aplicar agrega el texto a la configuración pendiente. Después guarda los cambios de Empresa.</p>
    <div class="company-editor-actions"><button class="btn btn-s" type="button" @click="cancel">Cancelar edición</button><button class="btn btn-p" type="button" @click="apply">Aplicar cláusula</button></div>
  </section>
</template>
