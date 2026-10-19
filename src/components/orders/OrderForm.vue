<script setup lang="ts">
import { ref } from "vue";
import type { Order } from "../../types/order";

defineProps<{ order: Order }>();

const showPassword = ref(false);
</script>

<template>
  <div class="form">
    <div class="block">
      <h2 class="sec">Cliente</h2>
      <div class="g2">
        <div class="fld"><label for="orden">Orden</label><input id="orden" readonly :value="order.id" /></div>
        <div class="fld"><label for="fecha">Fecha de recepción</label><input id="fecha" readonly :value="order.received" /></div>
      </div>
      <div class="fld"><label for="cliente">Cliente</label><input id="cliente" readonly :value="order.client" /></div>
      <div class="fld"><label for="tel">Teléfono</label><input id="tel" readonly :value="order.phone" /></div>
    </div>

    <div class="block">
      <h2 class="sec">Artículo</h2>
      <div class="g2">
        <div class="fld"><label for="art">Artículo</label><input id="art" readonly :value="order.item" /></div>
        <div class="fld"><label for="marca">Marca</label><input id="marca" readonly :value="order.brand" /></div>
        <div class="fld"><label for="modelo">Modelo</label><input id="modelo" readonly :value="order.model" /></div>
        <div class="fld"><label for="serie">No. de serie</label><input id="serie" readonly :value="order.serial" /></div>
      </div>
      <div class="ask">
        <b>¿Ha sido llevado a otro taller?</b>
        <div class="yn" role="group" aria-label="¿Ha sido llevado a otro taller?">
          <button type="button" :class="{ on: order.otherShop }" :aria-pressed="order.otherShop" @click="order.otherShop = true">Sí</button>
          <button type="button" :class="{ on: !order.otherShop }" :aria-pressed="!order.otherShop" @click="order.otherShop = false">No</button>
        </div>
      </div>
      <div class="ask">
        <b>¿Ha sufrido caída o humedad?</b>
        <div class="yn" role="group" aria-label="¿Ha sufrido caída o humedad?">
          <button type="button" :class="{ on: order.dropped }" :aria-pressed="order.dropped" @click="order.dropped = true">Sí</button>
          <button type="button" :class="{ on: !order.dropped }" :aria-pressed="!order.dropped" @click="order.dropped = false">No</button>
        </div>
      </div>
      <div v-if="order.dropped" class="fld"><label for="motivo">Motivo de la caída o humedad</label><input id="motivo" readonly :value="order.dropReason" /></div>
      <div class="fld">
        <label for="pass">Contraseña del equipo</label>
        <input id="pass" :type="showPassword ? 'text' : 'password'" readonly :value="order.password" />
        <button class="eye" type="button" :aria-label="showPassword ? 'Ocultar contraseña' : 'Mostrar contraseña'" @click="showPassword = !showPassword"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" /><circle cx="12" cy="12" r="3" /></svg></button>
      </div>
    </div>

    <div class="block">
      <h2 class="sec">Trabajo</h2>
      <div class="fld"><label for="trabajo">Trabajo a realizar</label><textarea id="trabajo" readonly rows="3" :value="order.work"></textarea></div>
      <div class="g3">
        <div class="fld"><label for="costo">Costo aproximado</label><input id="costo" readonly :value="order.cost" /></div>
        <div class="fld"><label for="abono">Abono</label><input id="abono" readonly :value="order.deposit" /></div>
      </div>
    </div>

    <div class="block">
      <h2 class="sec">Notas</h2>
      <div class="fld"><label class="sr" for="notas">Notas</label><textarea id="notas" readonly rows="3" :value="order.notes"></textarea></div>
    </div>
  </div>
</template>
