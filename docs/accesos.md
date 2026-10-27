# Base local y accesos

Decisiones del usuario, 26 de octubre de 2026.

La base SQLite reside en una sola computadora. No se implementa servidor compartido.

| Acceso | Regla confirmada |
| --- | --- |
| Completo | Acceso sin restricciones. El primer inicio debe crear un usuario completo con contraseña maestra. Siempre debe conservarse al menos uno activo. |
| Limitado | Notas, etapas, garantías y refacciones en Reparaciones; añadir registros de clientes. Sin pagos, edición ni borrado. Decisión del 27 de octubre de 2026; ver precisión abajo. |
| Personalizado | Selección individual de apartados y acciones: visualizar, crear, editar y borrar. Clientes, Reparaciones, Bitácora y Empresa. |

## Implementación actual

La migración `src-tauri/migrations/001_access.sql` está registrada para `sqlite:hrl_datos.db`, la conexión precargada de Tauri. Añade usuarios y permisos puntuales. Protege en SQLite el primer usuario completo y la conservación del último administrador activo. No crea una cuenta ni una contraseña predeterminada.

`password_hash` reserva el resultado de un hash de contraseña; esta migración no implementa autenticación, hashing ni recuperación. No se deben escribir contraseñas en texto claro. Los permisos almacenados todavía no se aplican a las operaciones existentes. La interfaz sigue usando localStorage para órdenes, Empresa y seguimiento; no se migraron datos del usuario.

Comprobaciones: `node --test tests/access-schema.test.mjs` prueba restricciones, referencias y conservación al reabrir un archivo SQLite. `cargo check --offline --manifest-path src-tauri/Cargo.toml` comprueba la integración de la migración con Rust. No equivale a una prueba de primer inicio de la app nativa.

## Siguientes entregas

Alta inicial y acceso con hash de contraseña en el backend; sesión local y comprobación de permisos en cada operación; administración de usuarios y permisos; migración transaccional de órdenes. Confirmar recuperación de la contraseña maestra antes de implementarla.

## Precisión del acceso limitado, 27 de octubre de 2026

El usuario confirmó: «notas etapas garantias refacciones añadir registros de clientes, sin pagos ni editar o borrar».

Se conservan como operaciones autorizadas el registro de notas, avance de etapas,
garantías, refacciones y alta de clientes. No se concede permiso genérico de
edición por permitir estas operaciones. Pagos, edición y borrado permanecen
restringidos. Falta precisar el alcance de consulta de registros existentes y
cómo distinguir agregar notas/avanzar etapas de editar contenido ya registrado.

La tabla SQLite actual solo distingue acciones genéricas de visualizar, crear,
editar y borrar. Esta decisión aún no se aplica en el backend ni en las vistas;
se necesitarán operaciones con permisos específicos para respetarla.
