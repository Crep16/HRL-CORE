import {test,beforeEach,afterEach,mock} from 'node:test';
import assert from 'node:assert/strict';
import {createSSRApp,nextTick} from 'vue';
import {renderToString} from 'vue/server-renderer';
import {createPinia,setActivePinia,disposePinia} from 'pinia';
import {mockIPC,clearMocks} from '@tauri-apps/api/mocks';
import {useAuthStore} from '../src/stores/useAuthStore.ts';
import {useBackupStore} from '../src/stores/useBackupStore.ts';
import {useBackupClose} from '../src/composables/useBackupClose.ts';
import {backupApi} from '../src/services/api/backups.ts';
import {windowApi} from '../src/services/api/window.ts';
globalThis.window={} as Window & typeof globalThis;
let pinia:ReturnType<typeof createPinia>;
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);});
afterEach(()=>{clearMocks();mock.restoreAll();disposePinia(pinia);});
function account(level:'complete'|'limited'|'custom'){
 useAuthStore().user={id:1,name:'Prueba',loginName:'admin',role:'administrator',accessLevel:level,active:true,createdAt:'2026-11-14'};
}
test('only complete access opens backups and changing session removes its master password and status',async()=>{
 const model=useBackupStore();for(const level of ['limited','custom'] as const){account(level);model.show();assert.equal(model.open,false);}
 account('complete');model.show();assert.equal(model.open,true);
 mockIPC((command,args)=>{assert.equal(command,'backup_status');assert.deepEqual(args,{payload:{masterPassword:'maestra-de-prueba'}});return {folder:'Carpeta de prueba',automatic:false,files:[]};});
 model.masterPassword='maestra-de-prueba';await model.unlock();assert.equal(model.unlocked,true);
 useAuthStore().clearSession();await nextTick();assert.equal(model.open,false);assert.equal(model.masterPassword,'');assert.equal(model.status,undefined);
 model.show(true);assert.equal(model.open,false);
 account('complete');model.show();model.masterPassword='maestra-de-prueba';
 mockIPC(()=>{throw {code:'UNAUTHENTICATED',message:'Sesión revocada'};});
 await model.unlock();assert.equal(useAuthStore().user,null);assert.equal(model.masterPassword,'');assert.equal(model.open,false);
});
test('backup form exposes a failed operation and manual creation refreshes its confirmed file list',async()=>{
 account('complete');const model=useBackupStore();model.show();model.masterPassword='maestra-de-prueba';let created=false;
 mockIPC(command=>{
  if(command==='backup_status')return {folder:'Prueba',automatic:false,files:created?[{name:'copia.hrlbackup',bytes:100}]:[]};
  assert.equal(command,'create_backup');if(!created){created=true;throw {code:'VALIDATION_ERROR',message:'No hay espacio'};}return 'copia.hrlbackup';
 });
 await model.unlock();await model.create();assert.equal(model.error,'No hay espacio');assert.equal(model.busy,false);
 await model.create();assert.equal(model.status?.files.length,1);assert.match(model.notice,/copia.hrlbackup/);
 model.close();assert.equal(model.masterPassword,'');
});
async function closeModel(prepare:()=>Promise<boolean>){
 let model!:ReturnType<typeof useBackupClose>;
 await renderToString(createSSRApp({setup(){model=useBackupClose(prepare);return ()=>null;}}).use(pinia));return model;
}
test('closing waits for the confirmed automatic backup before destroying the window',async()=>{
 const calls:string[]=[];let finish!:(value:string)=>void;
 mock.method(backupApi,'automatic',()=>{calls.push('backup');return new Promise<string>(resolve=>{finish=resolve;});});
 mock.method(windowApi,'finishClose',async()=>{calls.push('close');});
 const model=await closeModel(async()=>true);const pending=model.close();await Promise.resolve();
 assert.deepEqual(calls,['backup']);assert.equal(model.closing.value,true);
 await model.close();assert.deepEqual(calls,['backup']);
 finish('copia.hrlbackup');await pending;assert.deepEqual(calls,['backup','close']);
});
test('failed backup keeps the window open and discarding the new copy is explicit',async()=>{
 let closes=0;mock.method(backupApi,'automatic',async()=>{throw {code:'VALIDATION_ERROR',message:'Disco lleno'};});
 mock.method(windowApi,'finishClose',async()=>{closes++;});
 const model=await closeModel(async()=>true);await model.close();assert.equal(closes,0);assert.equal(model.error.value,'Disco lleno');
 await model.withoutBackup();assert.equal(closes,1);
});
test('draft guards prevent both backup and closing until the draft is resolved',async()=>{
 let calls=0;mock.method(backupApi,'automatic',async()=>{calls++;return null;});mock.method(windowApi,'finishClose',async()=>{calls++;});
 const model=await closeModel(async()=>false);await model.close();await model.withoutBackup();assert.equal(calls,0);assert.match(model.error.value,/cambios/);
});
