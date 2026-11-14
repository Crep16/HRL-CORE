use crate::access::{self, AccessState};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;
use tauri_plugin_sql::DbInstances;

#[cfg(test)]
#[path = "orders_deletion_tests.rs"]
mod deletion_tests;
#[cfg(test)]
#[path = "orders_tests.rs"]
mod tests;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    id: String,
    #[serde(default = "default_currency")]
    currency: String,
    client: String,
    phone: String,
    received: String,
    status: String,
    serial: String,
    item: String,
    brand: String,
    model: String,
    other_shop: bool,
    #[serde(default)]
    other_shop_reason: String,
    dropped: bool,
    drop_reason: String,
    password: String,
    #[serde(default)]
    failure: String,
    work: String,
    cost: String,
    deposit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deposit_method: Option<String>,
    notes: String,
    paid: bool,
    delivered: bool,
}

fn default_currency() -> String {
    "MXN".into()
}

fn stored_order(row:&sqlx::sqlite::SqliteRow)->Result<Order,String> {
    let raw:Option<String>=row.get("datos_json");
    let mut order:Order=serde_json::from_str(raw.as_deref().ok_or("Hay una orden incompleta que necesita revisión manual.")?)
        .map_err(|_|"Hay una orden ilegible. No se han modificado los datos.")?;
    order.failure=row.get("falla");
    validate(&order)?;
    Ok(order)
}

pub(crate) async fn repair_order(pool:&SqlitePool,state:&AccessState,order_id:&str)->Result<Order,String> {
    let mut tx=pool.begin().await.map_err(|_|"No se pudo abrir la base local.")?;
    state.check_scoped(&mut tx,"repairs.view",Some(order_id)).await?;
    let row=sqlx::query("SELECT datos_json,falla FROM Ordenes WHERE id=? AND coalesce((SELECT accion FROM Historial_papelera WHERE id_orden=Ordenes.id ORDER BY secuencia DESC LIMIT 1),'restored')<>'deleted'")
        .bind(order_id).fetch_optional(&mut *tx).await.map_err(|_|"No se pudo consultar la orden autorizada.")?.ok_or("La orden no existe o está en la papelera.")?;
    let mut order=stored_order(&row)?;order.paid=payment_state(&mut tx,order_id).await?.paid;
    Ok(order)
}
#[tauri::command]
pub async fn load_authorized_repair_order(order_id:String,instances:State<'_,DbInstances>,state:State<'_,AccessState>)->Result<Order,String> {
    repair_order(&access::pool(&instances).await?,&state,&order_id).await
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentState {
    pub total_minor: i64,
    pub collected_minor: i64,
    pub received_minor: i64,
    pub adjusted_minor: i64,
    pub refunded_minor: i64,
    pub credit_minor: i64,
    pub paid: bool,
}
pub(crate) async fn payment_state(db: &mut sqlx::SqliteConnection, id: &str) -> Result<PaymentState, String> {
    let row = sqlx::query("SELECT o.precio_centavos,t.* FROM Ordenes o JOIN Vista_cobros_orden t ON t.id=o.id WHERE o.id=?")
        .bind(id).fetch_one(db).await.map_err(|_| "No se pudo consultar el estado real del pago.")?;
    let total_minor:i64 = row.get("precio_centavos");
    let collected_minor:i64 = row.get("cobro_neto_centavos");
    Ok(PaymentState { total_minor, collected_minor, received_minor:row.get("cobrado_centavos"), adjusted_minor:row.get("ajustado_centavos"), refunded_minor:row.get("devuelto_centavos"), credit_minor:(collected_minor-total_minor).max(0), paid:collected_minor >= total_minor })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderList {
    orders: Vec<Order>,
    legacy_imported: bool,
}

fn money_minor(text: &str) -> Result<i64, String> {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-'))
        .collect();
    if cleaned.is_empty() || cleaned.contains('-') {
        return Err("Revisa el precio y el abono; deben ser importes positivos o cero.".into());
    }
    let separator = cleaned.rfind(['.', ',']);
    let (whole, fraction) = match separator {
        Some(index) if (1..=2).contains(&(cleaned.len() - index - 1)) => {
            (&cleaned[..index], &cleaned[index + 1..])
        }
        _ => (cleaned.as_str(), ""),
    };
    let whole: String = whole.chars().filter(char::is_ascii_digit).collect();
    let whole = if whole.is_empty() {
        0
    } else {
        whole
            .parse::<i64>()
            .map_err(|_| "Importe demasiado grande.")?
    };
    let fraction = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<i64>().map_err(|_| "Importe inválido.")? * 10,
        _ => fraction.parse::<i64>().map_err(|_| "Importe inválido.")?,
    };
    whole
        .checked_mul(100)
        .and_then(|n| n.checked_add(fraction))
        .filter(|n| *n <= 9_000_000_000_000)
        .ok_or("Importe demasiado grande.".into())
}

fn validate(order: &Order) -> Result<(i64, i64), String> {
    if !["MXN", "USD", "EUR"].contains(&order.currency.as_str()) {
        return Err("Selecciona una moneda válida.".into());
    }
    if order.id.trim().is_empty()
        || order.id != order.id.trim()
        || order.id.len() > 100
        || order.client.trim().is_empty()
        || order.phone.trim().is_empty()
        || order.item.trim().is_empty()
        || order.status.trim().is_empty()
    {
        return Err("Completa folio, cliente, teléfono, artículo y estado.".into());
    }
    if [
        &order.client,
        &order.phone,
        &order.received,
        &order.status,
        &order.serial,
        &order.item,
        &order.brand,
        &order.model,
        &order.other_shop_reason,
        &order.drop_reason,
        &order.password,
        &order.failure,
        &order.work,
        &order.notes,
    ]
    .iter()
    .any(|value| value.len() > 16000)
    {
        return Err("Un campo de la orden supera el tamaño permitido.".into());
    }
    Ok((money_minor(&order.cost)?, money_minor(&order.deposit)?))
}

#[cfg(test)]
pub(crate) async fn list(pool: &SqlitePool, state: &AccessState) -> Result<OrderList, String> {
    list_view(pool, state, false).await
}

pub(crate) async fn list_view(
    pool: &SqlitePool,
    state: &AccessState,
    deleted: bool,
) -> Result<OrderList, String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let can_view = state.check_scoped(&mut tx, "clients.view",None).await.is_ok();
    if deleted {
        state.check_scoped(&mut tx, "clients.view",None).await?;
        state.check_scoped(&mut tx, "orders.delete",None).await?;
    }
    if !can_view {
        state.check_scoped(&mut tx, "orders.create",None).await?;
    }
    let rows = if can_view {
        sqlx::query("SELECT datos_json,falla FROM Ordenes WHERE coalesce((SELECT accion FROM Historial_papelera WHERE id_orden=Ordenes.id ORDER BY secuencia DESC LIMIT 1),'restored')=? ORDER BY id")
            .bind(if deleted { "deleted" } else { "restored" })
            .fetch_all(&mut *tx)
            .await
            .map_err(|_| "No se pudieron consultar las órdenes.")?
    } else {
        Vec::new()
    };
    let mut orders = Vec::new();
    for row in rows {
        let mut order = stored_order(&row)?;
        order.paid = payment_state(&mut tx, &order.id).await?.paid;
        orders.push(order);
    }
    let imported: i64 =
        sqlx::query_scalar("SELECT count(*) FROM Importaciones_anteriores WHERE origen='orders.v1'")
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| "No se pudo consultar la importación.")?;
    tx.commit()
        .await
        .map_err(|_| "No se pudo completar la consulta.")?;
    Ok(OrderList {
        orders,
        legacy_imported: imported > 0,
    })
}

async fn write(
    db: &mut sqlx::SqliteConnection,
    order: &Order,
    original: Option<&Order>,
    author: &str,
    imported: bool,
) -> Result<(), String> {
    let (price, deposit) = validate(order)?;
    let json = serde_json::to_string(order).map_err(|_| "No se pudo preparar la orden.")?;
    let removed: i64 = sqlx::query_scalar("SELECT count(*) FROM Historial_papelera WHERE secuencia=(SELECT max(secuencia) FROM Historial_papelera WHERE id_orden=?) AND accion='deleted'")
        .bind(&order.id).fetch_one(&mut *db).await.map_err(|_| "No se pudo comprobar el estado de la orden.")?;
    if removed > 0 {
        return Err(
            "La orden está en la papelera. Restáurala antes de editarla; su folio se conserva."
                .into(),
        );
    }
    let existing =
        sqlx::query("SELECT id_equipo,datos_json,fecha_entrega,falla FROM Ordenes WHERE id=?")
            .bind(&order.id)
            .fetch_optional(&mut *db)
            .await
            .map_err(|_| "No se pudo consultar la orden.")?;
    if let Some(original) = original {
        if order.currency != original.currency {
            return Err("No se puede cambiar la moneda de una orden existente.".into());
        }
        let row = existing.ok_or("La orden ya no está disponible.")?;
        let mut saved = stored_order(&row)?;
        saved.paid = payment_state(db, &order.id).await?.paid;
        if &saved != original {
            return Err("La orden cambió desde que la abriste. Conserva el borrador y vuelve a consultar antes de guardar.".into());
        }
        if order.deposit != original.deposit || order.deposit_method != original.deposit_method || order.paid != original.paid {
            return Err(
                "Los pagos no se corrigen editando la orden. Conserva el importe registrado."
                    .into(),
            );
        }
        let equipment: String = row.get("id_equipo");
        let previous_delivery: Option<String> = row.get("fecha_entrega");
        sqlx::query("UPDATE Clientes SET nombre=?,telefono=? WHERE id=(SELECT id_cliente FROM Equipo WHERE id=?)")
            .bind(&order.client).bind(&order.phone).bind(&equipment).execute(&mut *db).await.map_err(|_| "No se pudo actualizar el cliente.")?;
        sqlx::query("UPDATE Equipo SET articulo=?,marca=?,modelo=?,serie=? WHERE id=?")
            .bind(&order.item)
            .bind(&order.brand)
            .bind(&order.model)
            .bind(&order.serial)
            .bind(equipment)
            .execute(&mut *db)
            .await
            .map_err(|_| "No se pudo actualizar el equipo.")?;
        sqlx::query("UPDATE Ordenes SET fecha_recepcion=?,estado=?,falla=?,trabajo=?,precio_centavos=?,paso_otro_taller=?,detalle_otro_taller=?,caida_humedad=?,detalle_caida_humedad=?,fecha_entrega=CASE WHEN ? THEN fecha_entrega WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,datos_json=? WHERE id=?")
            .bind(&order.received).bind(&order.status).bind(&order.failure).bind(&order.work).bind(price).bind(order.other_shop).bind(&order.other_shop_reason).bind(order.dropped).bind(&order.drop_reason).bind(order.delivered == original.delivered).bind(order.delivered).bind(json).bind(&order.id)
            .execute(&mut *db).await.map_err(|_| "No se pudo actualizar la orden.")?;
        let old =
            serde_json::to_value(original).map_err(|_| "No se pudo preparar el historial.")?;
        let new = serde_json::to_value(order).map_err(|_| "No se pudo preparar el historial.")?;
        for (key, field) in [
            ("client", "client"),
            ("phone", "phone"),
            ("received", "received"),
            ("status", "status"),
            ("serial", "serial"),
            ("item", "item"),
            ("brand", "brand"),
            ("model", "model"),
            ("work", "work"),
            ("failure", "failure"),
            ("cost", "price_minor"),
            ("otherShop", "other_shop"),
            ("otherShopReason", "other_shop_reason"),
            ("dropped", "dropped"),
            ("dropReason", "drop_reason"),
        ] {
            if old[key] != new[key] {
                let (previous, next) = if key == "cost" {
                    (money_minor(&original.cost)?.to_string(), price.to_string())
                } else {
                    (old[key].to_string(), new[key].to_string())
                };
                audit(db, &order.id, author, field, Some(previous), next).await?;
            }
        }
        if original.delivered != order.delivered {
            let next: Option<String> =
                sqlx::query_scalar("SELECT fecha_entrega FROM Ordenes WHERE id=?")
                    .bind(&order.id)
                    .fetch_one(&mut *db)
                    .await
                    .map_err(|_| "No se pudo consultar la entrega.")?;
            audit(
                db,
                &order.id,
                author,
                "delivered_at",
                previous_delivery,
                next.unwrap_or_else(|| "null".into()),
            )
            .await?;
        }
        crate::logbook::order_notes(db, &order.id, author, &original.notes, &order.notes).await?;
    } else {
        if existing.is_some() {
            return Err("El folio ya existe. Conserva el borrador y elige otro folio.".into());
        }
        let client = access::random_id();
        let equipment = access::random_id();
        sqlx::query("INSERT INTO Clientes(id,nombre,telefono) VALUES (?,?,?)")
            .bind(&client)
            .bind(&order.client)
            .bind(&order.phone)
            .execute(&mut *db)
            .await
            .map_err(|_| "No se pudo guardar el cliente.")?;
        sqlx::query(
            "INSERT INTO Equipo(id,id_cliente,articulo,marca,modelo,serie) VALUES (?,?,?,?,?,?)",
        )
        .bind(&equipment)
        .bind(client)
        .bind(&order.item)
        .bind(&order.brand)
        .bind(&order.model)
        .bind(&order.serial)
        .execute(&mut *db)
        .await
        .map_err(|_| "No se pudo guardar el equipo.")?;
        sqlx::query("INSERT INTO Ordenes(id,id_equipo,id_creador,fecha_recepcion,estado,falla,trabajo,precio_centavos,paso_otro_taller,detalle_otro_taller,caida_humedad,detalle_caida_humedad,fecha_entrega,datos_json,moneda) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,CASE WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,?,?)")
            .bind(&order.id).bind(equipment).bind(author).bind(&order.received).bind(&order.status).bind(&order.failure).bind(&order.work).bind(price).bind(order.other_shop).bind(&order.other_shop_reason).bind(order.dropped).bind(&order.drop_reason).bind(order.delivered).bind(json).bind(&order.currency)
            .execute(&mut *db).await.map_err(|_| "No se pudo guardar la orden.")?;
        if deposit > 0 {
            sqlx::query("INSERT INTO Abonos(id,id_orden,id_autor,importe_centavos,fecha_evento,metodo_pago) VALUES (?,?,?,?,CASE WHEN ? THEN '' ELSE strftime('%Y-%m-%dT%H:%M:%fZ','now') END,?)")
                .bind(access::random_id()).bind(&order.id).bind(author).bind(deposit).bind(imported).bind(if imported {None} else {order.deposit_method.as_deref()}).execute(&mut *db).await.map_err(|_| "No se pudo guardar el abono inicial.")?;
        }
        audit(db, &order.id, author, "created", None, order.id.clone()).await?;
    }
    Ok(())
}

async fn audit(
    db: &mut sqlx::SqliteConnection,
    order: &str,
    author: &str,
    field: &str,
    old: Option<String>,
    new: String,
) -> Result<(), String> {
    sqlx::query("INSERT INTO Historial_ordenes(id,id_orden,id_autor,campo,valor_anterior,valor_nuevo) VALUES (?,?,?,?,?,?)")
        .bind(access::random_id()).bind(order).bind(author).bind(field).bind(old).bind(new).execute(db).await.map_err(|_| "No se pudo guardar el historial; no se confirmó ningún cambio de la orden.")?;
    Ok(())
}

pub(crate) async fn save(
    pool: &SqlitePool,
    state: &AccessState,
    order: Order,
    original: Option<Order>,
) -> Result<Order, String> {
    let (_, deposit) = validate(&order)?;
    if original.is_none()&&order.failure.trim().is_empty(){return Err("Describe la falla reportada antes de guardar la nueva orden.".into());}
    if original.as_ref().is_some_and(|old| old.id != order.id) {
        return Err("No se puede cambiar el folio al editar.".into());
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let user = state
        .authorize_scoped(
            &mut tx,
            if original.is_some() {
                "orders.edit"
            } else {
                "orders.create"
            },
            None,
        )
        .await?;
    if original.is_none() && (deposit > 0 || order.paid) {
        state.authorize_scoped(&mut tx, "payments.add",None).await?;
    }
    if original.is_none() && deposit > 0 && order.deposit_method.as_ref().is_none_or(|s|s.trim().is_empty()||s.len()>200) {
        return Err("Escribe el método del abono inicial (hasta 200 caracteres).".into());
    }
    write(&mut tx, &order, original.as_ref(), &user.id, false).await?;
    let mut saved = order;
    saved.paid = payment_state(&mut tx, &saved.id).await?.paid;
    sqlx::query("UPDATE Ordenes SET datos_json=? WHERE id=?")
        .bind(serde_json::to_string(&saved).map_err(|_| "No se pudo preparar la orden guardada.")?).bind(&saved.id)
        .execute(&mut *tx).await.map_err(|_| "No se pudo confirmar el estado del pago.")?;
    tx.commit()
        .await
        .map_err(|_| "No se pudo confirmar la orden. Conserva el borrador.")?;
    Ok(saved)
}

pub(crate) async fn import(
    pool: &SqlitePool,
    state: &AccessState,
    orders: Vec<Order>,
) -> Result<(), String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let user = state.authorize(&mut tx, "users.manage").await?;
    let occupied: i64 = sqlx::query_scalar("SELECT (SELECT count(*) FROM Ordenes)+(SELECT count(*) FROM Importaciones_anteriores WHERE origen='orders.v1')").fetch_one(&mut *tx).await.map_err(|_| "No se pudo comprobar la importación.")?;
    if occupied > 0 {
        return Err("La base ya tiene órdenes o ya fue importada. Se requiere revisión manual para combinar registros sin sobrescribirlos.".into());
    }
    for order in orders {
        write(&mut tx, &order, None, &user.id, true).await?;
        sqlx::query("UPDATE Ordenes SET fecha_entrega=NULL WHERE id=?")
            .bind(&order.id)
            .execute(&mut *tx)
            .await
            .map_err(|_| "No se pudo conservar la entrega histórica.")?;
    }
    sqlx::query("INSERT INTO Importaciones_anteriores(origen,id_importador) VALUES ('orders.v1',?)")
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(|_| "No se pudo registrar la importación.")?;
    tx.commit().await.map_err(
        |_| "No se pudo confirmar la importación. Los originales siguen en este equipo.",
    )?;
    Ok(())
}

#[tauri::command]
pub async fn set_order_deleted(
    order: Order,
    deleted: bool,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<(), String> {
    set_deleted(&access::pool(&instances).await?, &state, order, deleted).await
}

pub(crate) async fn set_deleted(
    pool: &SqlitePool,
    state: &AccessState,
    order: Order,
    deleted: bool,
) -> Result<(), String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let user = state.authorize_scoped(&mut tx, "orders.delete",None).await?;
    let saved = sqlx::query("SELECT datos_json,falla FROM Ordenes WHERE id=?")
        .bind(&order.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| "No se pudo consultar la orden.")?
        .ok_or("La orden ya no está disponible o necesita revisión manual.")?;
    let mut saved = stored_order(&saved)?;
    saved.paid = payment_state(&mut tx, &order.id).await?.paid;
    if saved != order {
        return Err("La orden cambió desde que la abriste. Vuelve a consultarla antes de eliminarla o restaurarla.".into());
    }
    let action: Option<String> = sqlx::query_scalar("SELECT accion FROM Historial_papelera WHERE id_orden=? ORDER BY secuencia DESC LIMIT 1").bind(&order.id).fetch_optional(&mut *tx).await.map_err(|_| "No se pudo consultar la papelera.")?;
    if deleted == (action.as_deref() == Some("deleted")) {
        return Ok(());
    }
    sqlx::query("INSERT INTO Historial_papelera(id_orden,id_autor,accion) VALUES (?,?,?)").bind(&order.id).bind(&user.id).bind(if deleted { "deleted" } else { "restored" })
        .execute(&mut *tx).await.map_err(|_| "No se pudo guardar el cambio en la papelera. La orden conserva su estado anterior.")?;
    tx.commit()
        .await
        .map_err(|_| "No se pudo confirmar el cambio de la orden.")?;
    Ok(())
}

#[tauri::command]
pub async fn next_order_number(
    prefix: String,
    start: i64,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<i64, String> {
    if prefix.is_empty()
        || prefix.chars().count() > 12
        || !prefix
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
        || !(1..=999999999).contains(&start)
    {
        return Err("Revisa el prefijo y el siguiente número de orden.".into());
    }
    let pool = access::pool(&instances).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo consultar el siguiente folio.")?;
    state.check_scoped(&mut tx, "orders.create",None).await?;
    let mut number = start;
    loop {
        if number > 999999999 {
            return Err("Se agotó el rango de folios configurado.".into());
        }
        let occupied: i64 = sqlx::query_scalar("SELECT count(*) FROM Ordenes WHERE id=?")
            .bind(format!("{prefix}{number:03}"))
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| "No se pudo consultar el siguiente folio.")?;
        if occupied == 0 {
            return Ok(number);
        }
        number += 1;
    }
}

#[tauri::command]
pub async fn list_orders(
    deleted: Option<bool>,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<OrderList, String> {
    list_view(
        &access::pool(&instances).await?,
        &state,
        deleted.unwrap_or(false),
    )
    .await
}
#[tauri::command]
pub async fn save_order(
    order: Order,
    original: Option<Order>,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<Order, String> {
    save(&access::pool(&instances).await?, &state, order, original).await
}
#[tauri::command]
pub async fn import_orders(
    orders: Vec<Order>,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<(), String> {
    import(&access::pool(&instances).await?, &state, orders).await
}
