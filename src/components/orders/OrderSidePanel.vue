<script setup lang="ts">
import { ref } from "vue";
import { orderStatuses, statusTone } from "../../data/orderStatuses";
import type { Order } from "../../types/order";

defineProps<{ order: Order }>();

const menuOpen = ref(false);

const evidence = [
  { label: "Foto 1", kind: "photo", background: "linear-gradient(140deg,#A8CCE6,#5F9CC4)" },
  { label: "Foto 2", kind: "photo", background: "linear-gradient(140deg,#B9D5E8,#7FB0D2)" },
  { label: "Video 1", kind: "video", background: "linear-gradient(140deg,#6FA6C9,#3F7FAE)" },
  { label: "Foto 3", kind: "photo", background: "linear-gradient(140deg,#C6DCEA,#90B9D6)" },
];
</script>

<template>
  <aside class="side" aria-label="Estado y evidencias">
    <div class="block">
      <h2 class="sec">Estado actual</h2>
      <div class="combo">
        <i class="dot" :class="statusTone(order.status)"></i>
        <label class="sr" for="estado">Estado del artículo</label>
        <input id="estado" v-model="order.status" />
        <button type="button" aria-label="Ver estados predefinidos" :aria-expanded="menuOpen" @click="menuOpen = !menuOpen"><svg class="i" viewBox="0 0 24 24" aria-hidden="true" style="stroke-width: 2"><path d="m6 9 6 6 6-6" /></svg></button>
        <ul v-if="menuOpen" class="combo-menu">
          <li v-for="status in orderStatuses" :key="status.label"><button type="button" @click="order.status = status.label; menuOpen = false"><i class="dot" :class="status.tone"></i>{{ status.label }}</button></li>
        </ul>
      </div>
      <p class="help">Escribe el estado o elige uno de la lista.</p>
      <div>
        <div class="swrow"><span>Pagado</span><button class="sw" :class="{ on: order.paid }" type="button" role="switch" :aria-checked="order.paid" aria-label="Pagado" @click="order.paid = !order.paid"><span></span></button></div>
        <div class="swrow"><span>Entregado</span><button class="sw" :class="{ on: order.delivered }" type="button" role="switch" :aria-checked="order.delivered" aria-label="Entregado" @click="order.delivered = !order.delivered"><span></span></button></div>
      </div>
    </div>

    <div class="block">
      <h2 class="sec">Evidencias<small>{{ evidence.length }} archivos</small></h2>
      <div class="mosaic">
        <button v-for="(file, position) in evidence" :key="file.label" class="m" :class="{ big: position === 0 }" type="button" :aria-label="`Ver ${file.label}`" :style="{ background: file.background }">
          <svg v-if="file.kind === 'video'" class="i" viewBox="0 0 24 24" aria-hidden="true" style="fill: currentColor"><path d="m8 5 12 7-12 7z" /></svg>
          <svg v-else class="i" :class="{ lg: position === 0 }" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="2" /><circle cx="9" cy="9" r="2" /><path d="m21 15-3.1-3.1a2 2 0 0 0-2.8 0L6 21" /></svg>
          <span v-if="position === 0" class="cap">{{ file.label }}</span>
        </button>
        <button class="m add" type="button" aria-label="Agregar evidencia"><svg class="i lg" viewBox="0 0 24 24" aria-hidden="true" style="stroke-width: 2"><path d="M12 5v14M5 12h14" /></svg></button>
        <button class="m more" type="button">Ver todas</button>
      </div>
      <div class="pair">
        <button class="btn btn-p" type="button"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3z" /><circle cx="12" cy="13" r="3" /></svg>Capturar</button>
        <button class="btn btn-s" type="button"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><path d="m17 8-5-5-5 5" /><path d="M12 3v12" /></svg>Agregar</button>
      </div>
    </div>

    <div class="repa">
      <h3>Reparaciones</h3>
      <p>Línea de tiempo, garantías y publicación en la web.</p>
      <div class="facts"><span class="tag">1 cambio sin publicar</span><span class="tag">3 garantías</span></div>
      <button class="btn btn-w" type="button">Abrir gestión</button>
    </div>
  </aside>
</template>
