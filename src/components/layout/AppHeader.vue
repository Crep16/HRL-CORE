<script setup lang="ts">
import brandLogo from "../../assets/images/brand/hrl-symbol-original.svg";
import AppIcon from '../ui/AppIcon.vue';
import { useAccess } from '../../composables/useAccess';
const { user } = useAccess();
const levels = { complete: 'Acceso completo', limited: 'Acceso limitado', custom: 'Acceso personalizado' };
const tabIcons: Record<string, string> = { Clientes: 'clients', Reparaciones: 'repair', Bitácora: 'logbook', Empresa: 'company', Usuarios: 'users' };

defineProps<{ tabs: string[]; active: string; canPrint?: boolean; canRepair?: boolean }>();
const emit = defineEmits<{ change: [tab: string]; print: [kind: 'entry' | 'exit']; logout: [] }>();

const printIcon = "M6 9V2h12v7M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2";
</script>

<template>
  <header class="top">
    <div class="logo-tile"><img :src="brandLogo" alt="HRL – Héctor Refacciones Laptop" /></div>
    <nav class="tabs" aria-label="Secciones">
      <button v-for="tab in tabs" :key="tab" class="tab" :disabled="tab === 'Bitácora' || (tab === 'Reparaciones' && !canRepair)" :title="tab === 'Bitácora' ? 'Bitácora pendiente de implementar' : tab === 'Reparaciones' && !canRepair ? 'Selecciona una orden activa' : undefined" :class="{ on: tab === active }" type="button" :aria-current="tab === active ? 'page' : undefined" @click="emit('change', tab)"><AppIcon v-if="tabIcons[tab]" :name="tabIcons[tab]" />{{ tab }}</button>
    </nav>
    <div class="header-actions" role="group" aria-label="Impresión y herramientas">
    <button class="ghost" type="button" :disabled="!canPrint" title="Imprimir con la configuración guardada de Empresa" @click="emit('print','entry')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path :d="printIcon" /><rect x="6" y="14" width="12" height="8" /></svg>Imprimir nota entrada</button>
    <button class="ghost" type="button" :disabled="!canPrint" title="Imprimir con la configuración guardada de Empresa" @click="emit('print','exit')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path :d="printIcon" /><rect x="6" y="14" width="12" height="8" /></svg>Imprimir nota salida</button>
    <button class="ghost" type="button" disabled title="Disponible en una próxima etapa"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6" /></svg>Herramientas</button>
    </div>
    <div class="header-session">
      <div v-if="user" class="active-user" role="group" aria-label="Usuario activo">
        <AppIcon name="user" />
        <div class="active-user-copy"><strong :title="user.name">{{ user.name }}</strong><span>{{ levels[user.accessLevel] }}</span></div>
      </div>
      <button class="ghost out" type="button" @click="emit('logout')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><path d="m16 17 5-5-5-5" /><path d="M21 12H9" /></svg>Cerrar sesión</button>
    </div>
  </header>
</template>
