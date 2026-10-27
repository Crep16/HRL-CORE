# Esquema local

La migración 001 conserva usuarios y permisos. La migración 002 añade
clientes, equipos, órdenes, avances, notas, refacciones, garantías, pagos
e historial. Ambas están registradas para `sqlite:hrl_datos.db`.

Los importes se almacenan en unidades menores enteras (centavos para
MXN). Los pagos de este esquema son cobros positivos; la política y las
operaciones de devoluciones y correcciones siguen pendientes de HRL-9.
No se infiere que un estado de reparación salde o elimine un adeudo.

Las relaciones impiden borrar registros referenciados. Desactivar a un
usuario conserva su autoría. El historial no admite editar ni borrar
eventos y su catálogo de campos excluye claves. Los comandos de recepción
confirman el historial junto con la orden. El esquema por sí solo no crea
eventos automáticamente; los flujos restantes deben conectarse al backend.

La garantía guarda cobertura y duración; su inicio debe derivarse de la
entrega de la orden. No se almacena una fecha de inicio independiente.

Pruebas: `node --test tests/workshop-schema.test.mjs` verifica la actualización
sobre usuarios existentes, reapertura de una orden con sus relaciones,
restricciones e historial, y rollback ante un evento con autor inválido.
`cargo check --offline --manifest-path src-tauri/Cargo.toml` comprueba el
registro de la migración en Rust.

## Recepción conectada, HRL-14

La migración 004 añade la ficha completa serializada y el registro de la
importación anterior. Los comandos de recepción mantienen ficha y relaciones
en una transacción; la ficha conserva formatos y campos que todavía no
tienen operaciones propias. Las cantidades normalizadas se guardan en
centavos junto con la moneda. Una orden existente no cambia de moneda.

La consulta revalida permisos. Un permiso de crear sin visualizar no devuelve
órdenes existentes, pero permite solicitar un folio disponible y crear una
recepción. Folios repetidos y ediciones con una ficha original desactualizada
se rechazan. La interfaz conserva el borrador ante errores de guardado.

La importación de `hrl.orders.v1` requiere un administrador y una base vacía.
Se conserva el original; el marcador de importación se confirma junto con
todas las órdenes. Los autores son los operadores de la importación, no
autores históricos supuestos. Los datos anteriores no indican la fecha real
de cada cobro o entrega: el abono inicial histórico usa fecha vacía y la
entrega histórica no recibe una fecha inventada. La garantía no debe iniciar
automáticamente con una fecha de importación. La moneda antigua se toma de
la configuración de Empresa al importar, pues antes no se guardaba por orden.

El abono inicial nuevo requiere permiso de pagos. No se corrige un pago ni
se reduce un precio editando la ficha, mientras se resuelven las reglas de
devoluciones y correcciones. El seguimiento de Reparaciones aún guarda sus
abonos en localStorage y no se ha trasladado al libro de pagos SQLite.

El historial de alta y cambios de campos compatibles se confirma junto con
la orden y excluye la clave del equipo. La clave del equipo se conserva en la
ficha local; no debe confundirse con las claves de acceso protegidas por hash.

Pruebas Rust adicionales: reapertura con orden completa, rechazo sin sesión,
rechazo de edición desactualizada, rollback si falla el historial, importación
duplicada sin residuos, conservación de fechas históricas desconocidas y
rechazo de pagos/edición/importación para acceso Limitado. La verificación
nativa completa con una cuenta real y la revisión visual siguen pendientes.
