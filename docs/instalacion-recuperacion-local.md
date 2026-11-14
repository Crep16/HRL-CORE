# Instalación y recuperación local de HRL CORE

Procedimiento del 13 de noviembre de 2026. Referencias de Jira HRL-131 y HRL-180. La validación automatizada cubre SQLite y custodia DPAPI en directorios temporales. La instalación con cámara, impresora y personal del taller requiere registrar su ejecución en HRL-141 y HRL-38.

## Instalación y primer inicio

1. Generar el frontend con `npm.cmd run build` y el instalador con `npm.cmd run tauri build`. Los paquetes se producen en `src-tauri/target/release/bundle/`. Conservar la versión del repositorio y el SHA-256 del instalador entregado.
2. Ejecutar el instalador con el usuario Windows que operará la aplicación. Abrir HRL CORE con esa identidad y completar la clave maestra y el primer administrador. El servicio confirma ambas credenciales y su auditoría en una transacción.
3. Cerrar y abrir HRL CORE. Comprobar el inicio de sesión, la empresa y una orden de prueba autorizada. Registrar los resultados y las incidencias de instalación.

## Conjunto que debe respaldarse

La ruta se obtiene con `app.path().app_config_dir()` y el identificador `com.hector.hrl-core`. En Windows corresponde a `%APPDATA%\com.hector.hrl-core`. Confirmar la ruta efectiva con el perfil Windows de la instalación.

| Recurso | Ruta relativa | Uso |
| --- | --- | --- |
| Base normalizada | `hrl_core_3fn.db` | Usuarios, órdenes, libros, auditoría, secuencias y referencias a recursos |
| Recursos | `assets-3fn/` | Evidencias en `evidencias/` y logos en `company/` |
| Custodia | `keys/device-password-v1.dpapi` | Clave AES protegida por DPAPI del usuario Windows |

Los directorios de recursos aparecen al guardar sus primeros archivos. Conservar el directorio de configuración completo permite incluir también archivos auxiliares SQLite como `-wal` y `-shm` cuando existan. Las bases anteriores y sus recursos pertenecen a sus respaldos históricos.

## Respaldo manual consistente

1. Cerrar todas las instancias de HRL CORE y comprobar que sus procesos terminaron. Detener las escrituras antes de copiar el conjunto.
2. Copiar el directorio de configuración completo a un directorio de respaldo nuevo. Conservar la identidad Windows y el equipo de origen, la versión de HRL CORE y la fecha del respaldo.
3. Comprobar que la copia incluye base, archivos de evidencia/logo referenciados y custodia DPAPI. Registrar un manifiesto con rutas, tamaños y hashes SHA-256. Restringir el acceso al respaldo porque contiene información del taller.
4. Verificar la copia con la misma versión de la aplicación y la identidad Windows autorizada. Registrar el resultado de `PRAGMA integrity_check` y `PRAGMA foreign_key_check` mediante una herramienta SQLite sobre la copia, junto con las pruebas de lectura de órdenes y recursos.

La automatización de captura y restauración corresponde a HRL-157. El plugin de respaldo del esquema anterior quedó archivado con el backend histórico.

## Recuperación con la misma custodia Windows

1. Trabajar primero sobre una copia del respaldo y conservar el conjunto original. Comparar el manifiesto y comprobar que están todos los recursos referenciados.
2. Cerrar HRL CORE. Conservar por separado el directorio de configuración actual y colocar el conjunto completo validado en la ruta de destino correspondiente al usuario Windows original.
3. Abrir la misma versión de HRL CORE con la identidad Windows que protegió la clave. Iniciar sesión y comprobar órdenes, pagos, auditoría, evidencias, logo y una credencial de prueba autorizada.
4. Si la custodia falta o está corrupta y existen credenciales cifradas, el arranque devuelve el error de protección/recuperación. El servicio exige la clave existente. Recuperar el archivo protegido correcto antes de repetir la apertura.
5. Registrar versión, origen del respaldo, identidad y equipo de destino, resultados e incidencias. Conservar las copias previas hasta completar la comprobación.

DPAPI depende de las credenciales y del perfil Windows que protegió el archivo. La portabilidad a otro usuario, equipo o perfil reinstalado requiere una estrategia de custodia aprobada en HRL-157. El restablecimiento Ed25519 cambia las contraseñas de acceso de la aplicación y conserva la clave de cifrado del equipo.

## Comprobaciones pendientes de entrega

| Caso | Evidencia requerida |
| --- | --- |
| Instalación y desinstalación | Versión, instalador, entorno, primer inicio y reapertura observados |
| Recuperación conjunta | Manifiesto completo, integridad SQLite y lectura real de recursos y credenciales |
| Custodia de otro usuario Windows | Rechazo observado en el entorno de destino autorizado |
| Cámara y permisos del sistema | Captura concedida y denegada, cierre y liberación del dispositivo |
| Impresión | Resultados físicos en carta y tickets de 80/60 mm |
| Aceptación del taller | Participantes autorizados, recorrido, incidencias y decisión de aceptación |
