<script setup lang="ts">
<script setup lang="ts">
import { computed, ref } from "vue";
import brandLogo from "../Nueva carpeta/assets/9c703df255340c1c7277e580f6bc6146.png";

type Activity = { order: string; date: string; item: string; status: string; tone: string };
type Client = {
  id: string; name: string; phone: string; email: string; taxId: string; kind: string;
  since: string; address: string; city: string; notes: string; orders: number;
  spent: string; frequent: boolean; activity: Activity[];
};

const clients = ref<Client[]>([
  { id: "CL-001", name: "García Hernández", phone: "998 555 0101", email: "m.garcia@email.com", taxId: "GAHM9004128K2", kind: "Particular", since: "20 sep 2024", address: "Calle 24, Mza. 18, Lt. 6", city: "Cancún, Quintana Roo, 77500", notes: "Prefiere contacto por WhatsApp. Recoge los equipos por la tarde.", orders: 8, spent: "$12,450", frequent: true, activity: [
    { order: "ORD-024", date: "20 sep 2026", item: "Laptop HP Pavilion 14", status: "En reparación", tone: "blue" },
    { order: "ORD-018", date: "04 jun 2026", item: "Cambio de teclado", status: "Entregado", tone: "teal" },
    { order: "ORD-011", date: "12 ene 2026", item: "Mantenimiento preventivo", status: "Entregado", tone: "slate" },
  ] },
  { id: "CL-002", name: "Lucía Martínez", phone: "998 555 0182", email: "lucia.mtz@email.com", taxId: "MALU850923QJ4", kind: "Particular", since: "11 feb 2025", address: "Av. Nader 48, Supermanzana 2", city: "Cancún, Quintana Roo, 77500", notes: "Contactar antes de realizar cualquier reparación adicional.", orders: 3, spent: "$4,800", frequent: false, activity: [
    { order: "ORD-023", date: "18 sep 2026", item: "Laptop Lenovo IdeaPad", status: "Esperando autorización", tone: "amber" },
    { order: "ORD-009", date: "02 may 2025", item: "Limpieza interna", status: "Entregado", tone: "teal" },
  ] },
  { id: "CL-003", name: "Roberto López Torres", phone: "998 555 0234", email: "roberto.lt@email.com", taxId: "LOTR7803065P1", kind: "Particular", since: "06 mar 2025", address: "Calle 31, Región 92", city: "Cancún, Quintana Roo, 77516", notes: "Sin notas adicionales.", orders: 5, spent: "$8,200", frequent: true, activity: [
    { order: "ORD-021", date: "12 sep 2026", item: "Laptop Dell Inspiron", status: "Listo para entrega", tone: "teal" },
    { order: "ORD-014", date: "29 nov 2025", item: "Reemplazo de batería", status: "Entregado", tone: "slate" },
  ] },
  { id: "CL-004", name: "Fernanda Ramírez", phone: "998 555 0346", email: "fer.ramirez@email.com", taxId: "RARF9207142B7", kind: "Particular", since: "14 jul 2025", address: "Av. Las Torres 215", city: "Cancún, Quintana Roo, 77535", notes: "Entregar diagnóstico por correo electrónico.", orders: 2, spent: "$3,100", frequent: false, activity: [
    { order: "ORD-020", date: "08 sep 2026", item: "Laptop Acer Aspire 5", status: "En diagnóstico", tone: "blue" },
  ] },
  { id: "CL-005", name: "Héctor Hernández Ruiz", phone: "998 555 0468", email: "hector.hr@email.com", taxId: "HERH810221MN3", kind: "Empresa", since: "21 ago 2025", address: "Av. Tulum 320, Local 4", city: "Cancún, Quintana Roo, 77500", notes: "Facturación mensual. Solicitar constancia fiscal actualizada.", orders: 12, spent: "$26,750", frequent: true, activity: [
    { order: "ORD-019", date: "02 sep 2026", item: "Mantenimiento de equipo", status: "Esperando refacción", tone: "amber" },
    { order: "ORD-015", date: "17 feb 2026", item: "Diagnóstico de laptop", status: "Entregado", tone: "teal" },
  ] },
  { id: "CL-006", name: "Valeria Castro", phone: "998 555 0579", email: "val.cast@email.com", taxId: "CAVV9605188R6", kind: "Particular", since: "09 ene 2026", address: "Calle 10, Supermanzana 15", city: "Cancún, Quintana Roo, 77505", notes: "Sin notas adicionales.", orders: 1, spent: "$950", frequent: false, activity: [
    { order: "ORD-016", date: "18 ago 2026", item: "Respaldo de información", status: "Entregado", tone: "slate" },
  ] },
]);

const selectedId = ref("CL-001");
const search = ref("");
const frequentOnly = ref(false);
const showDialog = ref(false);
const draft = ref({ name: "", phone: "", email: "" });
const selected = computed(() => clients.value.find((client) => client.id === selectedId.value) ?? clients.value[0]);
const filteredClients = computed(() => {
  const query = search.value.trim().toLocaleLowerCase("es");
  return clients.value.filter((client) =>
    (!frequentOnly.value || client.frequent) && `${client.name} ${client.phone} ${client.id}`.toLocaleLowerCase("es").includes(query),
  );
});
const selectedIndex = computed(() => clients.value.findIndex((client) => client.id === selectedId.value));

function initials(name: string) {
  return name.split(/\s+/).slice(0, 2).map((part) => part[0]).join("").toLocaleUpperCase("es");
}

function moveSelection(step: number) {
  selectedId.value = clients.value[(selectedIndex.value + step + clients.value.length) % clients.value.length].id;
}

function addClient() {
  const name = draft.value.name.trim();
  if (!name) return;
  const clientNumber = clients.value.length + 1;
  const client: Client = {
    id: `CL-${String(clientNumber).padStart(3, "0")}`, name,
    phone: draft.value.phone.trim() || "Sin teléfono", email: draft.value.email.trim() || "Sin correo",
    taxId: "Pendiente", kind: "Particular", since: "Hoy", address: "Pendiente de captura", city: "",
    notes: "Sin notas adicionales.", orders: 0, spent: "$0", frequent: false, activity: [],
  };
  clients.value.unshift(client);
  selectedId.value = client.id;
  showDialog.value = false;
  draft.value = { name: "", phone: "", email: "" };
}
</script>

<template>
  <div class="app-shell">
    <header class="topbar">
      <div class="brand-tile"><img :src="brandLogo" alt="HRL, Héctor Refacciones Laptop" /></div>
      <span class="top-divider"></span>
      <div class="top-label"><small>ADMINISTRACIÓN</small><strong>Directorio de clientes</strong></div>
      <span class="top-spacer"></span>
      <span class="shop-status"><i></i>Taller principal</span>
    </header>

    <main class="workspace">
      <aside class="directory" aria-label="Directorio de clientes">
        <div class="directory-heading">
          <div><p class="eyebrow">DIRECTORIO</p><h1>Clientes</h1><p class="subtle">{{ clients.length }} registros</p></div>
          <button class="button primary new-button" type="button" @click="showDialog = true"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>Nuevo</button>
        </div>
        <label class="search-box"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7" /><path d="m20 20-4-4" /></svg><span class="sr-only">Buscar clientes</span><input v-model="search" type="search" placeholder="Nombre, teléfono o clave" /><kbd>/</kbd></label>
        <div class="filter-tabs" role="group" aria-label="Filtrar clientes">
          <button type="button" :class="{ active: !frequentOnly }" @click="frequentOnly = false">Todos</button>
          <button type="button" :class="{ active: frequentOnly }" @click="frequentOnly = true">Frecuentes</button>
        </div>
        <div class="list-caption"><span>CLIENTE</span><span>{{ filteredClients.length }} / {{ clients.length }}</span></div>
        <ul class="client-list">
          <li v-for="client in filteredClients" :key="client.id">
            <button class="client-row" :class="{ selected: selectedId === client.id }" type="button" @click="selectedId = client.id">
              <span class="avatar" :class="`avatar-${client.id.slice(-1)}`">{{ initials(client.name) }}</span>
              <span class="client-copy"><span class="client-line"><strong>{{ client.name }}</strong><small>{{ client.id }}</small></span><span class="client-line secondary-line"><span>{{ client.phone }}</span><i v-if="client.frequent" title="Cliente frecuente">★</i></span></span>
            </button>
          </li>
          <li v-if="filteredClients.length === 0" class="empty-state">No hay clientes que coincidan con la búsqueda.</li>
        </ul>
        <p class="legend"><i></i> Cliente frecuente</p>
      </aside>

      <section v-if="selected" class="profile" aria-label="Ficha del cliente">
        <header class="profile-header">
          <div class="identity"><span class="profile-avatar">{{ initials(selected.name) }}</span><div><p class="eyebrow">{{ selected.id }} <b>/</b> PERFIL DEL CLIENTE</p><h2>{{ selected.name }}</h2><p class="subtle">Cliente desde {{ selected.since }}</p></div></div>
          <div class="profile-actions">
            <div class="record-nav" role="group" aria-label="Navegar entre clientes"><button type="button" aria-label="Cliente anterior" title="Anterior" @click="moveSelection(-1)"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg></button><span>{{ selectedIndex + 1 }} de {{ clients.length }}</span><button type="button" aria-label="Cliente siguiente" title="Siguiente" @click="moveSelection(1)"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></button></div>
            <a class="button secondary" :href="`tel:${selected.phone.replaceAll(' ', '')}"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M22 16.9v3a2 2 0 0 1-2.2 2 19.8 19.8 0 0 1-8.6-3.1 19.4 19.4 0 0 1-6-6A19.8 19.8 0 0 1 2.1 4.2 2 2 0 0 1 4.1 2h3a2 2 0 0 1 2 1.7l.4 2.8a2 2 0 0 1-.6 1.7L7.5 9.6a16 16 0 0 0 6 6l1.4-1.4a2 2 0 0 1 1.7-.6l2.8.4a2 2 0 0 1 1.6 1.9Z" /></svg>Llamar</a>
          </div>
        </header>
        <div class="perforation"></div>

        <div class="profile-body">
          <div class="profile-main">
            <section class="detail-section"><h3 class="section-title">Datos de contacto</h3><div class="field-grid">
              <div class="data-field"><span>Teléfono</span><a :href="`tel:${selected.phone.replaceAll(' ', '')}">{{ selected.phone }}</a></div>
              <div class="data-field"><span>Correo electrónico</span><a :href="`mailto:${selected.email}`">{{ selected.email }}</a></div>
              <div class="data-field"><span>Tipo de cliente</span><strong>{{ selected.kind }}</strong></div>
              <div class="data-field"><span>RFC</span><strong>{{ selected.taxId }}</strong></div>
            </div></section>
            <section class="detail-section"><h3 class="section-title">Domicilio</h3><div class="field-grid address-grid">
              <div class="data-field"><span>Dirección</span><strong>{{ selected.address }}</strong></div>
              <div class="data-field"><span>Ciudad y código postal</span><strong>{{ selected.city || 'Pendiente de captura' }}</strong></div>
            </div></section>
            <section class="detail-section"><div class="section-heading"><h3 class="section-title">Actividad en el taller</h3><small>HISTORIAL RECIENTE</small></div>
              <div class="stats"><div><span>Órdenes</span><strong>{{ selected.orders }}</strong></div><div><span>Total registrado</span><strong>{{ selected.spent }}</strong></div><div><span>Frecuencia</span><strong>{{ selected.frequent ? 'Recurrente' : 'Ocasional' }}</strong></div></div>
              <div v-if="selected.activity.length" class="activity-list"><div v-for="item in selected.activity" :key="item.order" class="activity-row"><span class="activity-icon"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M8 2v4m8-4v4M3 10h18" /></svg></span><span class="activity-copy"><strong>{{ item.item }}</strong><small>{{ item.order }} <i></i> {{ item.date }}</small></span><span class="status" :class="`tone-${item.tone}`"><i></i>{{ item.status }}</span></div></div>
              <p v-else class="no-activity">Este cliente aún no tiene órdenes registradas.</p>
            </section>
            <section class="detail-section"><h3 class="section-title">Notas internas</h3><p class="notes">{{ selected.notes }}</p></section>
          </div>
          <aside class="profile-aside"><section class="detail-section"><h3 class="section-title">Resumen</h3><div class="active-status"><i></i>Registro activo<svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg></div><p class="aside-copy">La ficha está disponible para nuevas órdenes de servicio.</p></section>
            <section class="detail-section"><h3 class="section-title">Datos rápidos</h3><div class="quick-row"><span>Clave</span><strong>{{ selected.id }}</strong></div><div class="quick-row"><span>Registro</span><strong>{{ selected.since }}</strong></div><div class="quick-row"><span>Clasificación</span><strong>{{ selected.frequent ? 'Frecuente' : 'General' }}</strong></div></section>
            <div class="aside-note"><i>i</i><p>Los datos de contacto ayudan a mantener actualizado el seguimiento de cada equipo.</p></div>
          </aside>
        </div>
      </section>
    </main>

    <div v-if="showDialog" class="modal-backdrop" @click.self="showDialog = false"><section class="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
      <header class="dialog-header"><div><p class="eyebrow">DIRECTORIO</p><h2 id="dialog-title">Nuevo cliente</h2></div><button class="close-button" type="button" aria-label="Cerrar" @click="showDialog = false"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m18 6-12 12M6 6l12 12" /></svg></button></header>
      <form class="client-form" @submit.prevent="addClient"><label>Nombre completo<input v-model="draft.name" required autofocus placeholder="Nombre y apellidos" /></label><label>Teléfono<input v-model="draft.phone" type="tel" placeholder="998 555 0000" /></label><label>Correo electrónico<input v-model="draft.email" type="email" placeholder="nombre@correo.com" /></label><div class="dialog-actions"><button class="button secondary" type="button" @click="showDialog = false">Cancelar</button><button class="button primary" type="submit">Guardar cliente</button></div></form>
    </section></div>
  </div>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,500;600;700;800&family=Instrument+Sans:wght@400;500;600;700&display=swap");
:root{font-family:"Instrument Sans",system-ui,sans-serif;color:#12222d;background:#e7ecf0;font-synthesis:none;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased;-moz-osx-font-smoothing:grayscale}*{box-sizing:border-box}body{margin:0;min-width:320px;min-height:100vh}button,input{font:inherit}button,a{-webkit-tap-highlight-color:transparent}button{cursor:pointer}a{color:inherit;text-decoration:none}button:focus-visible,a:focus-visible,input:focus-visible{outline:2px solid #1b72ad;outline-offset:3px}#app{min-height:100vh}
.app-shell{--ink:#12222d;--muted:#60717d;--line:#d5dde4;--line-dark:#b3bfc9;--tint:#f3f6f8;--petrol:#17384a;--accent:#1b72ad;--accent-dark:#155c8c;min-height:100vh;background:#e7ecf0}.topbar{min-height:74px;display:flex;align-items:center;gap:18px;padding:0 28px;background:var(--petrol);color:#fff}.brand-tile{width:142px;height:52px;display:grid;flex:none;place-items:center;padding:5px 10px;border-radius:9px;background:#fff;box-shadow:0 2px 10px #05121c4d}.brand-tile img{width:100%;height:100%;object-fit:contain}.top-divider{width:1px;height:34px;background:#ffffff38}.top-label{display:flex;flex-direction:column;gap:4px}.top-label small{color:#9fb5c4;font-size:9px;font-weight:700;letter-spacing:.08em}.top-label strong{font-size:14px;font-weight:600}.top-spacer{flex:1}.shop-status{display:flex;align-items:center;gap:9px;color:#d5e1e9;font-size:12px}.shop-status i,.active-status>i,.legend i{width:8px;height:8px;border-radius:50%;background:#65c5a6}.shop-status i{box-shadow:0 0 0 4px #65c5a61f}.workspace{width:min(1680px,100%);min-height:calc(100vh - 74px);display:flex;align-items:flex-start;gap:23px;margin:auto;padding:20px 27px 28px}.directory{flex:0 0 325px;min-width:0;padding:10px 0}.directory-heading{display:flex;align-items:flex-end;justify-content:space-between;gap:10px;margin-bottom:20px}.eyebrow{margin:0 0 6px;color:#687984;font-size:9px;font-weight:700;letter-spacing:.08em}.directory-heading h1{margin:0;font:700 28px/1 "Bricolage Grotesque","Instrument Sans",sans-serif}.subtle{margin:6px 0 0;color:var(--muted);font-size:12px}.button{min-height:40px;display:inline-flex;align-items:center;justify-content:center;gap:8px;padding:0 13px;border:1px solid transparent;border-radius:8px;font-size:12px;font-weight:600;white-space:nowrap}.primary{color:#fff;background:var(--accent)}.primary:hover{background:var(--accent-dark)}.secondary{border-color:var(--line-dark);color:var(--ink);background:#fff}.secondary:hover{background:var(--tint)}.new-button{min-height:39px;padding:0 11px}.icon{width:17px;height:17px;flex:none;fill:none;stroke:currentColor;stroke-width:1.8;stroke-linecap:round;stroke-linejoin:round}.search-box{height:44px;display:flex;align-items:center;gap:9px;padding:0 10px 0 12px;border:1px solid var(--line-dark);border-radius:8px;background:#fff;color:#667782}.search-box:focus-within{border-color:var(--accent);box-shadow:0 0 0 3px #1b72ad1c}.search-box input{width:100%;min-width:0;border:0;outline:0;background:transparent;color:var(--ink);font-size:12px}.search-box input::placeholder{color:#778791}.search-box kbd{min-width:20px;height:20px;display:grid;place-items:center;border:1px solid var(--line);border-radius:4px;color:#778791;font-size:10px}.sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap}.filter-tabs{display:flex;gap:3px;margin:12px 0 17px;padding:3px;border-radius:8px;background:#d9e1e7}.filter-tabs button{flex:1;min-height:34px;border:0;border-radius:6px;background:transparent;color:#53636e;font-size:11px;font-weight:600}.filter-tabs button.active{background:#fff;color:var(--ink);box-shadow:0 1px 2px #12222d26}.list-caption{display:flex;justify-content:space-between;padding:0 11px 8px;color:#74848e;font-size:9px;font-weight:700;letter-spacing:.07em}.client-list{display:flex;flex-direction:column;gap:3px;margin:0;padding:0;list-style:none}.client-row{width:100%;min-height:64px;display:flex;align-items:center;gap:10px;padding:8px 10px;border:0;border-radius:9px;background:transparent;color:var(--ink);text-align:left}.client-row:hover{background:#ffffffa6}.client-row.selected{background:#fff;box-shadow:0 1px 2px #12222d0d,0 5px 13px -6px #12222d40}.avatar,.profile-avatar{display:grid;flex:none;place-items:center;border-radius:50%;background:#dcebf7;color:#17384a;font-weight:700}.avatar{width:39px;height:39px;font-size:11px}.avatar-2{background:#e8e4d8;color:#71613b}.avatar-3{background:#dcece7;color:#2c6d60}.avatar-4{background:#e9e0ee;color:#6d4f7c}.avatar-5{background:#f2e1d8;color:#885b45}.avatar-6{background:#e2e7ef;color:#4d607a}.client-copy{flex:1;min-width:0;display:flex;flex-direction:column;gap:5px}.client-line{min-width:0;display:flex;align-items:center;justify-content:space-between;gap:7px}.client-line strong{overflow:hidden;color:#1c2c37;font-size:12px;text-overflow:ellipsis;white-space:nowrap}.client-line small{flex:none;color:#70808a;font-size:9px}.secondary-line{color:#61717c;font-size:10px}.secondary-line>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.secondary-line i{color:#c58b24;font-style:normal}.empty-state{padding:17px 10px;color:var(--muted);font-size:12px;line-height:1.5}.legend{display:flex;align-items:center;gap:7px;margin:14px 9px 0;color:var(--muted);font-size:10px}.legend i{width:7px;height:7px;background:#c58b24}
.profile{flex:1 1 680px;min-width:0;overflow:hidden;border-radius:17px;background:#fff;box-shadow:0 1px 0 #12222d0d,0 16px 36px -16px #12222d3d;animation:profile-in .3s ease both}@keyframes profile-in{from{opacity:0;transform:translateY(5px)}to{opacity:1;transform:translateY(0)}}.profile-header{min-height:125px;display:flex;align-items:center;justify-content:space-between;gap:16px;padding:22px 29px}.identity{min-width:0;display:flex;align-items:center;gap:14px}.profile-avatar{width:56px;height:56px;background:#dcebf7;color:#155c8c;font:700 18px "Bricolage Grotesque",sans-serif}.identity>div{min-width:0}.identity .eyebrow{margin-bottom:5px}.identity .eyebrow b{padding:0 4px;color:#a2afb7}.identity h2{overflow-wrap:anywhere;margin:0;font:700 23px/1.12 "Bricolage Grotesque","Instrument Sans",sans-serif}.profile-actions{display:flex;align-items:center;gap:8px}.record-nav{height:39px;display:inline-flex;align-items:center;overflow:hidden;border:1px solid var(--line-dark);border-radius:8px;background:#fff}.record-nav button{width:34px;height:37px;display:grid;place-items:center;border:0;background:transparent;color:var(--ink)}.record-nav button:hover{background:var(--tint)}.record-nav span{padding:0 3px;color:var(--muted);font-size:10px;font-weight:600;white-space:nowrap}.perforation{position:relative;height:0;margin:0 17px;border-top:2px dashed #c3ced6}.perforation:before,.perforation:after{position:absolute;top:-12px;width:22px;height:22px;border-radius:50%;background:#e7ecf0;content:""}.perforation:before{left:-28px}.perforation:after{right:-28px}.profile-body{display:flex;align-items:stretch}.profile-main{flex:1 1 500px;min-width:0;display:flex;flex-direction:column;gap:24px;padding:27px 29px 32px}.detail-section{display:flex;flex-direction:column;gap:12px}.section-title{display:flex;align-items:center;gap:10px;margin:0;font:600 16px/1.2 "Bricolage Grotesque","Instrument Sans",sans-serif}.section-title:after{flex:1;height:1px;background:var(--line);content:""}.field-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px 23px}.data-field{min-width:0;display:flex;flex-direction:column;gap:5px;padding-bottom:8px;border-bottom:1px dotted var(--line-dark)}.data-field>span{color:#6d7d87;font-size:10px;font-weight:600}.data-field>strong,.data-field>a{overflow-wrap:anywhere;color:#243642;font-size:12px;font-weight:600}.data-field>a:hover{color:var(--accent-dark);text-decoration:underline;text-underline-offset:3px}.address-grid{grid-template-columns:1.1fr 1fr}.section-heading{display:flex;align-items:center;justify-content:space-between;gap:10px}.section-heading>small{flex:none;color:#778791;font-size:8px;font-weight:700;letter-spacing:.07em}.stats{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:7px}.stats>div{min-width:0;display:flex;flex-direction:column;gap:5px;padding:10px;border-radius:7px;background:var(--tint)}.stats span{color:#677984;font-size:9px}.stats strong{overflow:hidden;color:#20323e;font:700 15px/1.2 "Bricolage Grotesque",sans-serif;text-overflow:ellipsis;white-space:nowrap}.activity-list{display:flex;flex-direction:column}.activity-row{min-height:54px;display:flex;align-items:center;gap:9px;border-bottom:1px solid #e9eef1}.activity-row:last-child{border:0}.activity-icon{width:30px;height:30px;display:grid;flex:none;place-items:center;border:1px solid var(--line);border-radius:7px;color:#526671}.activity-icon .icon{width:14px;height:14px}.activity-copy{flex:1;min-width:0;display:flex;flex-direction:column;gap:4px}.activity-copy strong{overflow:hidden;color:#253641;font-size:11px;text-overflow:ellipsis;white-space:nowrap}.activity-copy small{display:flex;align-items:center;gap:6px;color:#72818a;font-size:9px}.activity-copy small i{width:3px;height:3px;border-radius:50%;background:#9aa7af}.status{flex:none;display:inline-flex;align-items:center;gap:5px;color:#53636e;font-size:9px}.status i{width:6px;height:6px;border-radius:50%;background:#7b8a96}.tone-blue i{background:#1b72ad}.tone-teal i{background:#1b9c8e}.tone-amber i{background:#d99a1e}.no-activity{margin:0;padding:11px;border-radius:7px;background:var(--tint);color:var(--muted);font-size:11px}.notes{margin:0;padding:11px 13px;border:1px dotted var(--line-dark);border-radius:7px;background:#fafbfc;color:#40515c;font-size:11px;line-height:1.55}.profile-aside{flex:0 0 235px;display:flex;flex-direction:column;gap:23px;padding:27px 19px;border-left:1px solid var(--line);background:var(--tint)}.profile-aside .section-title{font-size:15px}.active-status{min-height:39px;display:flex;align-items:center;gap:8px;padding:0 10px;border:1px solid var(--line);border-radius:7px;background:#fff;color:#354650;font-size:11px;font-weight:600}.active-status>i{width:7px;height:7px}.active-status .icon{width:15px;height:15px;margin-left:auto;color:#378574}.aside-copy{margin:0;color:var(--muted);font-size:10px;line-height:1.5}.quick-row{min-height:33px;display:flex;align-items:center;justify-content:space-between;gap:8px;border-bottom:1px solid var(--line);font-size:10px}.quick-row span{color:#657680}.quick-row strong{color:#283b47;font-size:10px;font-weight:600;text-align:right}.aside-note{display:flex;align-items:flex-start;gap:8px;margin-top:auto;padding-top:12px;border-top:1px solid var(--line)}.aside-note>i{width:16px;height:16px;display:grid;flex:none;place-items:center;border:1px solid #9cabb4;border-radius:50%;color:#5e707b;font:600 10px Georgia,serif}.aside-note p{margin:0;color:#667781;font-size:9px;line-height:1.5}
.modal-backdrop{position:fixed;inset:0;z-index:5;display:grid;place-items:center;padding:18px;background:#0d1e28a8;animation:fade-in .16s ease both}@keyframes fade-in{from{opacity:0}to{opacity:1}}.dialog{width:min(100%,420px);padding:23px;border-radius:12px;background:#fff;box-shadow:0 24px 70px #07131d52}.dialog-header{display:flex;justify-content:space-between;gap:14px;margin-bottom:20px}.dialog-header .eyebrow{margin-bottom:5px}.dialog-header h2{margin:0;font:700 23px/1.1 "Bricolage Grotesque",sans-serif}.close-button{width:34px;height:34px;display:grid;place-items:center;border:0;border-radius:7px;background:transparent;color:#52636e}.close-button:hover{background:var(--tint)}.client-form{display:flex;flex-direction:column;gap:14px}.client-form label{display:flex;flex-direction:column;gap:6px;color:#52636e;font-size:10px;font-weight:600}.client-form input{height:40px;padding:0 10px;border:1px solid var(--line-dark);border-radius:6px;background:#fff;color:var(--ink);font-size:12px}.client-form input::placeholder{color:#97a3aa}.dialog-actions{display:flex;justify-content:flex-end;gap:8px;margin-top:6px}
@media(max-width:1100px){.workspace{gap:16px;padding-right:18px;padding-left:18px}.directory{flex-basis:285px}.profile-header{align-items:flex-start;flex-direction:column;padding:22px}.profile-actions{align-self:flex-end}.profile-main{padding:24px 22px 28px}.profile-aside{flex-basis:205px;padding:24px 15px}}
@media(max-width:820px){.topbar{min-height:66px;padding:0 16px}.brand-tile{width:125px;height:46px}.workspace{flex-direction:column;padding:15px}.directory{flex:none;width:100%;padding:4px 0}.client-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr))}.profile{flex:none;width:100%}.profile-body{flex-direction:column}.profile-aside{flex:none;border-top:1px solid var(--line);border-left:0}.aside-note{margin-top:0}}
@media(max-width:560px){.topbar{gap:10px;padding:0 11px}.brand-tile{width:102px;height:39px;padding:4px 6px}.top-divider{height:27px}.top-label small{font-size:8px}.top-label strong{max-width:120px;font-size:10px}.shop-status{gap:6px;font-size:9px}.workspace{gap:12px;padding:11px}.directory-heading h1{font-size:25px}.client-list{grid-template-columns:1fr}.client-row{min-height:58px}.profile-header{gap:15px;padding:18px 15px}.identity{align-items:flex-start;gap:10px}.profile-avatar{width:44px;height:44px;font-size:14px}.identity h2{font-size:19px}.identity .eyebrow{font-size:8px}.profile-actions{width:100%;justify-content:space-between}.profile-main{gap:21px;padding:21px 15px 24px}.field-grid,.address-grid{grid-template-columns:1fr;gap:11px}.stats{gap:5px}.stats>div{padding:8px 7px}.stats span{font-size:8px}.stats strong{font-size:13px}.activity-row{gap:6px}.activity-copy strong{font-size:10px}.status{max-width:95px;font-size:8px}.section-heading>small{font-size:7px}.dialog{padding:20px 17px}}
</style>