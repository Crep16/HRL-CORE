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
eventos y su catálogo de campos excluye claves. La escritura transaccional
del historial junto con cada operación debe implementarse en el backend;
el esquema por sí solo no crea eventos automáticamente.

La garantía guarda cobertura y duración; su inicio debe derivarse de la
entrega de la orden. No se almacena una fecha de inicio independiente.

Pruebas: `node --test tests/workshop-schema.test.mjs` verifica la actualización
sobre usuarios existentes, reapertura de una orden con sus relaciones,
restricciones e historial, y rollback ante un evento con autor inválido.
`cargo check --offline --manifest-path src-tauri/Cargo.toml` comprueba el
registro de la migración en Rust.

Esta entrega no migra localStorage ni conecta las pantallas de órdenes a
SQLite. La prueba de reapertura usa un archivo SQLite temporal; la prueba
nativa completa de HRL-11 y la integración de HRL-14 siguen pendientes.
