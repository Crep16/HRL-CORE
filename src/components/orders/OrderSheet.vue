<script setup lang="ts">
import OrderForm from "./OrderForm.vue";
import OrderSidePanel from "./OrderSidePanel.vue";
import AppIcon from '../ui/AppIcon.vue';
import type { Order } from "../../types/order";
import { useAccess } from '../../composables/useAccess';
const { can } = useAccess();

defineProps<{ order: Order; position: number; total: number; editing: boolean; busy?: boolean; initialPayment?: boolean; trashed?: boolean }>();
const emit = defineEmits<{ goTo: [position: number]; remove: []; restore: []; edit: []; save: []; cancel: []; repairs: [] }>();
</script>

<template>
  <section class="sheet" :aria-label="`Orden ${order.id}`">
    <div class="sh-h">
      <div>
        <h1 class="num" tabindex="-1">{{ order.id }}</h1>
        <p v-if="trashed" class="subt">En la papelera · historial y cobros conservados</p>
        <p class="who">{{ order.client }}</p>
        <p class="subt">Recibida el {{ order.received }}</p>
      </div>
      <div class="sh-act">
        <button v-if="editing" class="btn btn-p" type="button" :disabled="busy" @click="emit('save')">Guardar cambios</button><button v-if="editing" class="btn btn-s" type="button" :disabled="busy" @click="emit('cancel')">Cancelar</button>
        <div v-if="!editing" class="navgrp" role="group" aria-label="Avanzar entre órdenes">
          <button type="button" aria-label="Primera orden" title="Primera" @click="emit('goTo', 0)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m11 17-5-5 5-5" /><path d="m18 17-5-5 5-5" /></svg></button>
          <button type="button" aria-label="Orden anterior" title="Anterior" @click="emit('goTo', position - 2)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg></button>
          <span>{{ position }} de {{ total }}</span>
          <button type="button" aria-label="Orden siguiente" title="Siguiente" @click="emit('goTo', position)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></button>
          <button type="button" aria-label="Última orden" title="Última" @click="emit('goTo', total - 1)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m6 17 5-5-5-5" /><path d="m13 17 5-5-5-5" /></svg></button>
        </div>
        <button v-if="!editing && !trashed" class="btn btn-s" type="button" :disabled="busy || !can('orders.edit')" @click="emit('edit')"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 20h9" /><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4z" /></svg>Editar</button>
        <button v-if="!editing && !trashed" class="btn btn-s" type="button" :disabled="busy || !can('orders.delete')" @click="emit('remove')"><AppIcon name="trash" />Eliminar</button>
        <button v-if="trashed" class="btn btn-p" type="button" :disabled="busy || !can('orders.delete')" @click="emit('restore')"><AppIcon name="restore" />Restaurar orden</button>
      </div>
    </div>
    <div class="perf" aria-hidden="true"></div>

    <div class="sh-b">
      <OrderForm :order="order" :editing="editing && !busy" :initial-payment="initialPayment" />
      <OrderSidePanel :order="order" :editing="editing && !busy" :initial-payment="initialPayment" :unavailable="trashed" @repairs="emit('repairs')" />
    </div>
  </section>
</template>
