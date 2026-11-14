<script setup lang="ts">
import AppIcon from "../ui/AppIcon.vue";
import SearchSelect from "../ui/SearchSelect.vue";
import OrderEvidence from "./OrderEvidence.vue";
import { useReceptionForm } from "../../composables/useReceptionForm";
import { orderStatuses, statusTone } from "../../data/orderStatuses";
import { formatMinorUnits } from "../../utils/minorUnits.ts";
const { auth,store, draft, canEdit, editing,editExisting,delivered,trashed } = useReceptionForm();
const emit = defineEmits<{ repairs: [] }>();
</script>
<template>
  <aside class="side" aria-label="Estado de la orden">
    <section class="block"><h2 class="sec"><AppIcon name="status" />{{ !editing && delivered ? 'Pago y entrega' : 'Estado actual' }}</h2>
      <SearchSelect v-if="editing" id="estado" v-model="draft.status" label="Estado del artículo" :options="orderStatuses.map(status=>status.label)" :disabled="!canEdit" allow-custom required />
      <template v-else-if="!delivered"><p class="status-value"><i class="dot" :class="statusTone(draft.status)" aria-hidden="true"></i>{{ draft.status }}</p><p class="help">Estado registrado en esta orden.</p></template>
      <p v-if="editing" class="help">Escribe el estado o elige uno de la lista.</p>
      <dl class="order-balance"><div class="swrow"><dt>Pagado</dt><dd class="status-value" :class="{ confirmed: !editing && store.currentOrder?.pendingMinor === 0 }">{{ editing ? 'Por confirmar' : store.currentOrder?.pendingMinor === 0 ? 'Sí' : 'Pendiente' }}</dd></div><div class="swrow"><dt>Entregado</dt><dd><button v-if="editExisting" class="sw" :class="{on:delivered}" type="button" role="switch" :aria-checked="delivered" aria-label="Entregado" :disabled="!canEdit" @click="delivered=!delivered"><span></span></button><strong v-else class="status-value" :class="{confirmed:delivered}">{{ delivered ? 'Sí' : 'Pendiente' }}</strong></dd></div></dl>
      <p v-if="editing" class="help">El pago se calcula con los abonos registrados.</p>
    </section>
    <details class="evidence-section"><summary>Evidencias</summary><OrderEvidence v-if="!editing && store.currentOrder" :key="store.currentOrder.id" :order-id="String(store.currentOrder.id)" :trashed="trashed" :disabled="store.isLoading" /><p v-else class="help">Guarda la orden para agregar evidencias.</p></details>
    <div class="repa">
      <h3><AppIcon name="repair" />Reparaciones</h3>
      <p>Avances, notas del técnico y garantías de esta orden.</p>
      <div class="facts"><span class="tag">Seguimiento técnico</span><span class="tag">Garantías</span></div>
      <button class="btn btn-w" type="button" :disabled="editing || trashed || store.isLoading || !store.currentOrder || !auth.can('repairs.view','repairs',store.currentOrder.id)" @click="emit('repairs')"><AppIcon name="repair" />Abrir gestión</button>
    </div>
    <details v-if="!editing && store.currentOrder" class="reception-details"><summary>Saldo de la orden</summary><dl class="order-balance"><div class="swrow"><dt>Pendiente</dt><dd>{{ formatMinorUnits(store.currentOrder.pendingMinor) }} {{ draft.currency }}</dd></div><div class="swrow"><dt>Saldo a favor</dt><dd>{{ formatMinorUnits(store.currentOrder.creditMinor) }} {{ draft.currency }}</dd></div></dl></details>
  </aside>
</template>
