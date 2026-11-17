import { useWorkshopAccess } from './useWorkshopAccess.ts';
import { computed, nextTick, onBeforeUnmount, ref, watch, useTemplateRef } from 'vue';
import CompanyNote from '../components/company/CompanyNote.vue';
import { cloneCompany, companyErrors, formatFolio, nextAvailableNumber, useCompany } from './useCompany.ts';
import { printCompanyNote } from '../utils/printCompanyNote.ts';
import type { CompanySection, NoteKind } from '../types/company.ts';
import { paperLabels } from '../types/company.ts';
import { useDraftLeaveGuard } from './useDraftLeaveGuard.ts';
import type { Order } from '../types/order.ts';
export function useCompanyWorkspace(props: { orders: Order[]; order?: Order }, emit: (event: 'dirtyChange', value: boolean) => void) {
const { can } = useWorkshopAccess();
const {company, persistCompany, companyStorageError} = useCompany();
const draft = ref(cloneCompany(company.value));
const section = ref<CompanySection>('general');
const noteKind = ref<NoteKind>('entry');
const clauseEditing = ref(false);
const message = ref('');
const saving = ref(false);
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
const preview = useTemplateRef<HTMLElement>('preview');
const notePreview = useTemplateRef<InstanceType<typeof CompanyNote>>('notePreview');
const errorSummary = useTemplateRef<HTMLElement>('errorSummary');
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
async function save() {
  if(saving.value || clauseEditing.value || readingLogo.value)return false;
  if (!can('company.edit')) { message.value = 'Tu acceso permite consultar Empresa, pero no modificarla.'; return false; }
  errors.value=companyErrors(draft.value);
  if (errors.value.length) {nextTick(()=>errorSummary.value?.focus()); return false;}
  saving.value=true;
  try{if (await persistCompany(draft.value)) {draft.value=cloneCompany(company.value); message.value='Configuración e historial guardados en este equipo.';return true;} else { message.value=companyStorageError.value;return false; }}
  finally{saving.value=false;}
}
function discard() {
  draft.value=cloneCompany(company.value); errors.value=[]; uploadError.value=''; confirmDiscard.value=false;
  message.value='Se restauró la última configuración guardada.';
}
const leaveGuard = useDraftLeaveGuard({save, discard});
async function prepareLeave() {
  if (clauseEditing.value || readingLogo.value || saving.value || printing.value) {
    message.value = clauseEditing.value ? 'Aplica o cancela la cláusula antes de salir de Empresa.' : 'Espera a que termine la operación antes de salir de Empresa.';
    return false;
  }
  return !dirty.value || await leaveGuard.request();
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
  try {await notePreview.value?.prepare();await nextTick();await printCompanyNote(note,draft.value);message.value='Se solicitó el diálogo de impresión. Elige ahí la impresora o Guardar como PDF.';}
  catch(error) {message.value=error instanceof Error ? error.message : 'No se pudo preparar la impresión.';}
  finally {printing.value=false;}
}
function preventLeave(event:BeforeUnloadEvent) {if(hasPending.value) {event.preventDefault();event.returnValue='';}}
window.addEventListener('beforeunload',preventLeave);
onBeforeUnmount(()=>{window.removeEventListener('beforeunload',preventLeave);emit('dirtyChange',false);});
return { leaveGuard, prepareLeave, can, company, draft, section, noteKind, clauseEditing, message, saving, errors, fieldError, focusError, uploadError, readingLogo, printing, confirmDiscard, preview, notePreview, errorSummary, fileInput, dirty, hasPending, nextNumber, nextFolio, sections, current, descriptions, totalsExample, goTo, save, discard, upload, pasteLogo, printTest };
}
