<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import logo from '../../assets/images/brand/hrl-original.svg';
const emit = defineEmits<{ preview: [] }>();
type View = 'login' | 'master' | 'first' | 'account';
const view = ref<View>('master');
const name = ref('');
const password = ref('');
const confirmation = ref('');
const show = ref(false);
const level = ref('limited');
const message = ref('');
const heading = ref<HTMLElement>();
const sections = ['Clientes', 'Reparaciones', 'Bitácora', 'Empresa'];
const actions = ['Visualizar', 'Crear', 'Editar', 'Borrar'];
const grants = ref<string[]>([]);
const title = computed(() => ({login:'Iniciar sesión', master:'Define la clave maestra', first:'Crea el primer administrador', account:'Crear una cuenta'})[view.value]);
const description = computed(() => ({login:'Escribe tu usuario y tu clave para entrar al taller.', master:'Protege la administración de usuarios de este equipo.', first:'Esta persona tendrá acceso completo a todas las secciones.', account:'Define el usuario y las acciones que podrá realizar.'})[view.value]);
watch(view, async () => {
  password.value=''; confirmation.value=''; show.value=false; message.value='';
  await nextTick(); heading.value?.focus();
});
function submit() {
  message.value='';
  if (view.value !== 'master' && !name.value.trim()) { message.value='Escribe el nombre del usuario.'; return; }
  if (view.value !== 'login' && password.value.length < 8) { message.value='La clave debe tener al menos 8 caracteres.'; return; }
  if (!password.value) { message.value='Escribe la clave.'; return; }
  if (view.value !== 'login' && password.value !== confirmation.value) { message.value='Las dos claves deben coincidir.'; return; }
  if (view.value === 'master') {view.value='first'; return;}
  message.value='Formulario validado. La conexión con las cuentas SQLite todavía está pendiente; no se ha guardado ninguna cuenta ni iniciado una sesión.';
  password.value=''; confirmation.value='';
}
</script>

<template>
  <div class="access-shell">
    <header class="access-brand"><img :src="logo" alt="HRL" width="106" height="46"><div><strong>Bitácora del taller</strong><span>Acceso en este equipo</span></div></header>
    <div class="access-layout">
      <aside class="access-guide" aria-label="Configuración de acceso">
        <h2>Tu taller,<br>en orden.</h2>
        <p>Un acceso para cada persona que trabaja contigo.</p>
        <ol v-if="view === 'master' || view === 'first'" class="access-steps">
          <li :aria-current="view === 'master' ? 'step' : undefined"><span>1</span><div><b>Clave maestra</b><small>Administración de usuarios</small></div></li>
          <li :aria-current="view === 'first' ? 'step' : undefined"><span>2</span><div><b>Primer administrador</b><small>Acceso completo</small></div></li>
          <li><span>3</span><div><b>Inicio de sesión</b><small>Entrada al taller</small></div></li>
        </ol>
        <p class="access-local">Base de datos local<br><strong>Una sola computadora</strong></p>
      </aside>
      <main class="access-main">
        <nav class="access-tabs" aria-label="Vistas de acceso">
          <button type="button" :aria-current="view==='login' ? 'page' : undefined" @click="view='login'">Iniciar sesión</button>
          <button type="button" :aria-current="view==='master'||view==='first' ? 'page' : undefined" @click="view='master'">Primer uso</button>
          <button type="button" :aria-current="view==='account' ? 'page' : undefined" @click="view='account'">Crear cuenta</button>
        </nav>
        <h1 ref="heading" tabindex="-1">{{ title }}</h1>
        <p class="access-lead">{{ description }}</p>
        <form class="access-form" @submit.prevent="submit">
          <div v-if="view!=='master'" class="access-field"><label for="access-name">{{ view==='login' ? 'Usuario' : 'Nombre completo' }}</label><input id="access-name" v-model="name" :autocomplete="view==='login' ? 'username' : 'name'" required maxlength="100"></div>
          <div v-if="view==='first'" class="access-role"><strong>Acceso completo</strong><span>Sin restricciones. Siempre debe existir un administrador activo.</span></div>
          <div v-if="view==='account'" class="access-field"><label for="access-level">Nivel de acceso</label><select id="access-level" v-model="level"><option value="complete">Completo</option><option value="limited">Limitado</option><option value="custom">Personalizado</option></select><p>{{ level==='complete' ? 'Todas las secciones y la administración de usuarios.' : level==='limited' ? 'Trabajo en Reparaciones. Las acciones exactas siguen por definir.' : 'Selecciona los apartados y las acciones permitidas.' }}</p></div>
          <fieldset v-if="view==='account' && level==='custom'" class="access-permissions"><legend>Permisos por apartado</legend><div v-for="section in sections" :key="section"><strong>{{ section }}</strong><div><label v-for="action in actions" :key="action"><input v-model="grants" type="checkbox" :value="`${section}:${action}`">{{ action }}</label></div></div></fieldset>
          <div class="access-field"><label for="access-password">{{ view==='master' ? 'Clave maestra' : 'Clave del usuario' }}</label><div class="access-password"><input id="access-password" v-model="password" :type="show?'text':'password'" :autocomplete="view==='login'?'current-password':'new-password'" :minlength="view==='login'?undefined:8" required aria-describedby="access-password-help"><button type="button" :aria-pressed="show" @click="show=!show">{{ show?'Ocultar':'Mostrar' }}</button></div><p id="access-password-help">{{ view==='login'?'Si no recuerdas tu clave, consulta al administrador.':'Usa al menos 8 caracteres. Puedes pegar una clave de tu gestor.' }}</p></div>
          <div v-if="view!=='login'" class="access-field"><label for="access-confirm">Repite la clave</label><input id="access-confirm" v-model="confirmation" :type="show?'text':'password'" autocomplete="new-password" required :aria-invalid="confirmation.length>0 && confirmation!==password" :aria-describedby="confirmation.length>0 && confirmation!==password ? 'access-match' : undefined"><p v-if="confirmation.length>0 && confirmation!==password" id="access-match" class="access-error">Las dos claves deben coincidir.</p></div>
          <p v-if="message" class="access-feedback" role="status">{{ message }}</p>
          <div class="access-actions"><button v-if="view==='first'" type="button" class="access-secondary" @click="view='master'">Atrás</button><button class="access-primary" type="submit">{{ view==='master'?'Continuar':view==='login'?'Comprobar formulario':view==='first'?'Validar administrador':'Validar cuenta' }}</button></div>
        </form>
        <p class="access-prototype">Vista previa de acceso. Aún no guarda cuentas ni autentica. No uses una clave real.</p>
        <button class="access-preview" type="button" @click="emit('preview')">Ver la aplicación en desarrollo</button>
      </main>
    </div>
  </div>
</template>

<style src="../../assets/styles/access.css"></style>
