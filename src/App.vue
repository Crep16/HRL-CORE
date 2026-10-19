<script setup lang="ts">
import { ref } from "vue";
import AppHeader from "./components/layout/AppHeader.vue";
import OrderList from "./components/orders/OrderList.vue";
import OrderSheet from "./components/orders/OrderSheet.vue";
import { useOrders } from "./composables/useOrders";

const tabs = ["Clientes", "Bitácora", "Empresa"];
const activeTab = ref(tabs[0]);

const { orders, selectedId, query, field, ascending, visible, index, selected, goTo, removeSelected } = useOrders();

function confirmRemove() {
  if (selected.value && window.confirm(`¿Eliminar la orden ${selected.value.id}?`)) removeSelected();
}
</script>

<template>
  <div class="app">
    <AppHeader :tabs="tabs" :active="activeTab" @change="activeTab = $event" />
    <div v-if="activeTab === tabs[0]" class="work">
      <OrderList v-model:query="query" v-model:field="field" :orders="visible" :total="orders.length" :selected-id="selectedId" @select="selectedId = $event" @toggle-sort="ascending = !ascending" />
      <OrderSheet v-if="selected" :order="selected" :position="index + 1" :total="orders.length" @go-to="goTo" @remove="confirmRemove" />
    </div>
    <p v-else class="placeholder-page">La sección {{ activeTab }} aún no está disponible.</p>
  </div>
</template>

<style src="./assets/styles/orders.css"></style>
