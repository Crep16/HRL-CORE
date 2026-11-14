<script setup lang="ts">
import AppIcon from "../ui/AppIcon.vue";
import SearchSelect from "../ui/SearchSelect.vue";
import AppNotice from "../ui/AppNotice.vue";
import { useUsersWorkspace } from "../../composables/useUsersWorkspace";
const { accounts, busy, user, search, loaded, message, failed, selectedGrants, draft, sections, actions, roles, filtered, newAccount, edit, retry, save } = useUsersWorkspace();
</script>

<template>
  <section class="users-workspace" aria-labelledby="users-title">
    <header class="users-heading">
      <div><h1 id="users-title"><AppIcon name="users" />Usuarios</h1><p>Cuentas de este equipo y acciones que puede realizar cada persona.</p></div>
      <span class="users-local"><AppIcon name="shield" />Acceso administrado con clave maestra</span>
    </header>
    <AppNotice :message="message" :tone="failed?'error':'status'" @dismiss="message=''"><button v-if="failed && !loaded" class="btn btn-s" :disabled="busy" type="button" @click="retry">Volver a consultar</button></AppNotice>
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
                <SearchSelect id="user-role" v-model="draft.accessLevel" label="Nivel de acceso" :options="[{value:'complete',label:'Completo'},{value:'limited',label:'Limitado'},{value:'custom',label:'Personalizado'}]" />
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
                  <table class="users-permissions"><caption class="sr">Permisos personalizados por apartado y acción</caption><thead><tr><th scope="col">Apartado</th><th v-for="action in actions" :key="action.id" scope="col">{{ action.name }}</th></tr></thead><tbody><tr v-for="section in sections" :key="section.id"><th scope="row">{{ section.name }}</th><td v-for="action in actions" :key="action.id"><label v-if="section.id !== 'logbook' || action.id === 'view'" :aria-label="`${action.name} en ${section.name}`"><input v-model="selectedGrants" type="checkbox" :value="`${section.id}:${action.id}`"><span class="sr">{{ action.name }} en {{ section.name }}</span></label><span v-else class="users-help">No aplica</span></td></tr></tbody></table>
                  <p>La Bitácora solo permite consultar movimientos; no se pueden crear, editar ni borrar manualmente.</p>
                </template>
                <template v-else-if="draft.accessLevel === 'complete'"><p class="users-role-title">Acceso completo</p><p>Sin restricciones de acceso. Puede administrar las cuentas y sus permisos.</p><div class="users-rule"><AppIcon name="users" /><span>Siempre debe conservarse al menos un administrador activo.</span></div></template>
                <template v-else><p class="users-role-title">Acceso limitado</p><ul class="users-role-list"><li>Consultar todas las órdenes y sus datos.</li><li>Registrar nuevas recepciones y avances técnicos.</li><li>Sin edición ni borrado de órdenes.</li></ul></template>
                <details v-if="draft.accessLevel!=='complete'" class="users-money-permissions"><summary>Operaciones de dinero</summary><p>Estos permisos son independientes de Editar en Reparaciones.</p><label><input v-model="selectedGrants" type="checkbox" value="payments:add"> Registrar abonos</label><label><input v-model="selectedGrants" type="checkbox" value="payments:refund"> Devolver todo el saldo a favor</label><p>Solo el acceso Completo puede corregir abonos.</p></details>
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
<style scoped>
.users-money-permissions{min-width:0;border:0;border-top:1px solid var(--line);margin:16px 0 0;padding:16px 0 0}.users-money-permissions legend{font-weight:700}.users-money-permissions label{display:flex;align-items:center;gap:12px;min-height:44px;cursor:pointer}.users-money-permissions input{width:18px;height:18px;accent-color:var(--acc)}.users-money-permissions p{color:var(--muted);font-size:14px;line-height:1.5}
</style>
