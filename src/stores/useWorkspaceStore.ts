import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { AccessScope } from '../types/ipc.ts';
import type { TemporaryAccessContext } from '../composables/useTemporaryAccessForm.ts';
export const useWorkspaceStore = defineStore('workspace', () => {
  const page = ref<AccessScope>('clients');
  const request = ref<TemporaryAccessContext | null>(null);
  const chooseOperation = ref(false);
  let resolveAccess: ((value:boolean)=>void) | undefined;
  function finishRequest(approved=false) { request.value=null;chooseOperation.value=false;resolveAccess?.(approved);resolveAccess=undefined; }
  function requestAccess(operation:string,scope:AccessScope,orderId?:string|number|null,allowOperationChoice=false) {
    finishRequest();chooseOperation.value=allowOperationChoice;request.value={operation,scope,orderId:orderId?Number(orderId):null};
    return new Promise<boolean>(resolve=>{resolveAccess=resolve;});
  }
  return { page, request, chooseOperation, requestAccess, finishRequest };
});
