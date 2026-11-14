-- NULL conserva la lectura de los antecedentes estructurados anteriores.
-- Una cadena vacía representa una nota borrada expresamente por el usuario.
ALTER TABLE ordenes ADD COLUMN antecedentes_equipo TEXT
    CHECK(antecedentes_equipo IS NULL OR length(antecedentes_equipo) <= 16000);
