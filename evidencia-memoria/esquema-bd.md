# Esquema SQLite

**[EVIDENCIA] Fuente:** `src-tauri/migrations/001_esquema_objetivo.sql`. Cambios posteriores: migraciones 002–006, indicadas por tabla/campo. Tipos SQLite se presentan como declarados; las restricciones completas se consultan en los SQL. La base vigente tiene migraciones numeradas 001–006; `legacy/` está separado y no corresponde a la línea vigente. Cada celda de fuente identifica la migración donde se define la tabla o columna.

## Usuarios, roles y permisos

| Tabla | Columnas (tipo) | Fuente |
|---|---|---|
| `usuarios` | `id INTEGER`, `nombre TEXT`, `nombre_acceso TEXT`, `hash_contrasena TEXT`, `id_nivel INTEGER`, `id_rol INTEGER`, `activo INTEGER`, `creado_en TEXT` | 001 |
| `roles` | `id INTEGER`, `codigo TEXT`, `nombre TEXT` | 001 |
| `niveles_acceso` | `id INTEGER`, `codigo TEXT`, `nombre TEXT` | 001 |
| `apartados` | `id INTEGER`, `codigo TEXT`, `nombre TEXT` | 001 |
| `acciones` | `id INTEGER`, `codigo TEXT`, `nombre TEXT` | 001 |
| `permisos` | `id INTEGER`, `id_apartado INTEGER`, `id_accion INTEGER`, `solo_completo INTEGER` | 001 |
| `usuario_permiso` | `id_usuario INTEGER`, `id_permiso INTEGER` | 001 |
| `claves_maestras` | `id INTEGER`, `hash TEXT` | 001 |
| `operaciones_auditoria` | `id INTEGER`, `codigo TEXT`, `descripcion TEXT` | 001 |

## Autorizaciones temporales

| Tabla | Columnas (tipo) | Fuente |
|---|---|---|
| `accesos_temporales` | `id INTEGER`, `id_receptor INTEGER`, `id_autorizador INTEGER`, `id_apartado INTEGER`, `id_orden INTEGER NULL`, `motivo TEXT`, `creado_en TEXT`, `finalizado_en TEXT NULL`, `motivo_finalizacion TEXT NULL` | 001 |
| `acceso_temporal_permiso` | `id_acceso INTEGER`, `id_permiso INTEGER` | 001 |

## Configuración de empresa

| Tabla | Columnas (tipo) | Fuente |
|---|---|---|
| `empresa` | `id INTEGER`, `nombre TEXT`, `rfc TEXT`, `telefono TEXT`, `whatsapp TEXT`, `correo TEXT`, `sitio_web TEXT`, `direccion TEXT`, `horario TEXT`, `pais TEXT`, `locale TEXT`, `zona_horaria TEXT`, `codigo_moneda TEXT`, `mostrar_impuesto INTEGER`, `tasa_impuesto_milesimas INTEGER`, `nombre_impuesto TEXT`, `dias_garantia_mano_obra INTEGER`, `dias_garantia_refacciones INTEGER`, `revision INTEGER` | 001 |
| `configuracion_documentos` | `id INTEGER`, `logo_ruta TEXT NULL`, `logo_nombre TEXT NULL`, `usar_logo_impresion INTEGER`, `usar_logo_web INTEGER`, `pie_entrada TEXT`, `pie_salida TEXT`, `imprimir_direccion INTEGER`, `pedir_firma INTEGER`, `referencia_web INTEGER`, `posicion_clausulas TEXT`, `numerar_clausulas INTEGER`, `papel TEXT`, `orientacion TEXT`, `copias INTEGER`, `revision INTEGER`, `prefijo_folio TEXT` | 001; `prefijo_folio` se agrega en 002 |
| `clausulas` | `id INTEGER`, `titulo TEXT`, `texto TEXT`, `aplica_entrada INTEGER`, `aplica_salida INTEGER`, `posicion INTEGER` | 001 |
| `secuencias_folio` | `prefijo TEXT`, `siguiente_numero INTEGER` | 001 |
| `monedas` | `codigo TEXT`, `nombre TEXT` | 001 |

## Órdenes y reparaciones

| Tabla | Columnas (tipo) | Fuente |
|---|---|---|
| `estados_orden` | `id INTEGER`, `codigo TEXT`, `nombre TEXT`, `orden INTEGER` | 001 |
| `etapas` | `id INTEGER`, `nombre TEXT`, `es_predefinida INTEGER`, `orden INTEGER` | 001 |
| `clientes` | `id INTEGER`, `nombre TEXT`, `telefono TEXT`, `correo TEXT NULL`, `creado_en TEXT` | 001 |
| `equipos` | `id INTEGER`, `id_cliente INTEGER`, `articulo TEXT`, `marca TEXT`, `modelo TEXT`, `serie TEXT` | 001 |
| `ordenes` | `id INTEGER`, `folio TEXT`, `id_equipo INTEGER`, `id_creador INTEGER`, `id_estado INTEGER NULL`, `estado_personalizado TEXT NULL`, `codigo_moneda TEXT`, `fecha_recepcion TEXT`, `fecha_estimada_entrega TEXT NULL`, `fecha_entrega TEXT NULL`, `entregado INTEGER`, `falla TEXT`, `trabajo TEXT`, `notas_recepcion TEXT`, `precio_centavos INTEGER`, `paso_otro_taller INTEGER`, `detalle_otro_taller TEXT`, `sufrio_golpes INTEGER`, `detalle_golpes TEXT`, `derrame_liquidos INTEGER`, `detalle_liquidos TEXT`, `revision INTEGER`, `eliminada_en TEXT NULL`, `creado_en TEXT`, `antecedentes_equipo TEXT NULL` | 001; `antecedentes_equipo` 003 |
| `credenciales_dispositivos` | `id_orden INTEGER`, `version INTEGER`, `id_clave TEXT`, `algoritmo TEXT`, `nonce BLOB`, `cifrado BLOB` | 001 |
| `diagnosticos` | `id_orden INTEGER`, `id_autor INTEGER`, `texto TEXT`, `revision INTEGER`, `actualizado_en TEXT` | 001 |
| `seguimientos` | `id_orden INTEGER`, `tecnico TEXT`, `nota_publica TEXT`, `nota_interna TEXT`, `modo_publicacion TEXT`, `revision INTEGER` | 001 |
| `orden_etapas` | `id INTEGER`, `id_orden INTEGER`, `id_etapa INTEGER NULL`, `nombre_personalizado TEXT NULL`, `posicion INTEGER`, `completada_en TEXT NULL`, `retirada_en TEXT NULL`, `completada_previamente INTEGER` | 001; últimos dos campos 005 |
| `nombres_etapas_ocultos` | `nombre TEXT` | 004 |
| `seguimiento_publicacion` | `id_orden INTEGER`, `clave TEXT`, `habilitado INTEGER` | 001 |
| `avances` | `id INTEGER`, `id_etapa_orden INTEGER`, `id_autor INTEGER`, `titulo TEXT`, `detalle TEXT`, `visible_cliente INTEGER`, `fecha_evento TEXT` | 001 |
| `notas` | `id INTEGER`, `id_orden INTEGER`, `id_autor INTEGER`, `texto TEXT`, `visible_cliente INTEGER`, `creado_en TEXT` | 001 |
| `refacciones` | `id INTEGER`, `id_orden INTEGER`, `id_autor INTEGER`, `nombre TEXT`, `cantidad INTEGER`, `precio_centavos INTEGER` | 001 |
| `garantias` | `id INTEGER`, `id_orden INTEGER`, `id_autor INTEGER`, `cobertura TEXT`, `dias INTEGER` | 001 |
| `evidencias` | `id INTEGER`, `codigo_archivo TEXT`, `id_orden INTEGER`, `id_autor INTEGER`, `nombre_original TEXT`, `ruta_relativa TEXT`, `tipo_archivo TEXT`, `mime TEXT`, `tamano_bytes INTEGER`, `origen TEXT`, `descripcion TEXT`, `descripcion_inicial TEXT`, `retirada INTEGER`, `revision INTEGER`, `creado_en TEXT` | 001 |

## Libros, auditoría y gastos (también presentes en esquema)

`gastos_directos(id INTEGER, id_orden INTEGER, id_autor INTEGER, descripcion TEXT, importe_centavos INTEGER, motivo TEXT, fecha_evento TEXT)`; `solicitudes_financieras(id TEXT, id_autor INTEGER, tipo TEXT, huella_solicitud BLOB, creado_en TEXT)`; `abonos(id INTEGER, id_orden INTEGER, id_autor INTEGER, metodo_pago TEXT NULL, importe_centavos INTEGER, fecha_pago TEXT NULL, motivo TEXT NULL, id_solicitud TEXT NULL, creado_en TEXT)`; `ajustes_abono(id INTEGER, id_abono INTEGER, id_autor INTEGER, importe_centavos INTEGER, motivo TEXT, id_solicitud TEXT NULL, creado_en TEXT)`; `devoluciones(id INTEGER, id_orden INTEGER, id_autor INTEGER, metodo_pago TEXT, importe_centavos INTEGER, motivo TEXT, id_solicitud TEXT NULL, creado_en TEXT)`; `auditoria(id INTEGER, id_autor INTEGER, id_operacion INTEGER, id_orden INTEGER NULL, entidad TEXT, id_entidad TEXT NULL, motivo TEXT NULL, resultado TEXT, creado_en TEXT)`; `auditoria_detalle(id INTEGER, id_auditoria INTEGER, campo TEXT, valor_anterior TEXT NULL, valor_nuevo TEXT NULL)`. Fuente de todas: 001.

## Las ocho tablas del portal

| Tabla | Columnas (tipo) | Fuente |
|---|---|---|
| `portal_configuracion` | `id INTEGER`, `proyecto TEXT`, `clave_api TEXT`, `uid_emisor TEXT`, `url_portal TEXT`, `pausado INTEGER` | 006 |
| `portal_ordenes` | `id_orden INTEGER`, `revision INTEGER`, `habilitado INTEGER`, `entrega_instante TEXT NULL`, `error_publicacion TEXT NULL` | 006 |
| `portal_codigos` | `codigo TEXT`, `id_orden INTEGER`, `activo INTEGER` | 006 |
| `portal_envios` | `id_publicacion INTEGER`, `estado TEXT`, `intentos INTEGER`, `siguiente_intento INTEGER`, `error_envio TEXT NULL`, `confirmado_en TEXT NULL` | 006 |
| `portal_publicaciones` | `id INTEGER`, `id_orden INTEGER`, `codigo TEXT`, `revision INTEGER`, `publicado INTEGER`, `huella TEXT`, `tipo TEXT NULL`, `marca TEXT NULL`, `modelo TEXT NULL`, `estado_tecnico TEXT NULL`, `recibido_en TEXT NULL`, `entrega_estimada TEXT NULL`, `entregado_en TEXT NULL`, `acceso_hasta TEXT NULL`, `nota_publica TEXT NULL`, `eventos_visibles INTEGER`, `garantias_visibles INTEGER`, `moneda TEXT NULL`, `precio_centavos INTEGER NULL`, `ultimo_abono INTEGER`, `ultimo_ajuste INTEGER`, `ultima_devolucion INTEGER` | 006 |
| `portal_publicacion_etapas` | `id_publicacion INTEGER`, `posicion INTEGER`, `nombre TEXT`, `completada INTEGER`, `completada_en TEXT NULL` | 006 |
| `portal_publicacion_eventos` | `id_publicacion INTEGER`, `posicion INTEGER`, `fecha TEXT`, `titulo TEXT`, `detalle TEXT` | 006 |
| `portal_publicacion_garantias` | `id_publicacion INTEGER`, `posicion INTEGER`, `cobertura TEXT`, `dias INTEGER` | 006 |

**[EVIDENCIA]** La vista `vista_saldos_orden` se crea en 001; no es tabla almacenada. El esquema impone SQLite `STRICT`. Para reproducir todos los tipos, claves foráneas, defaults y CHECK: abrir las migraciones citadas.
