-- Nombres de tablas y columnas aprobados por el usuario.
-- SQLx ejecuta esta migracion dentro de su transaccion.
-- No modifica filas, identificadores, codigos internos ni contenido JSON.
-- Retira los objetos derivados para recrearlos con los nombres aprobados.

DROP INDEX audit_events_author;
DROP INDEX audit_events_order;
DROP INDEX equipment_client;
DROP INDEX equipment_serial;
DROP INDEX logbook_author;
DROP INDEX logbook_date;
DROP INDEX logbook_order;
DROP INDEX order_parts_order;
DROP INDEX order_visibility_order;
DROP INDEX orders_equipment;
DROP INDEX payment_operations_order;
DROP INDEX payment_operations_payment;
DROP INDEX payments_order;
DROP INDEX repair_changes_order;
DROP INDEX repair_events_order;
DROP INDEX repair_notes_order;
DROP INDEX users_login_name;
DROP INDEX warranties_order;
DROP TRIGGER audit_events_no_delete;
DROP TRIGGER audit_events_no_update;
DROP TRIGGER direct_expense_audit;
DROP TRIGGER direct_expenses_no_delete;
DROP TRIGGER direct_expenses_no_update;
DROP TRIGGER first_user_must_be_complete;
DROP TRIGGER logbook_no_delete;
DROP TRIGGER logbook_no_update;
DROP TRIGGER logbook_order_insert;
DROP TRIGGER logbook_payment_insert;
DROP TRIGGER logbook_repair_insert;
DROP TRIGGER logbook_visibility_insert;
DROP TRIGGER order_visibility_no_delete;
DROP TRIGGER order_visibility_no_update;
DROP TRIGGER payment_operations_audit;
DROP TRIGGER payment_operations_no_delete;
DROP TRIGGER payment_operations_no_update;
DROP TRIGGER payment_requests_no_delete;
DROP TRIGGER payment_requests_no_update;
DROP TRIGGER payments_no_delete;
DROP TRIGGER payments_no_update;
DROP TRIGGER preserve_last_complete_user_delete;
DROP TRIGGER preserve_last_complete_user_update;
DROP TRIGGER repair_changes_no_delete;
DROP TRIGGER repair_changes_no_update;
DROP TRIGGER system_logbook_no_delete;
DROP TRIGGER system_logbook_no_update;
DROP TRIGGER temporary_access_end;
DROP TRIGGER temporary_access_no_delete;
DROP TRIGGER temporary_access_restrict_update;
DROP TRIGGER temporary_access_start;
DROP VIEW logbook_all;
DROP VIEW order_payment_totals;
DROP INDEX order_evidence_order;
DROP TRIGGER evidence_no_delete;
DROP TRIGGER evidence_preserve_original;

-- access_settings -> Configuracion_acceso
ALTER TABLE access_settings RENAME COLUMN master_hash TO hash_clave_maestra;
ALTER TABLE access_settings RENAME TO Configuracion_acceso;

-- audit_events -> Historial_ordenes
ALTER TABLE audit_events RENAME COLUMN order_id TO id_orden;
ALTER TABLE audit_events RENAME COLUMN author_id TO id_autor;
ALTER TABLE audit_events RENAME COLUMN field TO campo;
ALTER TABLE audit_events RENAME COLUMN old_value TO valor_anterior;
ALTER TABLE audit_events RENAME COLUMN new_value TO valor_nuevo;
ALTER TABLE audit_events RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE audit_events RENAME TO Historial_ordenes;

-- clients -> Clientes
ALTER TABLE clients RENAME COLUMN name TO nombre;
ALTER TABLE clients RENAME COLUMN phone TO telefono;
ALTER TABLE clients RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE clients RENAME TO Clientes;

-- company_settings -> Configuracion_empresa
ALTER TABLE company_settings RENAME COLUMN record_json TO datos_json;
ALTER TABLE company_settings RENAME TO Configuracion_empresa;

-- direct_expenses -> Gastos_orden
ALTER TABLE direct_expenses RENAME COLUMN order_id TO id_orden;
ALTER TABLE direct_expenses RENAME COLUMN author_id TO id_autor;
ALTER TABLE direct_expenses RENAME COLUMN description TO descripcion;
ALTER TABLE direct_expenses RENAME COLUMN amount_minor TO importe_centavos;
ALTER TABLE direct_expenses RENAME COLUMN reason TO motivo;
ALTER TABLE direct_expenses RENAME COLUMN occurred_at TO fecha_evento;
ALTER TABLE direct_expenses RENAME TO Gastos_orden;

-- equipment -> Equipo
ALTER TABLE equipment RENAME COLUMN client_id TO id_cliente;
ALTER TABLE equipment RENAME COLUMN item TO articulo;
ALTER TABLE equipment RENAME COLUMN brand TO marca;
ALTER TABLE equipment RENAME COLUMN model TO modelo;
ALTER TABLE equipment RENAME COLUMN serial TO serie;
ALTER TABLE equipment RENAME TO Equipo;

-- legacy_imports -> Importaciones_anteriores
ALTER TABLE legacy_imports RENAME COLUMN source TO origen;
ALTER TABLE legacy_imports RENAME COLUMN imported_by TO id_importador;
ALTER TABLE legacy_imports RENAME COLUMN imported_at TO fecha_importacion;
ALTER TABLE legacy_imports RENAME TO Importaciones_anteriores;

-- logbook_events -> Bitacora_ordenes
ALTER TABLE logbook_events RENAME COLUMN source TO origen;
ALTER TABLE logbook_events RENAME COLUMN source_id TO id_origen;
ALTER TABLE logbook_events RENAME COLUMN order_id TO id_orden;
ALTER TABLE logbook_events RENAME COLUMN author_id TO id_autor;
ALTER TABLE logbook_events RENAME COLUMN author_name TO nombre_autor;
ALTER TABLE logbook_events RENAME COLUMN operation TO operacion;
ALTER TABLE logbook_events RENAME COLUMN field TO campo;
ALTER TABLE logbook_events RENAME COLUMN old_value TO valor_anterior;
ALTER TABLE logbook_events RENAME COLUMN new_value TO valor_nuevo;
ALTER TABLE logbook_events RENAME COLUMN details_json TO detalles_json;
ALTER TABLE logbook_events RENAME COLUMN occurred_at TO fecha_evento;
ALTER TABLE logbook_events RENAME COLUMN event_date TO fecha_declarada;
ALTER TABLE logbook_events RENAME COLUMN reason TO motivo;
ALTER TABLE logbook_events RENAME COLUMN result TO resultado;
ALTER TABLE logbook_events RENAME TO Bitacora_ordenes;

-- order_parts -> Refacciones_orden
ALTER TABLE order_parts RENAME COLUMN order_id TO id_orden;
ALTER TABLE order_parts RENAME COLUMN author_id TO id_autor;
ALTER TABLE order_parts RENAME COLUMN name TO nombre;
ALTER TABLE order_parts RENAME COLUMN price_minor TO precio_centavos;
ALTER TABLE order_parts RENAME TO Refacciones_orden;

-- order_visibility_events -> Historial_papelera
ALTER TABLE order_visibility_events RENAME COLUMN sequence TO secuencia;
ALTER TABLE order_visibility_events RENAME COLUMN order_id TO id_orden;
ALTER TABLE order_visibility_events RENAME COLUMN author_id TO id_autor;
ALTER TABLE order_visibility_events RENAME COLUMN action TO accion;
ALTER TABLE order_visibility_events RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE order_visibility_events RENAME TO Historial_papelera;

-- orders -> Ordenes
ALTER TABLE orders RENAME COLUMN equipment_id TO id_equipo;
ALTER TABLE orders RENAME COLUMN created_by TO id_creador;
ALTER TABLE orders RENAME COLUMN received TO fecha_recepcion;
ALTER TABLE orders RENAME COLUMN status TO estado;
ALTER TABLE orders RENAME COLUMN failure TO falla;
ALTER TABLE orders RENAME COLUMN work TO trabajo;
ALTER TABLE orders RENAME COLUMN price_minor TO precio_centavos;
ALTER TABLE orders RENAME COLUMN currency TO moneda;
ALTER TABLE orders RENAME COLUMN other_shop TO paso_otro_taller;
ALTER TABLE orders RENAME COLUMN other_shop_reason TO detalle_otro_taller;
ALTER TABLE orders RENAME COLUMN dropped TO caida_humedad;
ALTER TABLE orders RENAME COLUMN drop_reason TO detalle_caida_humedad;
ALTER TABLE orders RENAME COLUMN delivered_at TO fecha_entrega;
ALTER TABLE orders RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE orders RENAME COLUMN record_json TO datos_json;
ALTER TABLE orders RENAME TO Ordenes;

-- payment_operations -> Movimientos_abonos
ALTER TABLE payment_operations RENAME COLUMN order_id TO id_orden;
ALTER TABLE payment_operations RENAME COLUMN author_id TO id_autor;
ALTER TABLE payment_operations RENAME COLUMN kind TO tipo;
ALTER TABLE payment_operations RENAME COLUMN payment_id TO id_abono;
ALTER TABLE payment_operations RENAME COLUMN amount_minor TO importe_centavos;
ALTER TABLE payment_operations RENAME COLUMN expected_minor TO importe_esperado_centavos;
ALTER TABLE payment_operations RENAME COLUMN method TO metodo_pago;
ALTER TABLE payment_operations RENAME COLUMN reason TO motivo;
ALTER TABLE payment_operations RENAME COLUMN occurred_at TO fecha_evento;
ALTER TABLE payment_operations RENAME TO Movimientos_abonos;

-- payment_requests -> Solicitudes_abonos
ALTER TABLE payment_requests RENAME COLUMN author_id TO id_autor;
ALTER TABLE payment_requests RENAME COLUMN request_json TO solicitud_json;
ALTER TABLE payment_requests RENAME TO Solicitudes_abonos;

-- payments -> Abonos
ALTER TABLE payments RENAME COLUMN order_id TO id_orden;
ALTER TABLE payments RENAME COLUMN author_id TO id_autor;
ALTER TABLE payments RENAME COLUMN amount_minor TO importe_centavos;
ALTER TABLE payments RENAME COLUMN occurred_at TO fecha_evento;
ALTER TABLE payments RENAME COLUMN method TO metodo_pago;
ALTER TABLE payments RENAME COLUMN reason TO motivo;
ALTER TABLE payments RENAME TO Abonos;

-- repair_changes -> Cambios_reparaciones
ALTER TABLE repair_changes RENAME COLUMN order_id TO id_orden;
ALTER TABLE repair_changes RENAME COLUMN author_id TO id_autor;
ALTER TABLE repair_changes RENAME COLUMN operation TO operacion;
ALTER TABLE repair_changes RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE repair_changes RENAME COLUMN details_json TO detalles_json;
ALTER TABLE repair_changes RENAME TO Cambios_reparaciones;

-- repair_events -> Avances_reparacion
ALTER TABLE repair_events RENAME COLUMN order_id TO id_orden;
ALTER TABLE repair_events RENAME COLUMN author_id TO id_autor;
ALTER TABLE repair_events RENAME COLUMN stage TO etapa;
ALTER TABLE repair_events RENAME COLUMN title TO titulo;
ALTER TABLE repair_events RENAME COLUMN detail TO detalle;
ALTER TABLE repair_events RENAME COLUMN public TO visible_cliente;
ALTER TABLE repair_events RENAME COLUMN occurred_at TO fecha_evento;
ALTER TABLE repair_events RENAME TO Avances_reparacion;

-- repair_notes -> Notas_reparacion
ALTER TABLE repair_notes RENAME COLUMN order_id TO id_orden;
ALTER TABLE repair_notes RENAME COLUMN author_id TO id_autor;
ALTER TABLE repair_notes RENAME COLUMN body TO texto;
ALTER TABLE repair_notes RENAME COLUMN public TO visible_cliente;
ALTER TABLE repair_notes RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE repair_notes RENAME TO Notas_reparacion;

-- repair_records -> Seguimiento_reparaciones
ALTER TABLE repair_records RENAME COLUMN order_id TO id_orden;
ALTER TABLE repair_records RENAME COLUMN record_json TO datos_json;
ALTER TABLE repair_records RENAME TO Seguimiento_reparaciones;

-- system_logbook_events -> Bitacora_sistema
ALTER TABLE system_logbook_events RENAME COLUMN record_id TO id_registro;
ALTER TABLE system_logbook_events RENAME COLUMN author_id TO id_autor;
ALTER TABLE system_logbook_events RENAME COLUMN author_name TO nombre_autor;
ALTER TABLE system_logbook_events RENAME COLUMN operation TO operacion;
ALTER TABLE system_logbook_events RENAME COLUMN details_json TO detalles_json;
ALTER TABLE system_logbook_events RENAME COLUMN reason TO motivo;
ALTER TABLE system_logbook_events RENAME COLUMN result TO resultado;
ALTER TABLE system_logbook_events RENAME COLUMN occurred_at TO fecha_evento;
ALTER TABLE system_logbook_events RENAME TO Bitacora_sistema;

-- temporary_access_grants -> Accesos_temporales
ALTER TABLE temporary_access_grants RENAME COLUMN recipient_id TO id_receptor;
ALTER TABLE temporary_access_grants RENAME COLUMN recipient_name TO nombre_receptor;
ALTER TABLE temporary_access_grants RENAME COLUMN approver_id TO id_autorizador;
ALTER TABLE temporary_access_grants RENAME COLUMN approver_name TO nombre_autorizador;
ALTER TABLE temporary_access_grants RENAME COLUMN scope TO apartado;
ALTER TABLE temporary_access_grants RENAME COLUMN record_id TO id_registro;
ALTER TABLE temporary_access_grants RENAME COLUMN operations_json TO operaciones_json;
ALTER TABLE temporary_access_grants RENAME COLUMN reason TO motivo;
ALTER TABLE temporary_access_grants RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE temporary_access_grants RENAME COLUMN ended_at TO fecha_finalizacion;
ALTER TABLE temporary_access_grants RENAME COLUMN end_reason TO motivo_finalizacion;
ALTER TABLE temporary_access_grants RENAME TO Accesos_temporales;

-- user_permissions -> Permisos_usuarios
ALTER TABLE user_permissions RENAME COLUMN user_id TO id_usuario;
ALTER TABLE user_permissions RENAME COLUMN section TO apartado;
ALTER TABLE user_permissions RENAME COLUMN action TO accion;
ALTER TABLE user_permissions RENAME TO Permisos_usuarios;

-- users -> Usuarios
ALTER TABLE users RENAME COLUMN name TO nombre;
ALTER TABLE users RENAME COLUMN password_hash TO hash_contrasena;
ALTER TABLE users RENAME COLUMN access_level TO nivel_acceso;
ALTER TABLE users RENAME COLUMN active TO activo;
ALTER TABLE users RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE users RENAME COLUMN login_name TO nombre_acceso;
ALTER TABLE users RENAME TO Usuarios;

-- warranties -> Garantias
ALTER TABLE warranties RENAME COLUMN order_id TO id_orden;
ALTER TABLE warranties RENAME COLUMN author_id TO id_autor;
ALTER TABLE warranties RENAME COLUMN coverage TO cobertura;
ALTER TABLE warranties RENAME COLUMN days TO dias;
ALTER TABLE warranties RENAME TO Garantias;

-- order_evidence -> Evidencias_orden
ALTER TABLE order_evidence RENAME COLUMN order_id TO id_orden;
ALTER TABLE order_evidence RENAME COLUMN original_name TO nombre_original;
ALTER TABLE order_evidence RENAME COLUMN mime TO tipo_imagen;
ALTER TABLE order_evidence RENAME COLUMN size_bytes TO tamano_bytes;
ALTER TABLE order_evidence RENAME COLUMN source TO origen;
ALTER TABLE order_evidence RENAME COLUMN note TO descripcion;
ALTER TABLE order_evidence RENAME COLUMN initial_note TO descripcion_inicial;
ALTER TABLE order_evidence RENAME COLUMN author_id TO id_autor;
ALTER TABLE order_evidence RENAME COLUMN created_at TO fecha_registro;
ALTER TABLE order_evidence RENAME COLUMN deleted TO retirada;
ALTER TABLE order_evidence RENAME TO Evidencias_orden;

CREATE INDEX idx_historial_ordenes_autor ON Historial_ordenes(id_autor, fecha_registro);

CREATE INDEX idx_historial_ordenes_orden ON Historial_ordenes(id_orden, fecha_registro);

CREATE INDEX idx_equipo_cliente ON Equipo(id_cliente);

CREATE INDEX idx_equipo_serie ON Equipo(serie);

CREATE INDEX idx_bitacora_autor ON Bitacora_ordenes(id_autor, id DESC);

CREATE INDEX idx_bitacora_fecha ON Bitacora_ordenes(fecha_evento DESC, id DESC);

CREATE INDEX idx_bitacora_orden ON Bitacora_ordenes(id_orden, id DESC);

CREATE INDEX idx_refacciones_orden ON Refacciones_orden(id_orden);

CREATE INDEX idx_papelera_orden ON Historial_papelera(id_orden, secuencia);

CREATE INDEX idx_ordenes_equipo ON Ordenes(id_equipo);

CREATE INDEX idx_movimientos_abonos_orden ON Movimientos_abonos(id_orden,fecha_evento,id);

CREATE INDEX idx_movimientos_abonos_abono ON Movimientos_abonos(id_abono);

CREATE INDEX idx_abonos_orden ON Abonos(id_orden, fecha_evento);

CREATE INDEX idx_cambios_reparaciones_orden ON Cambios_reparaciones(id_orden, id);

CREATE INDEX idx_avances_reparacion_orden ON Avances_reparacion(id_orden, fecha_evento);

CREATE INDEX idx_notas_reparacion_orden ON Notas_reparacion(id_orden);

CREATE UNIQUE INDEX idx_usuarios_nombre_acceso ON Usuarios(nombre_acceso);

CREATE INDEX idx_garantias_orden ON Garantias(id_orden);

CREATE TRIGGER proteger_borrado_historial_ordenes BEFORE DELETE ON Historial_ordenes
BEGIN SELECT RAISE(ABORT, 'No se puede borrar el historial de ordenes'); END;

CREATE TRIGGER proteger_edicion_historial_ordenes BEFORE UPDATE ON Historial_ordenes
BEGIN SELECT RAISE(ABORT, 'No se puede editar el historial de ordenes'); END;

CREATE TRIGGER registrar_gasto_bitacora AFTER INSERT ON Gastos_orden
BEGIN
 INSERT INTO Bitacora_ordenes(origen,id_origen,id_orden,id_autor,nombre_autor,operacion,campo,valor_nuevo,fecha_evento,motivo,detalles_json)
 VALUES ('expense',NEW.id,NEW.id_orden,NEW.id_autor,(SELECT nombre FROM Usuarios WHERE id=NEW.id_autor),'costs.add','direct_cost_minor',CAST(NEW.importe_centavos AS TEXT),NEW.fecha_evento,NEW.motivo,json_array(json_object('field','expenseDescription','oldValue',NULL,'newValue',NEW.descripcion),json_object('field','direct_cost_minor','oldValue',NULL,'newValue',NEW.importe_centavos)));
END;

CREATE TRIGGER proteger_borrado_gastos BEFORE DELETE ON Gastos_orden
BEGIN SELECT RAISE(ABORT,'No se pueden borrar los gastos'); END;

CREATE TRIGGER proteger_edicion_gastos BEFORE UPDATE ON Gastos_orden
BEGIN SELECT RAISE(ABORT,'No se pueden editar los gastos'); END;

CREATE TRIGGER exigir_primer_usuario_completo
BEFORE INSERT ON Usuarios
WHEN NOT EXISTS (SELECT 1 FROM Usuarios) AND (NEW.nivel_acceso <> 'complete' OR NEW.activo <> 1)
BEGIN
    SELECT RAISE(ABORT, 'El primer usuario debe tener acceso completo');
END;

CREATE TRIGGER proteger_borrado_bitacora_ordenes BEFORE DELETE ON Bitacora_ordenes
BEGIN SELECT RAISE(ABORT,'No se puede borrar la bitacora'); END;

CREATE TRIGGER proteger_edicion_bitacora_ordenes BEFORE UPDATE ON Bitacora_ordenes
BEGIN SELECT RAISE(ABORT,'No se puede editar la bitacora'); END;

CREATE TRIGGER registrar_orden_bitacora AFTER INSERT ON Historial_ordenes
BEGIN
    INSERT INTO Bitacora_ordenes(origen,id_origen,id_orden,id_autor,nombre_autor,operacion,campo,valor_anterior,valor_nuevo,fecha_evento)
    VALUES ('order',NEW.id,NEW.id_orden,NEW.id_autor,(SELECT nombre FROM Usuarios WHERE id=NEW.id_autor),CASE WHEN NEW.campo='created' THEN 'orders.create' ELSE 'orders.edit' END,NEW.campo,NEW.valor_anterior,NEW.valor_nuevo,NEW.fecha_registro);
END;

CREATE TRIGGER registrar_abono_bitacora AFTER INSERT ON Abonos
BEGIN
 INSERT INTO Bitacora_ordenes(origen,id_origen,id_orden,id_autor,nombre_autor,operacion,valor_nuevo,fecha_evento,fecha_declarada,motivo,detalles_json)
 VALUES ('payment',NEW.id,NEW.id_orden,NEW.id_autor,(SELECT nombre FROM Usuarios WHERE id=NEW.id_autor),'payments.add',CAST(NEW.importe_centavos AS TEXT),strftime('%Y-%m-%dT%H:%M:%fZ','now'),nullif(NEW.fecha_evento,''),NEW.motivo,
 json_array(json_object('field','amount_minor','oldValue',NULL,'newValue',NEW.importe_centavos),json_object('field','paymentMethod','oldValue',NULL,'newValue',NEW.metodo_pago)));
END;

CREATE TRIGGER registrar_reparacion_bitacora AFTER INSERT ON Cambios_reparaciones
BEGIN
    INSERT INTO Bitacora_ordenes(origen,id_origen,id_orden,id_autor,nombre_autor,operacion,detalles_json,fecha_evento)
    SELECT 'repair',CAST(NEW.id AS TEXT),NEW.id_orden,NEW.id_autor,u.nombre,NEW.operacion,NEW.detalles_json,NEW.fecha_registro
    FROM Usuarios u WHERE u.id=NEW.id_autor AND NEW.operacion<>'payments.add';
END;

CREATE TRIGGER registrar_papelera_bitacora AFTER INSERT ON Historial_papelera
BEGIN
    INSERT INTO Bitacora_ordenes(origen,id_origen,id_orden,id_autor,nombre_autor,operacion,fecha_evento)
    VALUES ('visibility',CAST(NEW.secuencia AS TEXT),NEW.id_orden,NEW.id_autor,(SELECT nombre FROM Usuarios WHERE id=NEW.id_autor),CASE WHEN NEW.accion='deleted' THEN 'orders.delete' ELSE 'orders.restore' END,NEW.fecha_registro);
END;

CREATE TRIGGER proteger_borrado_historial_papelera BEFORE DELETE ON Historial_papelera
BEGIN SELECT RAISE(ABORT, 'No se puede borrar el historial de papelera'); END;

CREATE TRIGGER proteger_edicion_historial_papelera BEFORE UPDATE ON Historial_papelera
BEGIN SELECT RAISE(ABORT, 'No se puede editar el historial de papelera'); END;

CREATE TRIGGER registrar_movimiento_abono_bitacora AFTER INSERT ON Movimientos_abonos
BEGIN
 INSERT INTO Bitacora_ordenes(origen,id_origen,id_orden,id_autor,nombre_autor,operacion,campo,valor_nuevo,fecha_evento,motivo,detalles_json)
 VALUES ('money',NEW.id,NEW.id_orden,NEW.id_autor,(SELECT nombre FROM Usuarios WHERE id=NEW.id_autor),
 CASE NEW.tipo WHEN 'refund' THEN 'payments.refund' ELSE 'payments.adjust' END,'amount_minor',CAST(NEW.importe_centavos AS TEXT),NEW.fecha_evento,NEW.motivo,
 json_array(json_object('field','amount_minor','oldValue',NULL,'newValue',NEW.importe_centavos),json_object('field','paymentReference','oldValue',NULL,'newValue',NEW.id_abono),json_object('field','paymentMethod','oldValue',NULL,'newValue',NEW.metodo_pago)));
END;

CREATE TRIGGER proteger_borrado_movimientos_abonos BEFORE DELETE ON Movimientos_abonos
BEGIN SELECT RAISE(ABORT,'No se pueden borrar los movimientos de abonos'); END;

CREATE TRIGGER proteger_edicion_movimientos_abonos BEFORE UPDATE ON Movimientos_abonos
BEGIN SELECT RAISE(ABORT,'No se pueden editar los movimientos de abonos'); END;

CREATE TRIGGER proteger_borrado_solicitudes_abonos BEFORE DELETE ON Solicitudes_abonos
BEGIN SELECT RAISE(ABORT,'No se pueden borrar las solicitudes de abonos'); END;

CREATE TRIGGER proteger_edicion_solicitudes_abonos BEFORE UPDATE ON Solicitudes_abonos
BEGIN SELECT RAISE(ABORT,'No se pueden editar las solicitudes de abonos'); END;

CREATE TRIGGER proteger_borrado_abonos BEFORE DELETE ON Abonos
BEGIN SELECT RAISE(ABORT,'No se pueden borrar los abonos'); END;

CREATE TRIGGER proteger_edicion_abonos BEFORE UPDATE ON Abonos
BEGIN SELECT RAISE(ABORT,'No se pueden editar los abonos'); END;

CREATE TRIGGER conservar_ultimo_administrador_al_borrar
BEFORE DELETE ON Usuarios
WHEN OLD.nivel_acceso = 'complete' AND OLD.activo = 1
 AND NOT EXISTS (SELECT 1 FROM Usuarios WHERE id <> OLD.id AND nivel_acceso = 'complete' AND activo = 1)
BEGIN
    SELECT RAISE(ABORT, 'Debe conservarse un usuario activo con acceso completo');
END;

CREATE TRIGGER conservar_ultimo_administrador_al_editar
BEFORE UPDATE ON Usuarios
WHEN OLD.nivel_acceso = 'complete' AND OLD.activo = 1
 AND (NEW.nivel_acceso <> 'complete' OR NEW.activo <> 1)
 AND NOT EXISTS (SELECT 1 FROM Usuarios WHERE id <> OLD.id AND nivel_acceso = 'complete' AND activo = 1)
BEGIN
    SELECT RAISE(ABORT, 'Debe conservarse un usuario activo con acceso completo');
END;

CREATE TRIGGER proteger_borrado_cambios_reparaciones BEFORE DELETE ON Cambios_reparaciones
BEGIN SELECT RAISE(ABORT, 'No se puede borrar el historial de reparaciones'); END;

CREATE TRIGGER proteger_edicion_cambios_reparaciones BEFORE UPDATE ON Cambios_reparaciones
BEGIN SELECT RAISE(ABORT, 'No se puede editar el historial de reparaciones'); END;

CREATE TRIGGER proteger_borrado_bitacora_sistema BEFORE DELETE ON Bitacora_sistema
BEGIN SELECT RAISE(ABORT,'No se puede borrar la bitacora del sistema'); END;

CREATE TRIGGER proteger_edicion_bitacora_sistema BEFORE UPDATE ON Bitacora_sistema
BEGIN SELECT RAISE(ABORT,'No se puede editar la bitacora del sistema'); END;

CREATE TRIGGER registrar_fin_acceso_temporal AFTER UPDATE ON Accesos_temporales
BEGIN
 INSERT INTO Bitacora_sistema(id_registro,id_autor,nombre_autor,operacion,detalles_json,motivo)
 VALUES (coalesce(NEW.id_registro,NEW.apartado),NEW.id_receptor,NEW.nombre_receptor,'access.temporary.end',
 json_array(json_object('field','temporaryGrant','oldValue',NULL,'newValue',NEW.id),json_object('field','authorizedBy','oldValue',NULL,'newValue',NEW.nombre_autorizador),json_object('field','temporaryScope','oldValue',NULL,'newValue',NEW.apartado)),NEW.motivo_finalizacion);
END;

CREATE TRIGGER proteger_borrado_accesos_temporales BEFORE DELETE ON Accesos_temporales
BEGIN SELECT RAISE(ABORT,'No se puede borrar el historial de accesos temporales'); END;

CREATE TRIGGER permitir_solo_cierre_acceso_temporal BEFORE UPDATE ON Accesos_temporales
WHEN OLD.fecha_finalizacion IS NOT NULL OR NEW.fecha_finalizacion IS NULL OR NEW.id IS NOT OLD.id
 OR NEW.id_receptor IS NOT OLD.id_receptor OR NEW.nombre_receptor IS NOT OLD.nombre_receptor
 OR NEW.id_autorizador IS NOT OLD.id_autorizador OR NEW.nombre_autorizador IS NOT OLD.nombre_autorizador
 OR NEW.apartado IS NOT OLD.apartado OR NEW.id_registro IS NOT OLD.id_registro
 OR NEW.operaciones_json IS NOT OLD.operaciones_json OR NEW.motivo IS NOT OLD.motivo OR NEW.fecha_registro IS NOT OLD.fecha_registro
BEGIN SELECT RAISE(ABORT,'El acceso temporal solo puede finalizarse una vez'); END;

CREATE TRIGGER registrar_inicio_acceso_temporal AFTER INSERT ON Accesos_temporales
BEGIN
 INSERT INTO Bitacora_sistema(id_registro,id_autor,nombre_autor,operacion,detalles_json,motivo)
 VALUES (coalesce(NEW.id_registro,NEW.apartado),NEW.id_receptor,NEW.nombre_receptor,'access.temporary.start',
 json_array(json_object('field','temporaryGrant','oldValue',NULL,'newValue',NEW.id),json_object('field','authorizedBy','oldValue',NULL,'newValue',NEW.nombre_autorizador),json_object('field','performedBy','oldValue',NULL,'newValue',NEW.nombre_receptor),json_object('field','temporaryScope','oldValue',NULL,'newValue',NEW.apartado),json_object('field','temporaryOperations','oldValue',NULL,'newValue',json(NEW.operaciones_json))),NEW.motivo);
END;

CREATE VIEW Vista_bitacora_completa AS
 SELECT id,id_orden,id_autor,nombre_autor,operacion,campo,valor_anterior,valor_nuevo,detalles_json,fecha_evento,fecha_declarada,motivo,resultado FROM Bitacora_ordenes
 UNION ALL
 SELECT -id,id_registro,id_autor,nombre_autor,operacion,NULL,NULL,NULL,detalles_json,fecha_evento,NULL,motivo,resultado FROM Bitacora_sistema;

CREATE VIEW Vista_cobros_orden AS
 SELECT o.id,
 coalesce((SELECT sum(importe_centavos) FROM Abonos WHERE id_orden=o.id),0) AS cobrado_centavos,
 coalesce((SELECT sum(importe_centavos) FROM Movimientos_abonos WHERE id_orden=o.id AND tipo='adjustment'),0) AS ajustado_centavos,
 coalesce((SELECT sum(importe_centavos) FROM Movimientos_abonos WHERE id_orden=o.id AND tipo='refund'),0) AS devuelto_centavos,
 coalesce((SELECT sum(importe_centavos) FROM Abonos WHERE id_orden=o.id),0)
 +coalesce((SELECT sum(importe_centavos) FROM Movimientos_abonos WHERE id_orden=o.id AND tipo='adjustment'),0)
 -coalesce((SELECT sum(importe_centavos) FROM Movimientos_abonos WHERE id_orden=o.id AND tipo='refund'),0) AS cobro_neto_centavos
 FROM Ordenes o;

CREATE INDEX idx_evidencias_orden ON Evidencias_orden(id_orden,retirada,fecha_registro);

CREATE TRIGGER proteger_borrado_evidencias BEFORE DELETE ON Evidencias_orden
BEGIN SELECT RAISE(ABORT,'Las evidencias deben conservarse'); END;

CREATE TRIGGER conservar_evidencia_original BEFORE UPDATE ON Evidencias_orden
WHEN NEW.id IS NOT OLD.id OR NEW.id_orden IS NOT OLD.id_orden
 OR NEW.nombre_original IS NOT OLD.nombre_original OR NEW.tipo_imagen IS NOT OLD.tipo_imagen
 OR NEW.tamano_bytes IS NOT OLD.tamano_bytes OR NEW.origen IS NOT OLD.origen
 OR NEW.descripcion_inicial IS NOT OLD.descripcion_inicial OR NEW.id_autor IS NOT OLD.id_autor
 OR NEW.fecha_registro IS NOT OLD.fecha_registro
BEGIN SELECT RAISE(ABORT,'La evidencia original no puede reemplazarse'); END;
