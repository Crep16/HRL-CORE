import { createApp,nextTick,h,Fragment } from 'vue';
import { createPinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import WorkshopApp from '../src/WorkshopApp.vue';
import BackupDialog from '../src/components/backup/BackupDialog.vue';
import { useAuthStore } from '../src/stores/useAuthStore';
import { useWorkspaceStore } from '../src/stores/useWorkspaceStore';
import { useCompanyStore } from '../src/stores/useCompanyStore';
import { defaultCompany } from '../src/composables/useCompany';
import '../src/assets/styles/tailwind.css';
import type { AccessScope,EditOrderPayload,OrderResponse } from '../src/types/ipc';

const order:OrderResponse={id:1,folio:'ORD-001',clientId:1,equipmentId:1,receivedBy:1,status:'En diagnóstico',currency:'MXN',receivedAt:'2026-11-05',estimatedDeliveryAt:null,reportedFailure:'El equipo no enciende.',plannedWork:'Revisar alimentación y realizar diagnóstico.',receptionNotes:'Se recibe con cargador.',priceMinor:150000,netMinor:50000,pendingMinor:100000,creditMinor:0,hasDevicePassword:false,client:{id:1,fullName:'Cliente de prueba',phone:'0000000000',email:null},equipment:{id:1,item:'Laptop',brand:'Marca',model:'Modelo',serial:'SERIE-PRUEBA'},history:{previousWorkshop:false,previousWorkshopDetail:'',sufferedImpact:false,impactDetail:'',liquidSpill:false,liquidDetail:''},revision:1,delivered:false,deliveredAt:null};
const settings={...defaultCompany(),name:'Taller de prueba',phone:'0000000000',nextNumber:2,revision:1};
const payment={totalMinor:150000,collectedMinor:50000,receivedMinor:50000,adjustedMinor:0,refundedMinor:0,creditMinor:0,paid:false};
const paymentRequests: unknown[] = [];
const draftGuardTest = { failSave: false, saveCalls: 0, logoutCalls: 0 };
Object.assign(window, { draftGuardTest });
Object.assign(window, { paymentRequests });
const publicSummary={technician:'Técnico de prueba',publicNote:'Diagnóstico en proceso.',warranties:[]};
const repair={stages:['Recibido','Diagnóstico','Entregado'],completed:1,...publicSummary,internalNote:'Revisar con fuente de laboratorio.',syncMode:'manual',events:[{id:1,date:'2026-11-05T10:00',title:'Equipo recibido',detail:'Se recibe con cargador.',stage:'Recibido',public:true}],parts:[],payments:[],visibility:[{label:'Línea de tiempo',enabled:true},{label:'Garantías',enabled:true},{label:'Resumen de pago',enabled:false},{label:'Notas del técnico',enabled:true}]};
const summary={id:1,folio:order.folio,clientName:order.client.fullName,serial:order.equipment.serial,item:order.equipment.item,status:order.status,receivedAt:order.receivedAt,currency:order.currency,priceMinor:order.priceMinor,netMinor:order.netMinor,pendingMinor:order.pendingMinor,creditMinor:0,delivered:false};
const photo={id:'11112222333344445555666677778888',orderId:'1',originalName:'foto-de-prueba.png',mime:'image/png',sizeBytes:68,source:'file',note:'Recepción',authorName:'Operador de prueba',createdAt:'2026-11-05T15:00:00Z',deleted:false,revision:1};
const photoBytes=Array.from(Uint8Array.from(atob('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ1sAAAAASUVORK5CYII='),c=>c.charCodeAt(0)));
const backups={folder:'Carpeta de pruebas / Respaldos',automatic:false,files:[] as {name:string;bytes:number}[]};
mockIPC((command,args)=>{
  if(command==='logout_user'){draftGuardTest.logoutCalls++;return null;}
  if(command==='save_normalized_company'){
    draftGuardTest.saveCalls++;
    if(draftGuardTest.failSave)throw {code:'CONFLICT',message:'La configuración cambió. Revisa el borrador.'};
    const {taxRateThousandths,...value}=(args as {payload:{value:typeof settings & {taxRateThousandths:number}}}).payload.value;
    Object.assign(settings,value,{taxRate:taxRateThousandths/1000,revision:settings.revision+1});
    return {...value,taxRateThousandths,revision:settings.revision};
  }
  if(['backup_status','create_backup','configure_backup','open_backup_folder'].includes(command)){
    const payload=(args as {payload:{masterPassword:string;automatic?:boolean}}).payload;
    if(payload.masterPassword!=='maestra-de-prueba')throw {code:'INVALID_CREDENTIALS',message:'Usuario o clave incorrectos.'};
    if(command==='backup_status')return structuredClone(backups);
    if(command==='create_backup'){const name='HRL-20261114-200000-prueba.hrlbackup';backups.files.push({name,bytes:4096});return name;}
    if(command==='configure_backup')backups.automatic=!!payload.automatic;
    return null;
  }
  if(command==='restore_backup')return true;
  if(command==='restart_after_restore')return null;
  if(command==='load_normalized_company'){const {taxRate,...value}=settings;return {...value,taxRateThousandths:taxRate*1000};}
  if(command==='list_reception_orders')return {items:[summary],total:1};
  if(command==='get_order'||command==='find_reception_order')return structuredClone(order);
  if(command==='edit_normalized_order'){
    const payload=(args as {payload:EditOrderPayload}).payload;
    if(payload.revision!==order.revision)throw {code:'CONFLICT',message:'La orden cambió. Actualiza la ficha.'};
    order.history=payload.value.history;order.plannedWork=payload.value.plannedWork;order.revision++;
    return structuredClone(order);
  }
  if(command==='normalized_repair_summary')return publicSummary;
  if(command==='load_normalized_repair')return {repair,revision:1,payment};
  if(command==='list_normalized_stages')return repair.stages;
  if(command==='list_normalized_payments')return {movements:[{id:'1',kind:'receipt',paymentId:null,amountMinor:50000,effectiveMinor:50000,method:'Efectivo',reason:null,occurredAt:'2026-11-05T15:00:00Z'}],payment};
  if(command==='record_normalized_payment'){
    const request=(args as {payload:{amountMinor:number;kind:string;expectedMinor:number}}).payload;
    paymentRequests.push(structuredClone(request));
    if(request.expectedMinor!==payment.collectedMinor)throw {code:'CONFLICT',message:'El saldo cambió. Actualiza los movimientos.'};
    payment.collectedMinor+=request.amountMinor;payment.receivedMinor+=request.amountMinor;
    order.netMinor=payment.collectedMinor;order.pendingMinor=order.priceMinor-order.netMinor;
    return structuredClone(payment);
  }
  if(command==='list_normalized_users')return [{id:1,name:'admin',role:'administrator',accessLevel:'complete',active:true,permissions:[]}];
  if(command==='list_normalized_logbook')return {entries:[{id:1,entity:'ordenes',entityId:'1',orderId:order.folio,authorId:'1',authorName:'Administrador de prueba',operation:'orders.create',details:[{field:'client',oldValue:null,newValue:order.client.fullName}],occurredAt:'2026-11-05T15:00:00Z',eventDate:null,currency:'MXN',reason:null,result:'success'}],total:1,page:0,pageSize:50,authors:[{id:'1',name:'Administrador de prueba'}]};
  if(command==='list_normalized_finances')return [{id:order.folio,databaseId:1,client:order.client.fullName,equipment:'Laptop Marca Modelo',status:order.status,currency:'MXN',totalMinor:150000,collectedMinor:50000,receivedMinor:50000,adjustedMinor:0,refundedMinor:0,creditMinor:0,balanceMinor:100000,costsMinor:0,profitMinor:150000,delivered:false,paid:false,classification:'pending',excluded:false,received:order.receivedAt,deliveredAt:null,settledAt:null,payments:[{amountMinor:50000,occurredAt:'2026-11-05T15:00:00Z',kind:'receipt'}],expenses:[]}];
  if(command==='list_normalized_evidence')return photo.deleted===Boolean((args as {deleted:boolean}).deleted)?[structuredClone(photo)]:[];
  if(command==='read_normalized_evidence')return photoBytes;
  if(command==='update_normalized_evidence'){
    const payload=(args as {payload:{note:string;deleted:boolean;expectedRevision:number}}).payload;
    if(payload.expectedRevision!==photo.revision)throw {code:'CONFLICT',message:'La foto cambió. Actualiza la galería.'};
    photo.note=payload.note;photo.deleted=payload.deleted;photo.revision++;
    return structuredClone(photo);
  }
  if(command==='finish_temporary_access')return null;
  throw {code:'VALIDATION_ERROR',message:`Comando de prueba no configurado: ${command} ${JSON.stringify(args)}`};
});
const pinia=createPinia(),auth=useAuthStore(pinia),company=useCompanyStore(pinia);
auth.user={id:1,name:'Administrador de prueba',loginName:'admin',role:'administrator',accessLevel:'complete',active:true,createdAt:'2026-11-05'};
const testLevel=new URLSearchParams(location.search).get('level');
if(testLevel==='limited'||testLevel==='custom')auth.user.accessLevel=testLevel;
auth.permissions=['clients.view','orders.create','orders.edit','orders.delete','repairs.view','repairs.edit','notes.add','stages.advance','parts.add','warranties.add','publication.edit','company.view','company.edit','users.manage','logbook.view','payments.add','payments.adjust','payments.refund','evidence.view','evidence.add','evidence.edit','evidence.delete'];
company.settings=settings;company.loaded=true;
createApp({render:()=>h(Fragment,null,[h(WorkshopApp),h(BackupDialog)])}).use(pinia).mount('#app');
await nextTick();
await new Promise(resolve=>setTimeout(resolve,250));
const page=new URLSearchParams(location.search).get('view')??'clients';
if(['clients','repairs','company','users','logbook'].includes(page))useWorkspaceStore(pinia).page=page as AccessScope;
await nextTick();
document.documentElement.dataset.testReady='true';
