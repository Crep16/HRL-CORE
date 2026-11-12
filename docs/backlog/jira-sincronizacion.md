# Preparación de HRL registro y estado de Jira

Revisión del 12 de noviembre de 2026. Fuente única de alcance: `docs/backlog-base (1).md`; `docs/jira-import (1).csv` confirma los 80 ítems base y el cambio de padre de HRL-64. No se tomó alcance de copias anteriores.

## Estado real del destino

**El proyecto nuevo HRL registro no se ha creado ni cargado.** La consulta `listJiraProjects` devuelve HRL CORE con clave HRL (ID 10066, 150 ítems). La clave requerida sigue ocupada. No se reinició el contador, renombró ni borró ese proyecto, ni se actualizó su backlog con esta revisión.

El conector de Atlassian disponible no ofrece crear/renombrar un proyecto Jira ni liberar su clave anterior. El acceso del navegador informó que el host no estaba disponible; el intento de vincular la ventana Opera agotó la autorización de control. Falta acceso administrativo para revisar una vía que conserve el proyecto previo y permita usar HRL en uno nuevo. No se sustituye la solicitud por otra clave sin acordarlo.

## Entregables preparados

| Archivo | Contenido |
| --- | --- |
| [resumen.md](resumen.md) | Nueve epics, distribución de puntos, discrepancias y cinco ítems faltantes |
| [jira-proyecto-nuevo.json](jira-proyecto-nuevo.json) | Manifiesto con descripciones completas, jerarquía, estado documental, puntos y condiciones previas |
| [jira-import-actualizado.csv](jira-import-actualizado.csv) | 133 filas: 80 ítems base y 53 subtareas, claves solicitadas explícitas y Parent ID |
| [jira-correspondencia.json](jira-correspondencia.json) | Referencias actuales y claves solicitadas; claves reales del nuevo destino todavía nulas |
| [Histórico de correspondencia](historico/jira-correspondencia-2026-11-11.json) | Mapa de la sincronización anterior, sin aplicarlo como alcance actual |
| [Histórico de sincronización](historico/jira-sincronizacion-2026-11-11.md) | Registro previo de Jira, conservado para trazabilidad |

El manifiesto y CSV contienen los títulos con [P03], [S305], [S306], [S401], [A05], [S206], [S307], [S402] y [S403]. HRL-64 apunta a HRL-77. HRL-66 no aparece como fila base ni se crea para consumir un número. Los cinco ítems faltantes no tienen filas, claves ni estimaciones.

## Condiciones de numeración y jerarquía

- Las `requestedKey` son objetivos de importación, no claves ya creadas. La correspondencia real no se rellena hasta comprobar el destino.
- Un proyecto nuevo debe empezar desde HRL-1. Hay 80 ítems pero el último identificador base es HRL-81 porque se omite HRL-66; una creación secuencial simple desplazaría las referencias posteriores.
- HRL-64 tiene padre HRL-77, que aparece después por número. Validar el tratamiento de `Issue ID`/`Parent ID` y claves explícitas del importador antes de cargar; con API sería necesario resolver el padre después de su creación.
- Las 53 subtareas HRL-82 a HRL-134 se cargan después de todos los ítems base. No crear un resumen adicional ni otras tareas antes de fijar la numeración prevista.
- Verificar tipos de incidencia, prioridades, campos de puntos y estados reales del proyecto nuevo; no reutilizar IDs de campos/tipos o sprints del proyecto anterior sin comprobarlos.
- `Sprint documental` es una columna auxiliar. Los cuatro sprints se crean o verifican en el tablero nuevo y después se asignan; los IDs 104–107 pertenecen al tablero anterior y no son destino de esta carga.
- Las columnas CSV se deben revisar/mapear en el importador administrativo. El CSV está preparado para revisión; no se ha probado ni importado y no certifica compatibilidad automática del importador.

## Alcance y puntos verificados por lectura

| Sprint | Ítems | Puntos | HRL-71 ítems/puntos | HRL-77 ítems/puntos |
| --- | ---: | ---: | ---: | ---: |
| 1 | 10 | 55 | 0/0 | 0/0 |
| 2 | 11 | 63 | 0/0 | 1/5 |
| 3 | 15 | 64 | 3/16 | 1/5 |
| 4 | 30 | 167 | 1/8 | 3/26 |
| Backlog | 5 | 37 | 1/5 | 0/0 |
| Total | 71 | 386 | 5/29 | 5/36 |

Los puntos se asignan solo a historias/tareas. Los nueve epics y las 53 subtareas no suman puntos adicionales. Sprint 4 concentra 47,9 % de los 349 puntos programados y 43,3 % de los 386 conocidos. No se aplicó el rebalanceo propuesto.

## Estados preparados, sin afirmar ejecución

Hecho documental expresa implementación localizada por inspección, no pruebas ejecutadas. En progreso conserva brechas concretas; Pendiente no implica inexistencia de infraestructura externa fuera del repo; No verificado mantiene los límites de comprobación. En particular HRL-64 y HRL-81 no acreditan hardware ni personal del taller. Los estados nativos solicitados se mapearán según el flujo real del proyecto nuevo.

Las cinco tareas de sincronización quedan Pendientes. HRL-78/79/80 tienen pruebas/configuración/procedimientos parciales localizados y quedan En progreso. No se ejecutaron builds, suites, instalación, recuperación ni pruebas con personas.

## Historial y alcance incompleto

Los registros del 11 de noviembre se archivaron como históricos y no se modificó el proyecto Jira al que se refieren. Las 33 subtareas retenidas de aquella revisión se renumeraron para no colisionar con los nuevos ítems base; la tabla de renumeración está en el JSON de correspondencia. Se descartaron del alcance actual las tres subtareas de la historia retirada HRL-66 y se añadieron 20 subtareas para las tareas nuevas.

**Faltan cinco ítems por incorporar a HRL-77.** No se inventan para completar la captura original.

