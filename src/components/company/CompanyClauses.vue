<script setup lang="ts">
import SearchSelect from '../ui/SearchSelect.vue';
import AppNotice from '../ui/AppNotice.vue';
import type { CompanySettings } from '../../types/company';
import { useCompanyClauses } from '../../composables/useCompanyClauses';
const settings = defineModel<CompanySettings>({ required: true });
const emit = defineEmits<{ editing: [value: boolean] }>();
const {editor,isNew,error,removed,activeCount,edit,cancel,apply,move,remove,undo} = useCompanyClauses(settings, value => emit('editing', value));
</script>

<template>
  <div class="company-clause-controls">
    <SearchSelect id="clause-position" v-model="settings.clausePosition" label="Dónde se imprimen" :options="[{value:'foot',label:'Al pie de la nota'},{value:'back',label:'En el reverso'}]" />
    <label class="company-check"><input v-model="settings.numberClauses" type="checkbox" /> Numerar cláusulas</label>
  </div>
  <p class="company-help">Textos de ejemplo del prototipo. Revísalos antes de utilizarlos con clientes.</p>
  <div class="company-section-toolbar"><p>{{ settings.clauses.length }} cláusulas · {{ activeCount }} activas</p><button id="clause-add" class="btn btn-s" type="button" :disabled="!!editor" @click="edit()">Agregar cláusula</button></div>
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
  <AppNotice v-if="removed" :message="`Se quitó «${removed.clause.title}».`"><button class="lnk" type="button" @click="undo">Deshacer</button></AppNotice>
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
