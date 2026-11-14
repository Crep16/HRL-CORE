<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { useReceptionForm } from "../../composables/useReceptionForm";
import { formatMinorUnits } from "../../utils/minorUnits.ts";
const { store, historyOpen, historyPage, historyPages, loadHistory, closeHistory } = useReceptionForm();
const dialog = ref<HTMLDialogElement>();
const selectedId = ref<number | null>(null);
const selected = computed(() => store.history.find(event => event.id === selectedId.value) ?? store.history[0]);
let previousFocus: HTMLElement | null = null;
const labels: Record<string, string> = { priceMinor: "Precio", amountMinor: "Importe", method: "Método de pago", folio: "Folio", clientId: "ID del cliente", equipmentId: "ID del equipo" };
const operations: Record<string, string> = { "orders.create": "Orden creada", "orders.edit": "Orden modificada", "orders.delete": "Orden enviada a papelera", "orders.restore": "Orden restaurada", "payments.add": "Abono registrado", "payments.refund": "Devolución registrada", "payments.adjust": "Abono corregido", "repairs.edit": "Reparación actualizada" };
const results: Record<string, string> = { success: "Correcto", denied: "Rechazado", error: "Error" };
function dateLabel(text: string) {
  const date = new Date(text);
  return Number.isNaN(date.getTime()) ? text : date.toLocaleString("es-MX", { dateStyle: "medium", timeStyle: "short" });
}
function value(field: string, text: string | null) {
  if (text === null) return "Sin valor";
  if (["priceMinor", "amountMinor"].includes(field) && /^-?\d+$/.test(text) && Number.isSafeInteger(Number(text))) {
    return `${formatMinorUnits(Number(text))} ${store.currentOrder?.currency ?? ""}`;
  }
  return text;
}
watch(historyOpen, async (open) => {
  await nextTick();
  if (open) {
    selectedId.value = null;
    previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    if (!dialog.value?.open) dialog.value?.showModal();
  } else { dialog.value?.close(); previousFocus?.focus(); }
});
onBeforeUnmount(() => { dialog.value?.close(); });
</script>
<template>
  <dialog ref="dialog" class="workshop-dialog order-history-dialog" aria-labelledby="order-history-title" @cancel.prevent="closeHistory">
    <header><h2 id="order-history-title">Historial · {{ store.currentOrder?.folio }}</h2><button type="button" autofocus class="btn btn-s" @click="closeHistory">Cerrar</button></header>
    <p v-if="store.isHistoryLoading" role="status">Consultando el historial…</p>
    <p v-if="store.historyError" role="alert" class="workshop-error">{{ store.historyError.message }}</p>
    <div v-if="store.history.length" class="history-layout" :aria-busy="store.isHistoryLoading">
      <section class="history-movements" aria-labelledby="history-movements-title">
        <header><h3 id="history-movements-title">Movimientos</h3><span>{{ store.historyTotal }} registros</span></header>
        <div class="history-table-scroll">
          <table class="history-table">
            <thead><tr><th scope="col">Fecha de registro</th><th scope="col">Operación</th><th scope="col">Usuario</th></tr></thead>
            <tbody><tr v-for="event in store.history" :key="event.id" :class="{ selected: event.id === selected?.id }">
              <td><time :datetime="event.createdAt">{{ dateLabel(event.createdAt) }}</time></td>
              <td><button type="button" class="history-select" :aria-pressed="event.id === selected?.id" aria-controls="history-detail" @click="selectedId = event.id">{{ operations[event.operation] ?? 'Cambio registrado' }}<small>Registro {{ event.id }}</small></button></td>
              <td>{{ event.authorName }}</td>
            </tr></tbody>
          </table>
        </div>
      </section>
      <section v-if="selected" id="history-detail" class="history-detail" aria-labelledby="history-detail-title">
        <h3 id="history-detail-title">Detalle del movimiento</h3>
        <h4>{{ operations[selected.operation] ?? 'Cambio registrado' }}</h4>
        <dl class="history-facts">
          <div><dt>Realizado por</dt><dd>{{ selected.authorName }}</dd></div>
          <div><dt>Registrado el</dt><dd>{{ dateLabel(selected.createdAt) }}</dd></div>
          <div><dt>Resultado</dt><dd>{{ results[selected.result] ?? selected.result }}</dd></div>
          <div><dt>Motivo</dt><dd>{{ selected.reason || 'No registrado para esta operación' }}</dd></div>
        </dl>
        <div v-for="change in selected.changes" :key="change.field" class="history-change">
          <h4>{{ labels[change.field] ?? change.field }}</h4>
          <dl><div><dt>Antes</dt><dd>{{ value(change.field, change.oldValue) }}</dd></div><div><dt>Después</dt><dd>{{ value(change.field, change.newValue) }}</dd></div></dl>
        </div>
        <p class="help">Registro interno de consulta. Los movimientos no se pueden editar ni borrar.</p>
      </section>
    </div>
    <p v-if="!store.isHistoryLoading && !store.historyError && !store.history.length">No hay eventos registrados para esta orden.</p>
    <footer><span>Página {{ historyPage + 1 }} de {{ historyPages }}</span><button type="button" class="btn btn-s" :disabled="store.isHistoryLoading || historyPage === 0" @click="loadHistory(historyPage - 1)">Anterior</button><button type="button" class="btn btn-s" :disabled="store.isHistoryLoading || historyPage + 1 >= historyPages" @click="loadHistory(historyPage + 1)">Siguiente</button><button type="button" class="btn btn-s" :disabled="store.isHistoryLoading" @click="loadHistory(historyPage)">Actualizar</button></footer>
  </dialog>
</template>
