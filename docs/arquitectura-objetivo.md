# Arquitectura objetivo de HRL CORE

## 1. Conceptos (para exponerlos ante el profesor)

| Concepto | Qué es | Dónde se aplica aquí |
|---|---|---|
| **POO** | Paradigma: modelar con objetos que encapsulan datos y comportamiento | Structs + `impl` + `trait` en Rust; clases de dominio en TypeScript |
| **Arquitectura en capas** | Patrón: separar responsabilidades en niveles que solo dependen hacia abajo | Backend Rust: commands → services → repositories → db |
| **MVC** | Patrón: Modelo-Vista-Controlador | Equivalente del backend: `commands` = controlador, `services`+`domain` = modelo |
| **MVVM** | Patrón: Modelo-Vista-VistaModelo, con enlace de datos reactivo | Frontend Vue: componentes = Vista, Pinia/composables = VistaModelo, tipos/servicios API = Modelo |

POO es un **paradigma**; capas, MVC y MVVM son **patrones de arquitectura**. Se combinan: la arquitectura define las
capas y la POO define cómo se construyen las piezas dentro de cada capa.

## 2. Vista general

```
┌──────────────────────── Frontend (Vue 3) · MVVM ────────────────────────┐
│  View (.vue)  ⇄  ViewModel (Pinia store / composable)  →  Model (tipos) │
│                                   │                                      │
│                        services/api  (único invoke de Tauri)             │
└───────────────────────────────────┼──────────────────────────────────────┘
                                    │ comandos Tauri
┌───────────────────────────────────▼──────────────────────────────────────┐
│ Backend (Rust) · Capas                                                    │
│  commands  →  services  →  repositories  →  db (SQLite)                   │
│                  │                                                        │
│               domain (structs, enums, reglas puras)                       │
└──────────────────────────────────────────────────────────────────────────┘
```

## 3. Responsabilidad de cada capa

**commands** — Traducen la petición a una llamada de servicio. Validan la forma (tipos, campos obligatorios),
nunca reglas de negocio.

**services** — Corazón del sistema. Ejemplos:
- `PaymentService`: valida permisos, no exceder el pendiente, devolución completa del saldo a favor,
  idempotencia por `id_solicitud`/`huella_solicitud`, escribe auditoría.
- `OrderService`: folio sin colisión (`secuencias_folio`), papelera (`eliminada_en`), historial de cambios.
- `RepairService`: guardado con control de revisión (rechaza guardados desactualizados).
- `AccessService`: autenticación, clave maestra, accesos temporales, último administrador.
- `AuditService`: único punto que escribe `auditoria` y `auditoria_detalle`.

Cada servicio abre **una transacción**, ejecuta el cambio y la auditoría, y confirma o revierte todo junto.

**repositories** — SQL puro y mapeo a structs. Sin reglas de negocio. Uno por agregado.

**domain** — Tipos como `Money` (centavos, no `f64`), `OrderStatus`, `PermissionKey { apartado, accion }`,
`Balance { precio, cobrado, ajustado, devuelto }` con métodos `net()`, `pending()`, `credit()`.

## 4. Estructura de carpetas propuesta

```
src-tauri/src/
  commands/   orders.rs payments.rs repairs.rs evidence.rs company.rs users.rs access.rs audit.rs
  services/   order_service.rs payment_service.rs repair_service.rs ... audit_service.rs
  repositories/ order_repo.rs payment_repo.rs ...
  domain/     money.rs balance.rs permissions.rs ...
  db/         connection.rs migrations.rs
  error.rs  main.rs
src-tauri/migrations/
  legacy/               migraciones 001–013 del esquema anterior (archivadas, no se ejecutan)
  001_esquema_objetivo.sql   nueva línea base

src/                    (estado actual → destino)
  assets/               se queda
  components/           se queda, pero solo presentación
  composables/          se queda; aquí vive la lógica de pantalla reutilizable (useOrderForm, useDraftGuard...)
  types/                se queda; es el MODEL (Order, Money, Payment...)
  utils/                se queda, solo funciones puras
  data/                 se queda, solo catálogos estáticos (por confirmar su contenido)
  views/        (nuevo) ClientsView.vue RepairsView.vue LogbookView.vue CompanyView.vue UsersView.vue AccessView.vue
  stores/       (nuevo) useOrdersStore.ts usePaymentsStore.ts useSessionStore.ts ...
  services/api/ (nuevo) ordersApi.ts paymentsApi.ts ...  ← único lugar con invoke()
  WorkshopApp.vue       se divide en views/ y queda como contenedor/navegación
  App.vue, main.ts      se quedan (main.ts registra Pinia)
```

Nota: el proyecto ya tiene `components`, `composables` y `types`, así que el frontend no se rehace; se
**completa** con las capas que faltan (`stores`, `services/api`, `views`) y se adelgaza `WorkshopApp.vue`.

## 5. Del esquema anterior al nuevo (qué se mueve a dónde)

| Antes | Ahora |
|---|---|
| `Ordenes.datos_json` | Columnas de `ordenes` + `clientes` + `equipos` + `abonos` |
| `Seguimiento_reparaciones.datos_json` | `seguimientos`, `orden_etapas`, `avances`, `notas`, `refacciones`, `garantias`, `seguimiento_publicacion` |
| `Configuracion_empresa.datos_json` | `empresa`, `configuracion_documentos`, `clausulas`, `secuencias_folio` |
| `Historial_ordenes`, `Cambios_reparaciones`, `Historial_papelera`, `Bitacora_ordenes`, `Bitacora_sistema` | `auditoria` + `auditoria_detalle` |
| `Movimientos_abonos` (tipo adjustment/refund) | `ajustes_abono` y `devoluciones` |
| `Solicitudes_abonos` (JSON) | `id_solicitud` + `huella_solicitud` en cada tabla del libro |
| `Permisos_usuarios` con CHECK | `permisos` (catálogo) + `usuario_permiso` |
| `Accesos_temporales.operaciones_json` | `acceso_temporal_permiso` |
| `Importaciones_anteriores` | Eliminada (residuo de migración) |
| `Ordenes.id` = folio | `ordenes.id` interno + `ordenes.folio` UNIQUE |
| Papelera por eventos | `ordenes.eliminada_en` + eventos en `auditoria` |

## 6. Puntos a defender ante el profesor

1. **Normalización**: 1FN (sin listas/JSON en celdas), 2FN (sin dependencias parciales; claves compuestas solo en tablas puente),
   3FN (sin datos derivados: saldos en vistas, nombres vía JOIN, `id_orden` de ajustes vía `abonos`).
2. **Excepción justificada**: `auditoria.id_entidad` es una referencia polimórfica sin FK (identifica registros de varias tablas).
3. **Por qué SQL solo protege integridad**: FK, CHECK, UNIQUE e inmutabilidad en la base; reglas de negocio en `services`,
   donde se pueden probar y leer.
4. **Dinero en centavos enteros** para evitar errores de redondeo.
5. **Transacción única cambio + auditoría**: nada queda sin rastro.
