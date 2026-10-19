<script setup lang="ts">
import { computed, ref } from "vue";
import type { Client } from "../types/client";
import { initials } from "../utils/text";

const props = defineProps<{ clients: Client[]; selectedId: string }>();
const emit = defineEmits<{ select: [id: string]; create: [] }>();

const search = ref("");
const frequentOnly = ref(false);

const filteredClients = computed(() => {
  const query = search.value.trim().toLocaleLowerCase("es");
  return props.clients.filter((client) =>
    (!frequentOnly.value || client.frequent) && `${client.name} ${client.phone} ${client.id}`.toLocaleLowerCase("es").includes(query),
  );
});
</script>

<template>
  <aside class="directory" aria-label="Directorio de clientes">
    <div class="directory-heading">
      <div><p class="eyebrow">DIRECTORIO</p><h1>Clientes</h1><p class="subtle">{{ clients.length }} registros</p></div>
      <button class="button primary new-button" type="button" @click="emit('create')"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>Nuevo</button>
    </div>
    <label class="search-box"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7" /><path d="m20 20-4-4" /></svg><span class="sr-only">Buscar clientes</span><input v-model="search" type="search" placeholder="Nombre, teléfono o clave" /><kbd>/</kbd></label>
    <div class="filter-tabs" role="group" aria-label="Filtrar clientes">
      <button type="button" :class="{ active: !frequentOnly }" @click="frequentOnly = false">Todos</button>
      <button type="button" :class="{ active: frequentOnly }" @click="frequentOnly = true">Frecuentes</button>
    </div>
    <div class="list-caption"><span>CLIENTE</span><span>{{ filteredClients.length }} / {{ clients.length }}</span></div>
    <ul class="client-list">
      <li v-for="client in filteredClients" :key="client.id">
        <button class="client-row" :class="{ selected: selectedId === client.id }" type="button" @click="emit('select', client.id)">
          <span class="avatar" :class="`avatar-${client.id.slice(-1)}`">{{ initials(client.name) }}</span>
          <span class="client-copy"><span class="client-line"><strong>{{ client.name }}</strong><small>{{ client.id }}</small></span><span class="client-line secondary-line"><span>{{ client.phone }}</span><i v-if="client.frequent" title="Cliente frecuente">★</i></span></span>
        </button>
      </li>
      <li v-if="filteredClients.length === 0" class="empty-state">No hay clientes que coincidan con la búsqueda.</li>
    </ul>
    <p class="legend"><i></i> Cliente frecuente</p>
  </aside>
</template>
