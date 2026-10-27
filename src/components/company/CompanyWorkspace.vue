<script setup lang="ts">
import { useAccess } from '../../composables/useAccess';
const { can } = useAccess();
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import CompanyClauses from './CompanyClauses.vue';
import CompanyNote from './CompanyNote.vue';
import brandLogo from '../../assets/images/brand/hrl-original.svg';
import { cloneCompany, companyErrors, formatFolio, nextAvailableNumber, useCompany } from '../../composables/useCompany';
import { printCompanyNote } from '../../utils/printCompanyNote';
import type { CompanySection, NoteKind } from '../../types/company';
import { paperLabels } from '../../types/company';
import type { Order } from '../../types/order';

const props = defineProps<{orders: Order[]; order?: Order}>();
const emit = defineEmits<{ dirtyChange: [value: boolean] }>();
const {company, companyStorageError, saveCompany} = useCompany();
const draft = ref(cloneCompany(company.value));
const section = ref<CompanySection>('general');
const noteKind = ref<NoteKind>('entry');
const clauseEditing = ref(false);
const message = ref('');
const errors = ref<string[]>([]);
const errorTargets: {pattern:RegExp;section:CompanySection;id:string}[] = [
  {pattern:/nombre/,section:'general',id:'company-name'}, {pattern:/correo/,section:'general',id:'company-email'},
  {pattern:/página/,section:'general',id:'company-website'}, {pattern:/prefijo/,section:'folios',id:'company-prefix'},
  {pattern:/siguiente número/,section:'folios',id:'company-next'}, {pattern:/copias/,section:'paper',id:'company-copies'},
  {pattern:/mano de obra/,section:'folios',id:'company-labor'}, {pattern:/refacciones/,section:'folios',id:'company-parts'},
  {pattern:/impuesto debe/,section:'region',id:'company-tax-rate'}, {pattern:/abreviatura/,section:'region',id:'company-tax-name'},
  {pattern:/mensajes/,section:'notes',id:'entry-footer'}, {pattern:/cláusula/,section:'clauses',id:'company-section-title'},
];
const fieldError = (id:string) => errors.value.find(error=>errorTargets.find(target=>target.pattern.test(error))?.id===id);
function focusError(error:string) {
  const target=errorTargets.find(target=>target.pattern.test(error));
  if(!target) return;
  section.value=target.section;
  nextTick(()=>document.getElementById(target.id)?.focus());
}
const uploadError = ref('');
const readingLogo = ref(false);
const printing = ref(false);
const confirmDiscard = ref(false);
const preview = ref<HTMLElement>();
const errorSummary = ref<HTMLElement>();
const fileInput = ref<HTMLInputElement>();
const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(company.value));
const hasPending = computed(() => dirty.value || clauseEditing.value || readingLogo.value);
watch(hasPending, value => emit('dirtyChange',value),{immediate:true});
watch(dirty, () => {message.value = ''; confirmDiscard.value = false;});
const nextNumber = computed(() => nextAvailableNumber(draft.value, props.orders));
const nextFolio = computed(() => Number.isInteger(nextNumber.value) && nextNumber.value > 0 ? formatFolio(draft.value.prefix,nextNumber.value) : 'Revisa el siguiente número');
const activeClauses = computed(() => draft.value.clauses.filter(c=>c.entry||c.exit).length);
const sections = computed(() => [
  {id:'general',title:'Datos generales',detail:draft.value.name || 'Nombre y contacto'},
  {id:'logo',title:'Logo',detail:draft.value.logoPrint || draft.value.logoWeb ? draft.value.logoName || 'Logo HRL' : 'No se muestra'},
  {id:'notes',title:'Notas impresas',detail:'Mensajes y firma'},
  {id:'clauses',title:'Cláusulas',detail:`${draft.value.clauses.length} textos, ${activeClauses.value} activos`},
  {id:'paper',title:'Hoja de impresión',detail:`${paperLabels[draft.value.paper]} · ${draft.value.copies || 0} copias`},
  {id:'folios',title:'Folios y garantías',detail:`Siguiente: ${nextFolio.value}`},
  {id:'region',title:'Región e impuestos',detail:`${draft.value.currency} · ${draft.value.taxName} ${draft.value.taxRate || 0} %`},
] as const);
const current = computed(() => sections.value.find(item=>item.id===section.value)!);
const descriptions: Record<CompanySection,string> = {
  general:'Identificación y contacto que aparecen en tus notas.', logo:'Imagen de la empresa y lugares donde se muestra.',
  notes:'Mensajes y datos que acompañan cada nota.', clauses:'Condiciones del servicio, orden y notas donde se imprimen.',
  paper:'Tamaño, orientación y copias de tus notas.', folios:'Numeración de nuevas órdenes y garantías sugeridas.', region:'Formato de importes, zona horaria e impuesto incluido en el total.',
};
const totalsExample = computed(() => {
  const rate=Number(draft.value.taxRate) || 0;
  const money=(n:number)=>new Intl.NumberFormat(draft.value.locale,{style:'currency',currency:draft.value.currency}).format(n);
  return `De un total de ${money(1000)}, el subtotal es ${money(1000/(1+rate/100))} y ${draft.value.taxName} es ${money(1000-1000/(1+rate/100))}.`;
});
function goTo(id:CompanySection) {
  if (clauseEditing.value || readingLogo.value) {message.value='Aplica o cancela la edición actual antes de cambiar de sección.'; return;}
  section.value=id; errors.value=[]; message.value='';
  nextTick(()=>document.getElementById('company-section-title')?.focus());
}
function selectSection(event:Event) {
  const select=event.target as HTMLSelectElement;
  goTo(select.value as CompanySection);
  select.value=section.value;
}
function save() {
  if (!can('company.edit')) { message.value = 'Tu acceso permite consultar Empresa, pero no modificarla.'; return; }
  errors.value=companyErrors(draft.value);
  if (errors.value.length) {nextTick(()=>errorSummary.value?.focus()); return;}
  if (saveCompany(draft.value)) {draft.value=cloneCompany(company.value); message.value='Configuración guardada en este equipo.';}
}
function discard() {
  draft.value=cloneCompany(company.value); errors.value=[]; uploadError.value=''; confirmDiscard.value=false;
  message.value='Se restauró la última configuración guardada.';
}
async function readLogo(file:Blob, name:string) {
  uploadError.value='';
  if (!['image/png','image/jpeg'].includes(file.type)) {uploadError.value='Elige una imagen PNG o JPG.';return;}
  if (file.size>1024*1024) {uploadError.value='La imagen supera 1 MB. Elige una versión más ligera.';return;}
  readingLogo.value=true;
  try {
    const url=await new Promise<string>((resolve,reject)=>{const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));reader.onerror=reject;reader.readAsDataURL(file);});
    const image=new Image(); image.src=url; await image.decode();
    if (image.naturalWidth>10000 || image.naturalHeight>10000) throw new Error();
    draft.value.logo=url; draft.value.logoName=`${name} · ${image.naturalWidth} × ${image.naturalHeight} px`; draft.value.logoPrint=true; draft.value.logoWeb=true;
  } catch {uploadError.value='No se pudo leer la imagen. Prueba con otro archivo PNG o JPG.';}
  finally {readingLogo.value=false; if(fileInput.value) fileInput.value.value='';}
}
function upload(event:Event) {const file=(event.target as HTMLInputElement).files?.[0];if(file) void readLogo(file,file.name);}
async function pasteLogo() {
  uploadError.value='';
  try {
    if (!navigator.clipboard?.read) throw new Error();
    const items=await navigator.clipboard.read();
    for (const item of items) {const type=item.types.find(t=>['image/png','image/jpeg'].includes(t));if(type) {await readLogo(await item.getType(type),'Imagen del portapapeles');return;}}
    uploadError.value='No hay una imagen PNG o JPG en el portapapeles.';
  } catch {uploadError.value='No se pudo acceder al portapapeles. Usa Subir archivo.';}
}
async function printTest() {
  const note=preview.value?.querySelector<HTMLElement>('.company-note');
  if (!note) return;
  errors.value=companyErrors(draft.value);
  if(errors.value.length) {nextTick(()=>errorSummary.value?.focus());return;}
  printing.value=true; message.value='';
  try {await printCompanyNote(note,draft.value);message.value='Se solicitó el diálogo de impresión. Elige ahí la impresora o Guardar como PDF.';}
  catch(error) {message.value=error instanceof Error ? error.message : 'No se pudo preparar la impresión.';}
  finally {printing.value=false;}
}
function preventLeave(event:BeforeUnloadEvent) {if(hasPending.value) {event.preventDefault();event.returnValue='';}}
window.addEventListener('beforeunload',preventLeave);
onBeforeUnmount(()=>{window.removeEventListener('beforeunload',preventLeave);emit('dirtyChange',false);});
</script>

<template>
  <main class="company-workspace" aria-labelledby="company-title">
    <header class="company-heading"><div><h1 id="company-title">Empresa</h1><p>Configura tus datos y el formato de las notas del taller.</p></div><span class="company-save-state" :class="{pending:hasPending}" role="status">{{ hasPending ? 'Cambios sin guardar' : 'Sin cambios pendientes' }}</span></header>
    <p v-if="companyStorageError" class="company-error" role="alert">{{ companyStorageError }}</p>
    <div v-if="message" class="company-feedback" role="status"><span>{{ message }}</span><button class="company-icon-button" type="button" aria-label="Cerrar mensaje" @click="message=''">×</button></div>
    <div class="company-layout">
      <div class="company-mobile-nav company-field"><label for="company-section-select">Sección de Empresa</label><select id="company-section-select" :value="section" @change="selectSection"><option v-for="item in sections" :key="item.id" :value="item.id">{{ item.title }}</option></select></div>
      <nav class="company-nav" aria-label="Configuración de Empresa">
        <button v-for="item in sections" :key="item.id" type="button" :class="{active:section===item.id}" :aria-current="section===item.id ? 'page' : undefined" @click="goTo(item.id)"><strong>{{ item.title }}</strong><span>{{ item.detail }}</span></button>
        <p>Guardado en este equipo.<br />Web pendiente de conectar.</p>
      </nav>
      <form class="company-panel" novalidate aria-labelledby="company-section-title" @submit.prevent="save">
        <fieldset class="company-permission-fields" :disabled="!can('company.edit')">
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
            <div class="company-field"><label for="company-paper">Tamaño del papel</label><select id="company-paper" v-model="draft.paper"><option v-for="(label,value) in paperLabels" :key="value" :value="value">{{ label }}</option></select></div>
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
              <div class="company-field"><label for="company-locale">Formato regional</label><select id="company-locale" v-model="draft.locale"><option value="es-MX">Español (México)</option><option value="en-US">Inglés (Estados Unidos)</option><option value="es-ES">Español (España)</option></select></div>
              <div class="company-field"><label for="company-currency">Moneda</label><select id="company-currency" v-model="draft.currency"><option value="MXN">Peso mexicano (MXN)</option><option value="USD">Dólar estadounidense (USD)</option><option value="EUR">Euro (EUR)</option></select></div>
              <div class="company-field"><label for="company-timezone">Zona horaria</label><select id="company-timezone" v-model="draft.timezone"><option value="America/Cancun">Cancún</option><option value="America/Mexico_City">Ciudad de México</option><option value="America/Tijuana">Tijuana</option><option value="America/Hermosillo">Hermosillo</option><option value="UTC">UTC</option></select></div>
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
        <div ref="preview" class="company-preview-paper" :class="{'preview-landscape':draft.orientation==='landscape' && draft.paper==='letter','preview-letter':draft.paper==='letter'}"><CompanyNote :settings="draft" :kind="noteKind" :order="order" :folio="section==='folios' ? nextFolio : undefined" /></div>
        <p class="company-help">Se actualiza mientras editas. La paginación final depende del contenido y del papel; confirma los márgenes en la vista de impresión.</p>
      </aside>
    </div>
  </main>
</template>
<style src="../../assets/styles/company.css"></style>
