<script setup lang="ts">
import DraftLeaveDialog from '../components/ui/DraftLeaveDialog.vue';
import CompanyClauses from '../components/company/CompanyClauses.vue';
import SearchSelect from '../components/ui/SearchSelect.vue';
import AppNotice from '../components/ui/AppNotice.vue';
import CompanyNote from '../components/company/CompanyNote.vue';
import brandLogo from '../assets/images/brand/hrl-original.svg';
import type { CompanySection } from '../types/company';
import { paperLabels } from '../types/company';
import type { Order } from '../types/order';
import { useCompanyWorkspace } from '../composables/useCompanyWorkspace';
const props = defineProps<{orders: Order[]; order?: Order}>();
const emit = defineEmits<{dirtyChange:[value:boolean]}>();
const { leaveGuard, prepareLeave, can, draft, section, noteKind, clauseEditing, message, saving, errors, fieldError, focusError, uploadError, readingLogo, printing, confirmDiscard, fileInput, dirty, hasPending, nextFolio, sections, current, descriptions, totalsExample, goTo, save, discard, upload, pasteLogo, printTest } = useCompanyWorkspace(props, emit);
defineExpose({prepareLeave});
</script>

<template>
  <main class="company-workspace" aria-labelledby="company-title">
    <header class="company-heading"><div><h1 tabindex="-1" id="company-title">Empresa</h1><p>Configura tus datos y el formato de las notas del taller.</p></div><span class="company-save-state" :class="{pending:hasPending}" role="status">{{ hasPending ? 'Cambios sin guardar' : 'Sin cambios pendientes' }}</span></header>
    <AppNotice :message="message" @dismiss="message=''" />
    <div class="company-layout">
      <SearchSelect class="company-mobile-nav company-field" id="company-section-select" :model-value="section" label="Sección de Empresa" :options="sections.map(item=>({value:item.id,label:item.title}))" @update:model-value="goTo($event as CompanySection)" />
      <nav class="company-nav" aria-label="Configuración de Empresa">
        <button v-for="item in sections" :key="item.id" type="button" :class="{active:section===item.id}" :aria-current="section===item.id ? 'page' : undefined" @click="goTo(item.id)"><strong>{{ item.title }}</strong><span>{{ item.detail }}</span></button>
        <p>Guardado en este equipo.<br />Web pendiente de conectar.</p>
      </nav>
      <form class="company-panel" novalidate aria-labelledby="company-section-title" @submit.prevent="save">
        <fieldset class="company-permission-fields" :disabled="!can('company.edit') || saving">
        <div class="company-panel-heading"><h2 id="company-section-title" tabindex="-1">{{ current.title }}</h2><p>{{ descriptions[section] }}</p></div>
        <div v-if="errors.length" ref="errorSummary" class="company-error" tabindex="-1" role="alert"><strong>Revisa la configuración antes de guardar:</strong><ul><li v-for="error in errors" :key="error"><button class="company-error-link" type="button" @click="focusError(error)">{{ error }}</button></li></ul></div>
        <div class="company-fields">
          <template v-if="section==='general'">
            <fieldset><legend>Negocio</legend><div class="company-grid">
              <div class="company-field"><label for="company-name">Nombre de la empresa <span>(obligatorio)</span></label><input id="company-name" v-model="draft.name" maxlength="100" autocomplete="organization" required :aria-invalid="!!fieldError('company-name')" :aria-describedby="fieldError('company-name') ? 'company-name-error' : undefined" /><p v-if="fieldError('company-name')" id="company-name-error" class="company-field-error">{{ fieldError('company-name') }}</p></div>
              <div class="company-field"><label for="company-tax-id">RFC o identificación fiscal</label><input id="company-tax-id" v-model="draft.taxId" maxlength="30" /></div>
            </div></fieldset>
            <fieldset><legend>Contacto</legend><div class="company-grid">
              <div class="company-field"><label for="company-phone">Teléfono</label><input id="company-phone" v-model="draft.phone" type="tel" maxlength="30" autocomplete="tel" /></div>
              <div class="company-field"><label for="company-whatsapp">WhatsApp</label><input id="company-whatsapp" v-model="draft.whatsapp" type="tel" maxlength="30" /></div>
              <div class="company-field"><label for="company-email">Correo</label><input id="company-email" v-model="draft.email" type="email" maxlength="120" autocomplete="email" :aria-invalid="!!fieldError('company-email')" :aria-describedby="fieldError('company-email') ? 'company-email-error' : undefined" /><p v-if="fieldError('company-email')" id="company-email-error" class="company-field-error">{{ fieldError('company-email') }}</p></div>
              <div class="company-field"><label for="company-website">Página en internet</label><input id="company-website" v-model="draft.website" maxlength="160" placeholder="negocio.mx" :aria-invalid="!!fieldError('company-website')" :aria-describedby="fieldError('company-website') ? 'company-website-error' : undefined" /><p v-if="fieldError('company-website')" id="company-website-error" class="company-field-error">{{ fieldError('company-website') }}</p></div>
            </div></fieldset>
            <fieldset><legend>Ubicación y horario</legend><div class="company-grid">
              <div class="company-field company-span"><label for="company-address">Dirección</label><input id="company-address" v-model="draft.address" maxlength="240" autocomplete="street-address" /></div>
              <div class="company-field company-span"><label for="company-hours">Horario de atención</label><input id="company-hours" v-model="draft.hours" maxlength="120" /></div>
            </div></fieldset>
          </template>
          <template v-else-if="section==='logo'">
            <div class="company-logo-preview"><img v-if="draft.logoPrint || draft.logoWeb" :src="draft.logo || brandLogo" alt="Logo seleccionado para la empresa" /><p>{{ !draft.logoPrint && !draft.logoWeb ? 'El logo no se muestra en las notas ni en la web.' : draft.logoName || 'Logo HRL incluido en la aplicación' }}</p></div>
            <input ref="fileInput" class="sr" type="file" accept="image/png,image/jpeg" aria-label="Archivo del logo" tabindex="-1" @change="upload" />
            <div class="company-button-row"><button class="btn btn-s" type="button" :disabled="readingLogo" @click="fileInput?.click()">Subir archivo</button><button class="btn btn-s" type="button" :disabled="readingLogo" @click="pasteLogo">Pegar del portapapeles</button></div>
            <p class="company-help">PNG o JPG, hasta 1 MB. Recomendado: 600 × 300 px, con fondo blanco o transparente.</p>
            <p v-if="readingLogo" role="status">Leyendo imagen…</p><p v-if="uploadError" class="company-error" role="alert">{{ uploadError }}</p>
            <fieldset><legend>Dónde se muestra</legend>
              <label class="company-check"><input v-model="draft.logoPrint" type="checkbox" /> En las notas impresas</label>
              <label class="company-check"><input v-model="draft.logoWeb" type="checkbox" /> En la web de seguimiento cuando esté conectada</label>
            </fieldset>
            <div class="company-button-row"><button class="lnk" type="button" :disabled="readingLogo || (!draft.logoPrint && !draft.logoWeb && !draft.logo)" @click="draft.logo='';draft.logoName='';draft.logoPrint=false;draft.logoWeb=false">Quitar logo</button><button class="lnk" type="button" :disabled="readingLogo || (!draft.logo && draft.logoPrint && draft.logoWeb)" @click="draft.logo='';draft.logoName='';draft.logoPrint=true;draft.logoWeb=true">Restablecer logo HRL</button></div>
          </template>
          <template v-else-if="section==='notes'">
            <div class="company-field"><label for="entry-footer">Mensaje al pie de la nota de entrada</label><textarea id="entry-footer" v-model="draft.entryFooter" rows="3" maxlength="200" aria-describedby="entry-count"></textarea><span id="entry-count" class="company-count">{{ draft.entryFooter.length }} de 200</span></div>
            <div class="company-field"><label for="exit-footer">Mensaje al pie de la nota de salida</label><textarea id="exit-footer" v-model="draft.exitFooter" rows="3" maxlength="200" aria-describedby="exit-count"></textarea><span id="exit-count" class="company-count">{{ draft.exitFooter.length }} de 200</span></div>
            <label class="company-check"><input v-model="draft.printAddress" type="checkbox" /> Imprimir dirección y RFC</label>
            <label class="company-check"><input v-model="draft.signature" type="checkbox" /> Pedir firma del cliente</label>
            <label class="company-check"><input v-model="draft.webReference" type="checkbox" /> Imprimir referencia de la orden</label>
            <p class="company-help">La referencia corresponde al folio interno. El portal web de seguimiento aún no está conectado.</p>
          </template>
          <CompanyClauses v-else-if="section==='clauses'" v-model="draft" @editing="clauseEditing=$event" />
          <template v-else-if="section==='paper'">
            <SearchSelect id="company-paper" v-model="draft.paper" label="Tamaño del papel" :options="Object.entries(paperLabels).map(([value,label])=>({value,label}))" />
            <fieldset v-if="draft.paper==='letter'" class="company-inline-options"><legend>Orientación</legend><label class="company-check"><input v-model="draft.orientation" type="radio" value="portrait" /> Vertical</label><label class="company-check"><input v-model="draft.orientation" type="radio" value="landscape" /> Horizontal</label></fieldset>
            <div class="company-field"><label for="company-copies">Copias por nota</label><div class="company-stepper"><button type="button" :disabled="draft.copies<=1" aria-label="Menos copias" @click="draft.copies=Math.max(1,Number(draft.copies)-1)">−</button><input id="company-copies" v-model.number="draft.copies" type="number" min="1" max="5" step="1" /><button type="button" :disabled="draft.copies>=5" aria-label="Más copias" @click="draft.copies=Math.min(5,Number(draft.copies)+1)">+</button></div></div>
            <p class="company-help">Elige la impresora o Guardar como PDF en el diálogo del sistema. {{ draft.paper!=='letter' ? 'Cada copia se prepara por separado, con el ancho de ticket seleccionado.' : 'Las copias se agrupan de dos en dos en la misma hoja, con una línea de corte. Si el contenido es extenso, ambas continúan juntas en la hoja siguiente.' }}</p>
            <p v-if="draft.clausePosition==='back'" class="company-help">Las cláusulas van en una página aparte. Para un reverso físico, activa doble cara en la impresora.</p>
            <button class="btn btn-s" type="button" :disabled="printing" @click="printTest">{{ printing ? 'Preparando impresión…' : 'Imprimir página de prueba' }}</button>
          </template>
          <template v-else-if="section==='folios'">
            <div class="company-grid">
              <div class="company-field"><label for="company-prefix">Prefijo del folio</label><input id="company-prefix" v-model="draft.prefix" maxlength="12" spellcheck="false" /></div>
              <div class="company-field"><label for="company-next">Siguiente número</label><input id="company-next" v-model.number="draft.nextNumber" type="number" min="1" max="999999999" step="1" /></div>
              <div class="company-field"><label for="company-labor">Garantía de mano de obra (días)</label><input id="company-labor" v-model.number="draft.laborDays" type="number" min="0" max="3650" step="1" /></div>
              <div class="company-field"><label for="company-parts">Garantía de refacciones (días)</label><input id="company-parts" v-model.number="draft.partsDays" type="number" min="0" max="3650" step="1" /></div>
            </div>
            <div class="company-folio-example"><span>Próxima orden disponible</span><strong>{{ nextFolio }}</strong></div>
            <p class="company-help">Los folios existentes se omiten para evitar duplicados. Las órdenes anteriores conservan su número.</p>
            <p class="company-help">Las garantías se sugieren al agregarlas en Reparaciones; puedes ajustar los días para cada concepto. Un valor de 0 indica que no hay garantía sugerida.</p>
          </template>
          <template v-else-if="section==='region'">
            <div class="company-grid">
              <div class="company-field"><label for="company-country">País</label><input id="company-country" v-model="draft.country" maxlength="80" /></div>
              <SearchSelect id="company-locale" v-model="draft.locale" label="Formato regional" :options="[{value:'es-MX',label:'Español (México)'},{value:'en-US',label:'Inglés (Estados Unidos)'},{value:'es-ES',label:'Español (España)'}]" />
              <SearchSelect id="company-currency" v-model="draft.currency" label="Moneda" :options="[{value:'MXN',label:'Peso mexicano (MXN)'},{value:'USD',label:'Dólar estadounidense (USD)'},{value:'EUR',label:'Euro (EUR)'}]" />
              <SearchSelect id="company-timezone" v-model="draft.timezone" label="Zona horaria" :options="[{value:'America/Cancun',label:'Cancún'},{value:'America/Mexico_City',label:'Ciudad de México'},{value:'America/Tijuana',label:'Tijuana'},{value:'America/Hermosillo',label:'Hermosillo'},{value:'UTC',label:'UTC'}]" />
            </div>
            <p class="company-help">El formato regional ajusta fechas e importes, y mantiene la interfaz en español. Cambiar moneda no convierte los importes existentes.</p>
            <label class="company-check"><input v-model="draft.showTax" type="checkbox" /> Desglosar impuesto incluido en el total de salida</label>
            <div class="company-grid"><div class="company-field"><label for="company-tax-rate">Porcentaje del impuesto (%)</label><input id="company-tax-rate" v-model.number="draft.taxRate" type="number" min="0" max="100" step="0.01" /></div><div class="company-field"><label for="company-tax-name">Abreviatura</label><input id="company-tax-name" v-model="draft.taxName" maxlength="12" /></div></div>
            <p class="company-calculation">{{ totalsExample }}</p>
          </template>
        </div>
        <footer class="company-save-bar">
          <p v-if="clauseEditing">Aplica o cancela la cláusula antes de guardar.</p>
          <div v-if="confirmDiscard" class="company-discard-confirm" role="alert"><span>¿Descartar todos los cambios de Empresa?</span><button class="btn btn-s" type="button" @click="confirmDiscard=false">Seguir editando</button><button class="btn btn-s" type="button" @click="discard">Sí, descartar</button></div>
          <div v-else class="company-save-actions"><button class="btn btn-s" type="button" :disabled="!dirty || clauseEditing || readingLogo" @click="confirmDiscard=true">Descartar cambios</button><button class="btn btn-p" type="submit" :disabled="!dirty || clauseEditing || readingLogo">Guardar cambios</button></div>
        </footer>
        </fieldset>
      </form>
      <aside class="company-preview" aria-labelledby="company-preview-title">
        <div class="company-preview-heading"><div><h2 id="company-preview-title">Así sale en la nota</h2><p>Vista previa · {{ order?.id || 'sin orden seleccionada' }}</p></div><div class="company-note-toggle" role="group" aria-label="Nota a previsualizar"><button type="button" :aria-pressed="noteKind==='entry'" :class="{active:noteKind==='entry'}" @click="noteKind='entry'">Entrada</button><button type="button" :aria-pressed="noteKind==='exit'" :class="{active:noteKind==='exit'}" @click="noteKind='exit'">Salida</button></div></div>
        <div class="company-preview-meta"><span>{{ paperLabels[draft.paper] }} · {{ draft.paper!=='letter' ? 'Vertical' : draft.orientation==='portrait' ? 'Vertical' : 'Horizontal' }}</span><span>{{ draft.copies }} {{ draft.copies===1 ? 'copia' : 'copias' }}</span></div>
        <div ref="preview" class="company-preview-paper" :class="{'preview-landscape':draft.orientation==='landscape' && draft.paper==='letter','preview-letter':draft.paper==='letter'}"><CompanyNote ref="notePreview" :settings="draft" :kind="noteKind" :order="order" :folio="section==='folios' ? nextFolio : undefined" /></div>
        <p class="company-help">Se actualiza mientras editas. La paginación final depende del contenido y del papel; confirma los márgenes en la vista de impresión.</p>
      </aside>
    </div>
    <DraftLeaveDialog id="company-leave" area="Empresa" :open="leaveGuard.open.value" :busy="leaveGuard.busy.value" :error="leaveGuard.error.value" @save="leaveGuard.save" @discard="leaveGuard.discard" @keep="leaveGuard.keep" />
  </main>
</template>
<style src="../assets/styles/company.css"></style>
