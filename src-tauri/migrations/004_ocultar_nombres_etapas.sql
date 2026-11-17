CREATE TABLE nombres_etapas_ocultos (
    nombre TEXT PRIMARY KEY COLLATE NOCASE CHECK(length(trim(nombre)) BETWEEN 1 AND 100)
) STRICT;
