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

La migración 003 añade el identificador de acceso y la clave maestra protegida.
El backend implementa el alta inicial transaccional, verificación con Argon2id,
sesión en memoria y cierre de sesión. La clave maestra y la del administrador
son independientes. No se guardan claves de acceso en localStorage ni se
devuelven hashes a la interfaz. La interfaz SQL genérica no tiene permisos de
acceso desde el frontend; los comandos Rust usan el pool de la base precargada.

Al reiniciar se conservan las cuentas y se exige iniciar sesión otra vez.
Los usuarios desactivados no pueden entrar; consultar la sesión revalida su
estado. Las cuentas usan un nombre de acceso normalizado, único e insensible
a mayúsculas para las altas nuevas. Una base antigua con nombres duplicados
necesita revisión manual; no se renombra ni elimina ninguna cuenta automáticamente.

La recepción ya usa SQLite con autorización por operación. La importación
de órdenes antiguas requiere acceso Completo y conserva los originales.
Empresa y seguimiento siguen en localStorage; la integración completa de
sus permisos y almacenamiento está pendiente. El seguimiento antiguo no se
abre con usuarios restringidos para evitar conceder edición o pagos por error.

Decisión explícita del 27 de octubre: sin recuperación automática de la clave
maestra; procedimiento manual por definir. No hay cuenta, contraseña o puerta
de recuperación predeterminada. Si ya hay usuarios pero falta la configuración
maestra, el inicio se bloquea con un error de revisión manual y conserva la base.

Comprobaciones: `node --test tests/access-schema.test.mjs` prueba restricciones, referencias y conservación al reabrir un archivo SQLite. `cargo check --offline --manifest-path src-tauri/Cargo.toml` comprueba la integración de la migración con Rust. No equivale a una prueba de primer inicio de la app nativa.

## Siguientes entregas

Completar la integración de permisos en las operaciones locales y la migración
transaccional de órdenes. El usuario decidió que no habrá recuperación automática
de la clave maestra; el procedimiento manual sigue por definir.

## Precisión del acceso limitado, 27 de octubre de 2026

El usuario confirmó: «notas etapas garantias refacciones añadir registros de clientes, sin pagos ni editar o borrar».

Se conservan como operaciones autorizadas el registro de notas, avance de etapas,
garantías, refacciones y alta de clientes. No se concede permiso genérico de
edición por permitir estas operaciones. Pagos, edición y borrado permanecen
restringidos. Después confirmó que puede consultar todas las órdenes y sus datos
de cliente y equipo. Agregar notas y avanzar etapas son operaciones específicas;
no autorizan la edición del contenido ya registrado.

La tabla de permisos personalizados distingue visualizar, crear, editar y borrar.
El backend añade un catálogo específico: consulta/alta/edición de órdenes,
consulta de reparaciones, agregar notas, avanzar etapas, agregar garantías,
agregar refacciones y agregar pagos. El acceso limitado se comprueba por esa
lista específica; no recibe un permiso genérico de editar Reparaciones.

## Administración de usuarios

El acceso completo puede crear, editar y desactivar cuentas desde Usuarios,
con confirmación de clave maestra. La cuenta y sus permisos se guardan en una
transacción. Se rechazan nombres duplicados y la desactivación del último
administrador. Las respuestas excluyen hashes y claves. Cada autorización
consulta de nuevo el usuario activo y sus permisos en SQLite, sin confiar
en el rol de la interfaz. Desactivar una cuenta conserva su autoría histórica.

HRL-14 conecta consulta, creación y edición de órdenes con este catálogo.
La integración de seguimiento continúa en HRL-21. Los permisos personalizados
de Empresa se aplican a sus controles y guardado de interfaz; todavía no son
una autorización de persistencia en Rust porque su almacenamiento sigue en
localStorage. HRL-13 permanece abierto hasta completar esos flujos.
