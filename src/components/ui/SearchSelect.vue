<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from 'vue';
type Option = string | { value: string; label: string };
const props = withDefaults(defineProps<{modelValue:string;options:readonly Option[];label?:string;id?:string;disabled?:boolean;required?:boolean;allowCustom?:boolean;placeholder?:string}>(),{label:'',disabled:false,required:false,allowCustom:false,placeholder:'Selecciona o escribe para buscar'});
const emit = defineEmits<{'update:modelValue':[value:string]}>();
const uid=useId();
const inputId=computed(()=>props.id || `search-select-${uid}`);
const items=computed(()=>props.options.map(option=>typeof option==='string'?{value:option,label:option}:option));
const selectedLabel=computed(()=>items.value.find(item=>item.value===props.modelValue)?.label ?? props.modelValue);
const text=ref(selectedLabel.value),open=ref(false),filtering=ref(false),active=ref(0);
const list=ref<HTMLElement>(),input=ref<HTMLInputElement>();
const normalize=(value:string)=>value.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLocaleLowerCase('es');
const matches=computed(()=>items.value.filter(item=>!filtering.value||normalize(item.label).includes(normalize(text.value))));
watch(selectedLabel,value=>{if(!filtering.value) text.value=value;});
watch(matches,()=>{active.value=0;});
watch(()=>props.disabled,value=>{if(value) close();});
async function reveal(){await nextTick();list.value?.children[active.value]?.scrollIntoView({block:'nearest'});}
async function show(){if(props.disabled)return;filtering.value=false;open.value=true;await nextTick();active.value=Math.max(0,matches.value.findIndex(item=>item.value===props.modelValue));await reveal();}
function choose(value:string){filtering.value=false;emit('update:modelValue',value);text.value=items.value.find(item=>item.value===value)?.label??value;open.value=false;void nextTick(()=>{if(props.modelValue!==value)text.value=selectedLabel.value;});}
async function focus(){await show();input.value?.select?.();}
function close(){const exact=items.value.find(item=>normalize(item.label)===normalize(text.value));if(filtering.value && exact)choose(exact.value);else if(filtering.value&&props.allowCustom)choose(text.value.trim());else{open.value=false;filtering.value=false;text.value=selectedLabel.value;}}
function toggle(){if(open.value)close();else{input.value?.focus();void show();}}
function type(){filtering.value=true;open.value=true;active.value=0;if(props.allowCustom)emit('update:modelValue',text.value);}
async function keydown(event:KeyboardEvent){
  if(['ArrowDown','ArrowUp','Home','End'].includes(event.key) && (open.value || event.key.startsWith('Arrow'))){
    event.preventDefault();if(!open.value){await show();return;}
    active.value=event.key==='Home'?0:event.key==='End'?matches.value.length-1:Math.max(0,Math.min(matches.value.length-1,active.value+(event.key==='ArrowDown'?1:-1)));await reveal();
  }else if(event.key==='Enter'&&open.value){event.preventDefault();const option=matches.value[active.value];if(option)choose(option.value);else if(props.allowCustom)close();}
  else if(event.key==='Escape'&&open.value){event.preventDefault();event.stopPropagation();filtering.value=false;open.value=false;text.value=selectedLabel.value;}
}
</script>
<template>
  <div class="search-select">
    <label v-if="label" :for="inputId">{{ label }}</label>
    <div class="search-select-control">
      <input :id="inputId" ref="input" v-model="text" role="combobox" aria-autocomplete="list" aria-haspopup="listbox" :aria-controls="`${inputId}-options`" :aria-expanded="open" :aria-activedescendant="open&&matches.length?`${inputId}-option-${active}`:undefined" :disabled="disabled" :required="required" :placeholder="placeholder" autocomplete="off" @focus="focus" @click="!open&&show()" @input="type" @keydown="keydown" @blur="close" />
      <button type="button" tabindex="-1" :disabled="disabled" :aria-label="`${open?'Cerrar':'Mostrar'} opciones${label?` de ${label}`:''}`" @pointerdown.prevent @click="toggle">▾</button>
    </div>
    <div v-if="open" class="search-select-dropdown">
      <ul :id="`${inputId}-options`" ref="list" role="listbox" :aria-label="label || 'Opciones disponibles'">
        <li v-for="(option,index) in matches" :id="`${inputId}-option-${index}`" :key="option.value" role="option" :aria-selected="option.value===modelValue" :class="{active:index===active}" @pointerdown.prevent @click="choose(option.value)" @pointermove="active=index">{{ option.label }}</li>
      </ul>
      <p v-if="!matches.length" role="status">{{ allowCustom?'Puedes usar el texto escrito como una opción nueva.':'No hay opciones que coincidan con la búsqueda.' }}</p>
    </div>
  </div>
</template>
<style scoped>
.search-select{position:relative;min-width:0;display:flex;flex-direction:column;gap:8px;width:100%}
.search-select>label{font-size:14px;line-height:20px;font-weight:600;color:var(--muted);margin:0}
.search-select-control{position:relative;display:flex;align-items:center}
.search-select-control input{width:100%;min-width:0;min-height:44px;padding:10px 44px 10px 12px!important;border:1px solid var(--line2);border-radius:8px;background:var(--tint);color:var(--ink);font:inherit;box-sizing:border-box}
.search-select-control input:focus-visible{outline:2px solid var(--acc);outline-offset:2px}
.search-select-control button{position:absolute;right:0;top:0;width:44px;height:100%;border:0;background:transparent;color:var(--ink);font-size:18px;cursor:pointer}
.search-select-control input:disabled{color:var(--muted);cursor:not-allowed}
.search-select-dropdown{position:absolute;top:100%;left:0;right:0;z-index:30;margin-top:4px;border:1px solid var(--line2);border-radius:8px;background:white;box-shadow:0 6px 18px #12222d26;overflow:hidden}
.search-select-dropdown ul{list-style:none;margin:0;padding:4px;max-height:220px;overflow-y:auto}
.search-select-dropdown li{min-height:44px;padding:10px 12px;box-sizing:border-box;color:var(--ink);font:15px/24px var(--body);cursor:pointer;overflow-wrap:anywhere}
.search-select-dropdown li.active{background:var(--acc-soft);color:var(--acc-ink)}
.search-select-dropdown li[aria-selected=true]{font-weight:600}
.search-select-dropdown p{margin:0;padding:12px;color:var(--muted);font:14px/1.5 var(--body)}
</style>
