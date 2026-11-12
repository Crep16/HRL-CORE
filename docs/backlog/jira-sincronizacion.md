# Sincronización de HRL CORE — 12 de noviembre de 2026

Se actualiza el proyecto existente **HRL CORE** (HRL, ID 10066), tablero 34, según la última instrucción del usuario. La propuesta de crear HRL registro queda sustituida.

La única fuente de alcance es `docs/backlog-base (1).md`: 80 ítems base (9 epics y 71 historias/tareas), más 53 subtareas documentadas desde HRL-82 a HRL-134. HRL-66 queda fuera del alcance vigente.

## Numeración visible

Las 71 historias/tareas llevan prefijos **[001] a [071]**, ordenados por referencia documental. Los nueve epics llevan [E01] a [E09]; cada subtarea usa el número de su padre y un sufijo, por ejemplo [039.1]. Se conservan los códigos originales entre corchetes.

Las claves nativas HRL-N no se reiniciaron. Jira Cloud mantiene abierta la [solicitud de reiniciar el contador](https://jira.atlassian.com/browse/JRACLOUD-34286). La referencia de los documentos, el número visible y la clave nativa son identificadores distintos; su correspondencia está en `jira-correspondencia.json`.

## Cambios aplicados

- Se reutilizaron las incidencias existentes y se actualizaron descripciones, prioridades, puntos, estados y jerarquía.
- Nuevos epics: base HRL-71 → Jira HRL-162 y base HRL-77 → Jira HRL-163.
- Se recuperaron las nueve tareas originales con códigos [P03], [S305], [S306], [S401], [A05], [S206], [S307], [S402] y [S403].
- Base HRL-64 pertenece al epic base HRL-77.
- Se crearon 20 subtareas nuevas; 33 subtareas previas se conservaron con referencias documentales actualizadas.
- Jira HRL-143 a HRL-146, correspondientes al portal retirado y sus tres subtareas, quedan cerradas y etiquetadas como reemplazadas. Se conservó su historial.
- El resumen auxiliar Jira HRL-161 no cuenta como historia ni suma puntos.

## Distribución de puntos

| Sprint | Historias/tareas | Puntos |
| --- | ---: | ---: |
| 1 | 10 | 55 |
| 2 | 11 | 63 |
| 3 | 15 | 64 |
| 4 | 30 | 167 |
| Backlog | 5 | 37 |
| Total | 71 | 386 |

Los sprints existentes 104–107 conservan fechas y ciclo de vida. El rebalanceo sugerido no se aplica sin acordar capacidad y alcance. **Sprint 4 supera el 40 %: 47,9 % de los puntos programados y 43,3 % de todos los puntos conocidos.**

## Estados y límites

Hecho → Terminado; En progreso → En curso; Pendiente → Por hacer; No verificado → En prueba, con etiqueta `no-verificado`. Hecho acredita implementación localizada por lectura estática, sin afirmar ejecución de pruebas.

Las cinco tareas de sincronización quedan Pendientes. HRL-64 y HRL-81 conservan No verificado; HRL-78/79/80 quedan En progreso. No se ejecutaron builds, pruebas ni validaciones nativas.

**Faltan cinco ítems por incorporar a la epic documental HRL-77.** No se inventaron nombres, claves, puntos ni criterios.

## Archivos de trazabilidad

- `jira-correspondencia.json`: mapa vigente con claves reales y números visibles.
- `jira-proyecto-nuevo.json`: nombre histórico del manifiesto; su contenido ahora declara actualización del proyecto existente.
- `jira-import-actualizado.csv`: exportación documental con claves reales; no se importó ni certifica compatibilidad con un importador.
- `historico/`: registros de la revisión anterior, conservados sin alteración.

## Comprobación final

Se consultaron nuevamente las 133 incidencias del alcance vigente y se verificaron sus títulos, padres, estados, puntos y sprints sin diferencias. Los 71 números visibles principales son únicos. El resumen auxiliar HRL-161 se excluye de estas cantidades.

