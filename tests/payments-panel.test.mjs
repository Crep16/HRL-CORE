import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync, writeFileSync, unlinkSync } from 'node:fs';
import { parse, compileScript } from '@vue/compiler-sfc';
import ts from 'typescript';
import { createRenderer, h, ref, nextTick } from 'vue';

test('refund-only user retries the identical pending refund after a lost response and remount', async()=>{
 const {descriptor}=parse(readFileSync(new URL('../src/components/repairs/PaymentsPanel.vue',import.meta.url),'utf8'));
 const script=compileScript(descriptor,{id:'qa-payments',inlineTemplate:true});
 const source=script.content
  .replace(/import \{ invoke \} from [^;]+;/,"const invoke=(command,args)=>globalThis.moneyTest.invoke(command,args);")
  .replace(/import \{ useAccess \} from [^;]+;/,"const useAccess=()=>globalThis.moneyTest.access;")
  .replace(/import \{ useCompany \} from [^;]+;/,"const useCompany=()=>globalThis.moneyTest.company;")
  .replace(/import AppNotice from [^;]+;/,"const AppNotice={render(){return null;}};");
 const file=new URL(`.payments-panel-${process.pid}.mjs`,import.meta.url);
 const previousDocument=globalThis.document,previousMoney=globalThis.moneyTest;
 const previousDocumentClass=globalThis.Document,previousShadowRoot=globalThis.ShadowRoot;
 const storageDescriptor=Object.getOwnPropertyDescriptor(globalThis,'localStorage');
 const storage=new Map(),calls=[],saved=[];
 let app,serverApplied=false;
 const initial={totalMinor:60000,collectedMinor:75000,receivedMinor:75000,adjustedMinor:0,refundedMinor:0,creditMinor:15000,paid:true};
 const final={...initial,collectedMinor:60000,refundedMinor:15000,creditMinor:0};
 const current=ref(initial);
 globalThis.document={activeElement:null};
 globalThis.Document=class {};globalThis.ShadowRoot=class {};
 Object.defineProperty(globalThis,'localStorage',{configurable:true,value:{getItem:key=>storage.get(key)??null,setItem:(key,value)=>storage.set(key,value),removeItem:key=>storage.delete(key)}});
 globalThis.moneyTest={access:{can:permission=>permission==='payments.refund'},company:{company:ref({locale:'es-MX'})},async invoke(command,args){
  if(command==='list_payment_ledger')return {movements:[{id:'original',kind:'receipt',amountMinor:75000,effectiveMinor:75000,method:'Prueba',occurredAt:null,paymentId:null,reason:null}],payment:serverApplied?final:initial};
  if(command==='record_payment_operation'){
   calls.push(structuredClone(args.request));
   if(!serverApplied){serverApplied=true;throw new Error('Respuesta perdida después de confirmar');}
   return final;
  }
  throw new Error(command);
 }};
 const element=(tag,text='')=>({tag,tagName:tag.toUpperCase(),text,props:{},children:[],parent:null,value:'',listeners:{},addEventListener(name,fn){(this.listeners[name]??=[]).push(fn);},removeEventListener(){},getRootNode(){return {};},setAttribute(name,value){this.props[name]=value;},removeAttribute(name){delete this.props[name];}});
 const renderer=createRenderer({createElement:tag=>element(tag),createText:text=>element('#text',text),createComment:text=>element('#comment',text),setText:(node,text)=>node.text=text,setElementText:(node,text)=>{node.text=text;node.children=[];},patchProp:(node,key,_old,value)=>{node.props[key]=value;if(key==='value')node.value=value;},insert:(node,parent,anchor=null)=>{node.parent=parent;const index=anchor?parent.children.indexOf(anchor):-1;parent.children.splice(index<0?parent.children.length:index,0,node);},remove:node=>{const index=node.parent?.children.indexOf(node);if(index>=0)node.parent.children.splice(index,1);},parentNode:node=>node.parent,nextSibling:node=>node.parent?.children[node.parent.children.indexOf(node)+1]??null});
 const all=(node,predicate)=>[...(predicate(node)?[node]:[]),...node.children.flatMap(child=>all(child,predicate))];
 const text=node=>node.text+node.children.map(text).join('');
 const settle=async()=>{for(let i=0;i<12;i++){await Promise.resolve();await nextTick();}};
 let root;
 try{
  writeFileSync(file,ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText);
  const {default:Panel}=await import(file.href);
  const mount=()=>{root=element('root');app=renderer.createApp({setup:()=>()=>h(Panel,{orderId:'QA',currency:'MXN',payment:current.value,disabled:false,prepare:async()=>true,onRefreshed:value=>current.value=value,onSaved:value=>{saved.push(value);current.value=value;}})});app.mount(root);};
  mount();await settle();
  const buttons=()=>all(root,node=>node.tag==='button');
  assert.ok(!buttons().some(node=>text(node)==='Registrar abono'));
  assert.ok(!buttons().some(node=>text(node)==='Corregir abono'));
  buttons().find(node=>text(node)==='Devolver saldo a favor').props.onClick();await settle();
  for(const [id,value] of [['money-method','Transferencia libre'],['money-reason','Precio ajustado']]){
   const input=all(root,node=>node.props.id===id)[0];input.props['onUpdate:modelValue'](value);
  }
  await nextTick();await all(root,node=>node.tag==='form')[0].props.onSubmit({preventDefault(){}});await settle();
  assert.equal(calls.length,1);assert.equal(calls[0].amountMinor,15000);assert.equal(calls[0].method,'Transferencia libre');assert.equal(storage.size,1);
  app.unmount();mount();await settle();
  assert.equal(current.value.creditMinor,0);
  await buttons().find(node=>text(node)==='Reintentar operación pendiente').props.onClick();await settle();
  assert.equal(calls.length,2);assert.deepEqual(calls[1],calls[0]);assert.equal(storage.size,0);assert.equal(saved.length,1);
  assert.equal(buttons().find(node=>text(node)==='Devolver saldo a favor').props.disabled,true);
 }finally{
  app?.unmount();globalThis.document=previousDocument;globalThis.moneyTest=previousMoney;
  globalThis.Document=previousDocumentClass;globalThis.ShadowRoot=previousShadowRoot;
  if(storageDescriptor)Object.defineProperty(globalThis,'localStorage',storageDescriptor);else delete globalThis.localStorage;
  unlinkSync(file);
 }
});
