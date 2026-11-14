import assert from 'node:assert/strict';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import vue from '@vitejs/plugin-vue';
import { createSSRApp,h } from 'vue';
import { renderToString } from 'vue/server-renderer';

test('order refresh preserves selection, joins pending requests and uses the canonical saved payment state',async()=>{
 const previousWindow=globalThis.window,previousTauri=globalThis.isTauri;
 const previousStorage=Object.getOwnPropertyDescriptor(globalThis,'localStorage');
 const base={id:'A',client:'Prueba',phone:'123',received:'2026-10-28',status:'Listo para entrega',item:'Laptop',serial:'',brand:'',model:'',otherShop:false,otherShopReason:'',dropped:false,dropReason:'',password:'',work:'',cost:'750',deposit:'300',notes:'',paid:false,delivered:false,currency:'MXN'};
 let paid=false,resolveList=null,listCalls=0,hold=false,fail=false;
 globalThis.isTauri=true;
 Object.defineProperty(globalThis,'localStorage',{configurable:true,value:{getItem:()=>null}});
 globalThis.window={__TAURI_INTERNALS__:{async invoke(command){
  if(command==='access_status')return {setupComplete:true,user:{id:'qa',name:'Prueba',accessLevel:'complete'}};
  if(command==='access_permissions')return ['clients.view','orders.create','orders.edit','orders.delete'];
  if(command==='list_orders'){listCalls++;if(hold)await new Promise(resolve=>resolveList=resolve);if(fail)throw 'Error de consulta';return {orders:[{...base,paid},{...base,id:'B',paid:false}],legacyImported:true};}
  if(command==='save_order')return {...base,paid:true};
  throw new Error(command);
 }}};
 const server=await createServer({root:fileURLToPath(new URL('..',import.meta.url)),configFile:false,plugins:[vue()],server:{middlewareMode:true,hmr:false},appType:'custom',logLevel:'error'});
 try{
  const {useAccess}=await server.ssrLoadModule('/src/composables/useAccess.ts');await useAccess().refresh();
  const {useOrders}=await server.ssrLoadModule('/src/composables/useOrders.ts');let state;
  await renderToString(createSSRApp({setup(){state=useOrders();return ()=>h('div');}}));
  assert.equal(await state.load(),true);state.selectedId.value='A';
  hold=true;paid=true;const first=state.refresh(),second=state.refresh();
  assert.equal(first,second);assert.equal(state.selected.value.id,'A');assert.equal(state.orders.value.length,2);assert.equal(state.selected.value.paid,false);
  resolveList();assert.equal(await first,true);assert.equal(state.selected.value.paid,true);assert.equal(state.selectedId.value,'A');assert.equal(listCalls,2);
  hold=false;assert.equal(await state.saveOrder({...base,paid:false},{...base,paid:true}),true);assert.equal(state.selected.value.paid,true);
  fail=true;assert.equal(await state.refresh(),false);assert.equal(state.selected.value.id,'A');assert.equal(state.ready.value,false);assert.equal(state.storageError.value,'Error de consulta');
 }finally{await server.close();if(previousWindow===undefined)delete globalThis.window;else globalThis.window=previousWindow;if(previousTauri===undefined)delete globalThis.isTauri;else globalThis.isTauri=previousTauri;if(previousStorage)Object.defineProperty(globalThis,'localStorage',previousStorage);else delete globalThis.localStorage;}
});
