# Distribución de Usuarios

Rama `feat/HRL-13-users-workspace-layout`.

Se sustituye el formulario estrecho en una sola columna por una lista de
cuentas y un editor. La lista permite buscar por nombre o nivel, identifica
la sesión actual con «Tú» y muestra el estado de cada cuenta.

El editor distribuye los datos y las claves a la izquierda, y la descripción
del acceso o matriz de permisos a la derecha. La matriz conserva los mismos
apartados y acciones, con etiquetas accesibles para cada casilla. El guardado
permanece al pie del editor, fuera del área desplazable; las listas largas y
el contenido que no quepa se desplazan dentro de su propio panel.

La disposición de escritorio aprovecha la altura disponible mediante flex
y grid. En vistas estrechas o al ampliar mucho el texto, vuelve a una
disposición vertical que permite consultar todos los campos; no se oculta
contenido para evitar el desplazamiento.

Dirección conservada: herramienta de administración del taller HRL, ENERGY 1 /
RHYTHM 1 / MOTION 1. Se usan la tipografía, paleta e iconos existentes. La
selección usa el azul de HRL; el blanco distingue las superficies de trabajo.
No hay cuentas ni cifras inventadas: los recuentos proceden de la consulta.

Validación: compilación TypeScript/Vite y revisión de espacios. No había una
ventana nativa de HRL abierta para recorrer el formulario. No se crearon ni
editaron cuentas reales. La revisión visual completa sigue pendiente.

## Tamaño de la aplicación

Rama `fix/HRL-13-minimum-window-size`.

La ventana principal establece 1280 × 760 como tamaño inicial y 1120 × 680
como mínimo, en píxeles lógicos de Tauri. Antes permitía 360 × 520, que
comprimía los paneles de trabajo. El límite se aplica al volver a abrir
la aplicación nativa.

Una cantidad grande de cuentas o texto ampliado puede necesitar desplazarse.
La lista y el cuerpo del editor lo permiten dentro de sus paneles; la barra
para autorizar y guardar queda fuera de esa área.

PASS: 22 pruebas Node, compilación TypeScript/Vite, `cargo check --offline`
y revisión de espacios. Los campos `minWidth` y `minHeight` se verificaron
contra el esquema instalado de Tauri. Falta comprobar el tamaño mínimo y
la disposición mediante un recorrido de una ventana nativa abierta.
