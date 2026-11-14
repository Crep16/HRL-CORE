<script setup lang="ts">
import AppNotice from '../ui/AppNotice.vue';
import brandLogo from '../../assets/images/brand/hrl-symbol-original.svg';
import type { CompanySettings, NoteKind } from '../../types/company';
import type { Order } from '../../types/order';
import { useCompanyNote } from '../../composables/useCompanyNote';
const props = defineProps<{ settings: CompanySettings; kind: NoteKind; order?: Order; folio?: string }>();
const {clauses,total,repairSummary,summaryError,prepare,subtotal,money}=useCompanyNote(props);
defineExpose({prepare});
</script>

<template>
  <AppNotice :message="summaryError" tone="error" />
  <article class="company-note" :class="{ 'note-ticket':settings.paper !== 'letter', 'note-ticket60':settings.paper==='ticket60', 'note-landscape':settings.paper==='letter' && settings.orientation === 'landscape', 'clauses-back':settings.clausePosition === 'back' }" :aria-label="`Nota de ${kind === 'entry' ? 'entrada' : 'salida'}`">
    <div class="note-front">
      <header class="note-company">
        <div><h3>{{ settings.name || 'Nombre de la empresa' }}</h3>
          <p v-if="settings.printAddress && settings.address">{{ settings.address }}</p>
          <p v-if="settings.phone">Tel. {{ settings.phone }}</p>
          <p v-if="settings.whatsapp">WhatsApp {{ settings.whatsapp }}</p>
          <p v-if="settings.email">{{ settings.email }}</p>
          <p v-if="settings.website">{{ settings.website }}</p>
          <p v-if="settings.printAddress && settings.taxId">RFC: {{ settings.taxId }}</p>
          <p v-if="settings.hours">{{ settings.hours }}</p>
        </div>
        <img v-if="settings.logoPrint" :src="settings.logo || brandLogo" alt="Logo de la empresa" />
      </header>
      <div class="note-heading"><h4>Orden de {{ kind === 'entry' ? 'entrada' : 'salida' }}</h4><strong>{{ folio || order?.id || 'Sin orden' }}</strong></div>
      <p v-if="order" class="note-date">Fecha de recepción: {{ order.received }}</p>
      <dl v-if="order" class="note-fields">
        <div><dt>Cliente</dt><dd>{{ order.client || 'Sin registrar' }}</dd></div>
        <div><dt>Teléfono</dt><dd>{{ order.phone || 'Sin registrar' }}</dd></div>
        <div><dt>Artículo</dt><dd>{{ order.item || 'Sin registrar' }}</dd></div>
        <div><dt>Marca / modelo</dt><dd>{{ [order.brand, order.model].filter(Boolean).join(' ') || 'Sin registrar' }}</dd></div>
        <div><dt>Serie</dt><dd>{{ order.serial || 'Sin registrar' }}</dd></div>
      </dl>
      <p v-else class="note-empty">Selecciona una orden en Clientes para previsualizar sus datos.</p>
      <section v-if="order?.work" class="note-box"><h5>Trabajo registrado</h5><p>{{ order.work }}</p></section>
      <section v-if="order?.notes && kind === 'entry'" class="note-box"><h5>Notas de recepción</h5><p>{{ order.notes }}</p></section>
      <section v-if="kind==='exit' && repairSummary.publicNote" class="note-box"><h5>Nota del técnico para el cliente</h5><p>{{ repairSummary.publicNote }}</p></section>
      <section v-if="kind==='exit' && repairSummary.warranties.length" class="note-box"><h5>Garantías registradas</h5><p v-for="(warranty,index) in repairSummary.warranties" :key="index">{{ warranty.name }}: {{ warranty.days }} días a partir de la entrega.</p></section>
      <dl v-if="kind === 'exit'" class="note-totals">
        <template v-if="settings.showTax"><div><dt>Subtotal</dt><dd>{{ money(subtotal) }}</dd></div><div><dt>{{ settings.taxName }} {{ settings.taxRate }} %</dt><dd>{{ money(total - subtotal) }}</dd></div></template>
        <div><dt>Total registrado</dt><dd>{{ money(total) }}</dd></div>
      </dl>
      <p v-if="kind==='exit' && repairSummary.technician">Técnico: {{ repairSummary.technician }}</p>
      <p v-if="settings.webReference && order" class="note-reference">Referencia de la orden: {{ order.id }}</p>
      <section v-if="clauses.length && settings.clausePosition === 'foot'" class="note-clauses">
        <h5>Cláusulas</h5><ol :class="{unnumbered:!settings.numberClauses}"><li v-for="clause in clauses" :key="clause.id"><strong>{{ clause.title }}.</strong> {{ clause.text }}</li></ol>
      </section>
      <div v-if="settings.signature" class="note-signature">Firma del cliente</div>
      <p class="note-footer">{{ kind === 'entry' ? settings.entryFooter : settings.exitFooter }}</p>
    </div>
    <section v-if="clauses.length && settings.clausePosition === 'back'" class="note-reverse note-clauses">
      <h5>Cláusulas · reverso</h5><ol :class="{unnumbered:!settings.numberClauses}"><li v-for="clause in clauses" :key="clause.id"><strong>{{ clause.title }}.</strong> {{ clause.text }}</li></ol>
    </section>
  </article>
</template>
<style src="../../assets/styles/company-note.css"></style>
