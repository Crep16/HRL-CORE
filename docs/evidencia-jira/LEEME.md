# Importar las historias en Jira

1. En Jira, abre **Configuración del proyecto → Importar incidencias desde CSV** (el nombre puede variar según la edición y los permisos del proyecto).
2. Selecciona `historias-jira.csv` y confirma que la codificación sea UTF-8 y que la primera fila contiene encabezados.
3. Mapea `Summary` a Resumen, `Issue Type` a Tipo de incidencia, `Description` a Descripción, `Priority` a Prioridad, `Labels` a Etiquetas, `Sprint` al campo Sprint y `Epic Name` a Nombre de épica.
4. Revisa la vista previa y completa cualquier campo obligatorio adicional que exija el proyecto antes de confirmar la importación.

El CSV contiene 9 épicas (RF01–RF09) y 11 historias. Los valores de prioridad son `Alta` y `Media`; si el proyecto usa opciones distintas, configura la correspondencia en el importador. El campo Sprint se conserva como texto según la lista de origen; Jira puede requerir sprints preexistentes o una configuración adicional para asignarlo. `Epic Name` se llena en las filas Epic; el archivo no incluye una columna de vínculo de épica para las historias.