use crate::{
    domain::{contracts::*, security::DeviceCredential},
    error::AppError,
};
use sqlx::{Row, SqliteConnection};

#[derive(Default)]
pub struct OrderRepository;

pub trait OrderReader: Send + Sync {
    fn load_any<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        id: i64,
        include_deleted: bool,
    ) -> super::RepositoryFuture<'a, Option<OrderResponse>>;
}

impl OrderReader for OrderRepository {
    fn load_any<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        id: i64,
        include_deleted: bool,
    ) -> super::RepositoryFuture<'a, Option<OrderResponse>> {
        Box::pin(OrderRepository::load_any(self, db, id, include_deleted))
    }
}

pub struct RequestRecord {
    pub author: i64,
    pub kind: String,
    pub fingerprint: Vec<u8>,
    pub order_id: Option<i64>,
}

impl OrderRepository {
    pub async fn has_credentials(&self, db: &mut SqliteConnection) -> Result<bool, AppError> {
        Ok(
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM credenciales_dispositivos)")
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn update_order(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        equipment: i64,
        status: Option<i64>,
        value: &CreateOrderPayload,
        delivered: bool,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE ordenes SET id_equipo=?,id_estado=?,estado_personalizado=?,fecha_recepcion=?,fecha_estimada_entrega=?,falla=?,trabajo=?,notas_recepcion=?,precio_centavos=?,paso_otro_taller=?,detalle_otro_taller=?,sufrio_golpes=?,detalle_golpes=?,derrame_liquidos=?,detalle_liquidos=?,antecedentes_equipo=?,entregado=?,fecha_entrega=CASE WHEN ? THEN coalesce(fecha_entrega,strftime('%Y-%m-%d','now')) ELSE NULL END,revision=revision+1 WHERE id=? AND eliminada_en IS NULL")
            .bind(equipment).bind(status).bind(if status.is_none(){Some(&value.status)}else{None}).bind(&value.received_at).bind(&value.estimated_delivery_at).bind(&value.reported_failure).bind(&value.planned_work).bind(&value.reception_notes).bind(value.price_minor).bind(value.history.previous_workshop).bind(&value.history.previous_workshop_detail).bind(value.history.suffered_impact).bind(&value.history.impact_detail).bind(value.history.liquid_spill).bind(&value.history.liquid_detail).bind(&value.history.notes).bind(delivered).bind(delivered).bind(id).execute(db).await?;
        Ok(())
    }
    pub async fn remove_credential(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM credenciales_dispositivos WHERE id_orden=?")
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }
    pub async fn trash(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        deleted: bool,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE ordenes SET eliminada_en=CASE WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,revision=revision+1 WHERE id=?").bind(deleted).bind(id).execute(db).await?;
        Ok(())
    }
    pub async fn load(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<Option<OrderResponse>, AppError> {
        self.load_any(db, id, false).await
    }

    pub async fn list(
        &self,
        db: &mut SqliteConnection,
        payload: &ListOrdersPayload,
    ) -> Result<OrderListResponse, AppError> {
        // instr trata %, _ y comillas como texto literal; los valores siempre se enlazan.
        let folded = |column: &str| {
            let mut expression = column.to_owned();
            // SQLite lower solo cubre ASCII; se conserva la comparación literal del texto español.
            for (upper, lower) in [
                ('Á', 'á'),
                ('É', 'é'),
                ('Í', 'í'),
                ('Ó', 'ó'),
                ('Ú', 'ú'),
                ('Ü', 'ü'),
                ('Ñ', 'ñ'),
            ] {
                expression = format!("replace({expression},'{upper}','{lower}')");
            }
            format!("lower({expression})")
        };
        let filter = format!(
            " FROM ordenes o JOIN equipos e ON e.id=o.id_equipo JOIN clientes c ON c.id=e.id_cliente LEFT JOIN estados_orden s ON s.id=o.id_estado JOIN vista_saldos_orden b ON b.id_orden=o.id WHERE (o.eliminada_en IS NOT NULL)=?5 AND (?1='' OR (?2 IN ('all','order') AND instr({},?1)>0) OR (?2 IN ('all','client') AND instr({},?1)>0) OR (?2 IN ('all','serial') AND instr({},?1)>0))",
            folded("o.folio"),
            folded("c.nombre"),
            folded("e.serie")
        );
        let field = match payload.field {
            OrderSearchField::All => "all",
            OrderSearchField::Order => "order",
            OrderSearchField::Client => "client",
            OrderSearchField::Serial => "serial",
        };
        let direction = if payload.oldest_first { "ASC" } else { "DESC" };
        let total = sqlx::query_scalar::<_, i64>(&format!("SELECT count(*){filter}"))
            .bind(&payload.query)
            .bind(field)
            .bind(payload.limit)
            .bind(payload.offset)
            .bind(payload.trashed)
            .fetch_one(&mut *db)
            .await?;
        let rows = sqlx::query(&format!("SELECT o.id,o.folio,c.nombre,e.serie,e.articulo,coalesce(s.nombre,o.estado_personalizado) AS estado,o.fecha_recepcion,o.codigo_moneda,o.precio_centavos,b.cobro_neto_centavos,b.pendiente_centavos,b.saldo_favor_centavos,o.entregado{filter} ORDER BY o.id {direction} LIMIT ?3 OFFSET ?4"))
            .bind(&payload.query).bind(field)
            .bind(payload.limit).bind(payload.offset).bind(payload.trashed).fetch_all(db).await?;
        let items = rows
            .into_iter()
            .map(|r| {
                Ok(OrderListItem {
                    id: r.try_get("id")?,
                    folio: r.try_get("folio")?,
                    client_name: r.try_get("nombre")?,
                    serial: r.try_get("serie")?,
                    item: r.try_get("articulo")?,
                    status: r.try_get("estado")?,
                    received_at: r.try_get("fecha_recepcion")?,
                    currency: r.try_get("codigo_moneda")?,
                    price_minor: r.try_get("precio_centavos")?,
                    net_minor: r.try_get("cobro_neto_centavos")?,
                    pending_minor: r.try_get("pendiente_centavos")?,
                    credit_minor: r.try_get("saldo_favor_centavos")?,
                    delivered: r.try_get("entregado")?,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        Ok(OrderListResponse { items, total })
    }

    pub async fn find_id(
        &self,
        db: &mut SqliteConnection,
        reference: &str,
    ) -> Result<Option<i64>, AppError> {
        Ok(sqlx::query_scalar("SELECT id FROM ordenes WHERE eliminada_en IS NULL AND (folio=? OR id=?) ORDER BY (folio=?) DESC LIMIT 1")
            .bind(reference).bind(reference.parse::<i64>().ok()).bind(reference).fetch_optional(db).await?)
    }

    pub async fn history(
        &self,
        db: &mut SqliteConnection,
        payload: &OrderHistoryPayload,
    ) -> Result<OrderHistoryResponse, AppError> {
        let total = sqlx::query_scalar("SELECT count(*) FROM auditoria WHERE id_orden=?")
            .bind(payload.order_id)
            .fetch_one(&mut *db)
            .await?;
        let rows = sqlx::query("SELECT a.*,u.nombre,p.codigo AS operacion FROM auditoria a JOIN usuarios u ON u.id=a.id_autor JOIN operaciones_auditoria p ON p.id=a.id_operacion WHERE a.id_orden=? ORDER BY a.id DESC LIMIT ? OFFSET ?")
            .bind(payload.order_id).bind(payload.limit).bind(payload.offset).fetch_all(&mut *db).await?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let id: i64 = r.try_get("id")?;
            let changes = sqlx::query("SELECT campo,valor_anterior,valor_nuevo FROM auditoria_detalle WHERE id_auditoria=? ORDER BY id")
                .bind(id).fetch_all(&mut *db).await?.into_iter().map(|d| Ok(OrderHistoryChange {
                    field: d.try_get("campo")?, old_value: d.try_get("valor_anterior")?, new_value: d.try_get("valor_nuevo")?,
                })).collect::<Result<Vec<_>, AppError>>()?;
            items.push(OrderHistoryEvent {
                id,
                author_id: r.try_get("id_autor")?,
                author_name: r.try_get("nombre")?,
                operation: r.try_get("operacion")?,
                entity: r.try_get("entidad")?,
                entity_id: r.try_get("id_entidad")?,
                reason: r.try_get("motivo")?,
                result: r.try_get("resultado")?,
                created_at: r.try_get("creado_en")?,
                changes,
            });
        }
        Ok(OrderHistoryResponse { items, total })
    }

    pub async fn clients_matching(
        &self,
        db: &mut SqliteConnection,
        input: &ClientInput,
    ) -> Result<Vec<i64>, AppError> {
        Ok(sqlx::query_scalar("SELECT id FROM clientes WHERE nombre=? AND telefono=? AND correo IS ? ORDER BY id LIMIT 2").bind(&input.full_name).bind(&input.phone).bind(&input.email).fetch_all(db).await?)
    }
    pub async fn client(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<Option<ClientOutput>, AppError> {
        sqlx::query("SELECT id,nombre,telefono,correo FROM clientes WHERE id=?")
            .bind(id)
            .fetch_optional(db)
            .await?
            .map(|r| {
                Ok(ClientOutput {
                    id: r.try_get("id")?,
                    full_name: r.try_get("nombre")?,
                    phone: r.try_get("telefono")?,
                    email: r.try_get("correo")?,
                })
            })
            .transpose()
    }
    pub async fn insert_client(
        &self,
        db: &mut SqliteConnection,
        input: &ClientInput,
    ) -> Result<i64, AppError> {
        Ok(
            sqlx::query("INSERT INTO clientes(nombre,telefono,correo) VALUES (?,?,?)")
                .bind(&input.full_name)
                .bind(&input.phone)
                .bind(&input.email)
                .execute(db)
                .await?
                .last_insert_rowid(),
        )
    }
    pub async fn equipment(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<Option<(i64, EquipmentOutput)>, AppError> {
        sqlx::query("SELECT id,id_cliente,articulo,marca,modelo,serie FROM equipos WHERE id=?")
            .bind(id)
            .fetch_optional(db)
            .await?
            .map(|r| {
                Ok((
                    r.try_get("id_cliente")?,
                    EquipmentOutput {
                        id: r.try_get("id")?,
                        item: r.try_get("articulo")?,
                        brand: r.try_get("marca")?,
                        model: r.try_get("modelo")?,
                        serial: r.try_get("serie")?,
                    },
                ))
            })
            .transpose()
    }
    pub async fn equipment_matching(
        &self,
        db: &mut SqliteConnection,
        client: i64,
        input: &EquipmentInput,
    ) -> Result<Vec<i64>, AppError> {
        Ok(sqlx::query_scalar("SELECT id FROM equipos WHERE id_cliente=? AND articulo=? AND marca=? AND modelo=? AND serie=? ORDER BY id LIMIT 2")
            .bind(client).bind(&input.item).bind(&input.brand).bind(&input.model).bind(&input.serial).fetch_all(db).await?)
    }
    pub async fn insert_equipment(
        &self,
        db: &mut SqliteConnection,
        client: i64,
        input: &EquipmentInput,
    ) -> Result<i64, AppError> {
        Ok(sqlx::query(
            "INSERT INTO equipos(id_cliente,articulo,marca,modelo,serie) VALUES (?,?,?,?,?)",
        )
        .bind(client)
        .bind(&input.item)
        .bind(&input.brand)
        .bind(&input.model)
        .bind(&input.serial)
        .execute(db)
        .await?
        .last_insert_rowid())
    }
    pub async fn reserve_folio(
        &self,
        db: &mut SqliteConnection,
        prefix: &str,
        start: i64,
    ) -> Result<String, AppError> {
        sqlx::query("INSERT INTO secuencias_folio(prefijo,siguiente_numero) VALUES (?,?) ON CONFLICT(prefijo) DO NOTHING").bind(prefix).bind(start).execute(&mut *db).await?;
        loop {
            let number:Option<i64>=sqlx::query_scalar("UPDATE secuencias_folio SET siguiente_numero=siguiente_numero+1 WHERE prefijo=? AND siguiente_numero<=999999999 RETURNING siguiente_numero-1").bind(prefix).fetch_optional(&mut *db).await?;
            let number = number.ok_or_else(|| {
                AppError::Conflict("Se agotó el rango de folios configurado.".into())
            })?;
            let folio = format!("{prefix}{number:03}");
            let used: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM ordenes WHERE folio=?)")
                    .bind(&folio)
                    .fetch_one(&mut *db)
                    .await?;
            if !used {
                return Ok(folio);
            }
        }
    }
    pub async fn status(
        &self,
        db: &mut SqliteConnection,
        text: &str,
    ) -> Result<Option<i64>, AppError> {
        Ok(sqlx::query_scalar(
            "SELECT id FROM estados_orden WHERE codigo=? OR nombre=? ORDER BY id LIMIT 1",
        )
        .bind(text)
        .bind(text)
        .fetch_optional(db)
        .await?)
    }
    pub async fn currency_exists(
        &self,
        db: &mut SqliteConnection,
        currency: &str,
    ) -> Result<bool, AppError> {
        Ok(
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM monedas WHERE codigo=?)")
                .bind(currency)
                .fetch_one(db)
                .await?,
        )
    }
    pub async fn insert_order(
        &self,
        db: &mut SqliteConnection,
        folio: &str,
        equipment: i64,
        author: i64,
        status: Option<i64>,
        payload: &CreateOrderPayload,
    ) -> Result<i64, AppError> {
        Ok(sqlx::query("INSERT INTO ordenes(folio,id_equipo,id_creador,id_estado,estado_personalizado,codigo_moneda,fecha_recepcion,fecha_estimada_entrega,falla,trabajo,notas_recepcion,precio_centavos,paso_otro_taller,detalle_otro_taller,sufrio_golpes,detalle_golpes,derrame_liquidos,detalle_liquidos,antecedentes_equipo) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(folio).bind(equipment).bind(author).bind(status).bind(if status.is_none(){Some(payload.status.as_str())}else{None}).bind(&payload.currency)
            .bind(&payload.received_at).bind(&payload.estimated_delivery_at).bind(&payload.reported_failure).bind(&payload.planned_work).bind(&payload.reception_notes).bind(payload.price_minor)
            .bind(payload.history.previous_workshop).bind(&payload.history.previous_workshop_detail).bind(payload.history.suffered_impact).bind(&payload.history.impact_detail).bind(payload.history.liquid_spill).bind(&payload.history.liquid_detail)
            .bind(&payload.history.notes)
            .execute(db).await?.last_insert_rowid())
    }
    pub async fn insert_credential(
        &self,
        db: &mut SqliteConnection,
        secret: &DeviceCredential,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO credenciales_dispositivos(id_orden,version,id_clave,algoritmo,nonce,cifrado) VALUES (?,?,?,?,?,?)")
            .bind(secret.order_id()).bind(secret.version()).bind(secret.key_id()).bind(secret.algorithm()).bind(secret.nonce().as_slice()).bind(secret.ciphertext()).execute(db).await?;
        Ok(())
    }
    pub async fn credential(
        &self,
        db: &mut SqliteConnection,
        order: i64,
    ) -> Result<Option<DeviceCredential>, AppError> {
        let row=sqlx::query("SELECT version,id_clave,algoritmo,nonce,cifrado FROM credenciales_dispositivos WHERE id_orden=?").bind(order).fetch_optional(db).await?;
        row.map(|r| {
            if r.try_get::<i64, _>("version")? != 1
                || r.try_get::<&str, _>("algoritmo")? != "AES-256-GCM"
            {
                return Err(AppError::Crypto);
            }
            let nonce: Vec<u8> = r.try_get("nonce")?;
            DeviceCredential::from_encrypted(
                order,
                r.try_get("id_clave")?,
                nonce.as_slice().try_into().map_err(|_| AppError::Crypto)?,
                r.try_get("cifrado")?,
            )
            .map_err(|_| AppError::Crypto)
        })
        .transpose()
    }
    pub async fn request(
        &self,
        db: &mut SqliteConnection,
        id: &str,
    ) -> Result<Option<RequestRecord>, AppError> {
        sqlx::query("SELECT s.id_autor,s.tipo,s.huella_solicitud,a.id_orden FROM solicitudes_financieras s LEFT JOIN abonos a ON a.id_solicitud=s.id WHERE s.id=?")
            .bind(id).fetch_optional(db).await?.map(|r|Ok(RequestRecord{author:r.try_get("id_autor")?,kind:r.try_get("tipo")?,fingerprint:r.try_get("huella_solicitud")?,order_id:r.try_get("id_orden")?})).transpose()
    }
    pub async fn initial_payment(
        &self,
        db: &mut SqliteConnection,
        order: i64,
        author: i64,
        payment: &InitialPaymentInput,
        fingerprint: &[u8; 32],
    ) -> Result<i64, AppError> {
        sqlx::query("INSERT INTO solicitudes_financieras(id,id_autor,tipo,huella_solicitud) VALUES (?,?,'receipt',?)").bind(&payment.request_id).bind(author).bind(fingerprint.as_slice()).execute(&mut *db).await?;
        Ok(sqlx::query("INSERT INTO abonos(id_orden,id_autor,metodo_pago,importe_centavos,fecha_pago,id_solicitud) VALUES (?,?,?,?,strftime('%Y-%m-%dT%H:%M:%fZ','now'),?)")
            .bind(order).bind(author).bind(&payment.method).bind(payment.amount_minor).bind(&payment.request_id).execute(db).await?.last_insert_rowid())
    }
    pub async fn load_any(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        include_deleted: bool,
    ) -> Result<Option<OrderResponse>, AppError> {
        let row=sqlx::query("SELECT o.*,coalesce(s.nombre,o.estado_personalizado) AS estado,b.cobro_neto_centavos,b.pendiente_centavos,b.saldo_favor_centavos,EXISTS(SELECT 1 FROM credenciales_dispositivos d WHERE d.id_orden=o.id) AS tiene_clave,e.id_cliente FROM ordenes o JOIN equipos e ON e.id=o.id_equipo LEFT JOIN estados_orden s ON s.id=o.id_estado JOIN vista_saldos_orden b ON b.id_orden=o.id WHERE o.id=? AND (? OR o.eliminada_en IS NULL)").bind(id).bind(include_deleted).fetch_optional(&mut *db).await?;
        let Some(r) = row else {
            return Ok(None);
        };
        let client_id = r.try_get("id_cliente")?;
        let equipment_id = r.try_get("id_equipo")?;
        let client = self
            .client(db, client_id)
            .await?
            .ok_or(AppError::NotFound)?;
        let (_, equipment) = self
            .equipment(db, equipment_id)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(Some(OrderResponse {
            id: r.try_get("id")?,
            folio: r.try_get("folio")?,
            client_id,
            equipment_id,
            received_by: r.try_get("id_creador")?,
            status: r.try_get("estado")?,
            currency: r.try_get("codigo_moneda")?,
            received_at: r.try_get("fecha_recepcion")?,
            estimated_delivery_at: r.try_get("fecha_estimada_entrega")?,
            reported_failure: r.try_get("falla")?,
            planned_work: r.try_get("trabajo")?,
            reception_notes: r.try_get("notas_recepcion")?,
            price_minor: r.try_get("precio_centavos")?,
            net_minor: r.try_get("cobro_neto_centavos")?,
            pending_minor: r.try_get("pendiente_centavos")?,
            credit_minor: r.try_get("saldo_favor_centavos")?,
            has_device_password: r.try_get("tiene_clave")?,
            device_password: None,
            client,
            equipment,
            history: EquipmentHistory {
                notes: r.try_get("antecedentes_equipo")?,
                previous_workshop: r.try_get("paso_otro_taller")?,
                previous_workshop_detail: r.try_get("detalle_otro_taller")?,
                suffered_impact: r.try_get("sufrio_golpes")?,
                impact_detail: r.try_get("detalle_golpes")?,
                liquid_spill: r.try_get("derrame_liquidos")?,
                liquid_detail: r.try_get("detalle_liquidos")?,
            },
            revision: r.try_get("revision")?,
            delivered: r.try_get("entregado")?,
            delivered_at: r.try_get("fecha_entrega")?,
        }))
    }
}
