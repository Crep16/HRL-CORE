import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync, writeFileSync, unlinkSync } from 'node:fs';
import { parse, compileScript } from '@vue/compiler-sfc';
import ts from 'typescript';
import { createRenderer, h, nextTick, ref } from 'vue';

test('temporary access dialog explains its scope, clears the password after rejection and cancels without granting',async()=>{
 const {descriptor}=parse(readFileSync(new URL('../src/components/access/TemporaryAccessDialog.vue',import.meta.url),'utf8'));
 const script=compileScript(descriptor,{id:'qa-approval-dialog',inlineTemplate:true});
 const source=script.content.replace(/import \{ useAccess \} from [^;]+;/,'const useAccess=()=>globalThis.approvalDialogTest;');
 const file=new URL(`.temporary-dialog-${process.pid}.mjs`,import.meta.url);
 const previous={document:globalThis.document,Document:globalThis.Document,ShadowRoot:globalThis.ShadowRoot,HTMLElement:globalThis.HTMLElement,test:globalThis.approvalDialogTest};
 const requested=ref(null),calls=[],answers=[];
 globalThis.document={activeElement:null};globalThis.Document=class {};globalThis.ShadowRoot=class {};globalThis.HTMLElement=class {};
 globalThis.approvalDialogTest={accessRequest:requested,async approveTemporary(...args){calls.push(args);throw 'Credenciales incorrectas';},resolveRequest(allowed){answers.push(allowed);requested.value=null;}};
 const find=(node,predicate)=>[...(predicate(node)?[node]:[]),...node.children.flatMap(child=>find(child,predicate))];
 const element=(tag,text='')=>({tag,tagName:tag.toUpperCase(),text,props:{},children:[],parent:null,open:false,value:'',listeners:{},showModal(){this.open=true;},close(){this.open=false;},focus(){globalThis.document.activeElement=this;},querySelector(selector){return find(this,node=>node.props.id===selector.slice(1))[0];},addEventListener(name,fn){(this.listeners[name]??=[]).push(fn);},removeEventListener(){},setAttribute(name,value){this.props[name]=value;},removeAttribute(name){delete this.props[name];},getRootNode(){return {};}});
 const renderer=createRenderer({createElement:tag=>element(tag),createText:text=>element('#text',text),createComment:text=>element('#comment',text),setText:(node,text)=>node.text=text,setElementText:(node,text)=>{node.text=text;node.children=[];},patchProp:(node,key,_old,value)=>{node.props[key]=value;if(key==='value')node.value=value;},insert:(node,parent,anchor=null)=>{node.parent=parent;const index=anchor?parent.children.indexOf(anchor):-1;parent.children.splice(index<0?parent.children.length:index,0,node);},remove:node=>{const index=node.parent?.children.indexOf(node);if(index>=0)node.parent.children.splice(index,1);},parentNode:node=>node.parent,nextSibling:node=>node.parent?.children[node.parent.children.indexOf(node)+1]??null});
 let app;
 const settle=async()=>{for(let i=0;i<6;i++){await Promise.resolve();await nextTick();}};
 try{
  writeFileSync(file,ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText);
  const {default:Dialog}=await import(file.href);const root=element('root');app=renderer.createApp({render:()=>h(Dialog)});app.mount(root);
  requested.value={operation:'repairs.edit',scope:'repairs',recordId:'A'};await settle();
  const dialog=find(root,node=>node.tag==='dialog')[0];assert.equal(dialog.open,true);
  const body=find(root,()=>true).map(node=>node.text).join(' ');assert.ok(body.includes('Tu sesión se conserva'));assert.ok(body.includes('Bitácora'));
  const byId=id=>find(root,node=>node.props.id===id)[0];
  assert.equal(byId('temporary-order').props.readonly,true);
  for(const [id,value] of [['temporary-user','Superior'],['temporary-password','test-only-password'],['temporary-reason','Editar técnico']])byId(id).props['onUpdate:modelValue'](value);
  await nextTick();await find(root,node=>node.tag==='form')[0].props.onSubmit({preventDefault(){}});await settle();
  assert.equal(calls.length,1);assert.equal(calls[0][1],'test-only-password');assert.equal(calls[0][3],'A');assert.equal(byId('temporary-password').value,'');
  assert.ok(find(root,node=>node.props.role==='alert').some(node=>node.text==='Credenciales incorrectas'));
  dialog.props.onCancel({preventDefault(){}});await settle();assert.equal(dialog.open,false);assert.equal(answers.at(-1),false);
 }finally{app?.unmount();globalThis.document=previous.document;globalThis.Document=previous.Document;globalThis.ShadowRoot=previous.ShadowRoot;globalThis.HTMLElement=previous.HTMLElement;globalThis.approvalDialogTest=previous.test;unlinkSync(file);}
});
