<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import logo from '../../assets/images/brand/hrl-original.svg';
import { useAccess } from '../../composables/useAccess';
const { setupComplete, loading, error, available, refresh, firstSetup, login } = useAccess();
const step = ref<'master' | 'first'>('master');
const name = ref('');
const password = ref('');
const confirmation = ref('');
const masterPassword = ref('');
const masterConfirmation = ref('');
const show = ref(false);
const message = ref('');
const heading = ref<HTMLElement>();
const view = computed(() => setupComplete.value ? 'login' : step.value);
const title = computed(() => ({ login: 'Iniciar sesión', master: 'Define la clave maestra', first: 'Crea el primer administrador' })[view.value]);
const description = computed(() => ({ login: 'Escribe tu usuario y tu clave para entrar al taller.', master: 'Protege la administración de usuarios de este equipo.', first: 'Esta persona tendrá acceso completo a todas las secciones.' })[view.value]);
function clearSecrets() { password.value = ''; confirmation.value = ''; masterPassword.value = ''; masterConfirmation.value = ''; show.value = false; }
onBeforeUnmount(clearSecrets);
watch(view, async () => { password.value = ''; confirmation.value = ''; show.value = false; message.value = ''; await nextTick(); heading.value?.focus(); });
async function submit() {
  if (loading.value || !available) return;
  message.value = '';
  if (view.value !== 'master' && !name.value.trim()) { message.value = 'Escribe el nombre del usuario.'; return; }
  if (!password.value) { message.value = 'Escribe la clave.'; return; }
  if (view.value !== 'login' && password.value.length < 8) { message.value = 'La clave debe tener al menos 8 caracteres.'; return; }
  if (view.value !== 'login' && password.value !== confirmation.value) { message.value = 'Las dos claves deben coincidir.'; return; }
  if (view.value === 'master') { masterPassword.value = password.value; masterConfirmation.value = confirmation.value; step.value = 'first'; return; }
  try {
    if (view.value === 'login') await login(name.value, password.value);
    else await firstSetup({ name: name.value, password: password.value, confirmation: confirmation.value, masterPassword: masterPassword.value, masterConfirmation: masterConfirmation.value });
  } finally { password.value = ''; confirmation.value = ''; show.value = false; }
}
function back() { clearSecrets(); step.value = 'master'; }
</script>

<template>
  <div class="access-shell">
    <header class="access-brand"><img :src="logo" alt="HRL" width="106" height="46"><div><strong>Bitácora del taller</strong><span>Acceso en este equipo</span></div></header>
    <div class="access-layout">
      <aside class="access-guide" aria-label="Configuración de acceso">
        <h2>Tu taller,<br>en orden.</h2>
        <p>Un acceso para cada persona que trabaja contigo.</p>
        <ol v-if="view !== 'login'" class="access-steps">
          <li :aria-current="view === 'master' ? 'step' : undefined"><span>1</span><div><b>Clave maestra</b><small>Administración de usuarios</small></div></li>
          <li :aria-current="view === 'first' ? 'step' : undefined"><span>2</span><div><b>Primer administrador</b><small>Acceso completo</small></div></li>
          <li><span>3</span><div><b>Inicio de sesión</b><small>Entrada al taller</small></div></li>
        </ol>
        <p class="access-local">Base de datos local<br><strong>Una sola computadora</strong></p>
      </aside>
      <main class="access-main" :aria-busy="loading">
        <h1 ref="heading" tabindex="-1">{{ title }}</h1>
        <p class="access-lead">{{ description }}</p>
        <p v-if="loading" class="access-feedback" role="status">Comprobando el acceso…</p>
        <p v-if="error" class="access-feedback" role="alert">{{ error }}</p>
        <button v-if="error && available" class="access-secondary" type="button" :disabled="loading" @click="refresh">Volver a comprobar</button>
        <form class="access-form" @submit.prevent="submit">
          <fieldset class="access-fields" :disabled="loading || !available">
            <div v-if="view !== 'master'" class="access-field"><label for="access-name">Usuario</label><input id="access-name" v-model="name" autocomplete="username" required maxlength="100"></div>
            <div v-if="view === 'first'" class="access-role"><strong>Acceso completo</strong><span>Sin restricciones. Siempre debe existir un administrador activo.</span></div>
            <div class="access-field"><label for="access-password">{{ view === 'master' ? 'Clave maestra' : 'Clave del usuario' }}</label><div class="access-password"><input id="access-password" v-model="password" :type="show ? 'text' : 'password'" :autocomplete="view === 'login' ? 'current-password' : 'new-password'" :minlength="view === 'login' ? undefined : 8" maxlength="1024" required aria-describedby="access-password-help"><button type="button" :aria-pressed="show" @click="show = !show">{{ show ? 'Ocultar' : 'Mostrar' }}</button></div><p id="access-password-help">{{ view === 'login' ? 'La clave maestra no sustituye la clave del usuario.' : 'Usa al menos 8 caracteres. Puedes pegar una clave de tu gestor.' }}</p></div>
            <div v-if="view !== 'login'" class="access-field"><label for="access-confirm">Repite la clave</label><input id="access-confirm" v-model="confirmation" :type="show ? 'text' : 'password'" autocomplete="new-password" required maxlength="1024" :aria-invalid="confirmation.length > 0 && confirmation !== password" :aria-describedby="confirmation.length > 0 && confirmation !== password ? 'access-match' : undefined"><p v-if="confirmation.length > 0 && confirmation !== password" id="access-match" class="access-error">Las dos claves deben coincidir.</p></div>
            <p v-if="message" class="access-feedback" role="alert">{{ message }}</p>
            <div class="access-actions"><button v-if="view === 'first'" type="button" class="access-secondary" @click="back">Atrás</button><button class="access-primary" type="submit">{{ view === 'master' ? 'Continuar' : view === 'login' ? 'Iniciar sesión' : 'Crear administrador' }}</button></div>
          </fieldset>
        </form>
        <p class="access-prototype">No hay recuperación automática de la clave maestra. Conserva la clave fuera de este equipo; el procedimiento manual sigue por definir.</p>
      </main>
    </div>
  </div>
</template>

<style src="../../assets/styles/access.css"></style>
