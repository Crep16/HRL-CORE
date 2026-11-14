use super::{Database, migrations};
use crate::error::AppError;
use sqlx::{Row, SqlitePool};

async fn seed(pool: &SqlitePool) {
    sqlx::raw_sql(
        "INSERT INTO claves_maestras VALUES (1,'hash-de-prueba');
         INSERT INTO usuarios(id,nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol)
         SELECT 1,'Administrador','admin','hash-de-prueba',n.id,r.id
         FROM niveles_acceso n JOIN roles r WHERE n.codigo='complete' AND r.codigo='administrator';
         INSERT INTO usuarios(id,nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol)
         SELECT 2,'Tecnico','tecnico','hash-de-prueba',n.id,r.id
         FROM niveles_acceso n JOIN roles r WHERE n.codigo='limited' AND r.codigo='technician';
         INSERT INTO clientes(id,nombre,telefono) VALUES (1,'Cliente de prueba','0000000000');
         INSERT INTO equipos(id,id_cliente,articulo) VALUES (1,1,'Laptop');
         INSERT INTO ordenes(id,folio,id_equipo,id_creador,id_estado,fecha_recepcion,falla,precio_centavos)
         SELECT 1,'ORD-001',1,1,id,'2026-11-04','Falla de prueba',60000 FROM estados_orden WHERE codigo='diagnosis';
         INSERT INTO ordenes(id,folio,id_equipo,id_creador,id_estado,fecha_recepcion,falla,precio_centavos)
         SELECT 2,'ORD-002',1,1,id,'2026-11-04','Otra falla',60000 FROM estados_orden WHERE codigo='diagnosis';",
    ).execute(pool).await.unwrap();
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

#[test]
fn baseline_is_explicit_reentrant_and_matches_the_sql_deliverable() {
    tauri::async_runtime::block_on(async {
        assert_eq!(
            migrations::BASELINE_SQL,
            include_str!("../../../schema.sql")
        );
        assert_eq!(migrations::migrator().iter().count(), 3);
        let db = Database::in_memory().await.unwrap();
        seed(db.pool()).await;
        migrations::apply(db.pool()).await.unwrap();
        assert_eq!(count(db.pool(), "_sqlx_migrations").await, 3);
        assert_eq!(count(db.pool(), "ordenes").await, 2);
        assert_eq!(
            sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
                .fetch_one(db.pool())
                .await
                .unwrap(),
            "ok"
        );
        assert!(
            sqlx::query("PRAGMA foreign_key_check")
                .fetch_all(db.pool())
                .await
                .unwrap()
                .is_empty()
        );
        let json_columns: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlite_schema s, pragma_table_info(s.name) p WHERE s.type='table' AND p.name LIKE '%json%'")
            .fetch_one(db.pool()).await.unwrap();
        assert_eq!(json_columns, 0);
    });
}

#[test]
fn every_pool_connection_enforces_foreign_keys_and_file_reopen_preserves_data() {
    tauri::async_runtime::block_on(async {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("fase1.sqlite");
        let db = Database::open_file(&file).await.unwrap();
        seed(db.pool()).await;
        let mut connections = Vec::new();
        for _ in 0..4 {
            let mut connection = db.pool().acquire().await.unwrap();
            assert_eq!(
                sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
                    .fetch_one(&mut *connection)
                    .await
                    .unwrap(),
                1
            );
            assert!(
                sqlx::query("INSERT INTO equipos(id_cliente,articulo) VALUES (999,'PC')")
                    .execute(&mut *connection)
                    .await
                    .is_err()
            );
            connections.push(connection);
        }
        drop(connections);
        db.close().await;
        let reopened = Database::open_file(&file).await.unwrap();
        assert_eq!(count(reopened.pool(), "ordenes").await, 2);
        assert_eq!(count(reopened.pool(), "_sqlx_migrations").await, 3);
        reopened.close().await;
    });
}

#[test]
fn legacy_and_changed_migration_checksums_are_rejected_without_replacing_data() {
    tauri::async_runtime::block_on(async {
        let legacy = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql("CREATE TABLE Ordenes(id TEXT PRIMARY KEY,datos_json TEXT); INSERT INTO Ordenes VALUES ('original','conservar');")
            .execute(&legacy).await.unwrap();
        assert!(matches!(
            migrations::apply(&legacy).await,
            Err(AppError::LegacyDatabase)
        ));
        assert_eq!(count(&legacy, "Ordenes").await, 1);
        let db = Database::in_memory().await.unwrap();
        seed(db.pool()).await;
        sqlx::query("UPDATE _sqlx_migrations SET checksum=zeroblob(48)")
            .execute(db.pool())
            .await
            .unwrap();
        assert!(matches!(
            migrations::apply(db.pool()).await,
            Err(AppError::Migration(_))
        ));
        assert_eq!(count(db.pool(), "ordenes").await, 2);
    });
}

#[test]
fn first_and_last_administrator_and_initial_setup_state_are_protected() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        assert_eq!(
            sqlx::query_scalar::<_, bool>("SELECT configurado FROM vista_configuracion_acceso")
                .fetch_one(pool)
                .await
                .unwrap(),
            false
        );
        assert!(sqlx::query("INSERT INTO usuarios(nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol) SELECT 'No admin','no-admin','hash',n.id,r.id FROM niveles_acceso n JOIN roles r WHERE n.codigo='limited' AND r.codigo='technician'")
            .execute(pool).await.is_err());
        seed(pool).await;
        assert!(
            sqlx::query_scalar::<_, bool>("SELECT configurado FROM vista_configuracion_acceso")
                .fetch_one(pool)
                .await
                .unwrap()
        );
        assert!(
            sqlx::query("UPDATE usuarios SET activo=0 WHERE id=1")
                .execute(pool)
                .await
                .is_err()
        );
        assert!(sqlx::query("UPDATE usuarios SET id_nivel=(SELECT id FROM niveles_acceso WHERE codigo='limited') WHERE id=1").execute(pool).await.is_err());
        assert!(
            sqlx::query("DELETE FROM usuarios WHERE id=1")
                .execute(pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query(
                "INSERT INTO usuario_permiso SELECT 2,id FROM permisos WHERE solo_completo=1"
            )
            .execute(pool)
            .await
            .is_err()
        );
    });
}

#[test]
fn all_catalogs_are_immutable_and_current_states_permissions_and_currencies_exist() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        for table in [
            "niveles_acceso",
            "roles",
            "apartados",
            "acciones",
            "permisos",
            "monedas",
            "estados_orden",
            "etapas",
            "operaciones_auditoria",
        ] {
            let column = if table == "monedas" { "codigo" } else { "id" };
            for statement in [
                format!("INSERT INTO {table} DEFAULT VALUES"),
                format!("UPDATE {table} SET {column}={column}"),
                format!("DELETE FROM {table}"),
            ] {
                let error = sqlx::query(&statement)
                    .execute(db.pool())
                    .await
                    .unwrap_err();
                assert!(error.to_string().contains("migra"), "{table}: {error}");
            }
        }
        let currencies: Vec<String> =
            sqlx::query_scalar("SELECT codigo FROM monedas ORDER BY codigo")
                .fetch_all(db.pool())
                .await
                .unwrap();
        assert_eq!(currencies, ["EUR", "MXN", "USD"]);
        for label in [
            "En diagnóstico",
            "Esperando autorización",
            "Autorizado",
            "Esperando refacción",
            "En reparación",
            "Listo para entrega",
            "Entregado",
        ] {
            assert_eq!(
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM estados_orden WHERE nombre=?")
                    .bind(label)
                    .fetch_one(db.pool())
                    .await
                    .unwrap(),
                1
            );
        }
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM permisos p JOIN apartados a ON a.id=p.id_apartado JOIN acciones c ON c.id=p.id_accion WHERE a.codigo='payments' AND c.codigo IN ('add','refund','adjust')")
            .fetch_one(db.pool()).await.unwrap(),3);
    });
}

#[test]
fn balances_keep_receipts_price_reductions_adjustments_and_total_refunds_separate() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        seed(pool).await;
        sqlx::raw_sql("INSERT INTO abonos(id,id_orden,id_autor,metodo_pago,importe_centavos) VALUES (1,1,1,'Método escrito libremente',75000),(2,2,1,NULL,50000);
            INSERT INTO ajustes_abono(id_abono,id_autor,importe_centavos,motivo) VALUES (2,1,-20000,'Corrección de captura');")
            .execute(pool).await.unwrap();
        let row = sqlx::query("SELECT * FROM vista_saldos_orden WHERE id_orden=1")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(row.get::<i64, _>("pendiente_centavos"), 0);
        assert_eq!(row.get::<i64, _>("saldo_favor_centavos"), 15000);
        let row = sqlx::query("SELECT * FROM vista_saldos_orden WHERE id_orden=2")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(row.get::<i64, _>("cobro_neto_centavos"), 30000);
        assert_eq!(row.get::<i64, _>("pendiente_centavos"), 30000);
        sqlx::query("INSERT INTO devoluciones(id_orden,id_autor,metodo_pago,importe_centavos,motivo) VALUES (1,1,'Método libre',15000,'Devolución total')").execute(pool).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT saldo_favor_centavos FROM vista_saldos_orden WHERE id_orden=1"
            )
            .fetch_one(pool)
            .await
            .unwrap(),
            0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT importe_centavos FROM abonos WHERE id=1")
                .fetch_one(pool)
                .await
                .unwrap(),
            75000
        );
        for amount in ["-1", "0", "1.25", "9000000000001", "'importe'"] {
            assert!(
                sqlx::query(&format!(
                    "INSERT INTO abonos(id_orden,id_autor,importe_centavos) VALUES (1,1,{amount})"
                ))
                .execute(pool)
                .await
                .is_err()
            );
        }
    });
}

#[test]
fn financial_requests_have_global_identity_author_type_and_immutable_fingerprint() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        seed(pool).await;
        sqlx::query("INSERT INTO solicitudes_financieras(id,id_autor,tipo,huella_solicitud) VALUES ('request-receipt-0001',1,'receipt',zeroblob(32))").execute(pool).await.unwrap();
        assert!(sqlx::query("INSERT INTO solicitudes_financieras(id,id_autor,tipo,huella_solicitud) VALUES ('request-receipt-0001',2,'refund',randomblob(32))").execute(pool).await.is_err());
        assert!(sqlx::query("INSERT INTO abonos(id_orden,id_autor,importe_centavos,id_solicitud) VALUES (1,2,100,'request-receipt-0001')").execute(pool).await.is_err());
        assert!(sqlx::query("INSERT INTO devoluciones(id_orden,id_autor,metodo_pago,importe_centavos,motivo,id_solicitud) VALUES (1,1,'Libre',100,'Prueba','request-receipt-0001')").execute(pool).await.is_err());
        sqlx::query("INSERT INTO abonos(id_orden,id_autor,importe_centavos,id_solicitud) VALUES (1,1,100,'request-receipt-0001')").execute(pool).await.unwrap();
        assert!(sqlx::query("INSERT INTO abonos(id_orden,id_autor,importe_centavos,id_solicitud) VALUES (1,1,100,'request-receipt-0001')").execute(pool).await.is_err());
        assert!(
            sqlx::query("UPDATE solicitudes_financieras SET huella_solicitud=randomblob(32)")
                .execute(pool)
                .await
                .is_err()
        );
    });
}

#[test]
fn all_books_and_audit_rows_reject_edits_and_deletion() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        seed(pool).await;
        sqlx::raw_sql("INSERT INTO abonos(id,id_orden,id_autor,importe_centavos) VALUES (1,1,1,75000);
          INSERT INTO ajustes_abono(id_abono,id_autor,importe_centavos,motivo) VALUES (1,1,-100,'Prueba');
          INSERT INTO devoluciones(id_orden,id_autor,metodo_pago,importe_centavos,motivo) VALUES (1,1,'Libre',14900,'Total');
          INSERT INTO gastos_directos(id_orden,id_autor,descripcion,importe_centavos,motivo) VALUES (1,1,'Costo',100,'Prueba');
          INSERT INTO auditoria(id,id_autor,id_operacion,entidad,id_orden) SELECT 1,1,id,'ordenes',1 FROM operaciones_auditoria WHERE codigo='orders.edit';
          INSERT INTO auditoria_detalle(id_auditoria,campo,valor_nuevo) VALUES (1,'precio','60000');")
          .execute(pool).await.unwrap();
        for table in [
            "abonos",
            "ajustes_abono",
            "devoluciones",
            "gastos_directos",
            "auditoria",
            "auditoria_detalle",
        ] {
            assert!(
                sqlx::query(&format!("UPDATE {table} SET id=id"))
                    .execute(pool)
                    .await
                    .is_err()
            );
            assert!(
                sqlx::query(&format!("DELETE FROM {table}"))
                    .execute(pool)
                    .await
                    .is_err()
            );
            assert_eq!(count(pool, table).await, 1);
        }
    });
}

#[test]
fn temporary_access_closes_only_once_and_keeps_granted_permissions() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        seed(pool).await;
        sqlx::query("INSERT INTO accesos_temporales(id,id_receptor,id_autorizador,id_apartado,id_orden,motivo) SELECT 1,2,1,id,1,'Autorización de prueba' FROM apartados WHERE codigo='repairs'").execute(pool).await.unwrap();
        sqlx::query(
            "INSERT INTO acceso_temporal_permiso SELECT 1,id FROM permisos WHERE solo_completo=1",
        )
        .execute(pool)
        .await
        .unwrap();
        assert!(
            sqlx::query("DELETE FROM acceso_temporal_permiso")
                .execute(pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("UPDATE accesos_temporales SET motivo='Reescrito'")
                .execute(pool)
                .await
                .is_err()
        );
        sqlx::query("UPDATE accesos_temporales SET finalizado_en='2026-11-04T01:00:00Z',motivo_finalizacion='Fin' WHERE id=1").execute(pool).await.unwrap();
        assert!(
            sqlx::query(
                "UPDATE accesos_temporales SET finalizado_en='2026-11-04T02:00:00Z' WHERE id=1"
            )
            .execute(pool)
            .await
            .is_err()
        );
        assert!(
            sqlx::query("DELETE FROM accesos_temporales")
                .execute(pool)
                .await
                .is_err()
        );
    });
}

#[test]
fn custom_states_ordered_stages_unknown_delivery_and_new_fields_are_preserved() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        seed(pool).await;
        sqlx::query("UPDATE ordenes SET id_estado=NULL,estado_personalizado='Esperando respuesta externa',entregado=1,fecha_entrega=NULL,fecha_estimada_entrega='2026-11-10',sufrio_golpes=1,derrame_liquidos=0 WHERE id=1").execute(pool).await.unwrap();
        sqlx::query("UPDATE clientes SET correo='cliente@example.invalid' WHERE id=1")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO diagnosticos(id_orden,id_autor,texto) VALUES (1,1,'Diagnóstico separado')",
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::raw_sql("INSERT INTO orden_etapas(id,id_orden,id_etapa,posicion) SELECT 1,1,id,0 FROM etapas WHERE nombre='Recibido';
          INSERT INTO orden_etapas(id,id_orden,nombre_personalizado,posicion) VALUES (2,1,'Limpieza de placa',1);
          INSERT INTO orden_etapas(id,id_orden,id_etapa,posicion) SELECT 3,1,id,2 FROM etapas WHERE nombre='Entregado';
          INSERT INTO avances(id_etapa_orden,id_autor,titulo,fecha_evento) VALUES (2,1,'Trabajo en placa','2026-11-04T01:00:00Z');")
          .execute(pool).await.unwrap();
        assert_eq!(count(pool, "orden_etapas").await, 3);
        assert!(sqlx::query("INSERT INTO orden_etapas(id_orden,nombre_personalizado,posicion) VALUES (1,'limpieza de placa',3)").execute(pool).await.is_err());
        assert!(
            sqlx::query("UPDATE ordenes SET falla=? WHERE id=1")
                .bind("x".repeat(16001))
                .execute(pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("UPDATE ordenes SET folio='ORD-002' WHERE id=1")
                .execute(pool)
                .await
                .is_err()
        );
        let row = sqlx::query(
            "SELECT entregado,fecha_entrega,sufrio_golpes,derrame_liquidos FROM ordenes WHERE id=1",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        assert!(row.get::<bool, _>("entregado"));
        assert_eq!(row.get::<Option<String>, _>("fecha_entrega"), None);
        assert!(row.get::<bool, _>("sufrio_golpes"));
        assert!(!row.get::<bool, _>("derrame_liquidos"));
    });
}

#[test]
fn evidence_paths_originals_and_encrypted_device_credentials_have_structural_constraints() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        seed(pool).await;
        let insert = "INSERT INTO evidencias(codigo_archivo,id_orden,id_autor,nombre_original,ruta_relativa,tipo_archivo,mime,tamano_bytes,origen) VALUES ('0123456789abcdef0123456789abcdef',1,1,'Prueba',?,'image','image/png',100,'file')";
        for path in [
            "../foto.png",
            "C:/foto.png",
            "/foto.png",
            "orden/../../foto.png",
            "orden\\foto.png",
        ] {
            assert!(sqlx::query(insert).bind(path).execute(pool).await.is_err());
        }
        sqlx::query(insert)
            .bind("orden/archivo.png")
            .execute(pool)
            .await
            .unwrap();
        assert!(
            sqlx::query("UPDATE evidencias SET ruta_relativa='otro.png'")
                .execute(pool)
                .await
                .is_err()
        );
        sqlx::query("UPDATE evidencias SET descripcion='Detalle',retirada=1,revision=revision+1")
            .execute(pool)
            .await
            .unwrap();
        assert!(
            sqlx::query("DELETE FROM evidencias")
                .execute(pool)
                .await
                .is_err()
        );
        assert!(sqlx::query("INSERT INTO credenciales_dispositivos VALUES (1,1,'clave-local','AES-256-GCM',zeroblob(11),zeroblob(16))").execute(pool).await.is_err());
        sqlx::query("INSERT INTO credenciales_dispositivos VALUES (1,1,'clave-local','AES-256-GCM',zeroblob(12),zeroblob(16))").execute(pool).await.unwrap();
        assert_eq!(count(pool, "credenciales_dispositivos").await, 1);
        assert!(sqlx::query("INSERT INTO credenciales_dispositivos VALUES (2,1,'clave-local','AES-256-GCM',zeroblob(12),zeroblob(16))").execute(pool).await.is_err());
        sqlx::query("INSERT INTO evidencias(codigo_archivo,id_orden,id_autor,nombre_original,ruta_relativa,tipo_archivo,mime,tamano_bytes,origen) VALUES ('1123456789abcdef0123456789abcdef',1,1,'Video','orden/video.mp4','video','video/mp4',30000000,'file'),('2123456789abcdef0123456789abcdef',1,1,'Documento','orden/archivo.pdf','document','application/pdf',1000,'file')").execute(pool).await.unwrap();
        assert_eq!(count(pool, "evidencias").await, 3);
        let columns: Vec<String> = sqlx::query("PRAGMA table_info(ordenes)")
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.get("name"))
            .collect();
        assert!(!columns.iter().any(|c| c.contains("contrasena")));
    });
}

#[test]
fn an_audit_failure_rolls_back_the_order_and_financial_request_in_one_transaction() {
    tauri::async_runtime::block_on(async {
        let db = Database::in_memory().await.unwrap();
        let pool = db.pool();
        seed(pool).await;
        sqlx::raw_sql("CREATE TRIGGER auditoria_fallo_prueba BEFORE INSERT ON auditoria BEGIN SELECT RAISE(ABORT,'Fallo de auditoría de prueba'); END;").execute(pool).await.unwrap();
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("UPDATE ordenes SET precio_centavos=50000 WHERE id=1")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("INSERT INTO solicitudes_financieras VALUES ('request-rollback-01',1,'receipt',zeroblob(32),'2026-11-04T00:00:00Z')").execute(&mut *tx).await.unwrap();
        assert!(sqlx::query("INSERT INTO auditoria(id_autor,id_operacion,entidad,id_orden) SELECT 1,id,'ordenes',1 FROM operaciones_auditoria WHERE codigo='orders.edit'").execute(&mut *tx).await.is_err());
        tx.rollback().await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT precio_centavos FROM ordenes WHERE id=1")
                .fetch_one(pool)
                .await
                .unwrap(),
            60000
        );
        assert_eq!(count(pool, "solicitudes_financieras").await, 0);
    });
}
