ALTER TABLE orden_etapas ADD COLUMN retirada_en TEXT;
ALTER TABLE orden_etapas ADD COLUMN completada_previamente INTEGER NOT NULL DEFAULT 0 CHECK(completada_previamente IN (0,1));
UPDATE orden_etapas SET completada_previamente=1 WHERE completada_en IS NOT NULL;
