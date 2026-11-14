import { computed } from 'vue';
import { useAuthStore } from '../stores/useAuthStore.ts';
import { useWorkspaceStore } from '../stores/useWorkspaceStore.ts';
import { useOrderStore } from '../stores/useOrderStore.ts';
export function useWorkshopAccess() {
  const auth = useAuthStore();
  const workspace = useWorkspaceStore();
  const orders = useOrderStore();
  const user = computed(() => auth.user ? { ...auth.user, id: String(auth.user.id) } : null);
  const can = (operation: string) => auth.can(operation, workspace.page, workspace.page === 'repairs' ? orders.currentOrder?.id ?? null : null);
  const temporary=computed(()=>auth.temporaryAccess);
  const requestAccess=(operation:string,scope:import('../types/ipc').AccessScope,orderId?:string|number|null)=>auth.can(operation,scope,orderId?Number(orderId):null)?Promise.resolve(true):workspace.requestAccess(operation,scope,orderId);
  return { user, temporary, can, requestAccess, baseCan: (operation: string) => auth.permissions.includes(operation) };
}
