<script setup lang="ts">
import { ref, watch } from "vue";
import type { Order } from "../../types/order";
import { useAccess } from '../../composables/useAccess';
const { can } = useAccess();

const props = defineProps<{ order: Order; editing: boolean; initialPayment?: boolean }>();

const showPassword = ref(false);
watch(() => props.order.id, () => { showPassword.value = false; });
</script>

<template>
  <div class="form">
    <div class="block">
      <h2 class="sec">Cliente</h2>
      <div class="g2">
        <div class="fld"><label for="orden">Orden</label><input id="orden" readonly :value="order.id" /></div>
        <div class="fld"><label for="fecha">Fecha de recepción</label><input id="fecha" :readonly="!editing" v-model="order.received" /></div>
      </div>
      <div class="g2">
        <div class="fld"><label for="cliente">Cliente</label><input id="cliente" :readonly="!editing" v-model="order.client" /></div>
        <div class="fld"><label for="tel">Teléfono</label><input id="tel" :readonly="!editing" v-model="order.phone" /></div>
      </div>
    </div>

    <div class="block">
      <h2 class="sec">Artículo</h2>
      <div class="g2">
        <div class="fld"><label for="art">Artículo</label><input id="art" :readonly="!editing" v-model="order.item" /></div>
        <div class="fld"><label for="marca">Marca</label><input id="marca" :readonly="!editing" v-model="order.brand" /></div>
        <div class="fld"><label for="modelo">Modelo</label><input id="modelo" :readonly="!editing" v-model="order.model" /></div>
        <div class="fld"><label for="serie">No. de serie</label><input id="serie" :readonly="!editing" v-model="order.serial" /></div>
      </div>
      <div class="ask">
        <b>¿Ha sido llevado a otro taller?</b>
        <div class="yn" role="group" aria-label="¿Ha sido llevado a otro taller?">
          <button :disabled="!editing" type="button" :class="{ on: order.otherShop }" :aria-pressed="order.otherShop" @click="order.otherShop = true">Sí</button>
          <button :disabled="!editing" type="button" :class="{ on: !order.otherShop }" :aria-pressed="!order.otherShop" @click="order.otherShop = false">No</button>
        </div>
      </div>
      <div v-if="order.otherShop" class="fld"><label for="otro-taller">Detalle de la visita a otro taller</label><input id="otro-taller" :readonly="!editing" v-model="order.otherShopReason" placeholder="Taller, trabajo realizado o diagnóstico previo" /></div>
      <div class="ask">
        <b>¿Ha sufrido caída o humedad?</b>
        <div class="yn" role="group" aria-label="¿Ha sufrido caída o humedad?">
          <button :disabled="!editing" type="button" :class="{ on: order.dropped }" :aria-pressed="order.dropped" @click="order.dropped = true">Sí</button>
          <button :disabled="!editing" type="button" :class="{ on: !order.dropped }" :aria-pressed="!order.dropped" @click="order.dropped = false">No</button>
        </div>
      </div>
      <div v-if="order.dropped" class="fld"><label for="motivo">Motivo de la caída o humedad</label><input id="motivo" :readonly="!editing" v-model="order.dropReason" /></div>
      <div class="fld">
        <label for="pass">Contraseña del equipo</label>
        <input id="pass" :type="showPassword ? 'text' : 'password'" :readonly="!editing" v-model="order.password" />
        <button class="eye" :class="{ revealed: showPassword }" :aria-pressed="showPassword" aria-controls="pass" type="button" :aria-label="showPassword ? 'Ocultar contraseña' : 'Mostrar contraseña'" @click="showPassword = !showPassword"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" /><circle v-if="showPassword" cx="12" cy="12" r="3" /><path v-else d="m3 3 18 18" /></svg></button>
      </div>
    </div>

    <div class="block">
      <h2 class="sec">Trabajo</h2>
      <div class="fld"><label for="trabajo">Trabajo a realizar</label><textarea id="trabajo" :readonly="!editing" rows="2" v-model="order.work"></textarea></div>
      <div class="g3">
        <div class="fld"><label for="costo">Costo aproximado</label><input id="costo" :readonly="!editing" v-model="order.cost" /></div>
        <div class="fld"><label for="abono">Abono inicial</label><input id="abono" :readonly="!editing || !initialPayment || !can('payments.add')" v-model="order.deposit" /></div>
      </div>
    </div>

    <div class="block">
      <h2 class="sec">Notas</h2>
      <div class="fld"><label class="sr" for="notas">Notas</label><textarea id="notas" :readonly="!editing" rows="3" v-model="order.notes"></textarea></div>
    </div>
  </div>
</template>
