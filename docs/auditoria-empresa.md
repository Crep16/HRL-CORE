# Empresa: implementación y auditoría de UX/UI

Fecha: 26 de octubre de 2026.

Referencia: `C:\Users\Hector\Documents\ChatGPT\memoria y proyecto de estadias\Bitácora HRL – Clientes.html`. Se extrajeron las siete pantallas de Empresa del HTML empaquetado, sin modificar el archivo original.

## Vistas implementadas

| Vista | Comportamiento |
| --- | --- |
| Datos generales | Nombre, identificación fiscal, contacto, dirección y horario; vista previa inmediata. Los datos de contacto ficticios del prototipo no se precargan. |
| Logo | Carga de PNG/JPG y pegado desde portapapeles cuando el entorno lo permite; validación de formato, lectura y límite de 1 MB; quitar/restablecer; visibilidad por destino. |
| Notas impresas | Mensajes independientes de entrada/salida, contador de caracteres, dirección/RFC, firma y referencia interna de la orden. |
| Cláusulas | Alta, edición, reordenación con teclado, selección de entrada/salida, numeración, posición al pie/reverso y eliminación con deshacer. |
| Hoja de impresión | Carta y tickets de 80 y 60 mm; orientación para carta, 1–5 copias y preparación de un documento para el diálogo de impresión del sistema. En carta, las copias se agrupan de dos en dos en una misma hoja, con línea de corte. |
| Folios y garantías | Prefijo y siguiente número conectados a nuevas órdenes, sin sobrescribir folios existentes; sugerencias independientes para mano de obra y refacciones. |
| Región e impuestos | Formato regional, moneda, zona horaria y desglose del impuesto incluido en el total. La interfaz permanece en español y no se convierten importes entre monedas. |

La configuración se guarda explícitamente en `hrl.company.v1`, dentro del almacenamiento local existente de la aplicación. El formulario mantiene un borrador separado de la configuración confirmada.

## Hallazgos corregidos

| Prioridad | Problema | Corrección y comprobación |
| --- | --- | --- |
| Alta | Empresa estaba deshabilitada y faltaban todas sus vistas. | Navegación funcional con las siete secciones identificadas en la referencia. Bitácora conserva su indicación de pendiente. |
| Alta | La navegación podía descartar un formulario pendiente. | Bloqueo de salida mientras haya cambios de Empresa o una orden en captura; descarte con confirmación dentro de la vista y aviso del navegador al cerrar con cambios. Se comprobó guardar, descartar y conservar el borrador. |
| Alta | Un error de almacenamiento podía aparentar un guardado correcto. | Solo se actualiza la configuración confirmada después de escribir correctamente. La prueba con cuota agotada conserva el formulario, los datos anteriores y un mensaje de error. |
| Alta | Entradas inválidas no tenían una recuperación clara. | Validación central de correo, web, folios, copias, garantías, impuesto y cláusulas. Resumen enfocable con accesos a los campos y errores junto a los campos de contacto. |
| Alta | El prototipo daba por disponible una publicación web aún inexistente. | Mensajes explícitos de guardado local y sincronización pendiente. La referencia impresa utiliza el folio interno, sin inventar un enlace de seguimiento. |
| Alta | La numeración podía ignorar las preferencias de Empresa. | Las órdenes nuevas usan el prefijo configurado, omiten números ocupados y avanzan el siguiente número al guardar; cancelar la captura no consume un folio. |
| Alta | Una nota de salida podía omitir el seguimiento registrado o exponer notas internas. | Se incluyen únicamente la nota pública del técnico, su nombre y las garantías guardadas. Se comprobó que la nota interna no aparece en el documento de impresión. |
| Media | Los formularios y la vista previa alargaban toda la página. | Tres columnas en escritorio, formulario con acciones siempre accesibles y vista previa con desplazamiento independiente para notas largas. Las siete vistas iniciales miden 1366 × 768 sin scroll de página en ese tamaño de ventana. |
| Media | El menú completo consumía demasiado espacio en móvil. | Selector de sección en móvil, campos apilados y controles táctiles de al menos 44 px en las nuevas vistas. |
| Media | Las opciones de impresión no tenían efecto real. | Documento aislado con CSS de papel/orientación, copias y página de reverso. Se comprobó la preparación de tres copias en carta horizontal y la solicitud del diálogo. |
| Media | Quitar una cláusula no ofrecía recuperación. | Deshacer antes de confirmar la configuración. Al descartar o guardar se elimina la operación pendiente de deshacer para impedir duplicados. |
| Media | La región elegida no se aplicaba al seguimiento. | Nuevas recepciones y marcas de tiempo de Reparaciones usan la zona configurada; los importes de seguimiento y notas usan la moneda/formato seleccionados. Se probaron separadores decimales mexicanos y europeos. |
| Media | Los controles podían perder foco al reordenar una cláusula. | El foco vuelve a la acción correspondiente o al título de la cláusula si la flecha queda deshabilitada. |

## Validación

- `npm run build`: compilación de Vue/TypeScript y Vite correcta.
- `npm run test:company`: nueve pruebas automatizadas correctas sobre folios, agrupación de copias, anchos de ticket, migración del formato anterior, validación, importes, zona horaria, fallos de guardado y recuperación de configuración corrupta. Usa el soporte de TypeScript de Node 24, disponible en este equipo; no añade dependencias de prueba al proyecto.
- Pruebas de navegador con Playwright en contexto aislado: guardado tras recargar, descarte, navegación protegida, cláusulas, notas de entrada/salida, logo, impresión y garantías sugeridas. No modificaron el almacenamiento del usuario en la aplicación abierta.
- Las siete vistas se comprobaron en 320 × 640, 375 × 812, 768 × 1024, 900 × 768, 1100 × 768, 1366 × 768, 1920 × 1080 y 812 × 375: sin desbordamiento horizontal.
- Regresión de recepción y Reparaciones: navegación, nueva orden, notas públicas/internas, historial, evidencias y contraseña; sin errores de ejecución en el navegador.
- Navegación con Tab y foco visible; prueba con preferencia de movimiento reducido.
- Contrastes calculados: texto secundario sobre fondo 5.08:1; botón primario 5.18:1; error 8.42:1; cambios pendientes 5.76:1; borde de campos 3.30:1.

## Cierre de la auditoría visual

- **Hard Gate: PASS en el alcance comprobado.** Sin desbordamiento horizontal en las 56 combinaciones de vista/tamaño; estados de error y navegación operables, sin nuevos enlaces ficticios.
- **Purpose Gate: PASS.** El azul existente identifica acciones y selección; papel blanco para lectura e impresión; bordes para agrupar campos. Se conserva el logo existente y no se generan recursos visuales nuevos.
- **Identidad: PASS.** Se mantienen las tipografías y el estilo de ficha del taller. Energía 2, ritmo 3, movimiento 1: jerarquía de títulos y secciones, acento limitado y ausencia de animación decorativa.
- **Calidad funcional: PASS.** Compilación, nueve pruebas persistentes y recorridos de navegador satisfactorios; estados de guardado comprobados también bajo fallo.

### Ajuste de ahorro de papel

Se verificó en navegador que la nota inicial completa produce una hoja con dos copias en carta vertical y carta horizontal. En vertical se apilan; en horizontal se colocan lado a lado. Cinco copias producen grupos de 2, 2 y 1. Una prueba con texto extenso confirmó que las continuaciones conservan ambos ejemplares en la misma hoja, sin perder texto ni reducir la tipografía para forzar el ajuste. Los tickets conservan copias separadas.

### Actualización de formatos y legibilidad

- Se retiró media carta del selector y del motor de impresión. Las configuraciones anteriores con ese valor se leen como carta, conservando nombre, contacto y demás preferencias.
- Los tickets tienen anchos físicos de 80 y 60 mm, con márgenes de 3 mm. Se comprobó que ambos generan copias independientes sin desbordamiento horizontal.
- Los campos principales usan texto de 16 px; se aumentaron etiquetas, botones, navegación, estados y vista previa. Se reforzó el contraste del texto secundario y de los bordes de los campos.
- Cliente y teléfono comparten una fila en escritorio. Se ajustaron anchos de columnas, márgenes y espacios para mantener las vistas iniciales de Clientes, Empresa y Reparaciones dentro de 1366 × 768. En móvil se conserva la distribución apilada.
- La impresión de carta usa texto mayor y conserva las copias emparejadas. El contenido largo continúa en hojas adicionales en lugar de reducirse para forzar su ajuste.

## Límites comprobables

- La impresión física y el diálogo nativo de la versión Tauri deben comprobarse con la impresora real. Las pruebas de navegador verifican el documento y la llamada de impresión; no afirman que se haya impreso papel.
- El reverso requiere doble cara o impresión manual en el diálogo del sistema. Las notas extensas pueden ocupar más de una página.
- El pegado de imágenes depende del permiso y de la compatibilidad del portapapeles del entorno; subir archivo es la alternativa disponible.
- El almacenamiento sigue siendo local. No se implementan en este cambio sincronización web, autenticación ni el módulo Bitácora.
- En móvil, formularios extendidos, cláusulas y notas largas conservan el desplazamiento necesario para mantener legibilidad y acceso a todos los datos.
