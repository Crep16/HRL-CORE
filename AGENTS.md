# AGENTS.md — HRL CORE

Instrucciones obligatorias para Codex (y cualquier agente) en este repositorio.
Léelas completas antes de modificar código. Si algo contradice una regla de aquí, **detente y pregunta**.

## 1. Qué es el proyecto

Aplicación de escritorio **offline-first** para un taller de reparación de equipos de cómputo:
recepción de equipos, seguimiento técnico, libro de dinero, evidencias fotográficas,
bitácora de auditoría, configuración de empresa y control de usuarios/permisos.

Stack: **Tauri + Rust** (backend local), **Vue 3 + Tailwind** (frontend), **SQLite** (datos).
Especificación funcional vigente: `docs/*.md` (`reglas-pagos.md`, `evidencias.md`,
`acceso-temporal.md`, `reparaciones-permisos.md`, `papelera-ordenes.md`, `bitacora.md`, `sqlite.md`).
**Esos documentos son la fuente de verdad del comportamiento. No cambies reglas de negocio.**

## 2. Objetivo de esta refactorización (en este orden)

1. Reemplazar la base de datos por `docs/esquema-objetivo.sql` (3FN, sin JSON de negocio).
2. Organizar el backend en **capas** y el frontend en **MVVM**.
3. Aplicar **POO** donde el lenguaje lo permite (ver §4).
4. No perder funcionalidad ni reglas ya documentadas.

## 3. Arquitectura obligatoria

### Backend Rust (`src-tauri/src/`) — arquitectura en capas

```
commands/      Reciben la petición de la UI, validan forma de entrada y delegan. SIN SQL, SIN reglas.
services/      Reglas de negocio: permisos, saldos, reintentos, revisiones, auditoría. Orquesta transacciones.
repositories/  ÚNICO lugar con SQL. Un repositorio por agregado (ordenes, abonos, usuarios...).
domain/        Structs, enums y reglas puras (sin I/O, sin SQLite, sin Tauri).
db/            Conexión, PRAGMA foreign_keys=ON por conexión, migraciones.
error.rs       Tipo de error único del backend.
```

Reglas de dependencia (solo hacia abajo): `commands → services → repositories → db`; `domain` no depende de nadie.
Un `command` nunca llama a un `repository`. Un `repository` nunca llama a un `service`.

### Frontend Vue (`src/`) — MVVM

```
components/            VIEW: solo presentación y eventos. Sin lógica de negocio ni llamadas a Tauri.
views/ (nueva)         VIEW de pantalla completa. WorkshopApp.vue se DIVIDE aquí (ver Fase 4); debe quedar solo como contenedor/enrutador.
stores/ (nueva, Pinia) VIEWMODEL: estado de pantalla compartido, orquestación. Verifica en package.json si Pinia está instalada; si no, instálala.
composables/           VIEWMODEL local: validación de formularios, guardia de borradores, lógica reutilizable de pantalla.
services/api/ (nueva)  Único lugar que llama a `invoke()` de Tauri.
types/                 MODEL: tipos TypeScript (y clases de dominio si se usan) alineados con el backend.
utils/                 Solo funciones puras y sin estado (formato de fechas/dinero). Si contiene reglas de negocio, muévelas a services (Rust) o a models.
data/                  Solo catálogos estáticos de presentación. Si contiene datos de prueba o de negocio, repórtalo; no lo uses como fuente de verdad.
```

Una `View` nunca importa de `services/api` directamente: pasa por un store o composable.

## 4. POO

- Rust: modela con `struct` + `impl` + `trait`. Los servicios son structs que reciben sus repositorios
  (inyección por constructor). Usa traits para los repositorios cuando permita probar con mocks.
- TypeScript: el dominio puede usar `class` (p. ej. `Money`, `Order`) con métodos; encapsula, no expongas campos mutables.
- No fuerces herencia. Prefiere composición. Documenta en el PR qué principio POO aplica cada clase
  (encapsulamiento, abstracción, polimorfismo).

## 5. Reglas de la base de datos

- Esquema único: `docs/esquema-objetivo.sql`. No agregues JSON como almacén de datos de negocio.
- Dinero siempre en **centavos enteros**. Nunca `f64` para importes.
- Los libros (`abonos`, `ajustes_abono`, `devoluciones`, `gastos_directos`) y la `auditoria` son **inmutables**.
- La auditoría se escribe desde `services/` **dentro de la misma transacción** que el cambio. Si falla, se revierte todo.
- Los saldos se calculan con `vista_saldos_orden`; no los guardes.
- Los reintentos financieros usan `id_solicitud` UNIQUE + `huella_solicitud`; mismo id con otra huella = error.
- `PRAGMA foreign_keys=ON` en **cada** conexión, antes de abrir transacción.
- No usar `ON UPDATE CASCADE`. Los ids no se renombran.
- Los catálogos (`permisos`, `apartados`, `acciones`...) solo cambian mediante migración nueva.
- Migraciones numeradas en `src-tauri/migrations/`, nunca edites una ya aplicada.
- **Excepción única y autorizada (Fase 1):** `src-tauri/migrations/` ya contiene migraciones del esquema anterior (hasta la 013).
  Como no hay datos de producción, se reinicia la línea base: mueve las existentes a `src-tauri/migrations/legacy/`
  (el cargador de migraciones NO debe ejecutarlas) y crea `001_esquema_objetivo.sql` como nueva base.
  Las bases locales de desarrollo se recrean desde cero. Cualquier cambio posterior sí sigue la regla normal (migración nueva).

## 6. Decisiones pendientes (NO resolver por tu cuenta)

- **Contraseña del equipo** (`ordenes.contrasena_equipo`): cifrar, hashear o retirar. Hasta decidir, no la registres
  en auditoría ni en documentos impresos (ya está documentado así).
- **Estados de orden y etapas**: el catálogo sembrado es provisional. Sustitúyelo por los valores reales de `orders.rs`
  y reporta la diferencia.
- **Usuarios y nombres históricos**: el esquema ya no copia `nombre_autor`. Si se quiere conservar el nombre mostrado en el
  momento del evento, hay que decidirlo explícitamente.
- **Migración de datos existentes**: no hay datos de producción confirmados. No escribas migración de datos sin confirmarlo.

## 7. Convenciones

- Idioma: nombres de tablas y columnas en español `snake_case`, plural. Códigos internos de contrato (`complete`, `payments.add`) se conservan.
- Código Rust/TS: identificadores en inglés o español, **pero consistente por capa**; decide y documenta en `docs/arquitectura-objetivo.md`.
- Comentarios y mensajes al usuario en español.
- Un commit por paso lógico, mensaje descriptivo.

## 8. Forma de trabajar

- **Una fase a la vez** (ver `docs/prompts-fases.md`). No adelantes fases.
- Antes de editar: lista los archivos que vas a tocar y por qué.
- No toques la UI en las fases 1–3.
- Cada fase termina con: pruebas pasando, `cargo check` / `cargo test` / `npm run build` sin errores y un resumen breve.
- Si una regla de `docs/*.md` no está clara, pregunta; no inventes.
- Prohibido: borrar pruebas para que pasen, deshabilitar triggers o FK, o dejar `TODO` sin registrar en el resumen.
