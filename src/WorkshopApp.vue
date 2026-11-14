<script setup lang="ts">
import AppHeader from './components/layout/AppHeader.vue';
import AppNotice from './components/ui/AppNotice.vue';
import ReceptionView from './views/ReceptionView.vue';
import CompanyView from './views/CompanyView.vue';
import UsersView from './views/UsersView.vue';
import RepairsView from './views/RepairsView.vue';
import LogbookView from './views/LogbookView.vue';
import CompanyNote from './components/company/CompanyNote.vue';
import TemporaryAccessDialog from './components/access/TemporaryAccessDialog.vue';
import { useWorkshop } from './composables/useWorkshop';
const {receptionTool,auth,orders,workspace,companyStore,company,companyDirty,notice,selected,publicOrder,tabs,active,can,canRepair,canPrint,printKind,printing,navigate,logout,finishTemporary,refreshOrder,print,requestAccess,prepareLeave,loadCompany}=useWorkshop();
defineExpose({prepareLeave});
</script>
<template>
  <div class="app normalized-workshop">
    <AppHeader :tabs="tabs" :active="active" :user="auth.user" :can-repair="canRepair" :can-print="canPrint" temporary-access :client-tools="workspace.page==='clients'" :can-view-history="!!selected && can('logbook')" :busy="orders.isLoading || printing || auth.isLoading" @change="navigate" @print="print" @logout="logout" @request-access="requestAccess" @lookup="receptionTool('lookup')" @history="receptionTool('history')" @refresh="receptionTool('refresh')" />
    <aside v-if="auth.temporaryAccess" class="temporary-banner" role="status"><span>Autorización temporal activa en {{ active }}. Tu usuario sigue siendo quien realiza las operaciones.</span><button class="btn btn-s" @click="finishTemporary">Finalizar acceso temporal</button></aside>
    <AppNotice :message="notice || auth.error?.message || ''" @dismiss="notice='';auth.error=null" />
    <AppNotice v-if="companyStore.error" :message="companyStore.error.message" tone="error"><button class="btn btn-s" :disabled="companyStore.isLoading" @click="loadCompany">Volver a consultar Empresa</button></AppNotice>
    <ReceptionView v-if="workspace.page==='clients'" ref="reception" embedded @repairs="navigate('Reparaciones')" />
    <CompanyView v-else-if="workspace.page==='company' && can('company') && companyStore.loaded" :orders="selected?[selected]:[]" :order="publicOrder" @dirty-change="companyDirty=$event" />
    <UsersView v-else-if="workspace.page==='users' && can('users')" />
    <LogbookView v-else-if="workspace.page==='logbook' && can('logbook')" />
    <RepairsView v-else-if="workspace.page==='repairs' && selected && can('repairs')" :key="selected.databaseId" ref="repair" :order="selected" @saved="refreshOrder" @back="navigate('Clientes')" />
    <section v-else class="sheet empty-work"><h1>{{ companyStore.isLoading ? 'Consultando configuración…' : 'Sección no disponible' }}</h1><p>Consulta los permisos de tu cuenta o solicita acceso temporal.</p></section>
    <div v-if="publicOrder" ref="printSource" class="sr" aria-hidden="true"><CompanyNote ref="printNote" :settings="company" :kind="printKind" :order="publicOrder" /></div>
    <TemporaryAccessDialog :request="workspace.request" :choose-operation="workspace.chooseOperation" @close="workspace.finishRequest()" @approved="workspace.finishRequest(true)" />
  </div>
</template>
<style src="./assets/styles/orders.css"></style>
<style src="./assets/styles/workspace.css"></style>
<style src="./assets/styles/readability.css"></style>
<style src="./assets/styles/app-header.css"></style>
<style src="./assets/styles/normalized-workshop.css"></style>
<style scoped>
.temporary-banner{margin:16px 24px;padding:16px;display:flex;align-items:center;justify-content:space-between;gap:16px;border:1px solid var(--line2);border-radius:10px;background:white}
</style>
