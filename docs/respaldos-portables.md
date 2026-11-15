# Respaldos portables

## Decisiones confirmadas por el usuario

El 14 de noviembre de 2026 el usuario definió estos requisitos:

1. El respaldo debe permitir recuperar la información aunque la computadora
   original deje de funcionar completamente.
2. Debe poder transportarse en un dispositivo fácil de llevar, como USB o disco
   externo, sin depender de un modelo de dispositivo específico.
3. Debe existir creación manual y automática. El automático se inicia al cerrar
   la aplicación y el archivo se conserva en una carpeta del programa, siguiendo
   el comportamiento de referencia descrito por el usuario para Eleventa.
4. La opción estará en **Herramientas → Respaldos**. Su acceso queda reservado
   a usuarios con acceso **Completo**, con validación de la clave maestra para
   configurar o restaurar. Limitado y Personalizado no reciben acceso a esta
   herramienta por permisos parciales ni préstamos temporales. La restricción
   se comprobará en Rust además de la presentación del menú.
5. El archivo portable se abre con **la clave maestra del taller vigente al
   crearlo**, según la respuesta explícita del usuario. No se introduce una
   contraseña de respaldo independiente.

Estos requisitos sustituyen la necesidad de decidir si HRL CORE debe ofrecer
recuperación portable y qué modos de creación se desean. No acreditan una
implementación o una restauración ya probada.

## Funcionamiento implementado

- Carpeta `Respaldos` dentro de los datos locales de HRL CORE, con una acción
  para abrirla. En Windows se ubica en
  `%APPDATA%\com.hector.hrl-core\Respaldos`, usando la ruta efectiva de Tauri.
- Producir un archivo portable con fecha y hora por ejecución, que pueda
  copiarse a USB o disco externo. El conjunto incluirá la base normalizada,
  fotografías, logo y las claves necesarias para recuperar las credenciales
  cifradas de los equipos.
- El paquete y su material criptográfico se protegen con la clave maestra.
  Para activar el automático se escribe una vez en Herramientas. La capacidad
  de cifrado derivada se conserva protegida por DPAPI; no se guarda la clave
  maestra en texto. Cambiarla invalida esa configuración: hay que reactivar
  el automático con la clave nueva. Las copias anteriores conservan su clave.
- Al cerrar, atender primero los borradores y las operaciones en curso. El
  respaldo solo representa información confirmada. Crear una copia consistente,
  verificar el archivo completo y después continuar con el cierre.
- Si falla por falta de espacio, permisos u otro motivo, se muestra el error
  y se permite reintentar. El flujo de salida distingue explícitamente
  entre respaldo confirmado y salida sin respaldo nuevo; no anunciar éxito
  antes de terminar ni dejar un archivo parcial como copia válida.
- Restaurar primero en un destino temporal, validar formato, integridad de la
  base, referencias a recursos y claves. Conservar la instalación y el respaldo
  originales hasta completar la validación. Probar otra identidad Windows sin
  depender del perfil o de la computadora de origen.

La restauración en una instalación con datos exige acceso Completo y su clave
maestra actual, además de la clave con que se creó el respaldo. En un primer
inicio vacío aparece **Restaurar un respaldo de otra instalación**: la clave
del archivo debe abrirlo y verificar la clave maestra de la base recuperada.
No permite sobrescribir una instalación configurada sin autorizarla.

Al restaurar, la instalación anterior queda en una carpeta `hrl-anterior-*`
junto al directorio de datos. El programa exige reiniciarse; se utilizan las
cuentas y los datos del respaldo, y la custodia AES se protege para el perfil
Windows de destino. El automático debe activarse de nuevo en esa instalación.

La creación usa una copia SQLite consistente mediante `VACUUM INTO`, con las
escrituras del servicio bloqueadas durante la captura. Comprueba integridad,
referencias, tamaños y descifrado de credenciales antes de confirmar el archivo.
El archivo se vuelve a leer y verificar antes de pasar de `.partial` a
`.hrlbackup`. No se copian los respaldos anteriores dentro de una copia nueva.

Formato versión 1: Argon2id v1.3 con 19456 KiB, dos iteraciones y paralelismo 1;
clave de 256 bits y AES-256-GCM por bloques de hasta 1 MiB. Un identificador
aleatorio del paquete y el índice autentican cada bloque; se comprueban el
final, las rutas, tamaños y hashes SHA-256 de los archivos. Permite verificar
archivos grandes sin cargarlos completos en memoria. La clave AES del equipo
solo viaja dentro del contenido cifrado, nunca como archivo de texto.

La protección portable de claves no se resuelve copiando únicamente
`device-password-v1.dpapi`: el proveedor actual está ligado al perfil Windows.
El restablecimiento de contraseñas de acceso por firma de soporte tampoco
sustituye la recuperación de las claves de cifrado.

## Aspectos pendientes de validar o ampliar

- Validación nativa de cierre, selector de archivo, reinicio y apertura de
  carpeta en el equipo de destino. Prueba de traslado entre dos identidades
  Windows distintas; no se sustituye por una ejecución con el mismo perfil.
- Conservación de versiones y eventual limpieza automática. No se autoriza
  eliminar copias antiguas mediante esta definición.
- Si además de la carpeta local debe existir una copia automática directa a un
  dispositivo externo y qué hacer cuando no esté conectado. El usuario confirmó
  almacenamiento local al cerrar y transportabilidad; no confirmó obligación de
  conectar un USB en cada salida.
- Recuperación de una instalación que no puede arrancar por base corrupta o
  custodia ausente: conservar su carpeta completa y usar una instalación vacía
  para restaurar. El formulario normal no evita la validación del arranque.

Si se olvida la clave maestra con que se creó una copia, el restablecimiento de
acceso de soporte no descifra ese archivo ni cambia su contraseña histórica.

Una copia guardada solamente en el disco de la computadora no cubre su pérdida
total. La recuperación pedida requiere conservar una copia del archivo en el
dispositivo transportable fuera de ese equipo.

## Seguimiento

Jira: HRL-157, HRL-158, HRL-159 y HRL-160. Los criterios deberán incluir creación
manual, automático al cerrar, archivo local transportable, recuperación sin
equipo de origen y rechazos sin sobrescribir datos ante paquete incompleto,
clave incorrecta, recursos ausentes o versión incompatible.

Estado: implementación local y pruebas automatizadas. `cargo check --offline`
y `npm.cmd run build` pasan. La suite Rust tiene 99 casos y la frontend 50,
todos aprobados. El navegador prueba el menú, la clave maestra, creación,
configuración y estado de restauración con IPC simulado. El cierre espera al
resultado confirmado y sus fallos conservan la ventana abierta en las pruebas
del composable. El caso Rust elimina la instalación de origen temporal y
restaura desde una copia independiente, recuperando abonos, evidencia y la
credencial cifrada; rechaza claves incorrectas, archivos alterados y recursos
ausentes sin reemplazar datos válidos. No se generan instaladores ni se actúa
sobre datos reales del taller.
