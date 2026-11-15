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

Estos requisitos sustituyen la necesidad de decidir si HRL CORE debe ofrecer
recuperación portable y qué modos de creación se desean. No acreditan una
implementación o una restauración ya probada.

## Diseño propuesto para la implementación

- Usar una carpeta `Respaldos` dentro de los datos locales de HRL CORE, con una
  acción para abrirla. La ruta deberá ser escribible por el usuario Windows.
- Producir un archivo portable con fecha y hora por ejecución, que pueda
  copiarse a USB o disco externo. El conjunto incluirá la base normalizada,
  fotografías, logo y las claves necesarias para recuperar las credenciales
  cifradas de los equipos.
- Proteger el paquete y su material criptográfico con una contraseña de respaldo
  independiente. La contraseña se configura en la aplicación; no se pide por
  chat ni se incorpora al repositorio. La preparación del automático debe
  resolver cómo conservar su capacidad de cifrado con protección local.
- Al cerrar, atender primero los borradores y las operaciones en curso. El
  respaldo solo representa información confirmada. Crear una copia consistente,
  verificar el archivo completo y después continuar con el cierre.
- Si falla por falta de espacio, permisos u otro motivo, mostrar el error y
  permitir reintentar. El flujo de salida deberá distinguir explícitamente
  entre respaldo confirmado y salida sin respaldo nuevo; no anunciar éxito
  antes de terminar ni dejar un archivo parcial como copia válida.
- Restaurar primero en un destino temporal, validar formato, integridad de la
  base, referencias a recursos y claves. Conservar la instalación y el respaldo
  originales hasta completar la validación. Probar otra identidad Windows sin
  depender del perfil o de la computadora de origen.

La protección portable de claves no se resuelve copiando únicamente
`device-password-v1.dpapi`: el proveedor actual está ligado al perfil Windows.
El restablecimiento de contraseñas de acceso por firma de soporte tampoco
sustituye la recuperación de las claves de cifrado.

## Aspectos pendientes de concretar

- Configuración y recuperación de la contraseña de respaldo, y personas
  autorizadas para configurar, crear y restaurar copias.
- Conservación de versiones y eventual limpieza automática. No se autoriza
  eliminar copias antiguas mediante esta definición.
- Si además de la carpeta local debe existir una copia automática directa a un
  dispositivo externo y qué hacer cuando no esté conectado. El usuario confirmó
  almacenamiento local al cerrar y transportabilidad; no confirmó obligación de
  conectar un USB en cada salida.
- Formato versionado, compatibilidad, captura consistente de SQLite/recursos y
  estrategia de validación antes de reemplazar datos.

Una copia guardada solamente en el disco de la computadora no cubre su pérdida
total. La recuperación pedida requiere conservar una copia del archivo en el
dispositivo transportable fuera de ese equipo.

## Seguimiento

Jira: HRL-157, HRL-158, HRL-159 y HRL-160. Los criterios deberán incluir creación
manual, automático al cerrar, archivo local transportable, recuperación sin
equipo de origen y rechazos sin sobrescribir datos ante paquete incompleto,
clave incorrecta, recursos ausentes o versión incompatible.

Estado de esta entrega: requisitos y propuesta registrados. La automatización,
el paquete portable y su restauración continúan pendientes de implementación
y validación. No se generan instaladores ni se actúa sobre datos reales del
taller al documentar estas decisiones.
