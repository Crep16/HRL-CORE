import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync,writeFileSync,unlinkSync } from 'node:fs';
import { parse,compileScript } from '@vue/compiler-sfc';
import ts from 'typescript';
import { createRenderer,h,nextTick,ref } from 'vue';

test('shared dropdown filters accents, selects by click and keyboard, restores invalid text and isolates IDs',async()=>{
 const source=readFileSync(new URL('../src/components/ui/SearchSelect.vue',import.meta.url),'utf8');
 const {descriptor}=parse(source);
 const script=compileScript(descriptor,{id:'qa-search-select',inlineTemplate:true});
 const compiled=ts.transpileModule(script.content,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText;
 const file=new URL(`.search-select-${process.pid}.mjs`,import.meta.url);
 const previousDocument=globalThis.document;globalThis.document={activeElement:null};
 const previousDocumentClass=globalThis.Document,previousShadowRoot=globalThis.ShadowRoot;
 globalThis.Document=class {};globalThis.ShadowRoot=class {};
 let app;
 try{
  writeFileSync(file,compiled);
  const {default:SearchSelect}=await import(file.href);
  const element=(tag,text='')=>({tag,text,children:[],props:{},listeners:{},parent:null,value:'',getRootNode(){return {};},addEventListener(name,listener){(this.listeners[name]??=[]).push(listener);},removeEventListener(){},setAttribute(name,value){this.props[name]=value;},removeAttribute(name){delete this.props[name];},scrollIntoView(){},focus(){globalThis.document.activeElement=this;this.props.onFocus?.();}});
  const renderer=createRenderer({
   createElement:tag=>element(tag),createText:text=>element('#text',text),createComment:text=>element('#comment',text),
   setText:(node,text)=>node.text=text,setElementText:(node,text)=>{node.text=text;node.children=[];},
   patchProp:(node,key,_old,value)=>{node.props[key]=value;if(key==='value')node.value=value;},
   insert:(node,parent,anchor=null)=>{if(node.parent){const old=node.parent.children.indexOf(node);if(old>=0)node.parent.children.splice(old,1);}node.parent=parent;const index=anchor?parent.children.indexOf(anchor):-1;parent.children.splice(index<0?parent.children.length:index,0,node);},
   remove:node=>{const index=node.parent?.children.indexOf(node);if(index>=0)node.parent.children.splice(index,1);},parentNode:node=>node.parent,nextSibling:node=>node.parent?.children[node.parent.children.indexOf(node)+1]??null,
  });
  const root=element('root'),selected=ref('repair'),disabled=ref(false);
  app=renderer.createApp({setup:()=>()=>h('div',[h(SearchSelect,{label:'Etapa',modelValue:selected.value,options:[{value:'received',label:'Recibido'},{value:'diagnosis',label:'Diagnóstico'},{value:'repair',label:'Reparación'}],disabled:disabled.value,'onUpdate:modelValue':value=>selected.value=value}),h(SearchSelect,{label:'Otra opción',modelValue:'',options:['A','B']})])});app.mount(root);
  const all=(node,predicate)=>[...(predicate(node)?[node]:[]),...node.children.flatMap(child=>all(child,predicate))];
  const input=all(root,node=>node.tag==='input')[0];
  const options=()=>all(root,node=>node.props.role==='option');
  const type=async text=>{input.value=text;for(const listener of input.listeners.input??[])listener({target:input});input.props.onInput?.({target:input});await nextTick();};
  input.focus();await nextTick();await nextTick();assert.equal(options().length,3);
  await type('diagnostico');assert.deepEqual(options().map(option=>option.text),['Diagnóstico']);
  options()[0].props.onClick();await nextTick();assert.equal(selected.value,'diagnosis');assert.equal(input.value,'Diagnóstico');assert.equal(options().length,0);
  input.focus();await nextTick();await nextTick();await type('repa');
  await input.props.onKeydown({key:'Enter',preventDefault(){},stopPropagation(){}});await nextTick();assert.equal(selected.value,'repair');
  input.focus();await nextTick();await type('no existe');assert.equal(options().length,0);input.props.onBlur();await nextTick();assert.equal(input.value,'Reparación');
  input.focus();await nextTick();await type('diagn');await input.props.onKeydown({key:'Escape',preventDefault(){},stopPropagation(){}});await nextTick();assert.equal(input.value,'Reparación');assert.equal(selected.value,'repair');
  disabled.value=true;await nextTick();assert.equal(input.props.disabled,true);
  const ids=all(root,node=>node.tag==='input').map(node=>node.props.id);assert.equal(new Set(ids).size,2);
 }finally{try{app?.unmount();}catch{}globalThis.document=previousDocument;globalThis.Document=previousDocumentClass;globalThis.ShadowRoot=previousShadowRoot;unlinkSync(file);}
});
