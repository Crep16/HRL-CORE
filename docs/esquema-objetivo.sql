-- ============================================================================
-- HRL CORE: ESQUEMA OBJETIVO NORMALIZADO (3FN)
-- ----------------------------------------------------------------------------
-- Reglas de diseño aplicadas respecto al esquema anterior:
--  1. Sin JSON como almacenamiento principal: cada dato tiene su tabla y columna.
--  2. Catálogos con FK en lugar de texto libre (estado, moneda, método de pago,
--     etapa, nivel de acceso, apartados y acciones).
--  3. PK interna (INTEGER) distinta del folio de negocio (ordenes.folio UNIQUE).
--  4. Una sola auditoría (auditoria + auditoria_detalle) escrita por la capa de
--     servicios dentro de la misma transacción; se eliminan los triggers que
--     copiaban entre bitácoras.
--  5. Ajustes y devoluciones son tablas separadas (antes una tabla con 'tipo').
--  6. Sin datos derivados almacenados (importe_esperado, nombres de autor
--     duplicados, id_orden redundante en ajustes). Los saldos se calculan en vistas.
--  7. Reintentos idempotentes con id_solicitud UNIQUE + huella_solicitud.
--  8. Los triggers que quedan solo protegen integridad (inmutabilidad, último
--     administrador, cierre único de accesos temporales).
-- Convención: tablas en plural, snake_case, español; códigos internos en inglés
-- solo donde ya forman parte del contrato con la app (ver catálogos).
-- Dinero siempre en centavos (INTEGER). Fechas en UTC ISO-8601 con sufijo Z.
-- ============================================================================

PRAGMA foreign_keys=ON;
BEGIN TRANSACTION;

-- ---------------------------------------------------------------------------
-- 1. CATÁLOGOS
-- ---------------------------------------------------------------------------
CREATE TABLE niveles_acceso (
    id     INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo TEXT NOT NULL UNIQUE,
    nombre TEXT NOT NULL
);

CREATE TABLE apartados (
    id     INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo TEXT NOT NULL UNIQUE,
    nombre TEXT NOT NULL
);

CREATE TABLE acciones (
    id     INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo TEXT NOT NULL UNIQUE,
    nombre TEXT NOT NULL
);

-- Combinaciones válidas apartado + acción. Reemplaza el CHECK compuesto anterior.
CREATE TABLE permisos (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    id_apartado INTEGER NOT NULL REFERENCES apartados(id) ON DELETE RESTRICT,
    id_accion   INTEGER NOT NULL REFERENCES acciones(id)  ON DELETE RESTRICT,
    UNIQUE(id_apartado, id_accion)
);

CREATE TABLE monedas (
    codigo TEXT PRIMARY KEY CHECK(length(codigo) = 3 AND codigo = upper(codigo)),
    nombre TEXT NOT NULL
);

CREATE TABLE estados_orden (
    id     INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo TEXT NOT NULL UNIQUE,
    nombre TEXT NOT NULL,
    orden  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE metodos_pago (
    id     INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(nombre)) > 0)
);

CREATE TABLE etapas (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre         TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(nombre)) > 0),
    es_predefinida INTEGER NOT NULL DEFAULT 0 CHECK(es_predefinida IN (0,1)),
    orden          INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE operaciones_auditoria (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo      TEXT NOT NULL UNIQUE,
    descripcion TEXT NOT NULL
);

-- ---------------------------------------------------------------------------
-- 2. SEGURIDAD
-- ---------------------------------------------------------------------------
CREATE TABLE clave_maestra (
    id   INTEGER PRIMARY KEY CHECK(id = 1),
    hash TEXT NOT NULL CHECK(length(hash) > 0)
);

CREATE TABLE usuarios (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre          TEXT NOT NULL CHECK(length(trim(nombre)) > 0),
    nombre_acceso   TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(nombre_acceso)) > 0),
    hash_contrasena TEXT NOT NULL CHECK(length(hash_contrasena) > 0),
    id_nivel        INTEGER NOT NULL REFERENCES niveles_acceso(id) ON DELETE RESTRICT,
    activo          INTEGER NOT NULL DEFAULT 1 CHECK(activo IN (0,1)),
    creado_en       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE usuario_permiso (
    id_usuario INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    id_permiso INTEGER NOT NULL REFERENCES permisos(id) ON DELETE RESTRICT,
    PRIMARY KEY(id_usuario, id_permiso)
);

-- Préstamo temporal de facultades. Los nombres se obtienen de usuarios (sin copias).
CREATE TABLE accesos_temporales (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    id_receptor         INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    id_autorizador      INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    id_apartado         INTEGER NOT NULL REFERENCES apartados(id) ON DELETE RESTRICT,
    id_orden            INTEGER REFERENCES ordenes(id) ON DELETE RESTRICT,
    motivo              TEXT NOT NULL CHECK(length(trim(motivo)) > 0),
    creado_en           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    finalizado_en       TEXT,
    motivo_finalizacion TEXT,
    CHECK(id_receptor <> id_autorizador)
);

-- Lista de operaciones prestadas (antes operaciones_json).
CREATE TABLE acceso_temporal_permiso (
    id_acceso  INTEGER NOT NULL REFERENCES accesos_temporales(id) ON DELETE RESTRICT,
    id_permiso INTEGER NOT NULL REFERENCES permisos(id) ON DELETE RESTRICT,
    PRIMARY KEY(id_acceso, id_permiso)
);

-- ---------------------------------------------------------------------------
-- 3. EMPRESA Y DOCUMENTOS (antes un solo JSON)
-- ---------------------------------------------------------------------------
CREATE TABLE empresa (
    id                       INTEGER PRIMARY KEY CHECK(id = 1),
    nombre                   TEXT NOT NULL DEFAULT '',
    rfc                      TEXT NOT NULL DEFAULT '',
    telefono                 TEXT NOT NULL DEFAULT '',
    whatsapp                 TEXT NOT NULL DEFAULT '',
    correo                   TEXT NOT NULL DEFAULT '',
    sitio_web                TEXT NOT NULL DEFAULT '',
    direccion                TEXT NOT NULL DEFAULT '',
    horario                  TEXT NOT NULL DEFAULT '',
    pais                     TEXT NOT NULL DEFAULT 'MX',
    locale                   TEXT NOT NULL DEFAULT 'es-MX',
    zona_horaria             TEXT NOT NULL DEFAULT 'America/Cancun',
    codigo_moneda            TEXT NOT NULL DEFAULT 'MXN' REFERENCES monedas(codigo) ON DELETE RESTRICT,
    mostrar_impuesto         INTEGER NOT NULL DEFAULT 0 CHECK(mostrar_impuesto IN (0,1)),
    tasa_impuesto_milesimas  INTEGER NOT NULL DEFAULT 0 CHECK(tasa_impuesto_milesimas BETWEEN 0 AND 100000),
    nombre_impuesto          TEXT NOT NULL DEFAULT 'IVA',
    dias_garantia_mano_obra  INTEGER NOT NULL DEFAULT 0 CHECK(dias_garantia_mano_obra >= 0),
    dias_garantia_refacciones INTEGER NOT NULL DEFAULT 0 CHECK(dias_garantia_refacciones >= 0),
    revision                 INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0)
);

CREATE TABLE configuracion_documentos (
    id                   INTEGER PRIMARY KEY CHECK(id = 1),
    logo_ruta            TEXT,   -- ruta relativa al almacenamiento de la app, nunca base64
    logo_nombre          TEXT,
    usar_logo_impresion  INTEGER NOT NULL DEFAULT 1 CHECK(usar_logo_impresion IN (0,1)),
    usar_logo_web        INTEGER NOT NULL DEFAULT 0 CHECK(usar_logo_web IN (0,1)),
    pie_entrada          TEXT NOT NULL DEFAULT '',
    pie_salida           TEXT NOT NULL DEFAULT '',
    imprimir_direccion   INTEGER NOT NULL DEFAULT 1 CHECK(imprimir_direccion IN (0,1)),
    firma                TEXT NOT NULL DEFAULT '',
    referencia_web       TEXT NOT NULL DEFAULT '',
    posicion_clausulas   TEXT NOT NULL DEFAULT 'end' CHECK(posicion_clausulas IN ('start','end')),
    numerar_clausulas    INTEGER NOT NULL DEFAULT 1 CHECK(numerar_clausulas IN (0,1)),
    papel                TEXT NOT NULL DEFAULT 'letter' CHECK(papel IN ('letter','ticket80','ticket60')),
    orientacion          TEXT NOT NULL DEFAULT 'portrait' CHECK(orientacion IN ('portrait','landscape')),
    copias               INTEGER NOT NULL DEFAULT 1 CHECK(copias BETWEEN 1 AND 10),
    revision             INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0)
);

CREATE TABLE clausulas (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    titulo         TEXT NOT NULL CHECK(length(trim(titulo)) > 0),
    texto          TEXT NOT NULL,
    aplica_entrada INTEGER NOT NULL DEFAULT 1 CHECK(aplica_entrada IN (0,1)),
    aplica_salida  INTEGER NOT NULL DEFAULT 0 CHECK(aplica_salida IN (0,1)),
    posicion       INTEGER NOT NULL DEFAULT 0
);

-- Generador de folios. Sustituye a nextNumber dentro del JSON de empresa.
CREATE TABLE secuencias_folio (
    prefijo          TEXT PRIMARY KEY,
    siguiente_numero INTEGER NOT NULL DEFAULT 1 CHECK(siguiente_numero > 0)
);

-- ---------------------------------------------------------------------------
-- 4. CLIENTES Y EQUIPOS
-- ---------------------------------------------------------------------------
CREATE TABLE clientes (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre    TEXT NOT NULL CHECK(length(trim(nombre)) > 0),
    telefono  TEXT NOT NULL DEFAULT '',
    creado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE equipos (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    id_cliente INTEGER NOT NULL REFERENCES clientes(id) ON DELETE RESTRICT,
    articulo   TEXT NOT NULL CHECK(length(trim(articulo)) > 0),
    marca      TEXT NOT NULL DEFAULT '',
    modelo     TEXT NOT NULL DEFAULT '',
    serie      TEXT NOT NULL DEFAULT ''
);

-- ---------------------------------------------------------------------------
-- 5. ÓRDENES
-- ---------------------------------------------------------------------------
CREATE TABLE ordenes (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    folio                 TEXT NOT NULL UNIQUE CHECK(length(trim(folio)) > 0),
    id_equipo             INTEGER NOT NULL REFERENCES equipos(id) ON DELETE RESTRICT,
    id_creador            INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    id_estado             INTEGER NOT NULL REFERENCES estados_orden(id) ON DELETE RESTRICT,
    codigo_moneda         TEXT NOT NULL DEFAULT 'MXN' REFERENCES monedas(codigo) ON DELETE RESTRICT,
    fecha_recepcion       TEXT NOT NULL,
    fecha_entrega         TEXT,
    falla                 TEXT NOT NULL DEFAULT '',
    trabajo               TEXT NOT NULL DEFAULT '',
    notas_recepcion       TEXT NOT NULL DEFAULT '',
    precio_centavos       INTEGER NOT NULL DEFAULT 0 CHECK(typeof(precio_centavos)='integer' AND precio_centavos >= 0),
    paso_otro_taller      INTEGER NOT NULL DEFAULT 0 CHECK(paso_otro_taller IN (0,1)),
    detalle_otro_taller   TEXT NOT NULL DEFAULT '',
    caida_humedad         INTEGER NOT NULL DEFAULT 0 CHECK(caida_humedad IN (0,1)),
    detalle_caida_humedad TEXT NOT NULL DEFAULT '',
    contrasena_equipo     TEXT NOT NULL DEFAULT '',  -- dato operativo; decidir cifrado o retiro (ver AGENTS.md)
    eliminada_en          TEXT,                      -- NULL = activa; fecha = en papelera
    creado_en             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    CHECK(paso_otro_taller = 1 OR detalle_otro_taller = ''),
    CHECK(caida_humedad = 1 OR detalle_caida_humedad = '')
);

CREATE TABLE seguimientos (
    id_orden     INTEGER PRIMARY KEY REFERENCES ordenes(id) ON DELETE RESTRICT,
    tecnico      TEXT NOT NULL DEFAULT '',
    nota_publica TEXT NOT NULL DEFAULT '',
    nota_interna TEXT NOT NULL DEFAULT '',
    revision     INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0)
);

-- Etapas marcadas como completadas por orden (antes "completed" dentro del JSON).
CREATE TABLE orden_etapas (
    id_orden      INTEGER NOT NULL REFERENCES ordenes(id) ON DELETE RESTRICT,
    id_etapa      INTEGER NOT NULL REFERENCES etapas(id)  ON DELETE RESTRICT,
    completada_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    PRIMARY KEY(id_orden, id_etapa)
);

-- Preferencias de publicación web por orden (antes "visibility" dentro del JSON).
CREATE TABLE seguimiento_publicacion (
    id_orden   INTEGER NOT NULL REFERENCES ordenes(id) ON DELETE RESTRICT,
    clave      TEXT NOT NULL CHECK(length(trim(clave)) > 0),
    habilitado INTEGER NOT NULL DEFAULT 0 CHECK(habilitado IN (0,1)),
    PRIMARY KEY(id_orden, clave)
);

CREATE TABLE avances (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    id_orden        INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor        INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    id_etapa        INTEGER NOT NULL REFERENCES etapas(id)   ON DELETE RESTRICT,
    titulo          TEXT NOT NULL CHECK(length(trim(titulo)) > 0),
    detalle         TEXT NOT NULL DEFAULT '',
    visible_cliente INTEGER NOT NULL DEFAULT 0 CHECK(visible_cliente IN (0,1)),
    fecha_evento    TEXT NOT NULL
);

CREATE TABLE notas (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    id_orden        INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor        INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    texto           TEXT NOT NULL CHECK(length(trim(texto)) > 0),
    visible_cliente INTEGER NOT NULL DEFAULT 0 CHECK(visible_cliente IN (0,1)),
    creado_en       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- Piezas utilizadas/cotizadas para la orden. No es inventario.
CREATE TABLE refacciones (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    id_orden        INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor        INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    nombre          TEXT NOT NULL CHECK(length(trim(nombre)) > 0),
    cantidad        INTEGER NOT NULL DEFAULT 1 CHECK(cantidad > 0),
    precio_centavos INTEGER NOT NULL CHECK(typeof(precio_centavos)='integer' AND precio_centavos >= 0)
);

CREATE TABLE garantias (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    id_orden  INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor  INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    cobertura TEXT NOT NULL CHECK(length(trim(cobertura)) > 0),
    dias      INTEGER NOT NULL CHECK(typeof(dias)='integer' AND dias >= 0)
);

-- Costo real pagado por el taller (distinto de refacciones: lo cotizado al cliente).
CREATE TABLE gastos_directos (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    id_orden         INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor         INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    descripcion      TEXT NOT NULL CHECK(length(trim(descripcion)) > 0),
    importe_centavos INTEGER NOT NULL CHECK(typeof(importe_centavos)='integer' AND importe_centavos > 0),
    motivo           TEXT NOT NULL CHECK(length(trim(motivo)) > 0),
    fecha_evento     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- ---------------------------------------------------------------------------
-- 6. DINERO (libro inmutable)
-- ---------------------------------------------------------------------------
CREATE TABLE abonos (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    id_orden          INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor          INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    id_metodo_pago    INTEGER REFERENCES metodos_pago(id) ON DELETE RESTRICT, -- NULL solo en importaciones históricas
    importe_centavos  INTEGER NOT NULL CHECK(typeof(importe_centavos)='integer' AND importe_centavos > 0),
    fecha_pago        TEXT,                -- fecha declarada; NULL si se desconoce (no cadena vacía)
    motivo            TEXT,
    id_solicitud      TEXT UNIQUE,         -- reintentos idempotentes
    huella_solicitud  TEXT,                -- hash de la solicitud para comparar reintentos
    creado_en         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- Corrección con signo de un abono original. La orden se obtiene vía abonos.
CREATE TABLE ajustes_abono (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    id_abono          INTEGER NOT NULL REFERENCES abonos(id)   ON DELETE RESTRICT,
    id_autor          INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    importe_centavos  INTEGER NOT NULL CHECK(typeof(importe_centavos)='integer' AND importe_centavos <> 0 AND abs(importe_centavos) <= 9000000000000),
    motivo            TEXT NOT NULL CHECK(length(trim(motivo)) > 0),
    id_solicitud      TEXT UNIQUE,
    huella_solicitud  TEXT,
    creado_en         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- Dinero devuelto al cliente (siempre positivo).
CREATE TABLE devoluciones (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    id_orden          INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor          INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    id_metodo_pago    INTEGER NOT NULL REFERENCES metodos_pago(id) ON DELETE RESTRICT,
    importe_centavos  INTEGER NOT NULL CHECK(typeof(importe_centavos)='integer' AND importe_centavos > 0 AND importe_centavos <= 9000000000000),
    motivo            TEXT NOT NULL CHECK(length(trim(motivo)) > 0),
    id_solicitud      TEXT UNIQUE,
    huella_solicitud  TEXT,
    creado_en         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- ---------------------------------------------------------------------------
-- 7. EVIDENCIAS (metadatos; los bytes viven en archivos)
-- ---------------------------------------------------------------------------
CREATE TABLE evidencias (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo_archivo      TEXT NOT NULL UNIQUE CHECK(length(codigo_archivo) = 32),
    id_orden            INTEGER NOT NULL REFERENCES ordenes(id)  ON DELETE RESTRICT,
    id_autor            INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    nombre_original     TEXT NOT NULL,
    tipo_imagen         TEXT NOT NULL CHECK(tipo_imagen IN ('image/jpeg','image/png','image/webp')),
    tamano_bytes        INTEGER NOT NULL CHECK(tamano_bytes > 0 AND tamano_bytes <= 20971520),
    origen              TEXT NOT NULL CHECK(origen IN ('camera','file')),
    descripcion         TEXT NOT NULL DEFAULT '' CHECK(length(descripcion) <= 500),
    descripcion_inicial TEXT NOT NULL DEFAULT '',
    retirada            INTEGER NOT NULL DEFAULT 0 CHECK(retirada IN (0,1)),
    revision            INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
    creado_en           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- ---------------------------------------------------------------------------
-- 8. AUDITORÍA ÚNICA (la escribe la capa de servicios en la misma transacción)
-- ---------------------------------------------------------------------------
CREATE TABLE auditoria (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    id_autor     INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    id_operacion INTEGER NOT NULL REFERENCES operaciones_auditoria(id) ON DELETE RESTRICT,
    id_orden     INTEGER REFERENCES ordenes(id) ON DELETE RESTRICT,  -- NULL si el evento es de sistema
    entidad      TEXT NOT NULL,   -- nombre de tabla afectada, p. ej. 'abonos', 'empresa'
    id_entidad   TEXT,            -- id del registro afectado (referencia polimórfica, sin FK)
    motivo       TEXT,
    resultado    TEXT NOT NULL DEFAULT 'success' CHECK(resultado IN ('success','denied','error')),
    creado_en    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE auditoria_detalle (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    id_auditoria   INTEGER NOT NULL REFERENCES auditoria(id) ON DELETE RESTRICT,
    campo          TEXT NOT NULL,
    valor_anterior TEXT,
    valor_nuevo    TEXT
);

-- ---------------------------------------------------------------------------
-- 9. ÍNDICES
-- ---------------------------------------------------------------------------
CREATE INDEX idx_usuarios_nivel          ON usuarios(id_nivel);
CREATE INDEX idx_equipos_cliente         ON equipos(id_cliente);
CREATE INDEX idx_equipos_serie           ON equipos(serie);
CREATE INDEX idx_ordenes_equipo          ON ordenes(id_equipo);
CREATE INDEX idx_ordenes_estado          ON ordenes(id_estado);
CREATE INDEX idx_ordenes_eliminada       ON ordenes(eliminada_en);
CREATE INDEX idx_avances_orden           ON avances(id_orden, fecha_evento);
CREATE INDEX idx_notas_orden             ON notas(id_orden);
CREATE INDEX idx_refacciones_orden       ON refacciones(id_orden);
CREATE INDEX idx_garantias_orden         ON garantias(id_orden);
CREATE INDEX idx_gastos_orden            ON gastos_directos(id_orden);
CREATE INDEX idx_abonos_orden            ON abonos(id_orden, creado_en);
CREATE INDEX idx_ajustes_abono           ON ajustes_abono(id_abono);
CREATE INDEX idx_devoluciones_orden      ON devoluciones(id_orden, creado_en);
CREATE INDEX idx_evidencias_orden        ON evidencias(id_orden, retirada, creado_en);
CREATE INDEX idx_auditoria_orden         ON auditoria(id_orden, id DESC);
CREATE INDEX idx_auditoria_autor         ON auditoria(id_autor, id DESC);
CREATE INDEX idx_auditoria_fecha         ON auditoria(creado_en DESC, id DESC);
CREATE INDEX idx_auditoria_detalle       ON auditoria_detalle(id_auditoria);
CREATE INDEX idx_accesos_receptor        ON accesos_temporales(id_receptor, finalizado_en);

-- ---------------------------------------------------------------------------
-- 10. VISTAS (se calculan al consultar; no duplican datos)
-- ---------------------------------------------------------------------------
CREATE VIEW vista_cobros_orden AS
SELECT o.id AS id_orden,
       o.folio,
       o.codigo_moneda,
       o.precio_centavos,
       coalesce((SELECT sum(a.importe_centavos) FROM abonos a WHERE a.id_orden = o.id), 0) AS cobrado_centavos,
       coalesce((SELECT sum(j.importe_centavos) FROM ajustes_abono j
                 JOIN abonos a ON a.id = j.id_abono WHERE a.id_orden = o.id), 0)           AS ajustado_centavos,
       coalesce((SELECT sum(d.importe_centavos) FROM devoluciones d WHERE d.id_orden = o.id), 0) AS devuelto_centavos
FROM ordenes o;

CREATE VIEW vista_saldos_orden AS
SELECT id_orden, folio, codigo_moneda, precio_centavos,
       cobrado_centavos, ajustado_centavos, devuelto_centavos,
       cobrado_centavos + ajustado_centavos - devuelto_centavos AS cobro_neto_centavos,
       max(precio_centavos - (cobrado_centavos + ajustado_centavos - devuelto_centavos), 0) AS pendiente_centavos,
       max((cobrado_centavos + ajustado_centavos - devuelto_centavos) - precio_centavos, 0) AS saldo_favor_centavos
FROM vista_cobros_orden;

-- ---------------------------------------------------------------------------
-- 11. TRIGGERS: solo integridad (la lógica de negocio vive en la capa de servicios)
-- ---------------------------------------------------------------------------
-- Inmutabilidad de libros y auditoría.
CREATE TRIGGER abonos_no_update            BEFORE UPDATE ON abonos            BEGIN SELECT RAISE(ABORT,'Los abonos son inmutables'); END;
CREATE TRIGGER abonos_no_delete            BEFORE DELETE ON abonos            BEGIN SELECT RAISE(ABORT,'Los abonos no se borran'); END;
CREATE TRIGGER ajustes_abono_no_update     BEFORE UPDATE ON ajustes_abono     BEGIN SELECT RAISE(ABORT,'Los ajustes son inmutables'); END;
CREATE TRIGGER ajustes_abono_no_delete     BEFORE DELETE ON ajustes_abono     BEGIN SELECT RAISE(ABORT,'Los ajustes no se borran'); END;
CREATE TRIGGER devoluciones_no_update      BEFORE UPDATE ON devoluciones      BEGIN SELECT RAISE(ABORT,'Las devoluciones son inmutables'); END;
CREATE TRIGGER devoluciones_no_delete      BEFORE DELETE ON devoluciones      BEGIN SELECT RAISE(ABORT,'Las devoluciones no se borran'); END;
CREATE TRIGGER gastos_directos_no_update   BEFORE UPDATE ON gastos_directos   BEGIN SELECT RAISE(ABORT,'Los gastos son inmutables'); END;
CREATE TRIGGER gastos_directos_no_delete   BEFORE DELETE ON gastos_directos   BEGIN SELECT RAISE(ABORT,'Los gastos no se borran'); END;
CREATE TRIGGER auditoria_no_update         BEFORE UPDATE ON auditoria         BEGIN SELECT RAISE(ABORT,'La auditoría es inmutable'); END;
CREATE TRIGGER auditoria_no_delete         BEFORE DELETE ON auditoria         BEGIN SELECT RAISE(ABORT,'La auditoría no se borra'); END;
CREATE TRIGGER auditoria_detalle_no_update BEFORE UPDATE ON auditoria_detalle BEGIN SELECT RAISE(ABORT,'La auditoría es inmutable'); END;
CREATE TRIGGER auditoria_detalle_no_delete BEFORE DELETE ON auditoria_detalle BEGIN SELECT RAISE(ABORT,'La auditoría no se borra'); END;
CREATE TRIGGER evidencias_no_delete        BEFORE DELETE ON evidencias        BEGIN SELECT RAISE(ABORT,'Las evidencias deben conservarse'); END;

-- Evidencias: solo descripción, retirada y revisión son editables.
CREATE TRIGGER evidencias_conservar_original BEFORE UPDATE ON evidencias
WHEN NEW.id IS NOT OLD.id OR NEW.codigo_archivo IS NOT OLD.codigo_archivo OR NEW.id_orden IS NOT OLD.id_orden
  OR NEW.id_autor IS NOT OLD.id_autor OR NEW.nombre_original IS NOT OLD.nombre_original
  OR NEW.tipo_imagen IS NOT OLD.tipo_imagen OR NEW.tamano_bytes IS NOT OLD.tamano_bytes
  OR NEW.origen IS NOT OLD.origen OR NEW.descripcion_inicial IS NOT OLD.descripcion_inicial
  OR NEW.creado_en IS NOT OLD.creado_en
BEGIN SELECT RAISE(ABORT,'La evidencia original no puede reemplazarse'); END;

-- Accesos temporales: solo se pueden cerrar, una vez.
CREATE TRIGGER accesos_temporales_no_delete BEFORE DELETE ON accesos_temporales
BEGIN SELECT RAISE(ABORT,'No se puede borrar el historial de accesos temporales'); END;

CREATE TRIGGER accesos_temporales_solo_cierre BEFORE UPDATE ON accesos_temporales
WHEN OLD.finalizado_en IS NOT NULL OR NEW.finalizado_en IS NULL OR NEW.id IS NOT OLD.id
  OR NEW.id_receptor IS NOT OLD.id_receptor OR NEW.id_autorizador IS NOT OLD.id_autorizador
  OR NEW.id_apartado IS NOT OLD.id_apartado OR NEW.id_orden IS NOT OLD.id_orden
  OR NEW.motivo IS NOT OLD.motivo OR NEW.creado_en IS NOT OLD.creado_en
BEGIN SELECT RAISE(ABORT,'El acceso temporal solo puede finalizarse una vez'); END;

-- Usuarios: el primer usuario debe ser administrador activo y siempre debe quedar uno.
CREATE TRIGGER usuarios_primer_administrador BEFORE INSERT ON usuarios
WHEN NOT EXISTS (SELECT 1 FROM usuarios)
 AND (NEW.activo <> 1 OR NEW.id_nivel <> (SELECT id FROM niveles_acceso WHERE codigo='complete'))
BEGIN SELECT RAISE(ABORT,'El primer usuario debe tener acceso completo'); END;

CREATE TRIGGER usuarios_conservar_admin_al_borrar BEFORE DELETE ON usuarios
WHEN OLD.activo = 1 AND OLD.id_nivel = (SELECT id FROM niveles_acceso WHERE codigo='complete')
 AND NOT EXISTS (SELECT 1 FROM usuarios WHERE id <> OLD.id AND activo = 1
                 AND id_nivel = (SELECT id FROM niveles_acceso WHERE codigo='complete'))
BEGIN SELECT RAISE(ABORT,'Debe conservarse un usuario activo con acceso completo'); END;

CREATE TRIGGER usuarios_conservar_admin_al_editar BEFORE UPDATE ON usuarios
WHEN OLD.activo = 1 AND OLD.id_nivel = (SELECT id FROM niveles_acceso WHERE codigo='complete')
 AND (NEW.activo <> 1 OR NEW.id_nivel <> OLD.id_nivel)
 AND NOT EXISTS (SELECT 1 FROM usuarios WHERE id <> OLD.id AND activo = 1
                 AND id_nivel = (SELECT id FROM niveles_acceso WHERE codigo='complete'))
BEGIN SELECT RAISE(ABORT,'Debe conservarse un usuario activo con acceso completo'); END;

-- ---------------------------------------------------------------------------
-- 12. DATOS DE CATÁLOGO (no son datos de clientes)
-- Ajustar estados, etapas y operaciones a los valores reales de orders.rs.
-- ---------------------------------------------------------------------------
INSERT INTO niveles_acceso(codigo,nombre) VALUES ('complete','Completo'),('limited','Limitado'),('custom','Personalizado');

INSERT INTO apartados(codigo,nombre) VALUES
 ('clients','Clientes'),('repairs','Reparaciones'),('logbook','Bitácora'),
 ('company','Empresa'),('payments','Pagos'),('users','Usuarios');

INSERT INTO acciones(codigo,nombre) VALUES
 ('view','Consultar'),('create','Crear'),('edit','Editar'),('delete','Eliminar'),
 ('add','Registrar'),('refund','Devolver');

INSERT INTO permisos(id_apartado,id_accion)
SELECT a.id, c.id FROM apartados a JOIN acciones c
 ON (a.codigo IN ('clients','repairs','logbook','company') AND c.codigo IN ('view','create','edit','delete'))
 OR (a.codigo = 'payments' AND c.codigo IN ('add','refund'));

-- El catálogo de permisos es de solo lectura tras el sembrado: sin estos triggers
-- se podría insertar una combinación inválida (p. ej. payments + view), algo que el
-- CHECK compuesto del esquema anterior sí impedía. Un permiso nuevo = nueva migración.
CREATE TRIGGER permisos_no_insert BEFORE INSERT ON permisos BEGIN SELECT RAISE(ABORT,'El catálogo de permisos solo cambia mediante migración'); END;
CREATE TRIGGER permisos_no_update BEFORE UPDATE ON permisos BEGIN SELECT RAISE(ABORT,'El catálogo de permisos solo cambia mediante migración'); END;
CREATE TRIGGER permisos_no_delete BEFORE DELETE ON permisos BEGIN SELECT RAISE(ABORT,'El catálogo de permisos solo cambia mediante migración'); END;

INSERT INTO monedas(codigo,nombre) VALUES ('MXN','Peso mexicano'),('USD','Dólar estadounidense');

INSERT INTO estados_orden(codigo,nombre,orden) VALUES
 ('received','Recibido',1),('diagnosis','En diagnóstico',2),('repair','En reparación',3),
 ('ready','Listo',4),('irreparable','Irreparable',5),('cancelled','Cancelado',6);

INSERT INTO etapas(nombre,es_predefinida,orden) VALUES
 ('Recepción',1,1),('Diagnóstico',1,2),('Reparación',1,3),('Pruebas',1,4),('Entrega',1,5);

INSERT INTO operaciones_auditoria(codigo,descripcion) VALUES
 ('orders.create','Alta de orden'),('orders.edit','Edición de orden'),
 ('orders.delete','Envío a papelera'),('orders.restore','Restauración desde papelera'),
 ('repairs.save','Guardado de seguimiento'),
 ('payments.add','Abono registrado'),('payments.adjust','Corrección de abono'),('payments.refund','Devolución'),
 ('costs.add','Gasto directo'),
 ('evidence.add','Evidencia agregada'),('evidence.edit','Evidencia editada'),
 ('evidence.remove','Evidencia retirada'),('evidence.restore','Evidencia restaurada'),
 ('company.edit','Cambio de configuración de empresa'),
 ('users.create','Alta de usuario'),('users.edit','Edición de usuario'),
 ('access.temporary.start','Inicio de acceso temporal'),('access.temporary.end','Fin de acceso temporal');

COMMIT;
