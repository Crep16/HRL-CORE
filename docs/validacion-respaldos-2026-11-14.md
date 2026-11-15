# Validación de respaldos portables — 14 de noviembre de 2026

Implementación: Herramientas → Respaldos, acceso Completo y clave maestra.
La copia manual y la copia al cerrar incluyen SQLite, fotografías, logo y la
clave necesaria para recuperar las credenciales cifradas. El archivo se abre
con la clave maestra vigente cuando se creó. La copia automática requiere
activarse una vez desde el formulario; no elimina copias anteriores.

## Evidencia ejecutada

- `cargo check --offline`: aprobado.
- `cargo test --offline`: 99 pruebas aprobadas. Las 43 advertencias provienen
  de funciones del backend legacy compiladas únicamente para pruebas.
- Suite frontend completa: 50 pruebas aprobadas, incluidas las cinco de
  `npm.cmd run test:backups`.
- `npm.cmd run build`: aprobado.
- `node tests/phase5-browser.mjs`: aprobado con IPC simulado; incluye menú,
  rechazo de clave, creación manual, configuración automática, restauración,
  estado de reinicio y ocultación para Limitado/Personalizado.
- Revisión visual a 1000 × 680 y 375 px: diálogo contenido, desplazamiento
  vertical y sin desbordamiento horizontal. Capturas locales en `artifacts/`.

La prueba Rust portable usa DPAPI real con instalaciones temporales. Guarda
un abono, logo, evidencia y contraseña de equipo cifrada; transporta el archivo
a otra carpeta, elimina el origen, restaura en una instalación vacía y verifica
los datos y el descifrado. Rechaza clave incorrecta y archivo alterado sin
reemplazar el destino; una evidencia ausente impide crear una copia válida.
También prueba permisos, activación/desactivación, cambio de clave maestra y
reparación de la configuración automática dañada. Este caso se volvió a
ejecutar tras los últimos ajustes del servicio.

El formato cifra y autentica cada bloque y su orden. Se probaron truncamiento,
mezcla de archivos, contraseñas incorrectas y rutas inválidas. La restauración
valida versión, integridad, relaciones y recursos antes del reemplazo; conserva
la instalación anterior en una carpeta hermana. No importa datos supuestos ni
modifica reglas de dinero, auditoría o migraciones.

## Antislop durante los cambios

Resultado: PASS para la interfaz revisada. Se conserva la identidad existente
de HRL: tipografía del proyecto, texto petróleo, fondo blanco y azul para la
acción principal. ENERGY 1 / RHYTHM 2 / MOTION 1. La jerarquía distingue crear,
configurar y restaurar por función; no se añadieron ilustraciones ni efectos.

Contrastes calculados: texto `#12222d` sobre blanco 16.25:1; blanco sobre azul
`#1b72ad` 5.18:1; borde `#7c8d99` sobre blanco 3.43:1; error `#91251d` sobre
`#fff0ee` 7.62:1. Hay foco visible, controles de 44 px, diálogo modal nativo,
foco al abrir y al cambiar de etapa, retorno de foco, estados de espera,
vacío, error y confirmación. Los mensajes describen acciones concretas.

## Validación pendiente

Falta ejecutar en Tauri el selector de archivos, abrir la carpeta, cerrar con
la X y reiniciar después de restaurar. La espera del cierre y sus fallos sí
tienen pruebas del composable; el navegador no acredita las APIs nativas.
Falta una restauración bajo otra identidad real de Windows o en otra PC;
las instalaciones temporales usaron la misma identidad de ejecución.

Jira HRL-157 y HRL-160 conservan la validación pendiente. No se generó
instalador ni se restauraron o borraron datos reales del taller.
