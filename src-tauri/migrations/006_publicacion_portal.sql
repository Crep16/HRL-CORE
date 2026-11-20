CREATE TABLE portal_configuracion (
 id INTEGER PRIMARY KEY CHECK(id=1), proyecto TEXT NOT NULL DEFAULT '', clave_api TEXT NOT NULL DEFAULT '',
 uid_emisor TEXT NOT NULL DEFAULT '', url_portal TEXT NOT NULL DEFAULT '', pausado INTEGER NOT NULL DEFAULT 1 CHECK(pausado IN(0,1))
);
INSERT INTO portal_configuracion(id) VALUES(1);
CREATE TABLE portal_ordenes (
 id_orden INTEGER PRIMARY KEY REFERENCES ordenes(id), revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0),
 habilitado INTEGER NOT NULL DEFAULT 1 CHECK(habilitado IN(0,1)), entrega_instante TEXT, error_publicacion TEXT
);
CREATE TABLE portal_codigos (
 codigo TEXT PRIMARY KEY, id_orden INTEGER NOT NULL REFERENCES ordenes(id), activo INTEGER NOT NULL CHECK(activo IN(0,1))
);
CREATE UNIQUE INDEX portal_codigo_activo ON portal_codigos(id_orden) WHERE activo=1;
CREATE TABLE portal_publicaciones (
 id INTEGER PRIMARY KEY, id_orden INTEGER NOT NULL REFERENCES ordenes(id), codigo TEXT NOT NULL REFERENCES portal_codigos(codigo),
 revision INTEGER NOT NULL CHECK(revision>0), publicado INTEGER NOT NULL CHECK(publicado IN(0,1)), huella TEXT NOT NULL,
 tipo TEXT, marca TEXT, modelo TEXT, estado_tecnico TEXT, recibido_en TEXT, entrega_estimada TEXT,
 entregado_en TEXT, acceso_hasta TEXT, nota_publica TEXT,
 eventos_visibles INTEGER NOT NULL CHECK(eventos_visibles IN(0,1)), garantias_visibles INTEGER NOT NULL CHECK(garantias_visibles IN(0,1)),
 moneda TEXT, precio_centavos INTEGER, ultimo_abono INTEGER NOT NULL, ultimo_ajuste INTEGER NOT NULL, ultima_devolucion INTEGER NOT NULL,
 UNIQUE(id_orden,revision)
);
CREATE TABLE portal_publicacion_etapas (
 id_publicacion INTEGER NOT NULL REFERENCES portal_publicaciones(id), posicion INTEGER NOT NULL,
 nombre TEXT NOT NULL, completada INTEGER NOT NULL CHECK(completada IN(0,1)), completada_en TEXT,
 PRIMARY KEY(id_publicacion,posicion)
);
CREATE TABLE portal_publicacion_eventos (
 id_publicacion INTEGER NOT NULL REFERENCES portal_publicaciones(id), posicion INTEGER NOT NULL,
 fecha TEXT NOT NULL, titulo TEXT NOT NULL, detalle TEXT NOT NULL, PRIMARY KEY(id_publicacion,posicion)
);
CREATE TABLE portal_publicacion_garantias (
 id_publicacion INTEGER NOT NULL REFERENCES portal_publicaciones(id), posicion INTEGER NOT NULL,
 cobertura TEXT NOT NULL, dias INTEGER NOT NULL CHECK(dias>=0), PRIMARY KEY(id_publicacion,posicion)
);
CREATE TABLE portal_envios (
 id_publicacion INTEGER PRIMARY KEY REFERENCES portal_publicaciones(id),
 estado TEXT NOT NULL DEFAULT 'pending' CHECK(estado IN('pending','sending','confirmed','superseded','error','conflict')),
 intentos INTEGER NOT NULL DEFAULT 0, siguiente_intento INTEGER NOT NULL DEFAULT 0, error_envio TEXT, confirmado_en TEXT
);
CREATE INDEX portal_envios_pendientes ON portal_envios(estado,siguiente_intento,id_publicacion);
CREATE TRIGGER portal_etapas_selladas BEFORE INSERT ON portal_publicacion_etapas WHEN EXISTS(SELECT 1 FROM portal_envios WHERE id_publicacion=NEW.id_publicacion) BEGIN SELECT RAISE(ABORT,'Publicación sellada'); END;
CREATE TRIGGER portal_eventos_sellados BEFORE INSERT ON portal_publicacion_eventos WHEN EXISTS(SELECT 1 FROM portal_envios WHERE id_publicacion=NEW.id_publicacion) BEGIN SELECT RAISE(ABORT,'Publicación sellada'); END;
CREATE TRIGGER portal_garantias_selladas BEFORE INSERT ON portal_publicacion_garantias WHEN EXISTS(SELECT 1 FROM portal_envios WHERE id_publicacion=NEW.id_publicacion) BEGIN SELECT RAISE(ABORT,'Publicación sellada'); END;
CREATE TRIGGER portal_publicaciones_inmutables_update BEFORE UPDATE ON portal_publicaciones BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
CREATE TRIGGER portal_publicaciones_inmutables_delete BEFORE DELETE ON portal_publicaciones BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
CREATE TRIGGER portal_etapas_inmutables_update BEFORE UPDATE ON portal_publicacion_etapas BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
CREATE TRIGGER portal_etapas_inmutables_delete BEFORE DELETE ON portal_publicacion_etapas BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
CREATE TRIGGER portal_eventos_inmutables_update BEFORE UPDATE ON portal_publicacion_eventos BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
CREATE TRIGGER portal_eventos_inmutables_delete BEFORE DELETE ON portal_publicacion_eventos BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
CREATE TRIGGER portal_garantias_inmutables_update BEFORE UPDATE ON portal_publicacion_garantias BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
CREATE TRIGGER portal_garantias_inmutables_delete BEFORE DELETE ON portal_publicacion_garantias BEGIN SELECT RAISE(ABORT,'Publicación inmutable'); END;
