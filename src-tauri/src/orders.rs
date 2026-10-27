use crate::access::{self, AccessState};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;
use tauri_plugin_sql::DbInstances;

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
    work: String,
    cost: String,
    deposit: String,
    notes: String,
    paid: bool,
    delivered: bool,
}

fn default_currency() -> String {
    "MXN".into()
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

pub(crate) async fn list(pool: &SqlitePool, state: &AccessState) -> Result<OrderList, String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let can_view = state.authorize(&mut tx, "clients.view").await.is_ok();
    if !can_view {
        state.authorize(&mut tx, "orders.create").await?;
    }
    let rows = if can_view {
        sqlx::query("SELECT record_json FROM orders ORDER BY id")
            .fetch_all(&mut *tx)
            .await
            .map_err(|_| "No se pudieron consultar las órdenes.")?
    } else {
        Vec::new()
    };
    let mut orders = Vec::new();
    for row in rows {
        let raw: Option<String> = row.get("record_json");
        let order: Order = serde_json::from_str(
            raw.as_deref()
                .ok_or("Hay una orden incompleta que necesita revisión manual.")?,
        )
        .map_err(|_| "Hay una orden ilegible. No se han modificado los datos.")?;
        validate(&order)?;
        orders.push(order);
    }
    let imported: i64 =
        sqlx::query_scalar("SELECT count(*) FROM legacy_imports WHERE source='orders.v1'")
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
) -> Result<(), String> {
    let (price, deposit) = validate(order)?;
    let json = serde_json::to_string(order).map_err(|_| "No se pudo preparar la orden.")?;
    let existing =
        sqlx::query("SELECT equipment_id,record_json,delivered_at FROM orders WHERE id=?")
            .bind(&order.id)
            .fetch_optional(&mut *db)
            .await
            .map_err(|_| "No se pudo consultar la orden.")?;
    if let Some(original) = original {
        if order.currency != original.currency {
            return Err("No se puede cambiar la moneda de una orden existente.".into());
        }
        let row = existing.ok_or("La orden ya no está disponible.")?;
        let saved: Order = serde_json::from_str(&row.get::<String, _>("record_json"))
            .map_err(|_| "La orden guardada necesita revisión manual.")?;
        if &saved != original {
            return Err("La orden cambió desde que la abriste. Conserva el borrador y vuelve a consultar antes de guardar.".into());
        }
        if order.deposit != original.deposit || order.paid != original.paid {
            return Err(
                "Los pagos no se corrigen editando la orden. Conserva el importe registrado."
                    .into(),
            );
        }
        if price < money_minor(&original.cost)? {
            return Err("Para reducir el precio hace falta revisar los cobros y devoluciones de esta orden. El precio anterior se ha conservado.".into());
        }
        let equipment: String = row.get("equipment_id");
        let previous_delivery: Option<String> = row.get("delivered_at");
        sqlx::query("UPDATE clients SET name=?,phone=? WHERE id=(SELECT client_id FROM equipment WHERE id=?)")
            .bind(&order.client).bind(&order.phone).bind(&equipment).execute(&mut *db).await.map_err(|_| "No se pudo actualizar el cliente.")?;
        sqlx::query("UPDATE equipment SET item=?,brand=?,model=?,serial=? WHERE id=?")
            .bind(&order.item)
            .bind(&order.brand)
            .bind(&order.model)
            .bind(&order.serial)
            .bind(equipment)
            .execute(&mut *db)
            .await
            .map_err(|_| "No se pudo actualizar el equipo.")?;
        sqlx::query("UPDATE orders SET received=?,status=?,work=?,price_minor=?,other_shop=?,other_shop_reason=?,dropped=?,drop_reason=?,delivered_at=CASE WHEN ? THEN delivered_at WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,record_json=? WHERE id=?")
            .bind(&order.received).bind(&order.status).bind(&order.work).bind(price).bind(order.other_shop).bind(&order.other_shop_reason).bind(order.dropped).bind(&order.drop_reason).bind(order.delivered == original.delivered).bind(order.delivered).bind(json).bind(&order.id)
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
                sqlx::query_scalar("SELECT delivered_at FROM orders WHERE id=?")
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
    } else {
        if existing.is_some() {
            return Err("El folio ya existe. Conserva el borrador y elige otro folio.".into());
        }
        let client = access::random_id();
        let equipment = access::random_id();
        sqlx::query("INSERT INTO clients(id,name,phone) VALUES (?,?,?)")
            .bind(&client)
            .bind(&order.client)
            .bind(&order.phone)
            .execute(&mut *db)
            .await
            .map_err(|_| "No se pudo guardar el cliente.")?;
        sqlx::query(
            "INSERT INTO equipment(id,client_id,item,brand,model,serial) VALUES (?,?,?,?,?,?)",
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
        sqlx::query("INSERT INTO orders(id,equipment_id,created_by,received,status,work,price_minor,other_shop,other_shop_reason,dropped,drop_reason,delivered_at,record_json,currency) VALUES (?,?,?,?,?,?,?,?,?,?,?,CASE WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,?,?)")
            .bind(&order.id).bind(equipment).bind(author).bind(&order.received).bind(&order.status).bind(&order.work).bind(price).bind(order.other_shop).bind(&order.other_shop_reason).bind(order.dropped).bind(&order.drop_reason).bind(order.delivered).bind(json).bind(&order.currency)
            .execute(&mut *db).await.map_err(|_| "No se pudo guardar la orden.")?;
        if deposit > 0 {
            sqlx::query("INSERT INTO payments(id,order_id,author_id,amount_minor,occurred_at) VALUES (?,?,?,?,strftime('%Y-%m-%dT%H:%M:%fZ','now'))")
                .bind(access::random_id()).bind(&order.id).bind(author).bind(deposit).execute(&mut *db).await.map_err(|_| "No se pudo guardar el abono inicial.")?;
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
    sqlx::query("INSERT INTO audit_events(id,order_id,author_id,field,old_value,new_value) VALUES (?,?,?,?,?,?)")
        .bind(access::random_id()).bind(order).bind(author).bind(field).bind(old).bind(new).execute(db).await.map_err(|_| "No se pudo guardar el historial; no se confirmó ningún cambio de la orden.")?;
    Ok(())
}

pub(crate) async fn save(
    pool: &SqlitePool,
    state: &AccessState,
    order: Order,
    original: Option<Order>,
) -> Result<(), String> {
    let (_, deposit) = validate(&order)?;
    if original.as_ref().is_some_and(|old| old.id != order.id) {
        return Err("No se puede cambiar el folio al editar.".into());
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let user = state
        .authorize(
            &mut tx,
            if original.is_some() {
                "orders.edit"
            } else {
                "orders.create"
            },
        )
        .await?;
    if original.is_none() && (deposit > 0 || order.paid) {
        state.authorize(&mut tx, "payments.add").await?;
    }
    write(&mut tx, &order, original.as_ref(), &user.id).await?;
    tx.commit()
        .await
        .map_err(|_| "No se pudo confirmar la orden. Conserva el borrador.")?;
    Ok(())
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
    let occupied: i64 = sqlx::query_scalar("SELECT (SELECT count(*) FROM orders)+(SELECT count(*) FROM legacy_imports WHERE source='orders.v1')").fetch_one(&mut *tx).await.map_err(|_| "No se pudo comprobar la importación.")?;
    if occupied > 0 {
        return Err("La base ya tiene órdenes o ya fue importada. Se requiere revisión manual para combinar registros sin sobrescribirlos.".into());
    }
    for order in orders {
        write(&mut tx, &order, None, &user.id).await?;
        sqlx::query("UPDATE payments SET occurred_at='' WHERE order_id=?")
            .bind(&order.id)
            .execute(&mut *tx)
            .await
            .map_err(|_| "No se pudo conservar el abono histórico.")?;
        sqlx::query("UPDATE orders SET delivered_at=NULL WHERE id=?")
            .bind(&order.id)
            .execute(&mut *tx)
            .await
            .map_err(|_| "No se pudo conservar la entrega histórica.")?;
    }
    sqlx::query("INSERT INTO legacy_imports(source,imported_by) VALUES ('orders.v1',?)")
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
    state.authorize(&mut tx, "orders.create").await?;
    let mut number = start;
    loop {
        if number > 999999999 {
            return Err("Se agotó el rango de folios configurado.".into());
        }
        let occupied: i64 = sqlx::query_scalar("SELECT count(*) FROM orders WHERE id=?")
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
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<OrderList, String> {
    list(&access::pool(&instances).await?, &state).await
}
#[tauri::command]
pub async fn save_order(
    order: Order,
    original: Option<Order>,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<(), String> {
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
