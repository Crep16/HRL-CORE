<script setup lang="ts">
import brandLogo from "../assets/images/brand-logo.png";
</script>

<template>
  <header class="topbar">
    <div class="brand-tile"><img :src="brandLogo" alt="HRL, Héctor Refacciones Laptop" /></div>
    <span class="top-divider"></span>
    <div class="top-label"><small>ADMINISTRACIÓN</small><strong>Directorio de clientes</strong></div>
    <span class="top-spacer"></span>
    <span class="shop-status"><i></i>Taller principal</span>
  </header>
</template>
