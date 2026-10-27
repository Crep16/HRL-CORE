<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useAccess } from '../../composables/useAccess';
const { can } = useAccess();
import { statusTone } from "../../data/orderStatuses";
import type { Order, SearchField } from "../../types/order";

const query = defineModel<string>("query", { required: true });
const field = defineModel<SearchField>("field", { required: true });

const props = defineProps<{ orders: Order[]; total: number; selectedId: string }>();
const emit = defineEmits<{ select: [id: string]; toggleSort: []; create: [] }>();

const page = ref(0);
const pageSize = 5;
const pages = computed(() => Math.max(1, Math.ceil(props.orders.length / pageSize)));
const pageOrders = computed(() => props.orders.slice(page.value * pageSize, (page.value + 1) * pageSize));
watch([query, field], () => { page.value = 0; }, { flush: 'sync' });
watch(() => props.orders.map(order => order.id).join(','), () => { page.value = Math.min(page.value, pages.value - 1); });
watch(() => props.selectedId, id => {
  const index = props.orders.findIndex(order => order.id === id);
  if (index >= 0) page.value = Math.floor(index / pageSize);
});
const reversed = ref(false);
function toggleSort() { reversed.value = !reversed.value; page.value = 0; emit('toggleSort'); }
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
      <button class="btn btn-p" type="button" :disabled="!can('orders.create')" @click="emit('create')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true" style="stroke-width: 2.2"><path d="M12 5v14M5 12h14" /></svg>Nueva orden</button>
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
      <button class="lnk" type="button" :aria-pressed="reversed" @click="toggleSort"><svg class="i sm sort-icon" :class="{ reversed }" viewBox="0 0 24 24" aria-hidden="true"><path d="m21 16-4 4-4-4" /><path d="M17 20V4" /><path d="m3 8 4-4 4 4" /><path d="M7 4v16" /></svg>Cambiar orden</button>

    </div>
    <ul class="olist">
      <li v-for="order in pageOrders" :key="order.id">
        <button class="orow" :class="{ sel: order.id === selectedId }" type="button" @click="emit('select', order.id)">
          <span class="o-top"><span class="o-name">{{ order.client }}</span><span class="o-id">{{ order.id }}</span></span>
          <span class="o-bot"><span class="st"><i class="dot" :class="statusTone(order.status)"></i>{{ order.status }}</span><span class="o-sn">{{ order.serial }}</span></span>
        </button>
      </li>
      <li v-if="orders.length === 0" class="empty-list">No hay órdenes que coincidan.</li>
    </ul>
    <nav class="order-pagination" aria-label="Páginas de órdenes">
      <div class="pagination-summary" aria-live="polite">
        <strong>Página {{ page + 1 }} de {{ pages }}</strong>
        <span v-if="orders.length">Órdenes {{ page * pageSize + 1 }}–{{ Math.min((page + 1) * pageSize, orders.length) }} de {{ orders.length }}</span>
        <span v-else>Sin resultados</span>
      </div>
      <div class="pagination-buttons">
        <button class="btn btn-s" type="button" :disabled="page === 0" aria-label="Página anterior de órdenes" @click="page--"><svg class="i sm" viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg>Anterior</button>
        <button class="btn btn-s" type="button" :disabled="page >= pages - 1" aria-label="Página siguiente de órdenes" @click="page++">Siguiente<svg class="i sm" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></button>
      </div>
    </nav>
  </aside>
</template>
