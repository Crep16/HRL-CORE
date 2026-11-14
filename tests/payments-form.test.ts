import { afterEach, beforeEach, test } from 'node:test';
import assert from 'node:assert/strict';
import { createSSRApp, reactive } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { createPinia, disposePinia, setActivePinia } from 'pinia';
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { usePaymentsPanel } from '../src/composables/usePaymentsPanel.ts';
import { useAuthStore } from '../src/stores/useAuthStore.ts';
import type { PaymentState, MoneyRequest } from '../src/types/payments.ts';

globalThis.window = {} as Window & typeof globalThis;
const pending = new Map<string, string>();
Object.defineProperty(globalThis, 'localStorage', {configurable:true,value:{
  getItem:(key:string)=>pending.get(key)??null,
  setItem:(key:string,value:string)=>pending.set(key,value),
  removeItem:(key:string)=>pending.delete(key),
}});
let pinia: ReturnType<typeof createPinia>;
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);pending.clear();});
afterEach(()=>{clearMocks();disposePinia(pinia);});

async function panel(record:(request:MoneyRequest)=>PaymentState|Promise<PaymentState>){
  const state:PaymentState={totalMinor:150000,collectedMinor:50000,receivedMinor:50000,adjustedMinor:0,refundedMinor:0,creditMinor:0,paid:false};
  const props=reactive({orderId:'1',currency:'MXN',payment:state,disabled:false,prepare:async()=>true});
  const saved:PaymentState[]=[];
  mockIPC((command,args)=>{
    if(command==='list_normalized_payments')return {movements:[],payment:props.payment};
    assert.equal(command,'record_normalized_payment');
    return record((args as {payload:MoneyRequest}).payload);
  });
  const auth=useAuthStore();
  auth.user={id:1,name:'Prueba',loginName:'admin',role:'administrator',accessLevel:'complete',active:true,createdAt:'2026-11-14'};
  auth.permissions=['payments.add'];
  let model!:ReturnType<typeof usePaymentsPanel>;
  await renderToString(createSSRApp({setup(){
    model=usePaymentsPanel(props,((event:string,value:PaymentState)=>{
      if(event==='saved'){props.payment=value;saved.push(value);}
    }) as Parameters<typeof usePaymentsPanel>[1]);
    return ()=>null;
  }}).use(pinia));
  await model.load();
  await model.open('receipt');
  model.method.value='Efectivo';
  return {model,props,saved,state};
}

test('payment confirmation accepts decimal text and the numeric value emitted by number inputs',async()=>{
  const requests:MoneyRequest[]=[];
  const {model,state,saved}=await panel(request=>{requests.push(request);return {...state,collectedMinor:50000+request.amountMinor};});
  model.amount.value=100.5 as unknown as string;
  await model.submit();
  assert.equal(requests[0].amountMinor,10050);
  assert.equal(saved.length,1);
  assert.equal(model.error.value,'');
  assert.equal(pending.size,0);
  await model.open('receipt');model.amount.value='20,05';model.method.value='Transferencia';
  await model.submit();
  assert.equal(requests[1].amountMinor,2005);
});

test('invalid and excessive payments show a message without sending a financial request',async()=>{
  let calls=0;
  const {model,state}=await panel(()=>{calls++;return state;});
  for(const value of ['', '0','-1','1000.001','1e3','1,000.00','1000.01']){
    model.amount.value=value;await model.submit();
    assert.ok(model.error.value,value);assert.equal(calls,0);assert.equal(pending.size,0);
  }
});

test('an uncertain payment retries its exact identifier and amount after the form is changed',async()=>{
  const requests:MoneyRequest[]=[];
  const {model,state}=await panel(request=>{
    requests.push(structuredClone(request));
    if(requests.length===1)throw {code:'RESULT_UNCERTAIN',message:'Resultado incierto de prueba'};
    return {...state,collectedMinor:60050};
  });
  model.amount.value='100.50';await model.submit();
  assert.equal(model.hasPending.value,true);
  model.amount.value='500';model.method.value='Otro método';
  await model.retryPending();
  assert.deepEqual(requests[1],requests[0]);assert.equal(pending.size,0);
  assert.equal(model.hasPending.value,false);
});

test('a definitive rejection shows the backend explanation and unlocks a corrected attempt',async()=>{
  const {model}=await panel(()=>{throw {code:'CONFLICT',message:'El saldo cambió. Actualiza los movimientos.'};});
  model.amount.value='100';await model.submit();
  assert.equal(model.error.value,'El saldo cambió. Actualiza los movimientos.');
  assert.equal(model.attempted.value,false);assert.equal(model.hasPending.value,false);
  assert.equal(pending.size,0);
});
