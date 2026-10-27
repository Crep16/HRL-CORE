<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAccess } from '../../composables/useAccess';
type Grant = { section: string; action: string };
type Account = { id: string; name: string; accessLevel: string; active: boolean; permissions: Grant[] };
const { refresh } = useAccess();
const accounts = ref<Account[]>([]);
const busy = ref(false);
const message = ref('');
const selectedGrants = ref<string[]>([]);
const empty = () => ({ id: null as string | null, name: '', password: '', confirmation: '', masterPassword: '', accessLevel: 'limited', active: true });
const draft = reactive(empty());
const sections = [{ id: 'clients', name: 'Clientes' }, { id: 'repairs', name: 'Reparaciones' }, { id: 'logbook', name: 'Bitácora' }, { id: 'company', name: 'Empresa' }];
const actions = [{ id: 'view', name: 'Visualizar' }, { id: 'create', name: 'Crear' }, { id: 'edit', name: 'Editar' }, { id: 'delete', name: 'Borrar' }];
const roles: Record<string, string> = { complete: 'Completo', limited: 'Limitado', custom: 'Personalizado' };
async function load() {
  busy.value = true;
  try { accounts.value = await invoke<Account[]>('list_users'); }
  catch (cause) { message.value = String(cause); }
  finally { busy.value = false; }
}
onMounted(load);
function clear() { Object.assign(draft, empty()); selectedGrants.value = []; message.value = ''; }
function edit(account: Account) { clear(); Object.assign(draft, { id: account.id, name: account.name, accessLevel: account.accessLevel, active: account.active }); selectedGrants.value = account.permissions.map(g => `${g.section}:${g.action}`); }
async function save() {
  if (busy.value) return;
  busy.value = true; message.value = '';
  try {
    await invoke('save_user', { input: { ...draft, permissions: selectedGrants.value.map(value => { const [section, action] = value.split(':'); return { section, action }; }) } });
    clear(); message.value = 'Cuenta y permisos guardados en este equipo.';
    await refresh(); await load();
  } catch (cause) { message.value = String(cause); }
  finally { draft.password = ''; draft.confirmation = ''; draft.masterPassword = ''; busy.value = false; }
}
</script>

<template>
  <section class="access-shell users-workspace" aria-label="Administración de usuarios">
    <div class="access-main">
      <h1>Usuarios de este equipo</h1>
      <p class="access-lead">Crea cuentas y define sus acciones. Cada cambio requiere tu clave maestra.</p>
      <p v-if="busy" role="status">Consultando o guardando usuarios…</p>
      <p v-if="message" class="access-feedback" role="status">{{ message }}</p>
      <ul aria-label="Cuentas existentes"><li v-for="account in accounts" :key="account.id">{{ account.name }} · {{ roles[account.accessLevel] }} · {{ account.active ? 'Activo' : 'Inactivo' }} <button class="access-secondary" :disabled="busy" type="button" @click="edit(account)">Editar {{ account.name }}</button></li></ul>
      <button class="access-secondary" :disabled="busy" type="button" @click="clear">Nueva cuenta</button>
      <h2>{{ draft.id ? 'Editar cuenta' : 'Crear cuenta' }}</h2>
      <form class="access-form" @submit.prevent="save">
        <fieldset class="access-fields" :disabled="busy">
          <div class="access-field"><label for="user-name">Usuario</label><input id="user-name" v-model="draft.name" autocomplete="off" maxlength="100" required></div>
          <div class="access-field"><label for="user-role">Acceso</label><select id="user-role" v-model="draft.accessLevel"><option value="complete">Completo</option><option value="limited">Limitado</option><option value="custom">Personalizado</option></select></div>
          <p v-if="draft.accessLevel === 'limited'">Consulta todas las órdenes y registra recepciones, notas, etapas, garantías y refacciones. Sin pagos, edición ni borrado.</p>
          <fieldset v-if="draft.accessLevel === 'custom'" class="access-permissions"><legend>Permisos por apartado</legend><div v-for="section in sections" :key="section.id"><strong>{{ section.name }}</strong><div><label v-for="action in actions" :key="action.id"><input v-model="selectedGrants" type="checkbox" :value="`${section.id}:${action.id}`">{{ action.name }}</label></div></div></fieldset>
          <label><input v-model="draft.active" type="checkbox"> Cuenta activa</label>
          <div class="access-field"><label for="user-password">Clave del usuario {{ draft.id ? '(deja vacío para conservarla)' : '' }}</label><input id="user-password" v-model="draft.password" type="password" autocomplete="new-password" :required="!draft.id" minlength="8" maxlength="1024"></div>
          <div class="access-field"><label for="user-confirm">Repite la clave del usuario</label><input id="user-confirm" v-model="draft.confirmation" type="password" autocomplete="new-password" :required="!draft.id || !!draft.password" maxlength="1024"></div>
          <div class="access-field"><label for="user-master">Clave maestra para autorizar el cambio</label><input id="user-master" v-model="draft.masterPassword" type="password" autocomplete="off" required maxlength="1024"></div>
          <div class="access-actions"><button class="access-primary" type="submit">Guardar cuenta y permisos</button></div>
        </fieldset>
      </form>
    </div>
  </section>
</template>

<style scoped>
.users-workspace{min-height:0;margin:0;max-width:none}.access-main{margin:auto}li{margin-bottom:12px;overflow-wrap:anywhere}.access-secondary{margin-block:4px}label:has(>input[type=checkbox]){min-height:44px;display:flex;align-items:center;gap:8px}
</style>
