<script setup lang="ts">
import AppIcon from '../ui/AppIcon.vue';
import SearchSelect from '../ui/SearchSelect.vue';
import OrderEvidence from './OrderEvidence.vue';
import { orderStatuses, statusTone } from "../../data/orderStatuses";
import type { Order } from "../../types/order";
import { computed } from 'vue';
import { orderAmount } from '../../utils/orderAmount';

const props=defineProps<{ order: Order; editing: boolean; initialPayment?: boolean; unavailable?: boolean; blocked?: boolean }>();
const paid=computed(()=>props.editing&&props.initialPayment ? orderAmount(props.order.deposit)>=orderAmount(props.order.cost) : props.order.paid);

const emit = defineEmits<{ repairs: [] }>();

</script>

<template>
  <aside class="side" aria-label="Estado y evidencias">
    <div class="block">
      <h2 class="sec"><AppIcon name="status" />{{ !editing && order.delivered ? 'Pago y entrega' : 'Estado actual' }}</h2>
      <template v-if="editing || !order.delivered">
      <SearchSelect v-if="editing" id="estado" v-model="order.status" label="Estado del artículo" :options="orderStatuses.map(status=>status.label)" allow-custom required />
      <p v-else class="status-value"><i class="dot" :class="statusTone(order.status)"></i> {{ order.status }}</p>
      <p v-if="editing" class="help">Escribe el estado o elige uno de la lista.</p>
      <p v-else class="help">Estado registrado en esta orden.</p>
      </template>
      <div>
        <div class="swrow"><span>Pagado</span><strong class="status-value" :class="{ confirmed: paid }">{{ paid ? 'Sí' : 'Pendiente' }}</strong></div>
        <div class="swrow"><span>Entregado</span><button v-if="editing" class="sw" :class="{ on: order.delivered }" type="button" role="switch" :aria-checked="order.delivered" aria-label="Entregado" @click="order.delivered = !order.delivered"><span></span></button><strong v-else class="status-value" :class="{ confirmed: order.delivered }">{{ order.delivered ? 'Sí' : 'Pendiente' }}</strong></div>
      </div>
      <p v-if="editing" class="help">El pago se calcula con los abonos registrados.</p>
    </div>

    <OrderEvidence :key="order.id" :order-id="order.id" :disabled="editing||blocked" :trashed="unavailable" />

    <div class="repa">
      <h3><AppIcon name="repair" />Reparaciones</h3>
      <p>Avances, notas del técnico y garantías de esta orden.</p>
      <div class="facts"><span class="tag">Seguimiento técnico</span><span class="tag">Garantías</span></div>
      <button class="btn btn-w" type="button" :disabled="editing || unavailable" @click="emit('repairs')"><AppIcon name="repair" />Abrir gestión</button>
    </div>
  </aside>
</template>
