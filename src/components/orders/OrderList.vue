<script setup lang="ts">
import { statusTone } from "../../data/orderStatuses";
import type { Order, SearchField } from "../../types/order";

const query = defineModel<string>("query", { required: true });
const field = defineModel<SearchField>("field", { required: true });

defineProps<{ orders: Order[]; total: number; selectedId: string }>();
const emit = defineEmits<{ select: [id: string]; toggleSort: []; create: [] }>();

const fields: { value: SearchField; label: string }[] = [
  { value: "all", label: "Todo" },
  { value: "order", label: "Orden" },
  { value: "client", label: "Cliente" },
  { value: "serial", label: "Serie" },
];
</script>

<template>
  <aside class="rail" aria-label="Órdenes">
    <div class="rail-h">
      <div>
        <h2 class="h-disp">Órdenes</h2>
        <p class="subt">{{ total }} en total</p>
      </div>
      <button class="btn btn-p" type="button" @click="emit('create')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true" style="stroke-width: 2.2"><path d="M12 5v14M5 12h14" /></svg>Nueva orden</button>
    </div>
    <div class="search">
      <label class="sr" for="q">Buscar órdenes</label>
      <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" /></svg>
      <input id="q" v-model="query" placeholder="Orden, cliente o serie" />
    </div>
    <div class="seg" role="group" aria-label="Buscar en">
      <button v-for="option in fields" :key="option.value" type="button" :class="{ on: field === option.value }" :aria-pressed="field === option.value" @click="field = option.value">{{ option.label }}</button>
    </div>
    <div class="tools">
      <button class="lnk" type="button" @click="emit('toggleSort')"><svg class="i sm" viewBox="0 0 24 24" aria-hidden="true"><path d="m21 16-4 4-4-4" /><path d="M17 20V4" /><path d="m3 8 4-4 4 4" /><path d="M7 4v16" /></svg>Cambiar orden</button>
      <button class="lnk" type="button">Búsqueda avanzada</button>
    </div>
    <ul class="olist">
      <li v-for="order in orders" :key="order.id">
        <button class="orow" :class="{ sel: order.id === selectedId }" type="button" @click="emit('select', order.id)">
          <span class="o-top"><span class="o-name">{{ order.client }}</span><span class="o-id">{{ order.id }}</span></span>
          <span class="o-bot"><span class="st"><i class="dot" :class="statusTone(order.status)"></i>{{ order.status }}</span><span class="o-sn">{{ order.serial }}</span></span>
        </button>
      </li>
      <li v-if="orders.length === 0" class="empty-list">No hay órdenes que coincidan.</li>
    </ul>
  </aside>
</template>
