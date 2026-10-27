# Incremento de base local, acceso y recepción

27 de octubre de 2026. Continuación de `avance-2026-10-27.md`.

## Ramas y commits

Las ramas son acumulativas por sus dependencias: esquema, autenticación,
permisos y recepción. Cada tarea conserva su commit y su rama.

| Ticket | Rama | Evidencia |
| --- | --- | --- |
| HRL-11 | `feat/HRL-11-workshop-schema` | `8eed26c`: migración de clientes, equipos, órdenes, seguimiento, garantías, pagos e historial; tres pruebas de esquema. |
| HRL-12 | `feat/HRL-12-local-authentication` | `8d33a1f`: alta transaccional, claves protegidas, sesión, formularios conectados y cierre de sesión. |
| HRL-13 | `feat/HRL-13-operation-permissions` | `8f27c2a`: catálogo de operaciones, comprobación en Rust, alta/edición/desactivación de usuarios y permisos personalizados. |
| HRL-14 | `feat/HRL-14-sqlite-orders` | Recepción conectada, importación conservando originales, autorización, folios únicos e historial transaccional. |

## Decisiones confirmadas

- No habrá recuperación automática de la clave maestra. Procedimiento manual
  por definir; no se añade una puerta de recuperación alternativa.
- El acceso Limitado puede consultar todas las órdenes y datos de cliente y
  equipo. Puede registrar recepciones, notas, etapas, garantías y refacciones;
  no pagos, edición ni borrado.

## Validación realizada

- PASS: 21 pruebas Node (`node --experimental-strip-types --test tests/*.test.mjs`).
- PASS: 8 pruebas Rust (`cargo test --offline --manifest-path src-tauri/Cargo.toml --lib`).
- PASS: TypeScript y Vite (`npm.cmd run build`).
- PASS: revisión de espacios (`git diff --check`).

Las pruebas Rust comprueban cuentas y órdenes tras reabrir bases temporales,
rechazo sin sesión o permiso, desactivación/revocación, protección del último
administrador, rollback de alta/importación/orden, folios duplicados, edición
desactualizada, conservación de moneda y exclusión de claves del historial.
No se usan credenciales reales ni se importa el almacenamiento del usuario
durante estas pruebas.

## Límites y siguientes criterios

La app nativa recompilada mostró el primer uso mediante accesibilidad. No
se pudo capturar ni automatizar la ventana para completar el recorrido.
No se creó el administrador real: sus claves debe configurarlas el usuario.
El recorrido visual/nativo completo queda pendiente y no se certifica la
interfaz como entrega final. Se mantienen los tickets en curso.

Reparaciones conserva su almacenamiento antiguo. Mientras se conectan sus
operaciones de agregar notas/avanzar etapas al backend, se abre solo para
acceso Completo. El catálogo de permisos sí está implementado y probado,
pero esto no acredita todos los flujos del acceso Limitado de HRL-13.
Empresa sigue en localStorage: los controles respetan el permiso de edición,
pero falta trasladar su persistencia y autorización a Rust.

No se implementó el borrado/archivo ni se reducen precios o corrigen pagos
desde la recepción. Esas operaciones esperan las reglas pendientes de HRL-9.
No se migraron los abonos de Reparaciones, ni se certificó impresión,
cámara o integración web. El registro inicial del historial avanza también
HRL-15; su revisión completa continúa pendiente.

`extract-reference.mjs` y `tmp-reference/` siguen fuera de estos commits;
ya estaban sin seguimiento antes de este trabajo.
