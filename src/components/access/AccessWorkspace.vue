<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import logo from "../../assets/images/brand/hrl-original.svg";
import { useAccessForm } from "../../composables/useAccessForm";

const { auth, recovery, name, password, confirmation, support, signature, newMasterPassword, newAdminPassword, successMessage, view, busy, title, errorMessage, hasSecrets, check, submit, back, toggleSupport, generateChallenge, verifySignature } = useAccessForm();
const heading = ref<HTMLElement>();
watch(title, async () => { await nextTick(); heading.value?.focus(); });
onMounted(check);
defineExpose({ prepareLeave: () => !busy.value && !hasSecrets.value });
</script>

<template>
  <section class="access-shell">
    <header class="access-brand">
      <img :src="logo" alt="HRL" width="106" height="46">
      <div><strong>Bitácora del taller</strong><span>Acceso en este equipo</span></div>
    </header>
    <div class="access-layout">
      <aside class="access-guide" aria-label="Configuración de acceso">
        <h2>Tu taller,<br>en orden.</h2>
        <p>Un acceso para cada persona que trabaja contigo.</p>
        <ol v-if="auth.setupComplete === false" class="access-steps">
          <li :aria-current="view === 'master' ? 'step' : undefined"><span>1</span><div><b>Clave maestra</b><small>Administración de usuarios</small></div></li>
          <li :aria-current="view === 'first' ? 'step' : undefined"><span>2</span><div><b>Primer administrador</b><small>Acceso completo</small></div></li>
          <li><span>3</span><div><b>Inicio de sesión</b><small>Entrada al taller</small></div></li>
        </ol>
        <p class="access-local">Base de datos local<br><strong>Una sola computadora</strong></p>
      </aside>
      <main class="access-main" :aria-busy="busy">
        <h1 ref="heading" tabindex="-1">{{ title }}</h1>
        <p v-if="busy" role="status" class="access-feedback">{{ support ? 'Procesando la solicitud de soporte…' : 'Comprobando el acceso…' }}</p>
        <p v-if="errorMessage" role="alert" class="access-feedback access-failure">{{ errorMessage }}</p>
        <p v-if="successMessage" role="status" class="access-feedback">{{ successMessage }}</p>
        <div v-if="auth.setupComplete === null">
          <p class="access-lead">Consulta el acceso configurado en este equipo.</p>
          <button type="button" class="access-secondary" :disabled="busy" @click="check">Volver a comprobar</button>
        </div>
        <template v-else-if="!support">
          <p class="access-lead">{{ view === 'login' ? 'Escribe tu usuario y tu clave para entrar al taller.' : view === 'master' ? 'Protege la administración de usuarios de este equipo.' : 'Esta persona tendrá acceso completo a todas las secciones.' }}</p>
          <form class="access-form" @submit.prevent="submit">
            <fieldset class="access-fields" :disabled="busy">
              <div v-if="view !== 'master'" class="access-field">
                <label for="access-name">Usuario</label>
                <input id="access-name" v-model="name" autocomplete="username" autocapitalize="none" spellcheck="false" maxlength="100" required>
              </div>
              <div v-if="view === 'first'" class="access-role"><strong>Acceso completo</strong><span>Sin restricciones. Siempre debe existir un administrador activo.</span></div>
              <div class="access-field">
                <label for="access-password">{{ view === 'master' ? 'Clave maestra' : 'Clave del usuario' }}</label>
                <input id="access-password" v-model="password" type="password" :autocomplete="view === 'login' ? 'current-password' : 'new-password'" :minlength="view === 'login' ? undefined : 8" maxlength="1024" required aria-describedby="access-help">
                <p id="access-help">{{ view === 'login' ? 'La clave maestra no sustituye la clave del usuario.' : 'Usa al menos 8 caracteres. Puedes pegar una clave de tu gestor.' }}</p>
              </div>
              <div v-if="view !== 'login'" class="access-field">
                <label for="access-confirm">Repite la clave</label>
                <input id="access-confirm" v-model="confirmation" type="password" autocomplete="new-password" minlength="8" maxlength="1024" required :aria-invalid="!!confirmation && confirmation !== password" :aria-describedby="confirmation && confirmation !== password ? 'access-match' : undefined">
                <p v-if="confirmation && confirmation !== password" id="access-match" class="access-error">Las dos claves deben coincidir.</p>
              </div>
              <div class="access-actions">
                <button v-if="view === 'first'" type="button" class="access-secondary" @click="back">Atrás</button>
                <button type="submit" class="access-primary">{{ view === 'master' ? 'Continuar' : view === 'first' ? 'Crear administrador' : 'Iniciar sesión' }}</button>
              </div>
            </fieldset>
          </form>
          <div v-if="view === 'login'" class="access-prototype">
            <button type="button" class="access-preview" :disabled="busy" @click="toggleSupport">Solicitar cambio de clave maestra con soporte</button>
          </div>
        </template>
        <section v-else class="access-recovery" aria-label="Recuperación por firma de soporte">
          <p class="access-lead">Envía el desafío Ed25519 al desarrollador. Su firma autoriza el cambio de la clave maestra y la del administrador indicado.</p>
          <button type="button" class="access-secondary" :disabled="busy" @click="generateChallenge">{{ recovery.challenge ? 'Generar otro desafío' : 'Generar desafío' }}</button>
          <template v-if="recovery.challenge">
            <p class="access-lead">Administrador: <strong>{{ recovery.challenge.adminUsername }}</strong>. El desafío caduca en {{ Math.floor(recovery.challenge.expiresInSeconds / 60) }} minutos; conserva esta ventana abierta.</p>
            <div class="access-field"><label for="recovery-challenge">Desafío para soporte</label><textarea id="recovery-challenge" :value="recovery.challenge.challenge" readonly rows="3" spellcheck="false"></textarea></div>
            <form class="access-form" @submit.prevent="verifySignature">
              <fieldset class="access-fields" :disabled="busy">
                <div class="access-field"><label for="recovery-signature">Firma del desarrollador</label><textarea id="recovery-signature" v-model="signature" minlength="128" maxlength="128" rows="3" autocomplete="off" spellcheck="false" required></textarea></div>
                <div class="access-field"><label for="recovery-master">Nueva clave maestra</label><input id="recovery-master" v-model="newMasterPassword" type="password" autocomplete="new-password" minlength="8" maxlength="1024" required></div>
                <div class="access-field"><label for="recovery-admin">Nueva clave del administrador</label><input id="recovery-admin" v-model="newAdminPassword" type="password" autocomplete="new-password" minlength="8" maxlength="1024" required></div>
                <div class="access-actions"><button type="submit" class="access-primary">Verificar firma y restablecer claves</button></div>
              </fieldset>
            </form>
          </template>
          <button type="button" class="access-preview" :disabled="busy" @click="toggleSupport">Volver al inicio de sesión</button>
        </section>
      </main>
    </div>
  </section>
</template>

<style src="../../assets/styles/access.css"></style>
