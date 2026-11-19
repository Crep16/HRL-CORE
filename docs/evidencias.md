# Evidencias locales por orden, HRL-20

## Ampliación normalizada, HRLP-67 (17 de noviembre de 2026)

La implementación normalizada admite imágenes JPG/PNG/WebP, documentos PDF y videos MP4, hasta 20 MB por archivo. En Clientes, selecciona una orden guardada, abre Evidencias y usa Seleccionar archivos. La galería muestra imágenes, reproduce MP4 con controles y permite guardar una copia del PDF o del video. Un códec no disponible muestra un mensaje y conserva la opción de guardar copia. La cámara sigue capturando únicamente fotos; no graba video.

Todos los tipos usan los mismos permisos de Clientes: Visualizar para consultar, Crear para agregar, Editar para describir y Borrar para retirar/restaurar o ver retiradas. Las descripciones siguen limitadas a 500 caracteres. No se eliminan físicamente los archivos al retirarlos. Se conservan revisión, autor, fecha, reintento con el identificador original y auditoría transaccional.

El servicio Rust valida tamaño, origen, MIME y estructura básica del formato: cabeceras de imagen, encabezado y marcador final PDF, o cajas MP4 con tamaños válidos y presencia de `ftyp`, `moov` y `mdat`. No es un analizador completo ni antivirus. La primera versión no admite Word, Excel, AVI, MOV ni otros formatos. PDF se guarda como copia para abrirlo con un lector; no se ejecuta un visor PDF dentro de la aplicación.

La tabla activa es `evidencias`; `tipo_archivo` contiene `image`, `video` o `document`. Los recursos están bajo `assets-3fn/evidencias/<id_orden>/<codigo_archivo>.<extensión>` junto a la base normalizada. La base sola no contiene los archivos. El esquema ya admitía estos tipos y no se añadió migración para esta ampliación.

Se ejecutaron 105 pruebas Rust, comprobación Rust, suites UI/fase 5/IPC, build y recorrido de Chrome con IPC simulado: carga de PDF/MP4, copia PDF, reproducción de un video de prueba, retiro/restauración y tres anchos de ventana. La validación de códecs y guardado de copias dentro del WebView instalado permanece en el recorrido nativo HRLP-63. La implementación y sus pruebas están registradas en el commit `37137e3`; el informe histórico se conserva en Git y en la memoria local.

Las secciones siguientes describen la implementación histórica anterior al esquema normalizado; sus rutas, nombres de tablas y comandos antiguos no sustituyen los anteriores.

Actualización de nombres: desde la migración 013, la tabla física se llama
`Evidencias_orden`, con `id_orden`, `nombre_original`, `tipo_imagen`, `tamano_bytes`,
`origen`, `descripcion`, `id_autor`, `fecha_registro` y `retirada`. Las referencias
a `order_evidence` más abajo corresponden a la migración histórica 012. Las carpetas
y nombres de archivos de fotografías no cambian por este renombrado.

Implementación del 29 de octubre de 2026. Elección del usuario: guardar imágenes
en el equipo mediante Tauri y continuar los avances sin ejecutar pruebas.

## Uso

En Clientes, guarda primero la orden y termina cualquier edición de la ficha.
El apartado Evidencias permite abrir la galería, tomar una fotografía con la
cámara web o seleccionar varios archivos desde el explorador. Una foto trasladada
del celular se selecciona igual que cualquier archivo de la computadora.

La importación copia el contenido: mover o borrar el archivo de origen no cambia
la evidencia guardada. Se aceptan JPG, PNG y WebP de hasta 20 MB por archivo.
La cámara solicita video sin micrófono, permite elegir dispositivo si hay varios,
capturar, revisar, repetir y agregar con una descripción opcional. Se apaga al
capturar, cerrar o desmontar el componente. Una captura pendiente pide confirmar
su descarte. Se muestran mensajes ante cámara ausente, ocupada o permiso rechazado.

La galería consulta SQLite y lee las fotos mediante comandos Rust autorizados.
Muestra la imagen completa, miniaturas, nombre, origen, tamaño, autor y fecha.
Las descripciones admiten 500 caracteres y se guardan expresamente. Cambiar de
foto intenta guardar una descripción pendiente; cerrar permite conservarla o
descartarla. Durante el guardado se bloquea cerrar el diálogo.

Retirar una foto la oculta de la lista activa sin borrar el archivo. Deshacer
revierte el último retiro y Ver fotos retiradas permite restauraciones posteriores,
incluso tras cerrar la aplicación. Una orden en papelera conserva sus evidencias;
primero debe restaurarse para modificarlas.

## Ubicación y respaldo

Raíz: `app.path().app_local_data_dir()/evidencias`.
En Windows, con el identificador actual, normalmente corresponde a:

```text
%LOCALAPPDATA%/com.hector.hrl-core/evidencias/
  <folio codificado en hexadecimal UTF-8>/
    <identificador único de foto>.jpg
    <identificador único de foto>.png
```

La codificación del folio evita que sus caracteres se interpreten como rutas.
Cada foto tiene un identificador aleatorio y no reemplaza otros archivos del
mismo nombre. Las rutas se construyen en Rust; la interfaz no envía rutas de disco
ni obtiene acceso general al sistema de archivos. La base no contiene imágenes
en base64 ni rutas absolutas del usuario.

Un respaldo debe copiar **la base SQLite y toda la carpeta `evidencias`**, con HRL
cerrado, conservando los identificadores y la estructura. La ubicación de SQLite
la gestiona el plugin SQL; no se supone que esté en la misma carpeta que las fotos.
Conservar solo el `.db` no respalda las imágenes. No se añade sincronización web
ni una herramienta automática de respaldo en este incremento.

## Persistencia, permisos e historial

- Migración `012_order_evidence.sql`, registrada en `src-tauri/src/lib.rs`.
- Comandos: `list_evidence`, `read_evidence`, `add_evidence`, `update_evidence`.
- Consultar exige Visualizar Clientes; agregar, Crear Clientes; describir, Editar
  Clientes; retirar/restaurar o consultar retiradas, Borrar Clientes. El acceso
  Limitado puede consultar/agregar; Completo puede todas las operaciones.
  Personalizado sigue sus permisos existentes. Se admite autorización temporal
  del apartado Clientes y se revalidan permisos en Rust.
- Crear requiere una orden persistida y activa. Los bytes originales se validan
  por tamaño y cabecera JPEG/PNG/WebP; la interfaz comprueba que el navegador
  puede decodificar la imagen. Esto no se presenta como validación antivirus.
- Se escribe el archivo y se sincroniza antes de confirmar referencia e historial.
  Si falla el registro o auditoría antes del commit, se revierte la transacción y
  se intenta retirar el archivo nuevo. Si el resultado del commit es incierto,
  se conserva el archivo y se reintenta con el mismo identificador y contenido.
  El reintento recupera el registro existente sin duplicar el evento. La cámara
  conserva foto y descripción de la primera solicitud durante ese reintento.
- Las fotos válidas de una selección múltiple se guardan aunque otras fallen;
  las fallidas quedan pendientes de reintento en esa ventana. Cerrar la aplicación
  pierde las solicitudes que no llegaron a guardarse; el archivo de origen no se mueve.
- Un cierre forzado entre la escritura del archivo y el commit puede dejar un
  archivo huérfano. No se muestra como evidencia sin fila confirmada en SQLite;
  no hay purga automática que pueda borrar imágenes válidas.
- La actualización usa revisión esperada para rechazar ediciones desactualizadas.
  La referencia original, archivo, autor y fecha son inmutables; solo cambian
  descripción, retiro y revisión. No existe borrado físico desde la interfaz.
- Bitácora registra altas, cambios de descripción, retiros y restauraciones por
  folio y autor, con identificador/nombre de foto, origen y cambios antes/después.
  El contenido binario de las fotos no se copia a Bitácora.

## Estado de entrega

`npm.cmd run build` completó TypeScript y Vite.
`cargo check --offline --manifest-path src-tauri/Cargo.toml --lib` completó Rust.
Son comprobaciones de compilación, no pruebas funcionales.

También completó `npm.cmd run tauri -- build --debug --no-bundle`, generando
`src-tauri/target/debug/hrl-core.exe` con la configuración normal de HRL.
No se abrió el ejecutable ni se construyó un instalador en este incremento.

No se ejecutaron suites, pruebas de cámara/importación/reinicio ni revisión visual
por la instrucción del usuario de omitir pruebas. No se modificó la base real del
taller ni se aplicaron manualmente migraciones sobre ella; la aplicación aplicará
la 012 al iniciar normalmente con su gestor de migraciones.

HRL-20 permanece En curso con la implementación registrada. No se declara Terminado
ni se acredita una prueba nativa de persistencia que no se realizó.
