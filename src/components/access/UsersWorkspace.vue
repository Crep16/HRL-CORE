<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAccess } from '../../composables/useAccess';
import AppIcon from '../ui/AppIcon.vue';
type Grant = { section: string; action: string };
type Account = { id: string; name: string; accessLevel: string; active: boolean; permissions: Grant[] };
const { refresh, user } = useAccess();
const accounts = ref<Account[]>([]);
const search = ref('');
const busy = ref(false);
const loaded = ref(false);
const message = ref('');
const failed = ref(false);
const selectedGrants = ref<string[]>([]);
const empty = () => ({ id: null as string | null, name: '', password: '', confirmation: '', masterPassword: '', accessLevel: 'limited', active: true });
const draft = reactive(empty());
const sections = [{ id: 'clients', name: 'Clientes' }, { id: 'repairs', name: 'Reparaciones' }, { id: 'logbook', name: 'Bitácora' }, { id: 'company', name: 'Empresa' }];
const actions = [{ id: 'view', name: 'Visualizar' }, { id: 'create', name: 'Crear' }, { id: 'edit', name: 'Editar' }, { id: 'delete', name: 'Borrar' }];
const roles: Record<string, string> = { complete: 'Completo', limited: 'Limitado', custom: 'Personalizado' };
const filtered = computed(() => {
  const text = search.value.trim().toLocaleLowerCase('es');
  return accounts.value.filter(account => `${account.name} ${roles[account.accessLevel]}`.toLocaleLowerCase('es').includes(text));
});
async function load() {
  busy.value = true;
  try { accounts.value = await invoke<Account[]>('list_users'); loaded.value = true; }
  catch (cause) { message.value = String(cause); failed.value = true; }
  finally { busy.value = false; }
}
onMounted(load);
function clear() { Object.assign(draft, empty()); selectedGrants.value = []; message.value = ''; failed.value = false; }
function focusName() { nextTick(() => document.getElementById('user-name')?.focus()); }
function newAccount() { clear(); focusName(); }
function edit(account: Account) { clear(); Object.assign(draft, { id: account.id, name: account.name, accessLevel: account.accessLevel, active: account.active }); selectedGrants.value = account.permissions.map(g => `${g.section}:${g.action}`); focusName(); }
async function retry() { message.value = ''; failed.value = false; await load(); }
async function save() {
  if (busy.value) return;
  busy.value = true; message.value = ''; failed.value = false;
  try {
    await invoke('save_user', { input: { ...draft, permissions: selectedGrants.value.map(value => { const [section, action] = value.split(':'); return { section, action }; }) } });
    clear(); message.value = 'Cuenta y permisos guardados en este equipo.';
    await refresh(); await load();
  } catch (cause) { message.value = String(cause); failed.value = true; }
  finally { draft.password = ''; draft.confirmation = ''; draft.masterPassword = ''; busy.value = false; }
}
</script>

<template>
  <section class="users-workspace" aria-labelledby="users-title">
    <header class="users-heading">
      <div><h1 id="users-title"><AppIcon name="users" />Usuarios</h1><p>Cuentas de este equipo y acciones que puede realizar cada persona.</p></div>
      <span class="users-local"><AppIcon name="shield" />Acceso administrado con clave maestra</span>
    </header>
    <div v-if="message" class="users-feedback" :class="{ error: failed }" :role="failed ? 'alert' : 'status'">{{ message }}<button v-if="failed && !loaded" class="btn btn-s" :disabled="busy" type="button" @click="retry">Volver a consultar</button></div>
    <div class="users-layout">
      <aside class="users-roster" aria-label="Cuentas de este equipo">
        <div class="users-roster-heading"><h2>Cuentas</h2><span v-if="loaded">{{ accounts.length }}</span></div>
        <div class="users-search"><AppIcon name="search" /><label class="sr" for="users-search">Buscar por usuario o nivel de acceso</label><input id="users-search" v-model="search" type="search" placeholder="Buscar usuario o acceso"></div>
        <div class="users-account-list" tabindex="0" aria-label="Lista de cuentas" :aria-busy="busy">
          <p v-if="busy && !loaded" class="users-empty" role="status">Recuperando cuentas…</p>
          <p v-else-if="failed && !loaded" class="users-empty">No se pudieron consultar las cuentas.</p>
          <p v-else-if="!filtered.length" class="users-empty">{{ search ? 'No hay cuentas que coincidan con la búsqueda.' : 'No hay cuentas para mostrar.' }}</p>
          <ul v-else>
            <li v-for="account in filtered" :key="account.id">
              <button class="users-account" :class="{ selected: draft.id === account.id }" :aria-current="draft.id === account.id ? 'true' : undefined" :disabled="busy" type="button" @click="edit(account)">
                <span class="users-account-name"><strong>{{ account.name }}</strong><span v-if="account.id === user?.id" class="users-self">Tú</span></span>
                <span class="users-account-meta"><span>{{ roles[account.accessLevel] }}</span><span class="users-account-state" :class="{ inactive: !account.active }">{{ account.active ? 'Activa' : 'Inactiva' }}</span></span>
              </button>
            </li>
          </ul>
        </div>
        <footer class="users-roster-footer"><span v-if="loaded">{{ filtered.length }} de {{ accounts.length }} cuentas</span><button class="btn btn-p" :disabled="busy" type="button" @click="newAccount"><AppIcon name="add" />Nueva cuenta</button></footer>
      </aside>

      <section class="users-editor" aria-labelledby="user-editor-title">
        <header class="users-editor-heading"><div><h2 id="user-editor-title">{{ draft.id ? 'Editar cuenta' : 'Nueva cuenta' }}</h2><p>{{ draft.id ? 'Actualiza los datos y autoriza el cambio al guardar.' : 'Define un usuario y el acceso que tendrá al taller.' }}</p></div><span class="users-mode">{{ roles[draft.accessLevel] }}</span></header>
        <form class="users-form" @submit.prevent="save">
          <fieldset class="users-form-fields" :disabled="busy">
            <div class="users-editor-body">
              <div class="users-account-fields">
                <div class="users-field"><label for="user-name">Usuario</label><input id="user-name" v-model="draft.name" autocomplete="off" maxlength="100" placeholder="Nombre de la persona" required></div>
                <div class="users-field"><label for="user-role">Nivel de acceso</label><select id="user-role" v-model="draft.accessLevel"><option value="complete">Completo</option><option value="limited">Limitado</option><option value="custom">Personalizado</option></select></div>
                <label class="users-active"><input v-model="draft.active" type="checkbox"><span><strong>Cuenta activa</strong><small>{{ draft.active ? 'Puede iniciar sesión en este equipo.' : 'No podrá iniciar sesión; se conserva su historial.' }}</small></span></label>
                <div class="users-passwords">
                  <div class="users-field"><label for="user-password">Clave del usuario</label><input id="user-password" v-model="draft.password" type="password" autocomplete="new-password" :required="!draft.id" minlength="8" maxlength="1024" aria-describedby="user-password-help"></div>
                  <div class="users-field"><label for="user-confirm">Repite la clave</label><input id="user-confirm" v-model="draft.confirmation" type="password" autocomplete="new-password" :required="!draft.id || !!draft.password" maxlength="1024" :aria-invalid="!!draft.confirmation && draft.confirmation !== draft.password" aria-describedby="user-password-help"></div>
                </div>
                <p id="user-password-help" class="users-help">{{ draft.id ? 'Deja las claves vacías para conservar la actual.' : 'Usa al menos 8 caracteres para la clave.' }}<span v-if="draft.confirmation && draft.confirmation !== draft.password" class="users-match-error"> Las dos claves deben coincidir.</span></p>
              </div>
              <section class="users-grants" aria-labelledby="user-grants-title">
                <h3 id="user-grants-title"><AppIcon name="shield" />Acciones permitidas</h3>
                <template v-if="draft.accessLevel === 'custom'">
                  <p>Marca los permisos de cada apartado.</p>
                  <table class="users-permissions"><caption class="sr">Permisos personalizados por apartado y acción</caption><thead><tr><th scope="col">Apartado</th><th v-for="action in actions" :key="action.id" scope="col">{{ action.name }}</th></tr></thead><tbody><tr v-for="section in sections" :key="section.id"><th scope="row">{{ section.name }}</th><td v-for="action in actions" :key="action.id"><label :aria-label="`${action.name} en ${section.name}`"><input v-model="selectedGrants" type="checkbox" :value="`${section.id}:${action.id}`"><span class="sr">{{ action.name }} en {{ section.name }}</span></label></td></tr></tbody></table>
                </template>
                <template v-else-if="draft.accessLevel === 'complete'"><p class="users-role-title">Acceso completo</p><p>Sin restricciones de acceso. Puede administrar las cuentas y sus permisos.</p><div class="users-rule"><AppIcon name="users" /><span>Siempre debe conservarse al menos un administrador activo.</span></div></template>
                <template v-else><p class="users-role-title">Acceso limitado</p><ul class="users-role-list"><li>Consultar todas las órdenes y sus datos.</li><li>Registrar nuevas recepciones.</li><li>Sin pagos, edición ni borrado.</li></ul><div class="users-rule"><AppIcon name="shield" /><span>Para elegir acciones por apartado, usa Personalizado.</span></div></template>
              </section>
            </div>
            <footer class="users-save-bar">
              <div class="users-field users-master"><label for="user-master"><AppIcon name="shield" />Clave maestra</label><input id="user-master" v-model="draft.masterPassword" type="password" autocomplete="off" placeholder="Necesaria para guardar" required maxlength="1024"></div>
              <div class="users-save-action"><span v-if="busy" role="status">Consultando o guardando…</span><button class="btn btn-p" type="submit"><AppIcon name="save" />Guardar cuenta</button></div>
            </footer>
          </fieldset>
        </form>
      </section>
    </div>
  </section>
</template>

<style src="../../assets/styles/users.css"></style>
