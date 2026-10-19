<script setup lang="ts">
import type { Client } from "../types/client";
import { initials, phoneHref } from "../utils/text";

defineProps<{ client: Client; position: number; total: number }>();
const emit = defineEmits<{ move: [step: number] }>();
</script>

<template>
  <section class="profile" aria-label="Ficha del cliente">
    <header class="profile-header">
      <div class="identity"><span class="profile-avatar">{{ initials(client.name) }}</span><div><p class="eyebrow">{{ client.id }} <b>/</b> PERFIL DEL CLIENTE</p><h2>{{ client.name }}</h2><p class="subtle">Cliente desde {{ client.since }}</p></div></div>
      <div class="profile-actions">
        <div class="record-nav" role="group" aria-label="Navegar entre clientes"><button type="button" aria-label="Cliente anterior" title="Anterior" @click="emit('move', -1)"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg></button><span>{{ position }} de {{ total }}</span><button type="button" aria-label="Cliente siguiente" title="Siguiente" @click="emit('move', 1)"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></button></div>
        <a class="button secondary" :href="phoneHref(client.phone)"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M22 16.9v3a2 2 0 0 1-2.2 2 19.8 19.8 0 0 1-8.6-3.1 19.4 19.4 0 0 1-6-6A19.8 19.8 0 0 1 2.1 4.2 2 2 0 0 1 4.1 2h3a2 2 0 0 1 2 1.7l.4 2.8a2 2 0 0 1-.6 1.7L7.5 9.6a16 16 0 0 0 6 6l1.4-1.4a2 2 0 0 1 1.7-.6l2.8.4a2 2 0 0 1 1.6 1.9Z" /></svg>Llamar</a>
      </div>
    </header>
    <div class="perforation"></div>

    <div class="profile-body">
      <div class="profile-main">
        <section class="detail-section"><h3 class="section-title">Datos de contacto</h3><div class="field-grid">
          <div class="data-field"><span>Teléfono</span><a :href="phoneHref(client.phone)">{{ client.phone }}</a></div>
          <div class="data-field"><span>Correo electrónico</span><a :href="`mailto:${client.email}`">{{ client.email }}</a></div>
          <div class="data-field"><span>Tipo de cliente</span><strong>{{ client.kind }}</strong></div>
          <div class="data-field"><span>RFC</span><strong>{{ client.taxId }}</strong></div>
        </div></section>
        <section class="detail-section"><h3 class="section-title">Domicilio</h3><div class="field-grid address-grid">
          <div class="data-field"><span>Dirección</span><strong>{{ client.address }}</strong></div>
          <div class="data-field"><span>Ciudad y código postal</span><strong>{{ client.city || 'Pendiente de captura' }}</strong></div>
        </div></section>
        <section class="detail-section"><div class="section-heading"><h3 class="section-title">Actividad en el taller</h3><small>HISTORIAL RECIENTE</small></div>
          <div class="stats"><div><span>Órdenes</span><strong>{{ client.orders }}</strong></div><div><span>Total registrado</span><strong>{{ client.spent }}</strong></div><div><span>Frecuencia</span><strong>{{ client.frequent ? 'Recurrente' : 'Ocasional' }}</strong></div></div>
          <div v-if="client.activity.length" class="activity-list"><div v-for="item in client.activity" :key="item.order" class="activity-row"><span class="activity-icon"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M8 2v4m8-4v4M3 10h18" /></svg></span><span class="activity-copy"><strong>{{ item.item }}</strong><small>{{ item.order }} <i></i> {{ item.date }}</small></span><span class="status" :class="`tone-${item.tone}`"><i></i>{{ item.status }}</span></div></div>
          <p v-else class="no-activity">Este cliente aún no tiene órdenes registradas.</p>
        </section>
        <section class="detail-section"><h3 class="section-title">Notas internas</h3><p class="notes">{{ client.notes }}</p></section>
      </div>
      <aside class="profile-aside"><section class="detail-section"><h3 class="section-title">Resumen</h3><div class="active-status"><i></i>Registro activo<svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg></div><p class="aside-copy">La ficha está disponible para nuevas órdenes de servicio.</p></section>
        <section class="detail-section"><h3 class="section-title">Datos rápidos</h3><div class="quick-row"><span>Clave</span><strong>{{ client.id }}</strong></div><div class="quick-row"><span>Registro</span><strong>{{ client.since }}</strong></div><div class="quick-row"><span>Clasificación</span><strong>{{ client.frequent ? 'Frecuente' : 'General' }}</strong></div></section>
        <div class="aside-note"><i>i</i><p>Los datos de contacto ayudan a mantener actualizado el seguimiento de cada equipo.</p></div>
      </aside>
    </div>
  </section>
</template>
