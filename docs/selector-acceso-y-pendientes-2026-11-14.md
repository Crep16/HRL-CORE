# Selector de acceso y pendientes de Jira

## Cambio autorizado el 14 de noviembre de 2026

El campo Usuario del inicio de sesión abre un desplegable propio de la
aplicación. Permite seleccionar un nombre directamente o filtrar al escribir,
sin distinguir mayúsculas ni acentos. Flechas, Inicio/Fin y Enter permiten
elegir con teclado; Escape cierra la lista. Seleccionar no inicia sesión ni
sustituye la contraseña del usuario.

`list_login_users` devuelve solamente los nombres de acceso activos, ordenados.
Está disponible antes de iniciar sesión para alimentar el selector solicitado.
No expone identificadores, roles, permisos, hashes ni contraseñas. La consulta
recorre command, servicio y repositorio; solo este último contiene SQL. No
cambia la sesión ni escribe auditoría. El servicio de inicio de sesión sigue
verificando la contraseña, el estado vigente y los límites de intentos.

El composable conserva la lista localmente durante la pantalla. Si falla su
carga muestra un aviso y reintento; permite escribir el nombre completo y
continuar con el acceso normal. El alta del primer administrador conserva el
campo de texto original. No requiere migraciones ni cuentas predeterminadas.

Se reutiliza `SearchSelect`, con mensaje vacío y límite de longitud configurables.
La vista presenta el formulario; el composable orquesta el estado a través del
store y la API. Los servicios y repositorios existentes encapsulan acceso y
lectura por composición; no se añade una jerarquía de clases.

## Validación ejecutada

- `cargo check --offline`: PASS.
- `cargo test --offline`: 97/97 PASS con perfil Windows real para DPAPI.
- `node --experimental-strip-types --test tests/*.test.ts`: 45/45 PASS.
- `npm.cmd run build`: PASS.
- `npm.cmd run test:browser`: PASS con componentes reales e IPC simulado.
- Navegador: apertura por clic, búsqueda `LU`, selección con flechas/Enter,
  selección directa, Escape, acceso con el nombre seleccionado, fallo de lista,
  acceso escrito manualmente tras ese fallo y primer inicio sin selector.
- Comprobación a 1000 px y 375 px, sin desbordamiento horizontal. Capturas:
  `artifacts/phase5/login-users.png`, `login-users-error.png` y `login-users-mobile.png`.
- `git diff --check`: PASS.

La prueba Rust comprueba base vacía, exclusión de inactivos, respuesta de solo
nombres, ausencia de sesión y de escrituras al listar, acceso normal y rechazo
de una cuenta desactivada después de consultar la lista. Las pruebas de navegador
no utilizan la base del taller ni acreditan operación de hardware físico.

## Antislop

Lectura: acceso operativo del taller con identidad HRL existente, energía 1,
ritmo 2 y movimiento 1. Se conserva el formulario, su paleta y tipografía; la
lista aparece junto al campo para reducir la escritura del usuario. Se usa el
mismo selector de la aplicación para mantener su comportamiento y presentación.

- Hard Gate PASS: etiquetas reales, estados de consulta/error/sin coincidencias,
  selección operativa y longitud máxima de 100 caracteres; sin contenido inventado.
- R-03 PASS: capturas de acceso a 1000/375 px y comprobación de ancho del documento.
- R-25 PASS: contraste calculado de texto/blanco 16,25:1; ayuda/blanco 6,04:1;
  opción activa 7,43:1; borde/blanco 3,43:1; foco/blanco 5,18:1.
- R-26/R-27 PASS: se probaron selección, acceso, error de lista y acceso manual;
  el reintento de consulta está probado en el composable.
- R-32/R-35 PASS: clic, flechas, Enter y Escape en Chrome; sin excepciones;
  foco visible y selección mediante `combobox`, `listbox` y `aria-activedescendant`.
- Purpose-Gate PASS: lista anclada al campo y flecha funcional para abrir/cerrar;
  sin nuevos recursos, tipografías, tarjetas, ilustraciones o animaciones.
- Liveliness PASS: valores 1/2/1 coherentes con el acceso existente; título,
  espacio entre campos, acento de acceso y marca HRL conservados.
- Craftsmanship y Quality Locks PASS: selector reutilizado, controles probados,
  ayuda específica y comportamiento de error conservando el acceso manual.

## Qué falta según Jira

Consulta: `project = HRL AND statusCategory != Done ORDER BY key ASC`.
Se recuperaron 43 incidencias abiertas: 9 tareas, 3 historias, 27 subtareas y
4 epics. Los padres y sus subtareas describen el mismo trabajo desglosado;
no representan 43 funciones independientes.

| Frente | Referencias vigentes | Falta para cerrar |
| --- | --- | --- |
| Portal web | HRL-10, HRL-31, HRL-32, HRL-36, HRL-50; epic HRL-162 | Acordar contrato, datos públicos, autorización y entorno; implementar publicación real, controles y pruebas contra ese servicio. |
| Custodia y recuperación local | HRL-130, HRL-132, HRL-37, HRL-181 | Validar recuperación conjunta de DB, recursos y clave en Windows; completar los recorridos de instalación cuando se autorice. Las pruebas DPAPI existentes no acreditan otro usuario o equipo. |
| Validación nativa | HRL-138, HRL-141 | Cámara concedida/denegada y liberación, impresión carta/ticket, ventana y reapertura con evidencia en el equipo de destino. |
| Aceptación del taller | HRL-38, HRL-182, HRL-183 | Acordar participantes y datos de prueba, ejecutar el recorrido, registrar incidencias y aceptación real. |
| Importación histórica, backlog futuro | HRL-147, HRL-148 a HRL-150 | Confirmar copia de origen, datos reales, correspondencias y conflictos antes de implementar la importación. |
| Video y documentos, backlog futuro | HRL-151 a HRL-155 | Definir formatos, tamaños, visualización y políticas antes de ampliar evidencias. |
| Traslado automático entre instalaciones, backlog futuro | HRL-157 a HRL-160 | Definir paquete, recuperación portable de claves, retención y compatibilidad; probar origen y destino. |

HRL-117, HRL-142, HRL-162 y HRL-163 son epics de seguimiento, cuyo cierre depende
del trabajo contenido. La descripción vigente de HRL-38 además menciona cinco
ítems de entrega sin identificar en la fuente; no tienen títulos ni alcance
suficientes para implementarlos o declararlos cubiertos.

El cierre del portal depende primero de HRL-10. Los otros recorridos principales
necesitan entorno Windows y participantes reales. Las tres historias de backlog
futuro deben confirmar su alcance de entrega. No se generó ni actualizó instalador,
conforme a la indicación del usuario. Esta consulta no cambia estados de Jira.
