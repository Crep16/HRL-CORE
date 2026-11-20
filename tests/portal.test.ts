import {afterEach,beforeEach,test} from 'node:test';
import assert from 'node:assert/strict';
import {createPinia,disposePinia,setActivePinia} from 'pinia';
import {mockIPC,clearMocks} from '@tauri-apps/api/mocks';
import {usePortalStore} from '../src/stores/usePortalStore.ts';
import {useAuthStore} from '../src/stores/useAuthStore.ts';
import {initialPortalConfig} from '../src/types/portal.ts';
import {toDataURL} from 'qrcode';

globalThis.window={} as Window & typeof globalThis;
let pinia:ReturnType<typeof createPinia>;
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);});
afterEach(()=>{clearMocks();disposePinia(pinia);});
const state={code:'K7QM-2XPA-9RTD',revision:1,enabled:true,state:'pending',error:null,confirmedAt:null,configured:true,paused:true};
test('portal discards a status response when the local session changes',async()=>{
  let resolve!:(value:unknown)=>void;mockIPC(()=>new Promise(r=>{resolve=r;}));
  const portal=usePortalStore(),auth=useAuthStore();const pending=portal.status(1);auth.clearSession();resolve(state);
  await assert.rejects(pending,(error:{code:string})=>error.code==='SESSION_CHANGED');assert.deepEqual(portal.statuses,{});
});
test('portal configuration draft survives a refresh and credentials are not kept in the store',async()=>{
  const config={...initialPortalConfig(),apiKey:'test-key',portalUrl:'https://portal.example.invalid'};
  let login:unknown;
  mockIPC((command,args)=>{if(command==='connect_portal'){login=args;return null;}return {config,connected:true,error:null};});
  const portal=usePortalStore();await portal.loadSettings();portal.draft.portalUrl='https://new.example.invalid';
  await portal.loadSettings();assert.equal(portal.draft.portalUrl,'https://new.example.invalid');
  await portal.connect('publisher@example.invalid','test-only-password');assert.deepEqual(login,{input:{email:'publisher@example.invalid',password:'test-only-password'}});
  assert.ok(!JSON.stringify(portal.$state).includes('test-only-password'));
});
test('portal sends the public action and keeps the confirmed status separate from a pending one',async()=>{
  mockIPC((command,args)=>{assert.equal(command,'portal_order_action');assert.deepEqual(args,{orderId:4,action:'withdraw'});return {...state,enabled:false};});
  const portal=usePortalStore();await portal.publication(4,'withdraw');assert.equal(portal.statuses[4]?.state,'pending');assert.equal(portal.statuses[4]?.enabled,false);
});
test('QR is generated locally from the public URL',async()=>{
  const data=await toDataURL('https://portal.example.invalid/seguimiento/K7QM-2XPA-9RTD',{width:256,margin:4,errorCorrectionLevel:'M'});
  assert.ok(data.startsWith('data:image/png;base64,'));assert.ok(Buffer.from(data.split(',')[1]!,'base64').length>100);
});

test('a delayed status cannot restore the previous code after rotation',async()=>{
  let resolve!:(value:unknown)=>void;
  const rotated={...state,code:'ABCD-EFGH-JKMN',revision:2};
  mockIPC(command=>command==='portal_order_status'?new Promise(r=>{resolve=r;}):rotated);
  const portal=usePortalStore(),previous=portal.status(1);
  await portal.publication(1,'rotate');resolve(state);
  await assert.rejects(previous,(error:{code:string})=>error.code==='CONFLICT');
  assert.equal(portal.statuses[1]?.code,rotated.code);
});

test('simultaneous status readers share a request and the next refresh reads again',async()=>{
  const responses:((value:unknown)=>void)[]=[];
  mockIPC(()=>new Promise(resolve=>responses.push(resolve)));
  const portal=usePortalStore(),previous=portal.status(1),latest=portal.status(1);
  assert.equal(responses.length,1);
  responses[0]!({...state,revision:2,enabled:false});
  const values=await Promise.all([previous,latest]);assert.deepEqual(values[0],values[1]);
  assert.equal(portal.statuses[1]?.state,'pending');assert.equal(portal.statuses[1]?.enabled,false);
  const refresh=portal.status(1);assert.equal(responses.length,2);
  responses[1]!({...state,revision:2,enabled:false,state:'confirmed'});await refresh;
  assert.equal(portal.statuses[1]?.state,'confirmed');
});

test('failed shared status reads can be retried instead of caching their failure',async()=>{
  let calls=0;mockIPC(()=>++calls===1?Promise.reject({code:'STORAGE_ERROR',message:'Error de prueba'}):state);
  const portal=usePortalStore();
  const results=await Promise.allSettled([portal.status(1),portal.status(1)]);
  assert.ok(results.every(result=>result.status==='rejected'));assert.equal(calls,1);
  await portal.status(1);assert.equal(calls,2);assert.equal(portal.statuses[1]?.code,state.code);
});

test('editing configuration while a refresh is pending preserves the draft',async()=>{
  const config={...initialPortalConfig(),apiKey:'test-key'};
  mockIPC(()=>({config,connected:false,error:null}));
  const portal=usePortalStore();await portal.loadSettings();
  let resolve!:(value:unknown)=>void;mockIPC(()=>new Promise(r=>{resolve=r;}));
  const refresh=portal.loadSettings();portal.draft.portalUrl='https://new.example.invalid';
  resolve({config,connected:false,error:null});await refresh;
  assert.equal(portal.draft.portalUrl,'https://new.example.invalid');assert.equal(portal.dirty,true);
});

test('an earlier settings response cannot undo a configuration saved later',async()=>{
  const old={...initialPortalConfig(),apiKey:'test-key'},saved={...old,portalUrl:'https://new.example.invalid'};
  let resolve!:(value:unknown)=>void,reads=0;
  mockIPC(command=>command==='portal_settings'?++reads===1?new Promise(r=>{resolve=r;}):{config:saved,connected:false,error:null}:null);
  const portal=usePortalStore(),previous=portal.loadSettings();portal.draft={...saved};
  await portal.configure();resolve({config:old,connected:false,error:null});
  await assert.rejects(previous,(error:{code:string})=>error.code==='CONFLICT');
  assert.equal(portal.settings?.config.portalUrl,saved.portalUrl);
});

test('a publication excludes duplicate actions and reads for the same order',async()=>{
  let resolve!:(value:unknown)=>void,calls=0;
  mockIPC(()=>{calls++;return new Promise(r=>{resolve=r;});});
  const portal=usePortalStore(),action=portal.publication(1,'withdraw');
  await assert.rejects(portal.publication(1,'rotate'),(error:{code:string})=>error.code==='CONFLICT');
  await assert.rejects(portal.status(1),(error:{code:string})=>error.code==='CONFLICT');
  assert.equal(calls,1);resolve({...state,enabled:false});await action;
  assert.equal(portal.busy,false);assert.equal(portal.statuses[1]?.enabled,false);
});
