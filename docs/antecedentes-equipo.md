# Antecedentes del equipo

Decisión del usuario del 14 de noviembre de 2026: sustituir las preguntas
separadas de otro taller y caída/humedad por un campo donde escribir lo necesario.

La ficha presenta un solo campo, **Antecedentes del equipo**, de texto libre,
opcional y con límite de 16000 caracteres. No interpreta palabras para asignar
automáticamente marcas de golpes, líquidos o intervención previa.

La migración nueva `003_antecedentes_equipo.sql` añade una columna TEXT nullable
en `ordenes`. No modifica migraciones aplicadas, documentos SQL de la base 001
ni los campos o valores históricos existentes. No importa datos de otras bases.

`history.notes` ausente o NULL permite presentar los antecedentes anteriores
con sus etiquetas y detalles. Cuando el usuario escribe una nota se guarda ese
texto. Una cadena vacía expresa que borró la nota y evita volver a mostrar los
antecedentes anteriores como si siguieran capturados. Los valores históricos
permanecen almacenados.

El alta y la edición usan los mismos permisos y revisiones de la orden. La
edición registra el valor anterior y nuevo como `equipmentHistory` en la misma
transacción que la orden. Un fallo de auditoría revierte el cambio. Rust y
SQLite validan el límite del texto. Las credenciales del equipo siguen fuera
de la auditoría y de las notas impresas.
