<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import brandLogo from "../../assets/images/brand/hrl-symbol-original.svg";
import AppIcon from '../ui/AppIcon.vue';
const header = ref<HTMLElement>();
let observer: ResizeObserver | undefined;
function measureHeader() {
  document.documentElement.style.setProperty('--app-header-bottom', `${Math.max(0, header.value?.getBoundingClientRect().bottom ?? 76)}px`);
}
onMounted(() => {
  observer = new ResizeObserver(measureHeader);
  if (header.value) observer.observe(header.value);
  window.addEventListener('scroll', measureHeader, { passive:true });
  measureHeader();
});
onUnmounted(() => { observer?.disconnect(); window.removeEventListener('scroll', measureHeader); });
const levels = { complete: 'Acceso completo', limited: 'Acceso limitado', custom: 'Acceso personalizado' };
const tabIcons: Record<string, string> = { Clientes: 'clients', Reparaciones: 'repair', Bitácora: 'logbook', Empresa: 'company', Usuarios: 'users' };

withDefaults(defineProps<{
  tabs: string[];
  active: string;
  user?: { name: string; loginName?: string; accessLevel: 'complete' | 'limited' | 'custom' } | null;
  heading?: string;
  canPrint?: boolean;
  canRepair?: boolean;
  printActions?: boolean;
  temporaryAccess?: boolean;
  busy?: boolean;
  clientTools?: boolean;
  canViewHistory?: boolean;
}>(), { printActions: true });
const emit = defineEmits<{ change: [tab: string]; print: [kind: 'entry' | 'exit']; logout: []; requestAccess: []; lookup: []; history: []; refresh: [] }>();
const toolsMenu = ref<HTMLDetailsElement>();
function closeTools() { if (toolsMenu.value) toolsMenu.value.open = false; }

const printIcon = "M6 9V2h12v7M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2";
</script>

<template>
  <header ref="header" class="top" :class="{ 'reception-header': !!heading }">
    <div class="logo-tile"><img :src="brandLogo" alt="HRL – Héctor Refacciones Laptop" /></div>
    <div v-if="heading" class="workshop-heading"><AppIcon name="clients" /><strong>{{ heading }}</strong></div>
    <nav v-else class="tabs" aria-label="Secciones">
      <button v-for="tab in tabs" :key="tab" class="tab" :disabled="tab === 'Reparaciones' && !canRepair" :title="tab === 'Reparaciones' && !canRepair ? 'Selecciona una orden activa' : undefined" :class="{ on: tab === active }" type="button" :aria-current="tab === active ? 'page' : undefined" @click="emit('change', tab)"><AppIcon v-if="tabIcons[tab]" :name="tabIcons[tab]" />{{ tab }}</button>
    </nav>
    <div class="header-actions" role="group" aria-label="Acciones del apartado">
    <button v-if="temporaryAccess && !printActions" class="ghost" type="button" :disabled="busy" @click="emit('requestAccess')"><AppIcon name="shield" />Solicitar acceso temporal</button>
    <template v-if="printActions">
    <button class="ghost" type="button" :disabled="!canPrint" title="Imprimir con la configuración guardada de Empresa" @click="emit('print','entry')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path :d="printIcon" /><rect x="6" y="14" width="12" height="8" /></svg>Imprimir nota entrada</button>
    <button class="ghost" type="button" :disabled="!canPrint" title="Imprimir con la configuración guardada de Empresa" @click="emit('print','exit')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path :d="printIcon" /><rect x="6" y="14" width="12" height="8" /></svg>Imprimir nota salida</button>
    <details ref="toolsMenu" class="header-tools" @keydown.esc.stop="closeTools"><summary class="ghost"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6" /></svg>Herramientas</summary><div class="header-tools-menu" @click="closeTools"><button v-if="temporaryAccess" type="button" :disabled="busy" @click="emit('requestAccess')"><AppIcon name="shield" />Solicitar acceso temporal</button><template v-if="clientTools"><button type="button" :disabled="busy" @click="emit('lookup')"><AppIcon name="search" />Consultar por folio o ID</button><button type="button" :disabled="busy" @click="emit('refresh')"><AppIcon name="restore" />Actualizar órdenes</button><button type="button" :disabled="busy || !canViewHistory" @click="emit('history')"><AppIcon name="logbook" />Historial de la orden</button></template></div></details>
    </template>
    </div>
    <div class="header-session">
      <div v-if="user" class="active-user" role="group" aria-label="Usuario activo">
        <AppIcon name="user" />
        <div class="active-user-copy"><strong :title="user.name">{{ user.loginName || user.name }}</strong><span>{{ levels[user.accessLevel] }}</span></div>
      </div>
      <button class="ghost out" type="button" :disabled="busy" @click="emit('logout')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><path d="m16 17 5-5-5-5" /><path d="M21 12H9" /></svg>Cerrar sesión</button>
    </div>
  </header>
</template>
