import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useUsersStore } from '../stores/useUsersStore.ts';
import { useAuthStore } from '../stores/useAuthStore.ts';
import type { Account } from '../types/users.ts';
export function useUsersWorkspace() {
  const store = useUsersStore();
  const auth = useAuthStore();
  const { accounts, isLoading: busy } = storeToRefs(store);
  const user = computed(() => auth.user);
  const search = ref(''); const loaded = ref(false); const message = ref(''); const failed = ref(false); const selectedGrants = ref<string[]>([]);
  const empty = () => ({ id:null as number|null,name:'',role:'technician',password:'',confirmation:'',masterPassword:'',accessLevel:'limited',active:true });
  const draft = reactive(empty());
  const sections = [{id:'clients',name:'Clientes'},{id:'repairs',name:'Reparaciones'},{id:'logbook',name:'Bitácora'},{id:'company',name:'Empresa'}];
  const actions = [{id:'view',name:'Visualizar'},{id:'create',name:'Crear'},{id:'edit',name:'Editar'},{id:'delete',name:'Borrar'}];
  const roles: Record<string,string> = {complete:'Completo',limited:'Limitado',custom:'Personalizado'};
  const filtered = computed(() => accounts.value.filter(account => `${account.name} ${roles[account.accessLevel]}`.toLocaleLowerCase('es').includes(search.value.trim().toLocaleLowerCase('es'))));
  async function load() { try { await store.load(); loaded.value=true; } catch {message.value=store.error?.message??'No se pudieron consultar los usuarios.'; failed.value=true;} }
  onMounted(load);
  function clear() { Object.assign(draft,empty()); selectedGrants.value=[]; message.value=''; failed.value=false; }
  function focusName() { nextTick(() => document.getElementById('user-name')?.focus()); }
  function newAccount() { clear();focusName(); }
  function edit(account:Account) {clear();Object.assign(draft,{id:account.id,name:account.name,role:account.role,accessLevel:account.accessLevel,active:account.active});selectedGrants.value=account.permissions.map(g=>`${g.section}:${g.action}`);focusName();}
  async function retry(){message.value='';failed.value=false;await load();}
  async function save(){
    if(busy.value)return;
    message.value='';failed.value=false;
    try { await store.save({...draft,role:draft.id?draft.role:draft.accessLevel==='complete'?'administrator':'technician',permissions:selectedGrants.value.map(value=>{const [section,action]=value.split(':');return {section:section!,action:action!};})});clear();message.value='Cuenta y permisos guardados en este equipo.';if(auth.can('users.manage','users'))await load(); }
    catch { message.value=store.error?.message??'No se pudo confirmar el guardado de la cuenta.';failed.value=true; }
    finally{draft.password='';draft.confirmation='';draft.masterPassword='';}
  }
  onBeforeUnmount(()=>{draft.password='';draft.confirmation='';draft.masterPassword='';});
  return {accounts,busy,user,search,loaded,message,failed,selectedGrants,draft,sections,actions,roles,filtered,load,newAccount,edit,retry,save};
}
