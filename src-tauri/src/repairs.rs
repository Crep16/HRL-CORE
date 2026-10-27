use crate::access::{self, AccessState};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;
use tauri_plugin_sql::DbInstances;

#[cfg(test)]
#[path = "repairs_tests.rs"]
mod tests;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Event {
    id: i64,
    date: String,
    title: String,
    detail: String,
    stage: String,
    public: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Warranty {
    name: String,
    days: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Part {
    name: String,
    price: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Payment {
    date: String,
    amount: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Visibility {
    label: String,
    enabled: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Repair {
    completed: u8,
    technician: String,
    public_note: String,
    internal_note: String,
    sync_mode: String,
    events: Vec<Event>,
    warranties: Vec<Warranty>,
    parts: Vec<Part>,
    payments: Vec<Payment>,
    visibility: Vec<Visibility>,
}
impl Default for Repair {
    fn default() -> Self {
        Self {
            completed: 0,
            technician: String::new(),
            public_note: String::new(),
            internal_note: String::new(),
            sync_mode: "manual".into(),
            events: vec![],
            warranties: vec![],
            parts: vec![],
            payments: vec![],
            visibility: vec![
                Visibility {
                    label: "Línea de tiempo".into(),
                    enabled: true,
                },
                Visibility {
                    label: "Garantías".into(),
                    enabled: true,
                },
                Visibility {
                    label: "Resumen de pago".into(),
                    enabled: false,
                },
                Visibility {
                    label: "Notas del técnico".into(),
                    enabled: true,
                },
            ],
        }
    }
}
#[derive(Serialize)]
pub struct Snapshot {
    repair: Repair,
    revision: i64,
}

fn validate(repair: &Repair) -> Result<(), String> {
    if repair.completed > 7 || !["manual", "auto"].contains(&repair.sync_mode.as_str()) {
        return Err("El seguimiento tiene un avance o modo inválido.".into());
    }
    if [
        &repair.technician,
        &repair.public_note,
        &repair.internal_note,
    ]
    .iter()
    .any(|s| s.len() > 64000)
        || repair.events.iter().any(|e| {
            e.id < 0
                || e.title.trim().is_empty()
                || e.title.len() > 16000
                || e.detail.len() > 64000
                || e.date.len() > 100
                || e.stage.len() > 100
        })
        || repair
            .warranties
            .iter()
            .any(|w| w.name.trim().is_empty() || w.days < 0)
        || repair
            .parts
            .iter()
            .any(|p| p.name.trim().is_empty() || !p.price.is_finite() || p.price < 0.)
        || repair
            .payments
            .iter()
            .any(|p| !p.amount.is_finite() || p.amount <= 0.)
    {
        return Err("Revisa los campos y los importes del seguimiento.".into());
    }
    Ok(())
}
async fn active_order(db: &mut sqlx::SqliteConnection, id: &str) -> Result<i64, String> {
    let row = sqlx::query("SELECT price_minor FROM orders WHERE id=? AND coalesce((SELECT action FROM order_visibility_events WHERE order_id=orders.id ORDER BY sequence DESC LIMIT 1),'restored')<>'deleted'")
        .bind(id).fetch_optional(db).await.map_err(|_|"No se pudo consultar la orden.")?.ok_or("La orden no está disponible. Si está en la papelera, debe restaurarse primero.")?;
    Ok(row.get("price_minor"))
}
async fn read(db: &mut sqlx::SqliteConnection, id: &str) -> Result<Snapshot, String> {
    let row = sqlx::query("SELECT record_json,revision FROM repair_records WHERE order_id=?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|_| "No se pudo recuperar el seguimiento.")?;
    if let Some(row) = row {
        let repair = serde_json::from_str(&row.get::<String, _>("record_json")).map_err(
            |_| "El seguimiento necesita revisión manual; los datos no se han modificado.",
        )?;
        validate(&repair)?;
        Ok(Snapshot {
            repair,
            revision: row.get("revision"),
        })
    } else {
        Ok(Snapshot {
            repair: Repair::default(),
            revision: 0,
        })
    }
}
async fn change(
    db: &mut sqlx::SqliteConnection,
    id: &str,
    author: &str,
    operation: &str,
) -> Result<(), String> {
    sqlx::query("INSERT INTO repair_changes(order_id,author_id,operation) VALUES (?,?,?)")
        .bind(id)
        .bind(author)
        .bind(operation)
        .execute(db)
        .await
        .map_err(|_| "No se pudo guardar la autoría del cambio.")?;
    Ok(())
}
async fn append_records(
    db: &mut sqlx::SqliteConnection,
    id: &str,
    author: &str,
    old: &Repair,
    new: &Repair,
) -> Result<(), String> {
    for event in &new.events[..new.events.len() - old.events.len()] {
        sqlx::query("INSERT INTO repair_events(id,order_id,author_id,stage,title,detail,public,occurred_at) VALUES (?,?,?,?,?,?,?,?)")
            .bind(access::random_id()).bind(id).bind(author).bind(&event.stage).bind(&event.title).bind(&event.detail).bind(event.public).bind(&event.date).execute(&mut *db).await.map_err(|_|"No se pudo guardar el avance.")?;
    }
    for (previous, next, public) in [
        (&old.public_note, &new.public_note, true),
        (&old.internal_note, &new.internal_note, false),
    ] {
        if next != previous {
            let body = next.strip_prefix(previous.as_str()).unwrap_or(next).trim();
            if !body.is_empty() {
                sqlx::query("INSERT INTO repair_notes(id,order_id,author_id,body,public) VALUES (?,?,?,?,?)").bind(access::random_id()).bind(id).bind(author).bind(body).bind(public).execute(&mut *db).await.map_err(|_|"No se pudo guardar la nota.")?;
            }
        }
    }
    for warranty in &new.warranties[old.warranties.len()..] {
        sqlx::query(
            "INSERT INTO warranties(id,order_id,author_id,coverage,days) VALUES (?,?,?,?,?)",
        )
        .bind(access::random_id())
        .bind(id)
        .bind(author)
        .bind(&warranty.name)
        .bind(warranty.days)
        .execute(&mut *db)
        .await
        .map_err(|_| "No se pudo guardar la garantía.")?;
    }
    for part in &new.parts[old.parts.len()..] {
        sqlx::query(
            "INSERT INTO order_parts(id,order_id,author_id,name,price_minor) VALUES (?,?,?,?,?)",
        )
        .bind(access::random_id())
        .bind(id)
        .bind(author)
        .bind(&part.name)
        .bind(minor(part.price)?)
        .execute(&mut *db)
        .await
        .map_err(|_| "No se pudo guardar la refacción.")?;
    }
    for payment in &new.payments[old.payments.len()..] {
        sqlx::query("INSERT INTO payments(id,order_id,author_id,amount_minor,occurred_at) VALUES (?,?,?,?,?)").bind(access::random_id()).bind(id).bind(author).bind(minor(payment.amount)?).bind(&payment.date).execute(&mut *db).await.map_err(|_|"No se pudo guardar el abono.")?;
    }
    Ok(())
}
fn minor(amount: f64) -> Result<i64, String> {
    let cents = (amount * 100.).round();
    if !amount.is_finite()
        || !(0. ..=9_000_000_000_000.).contains(&cents)
        || (amount * 100. - cents).abs() > 0.0001
    {
        return Err("El importe admite hasta dos decimales.".into());
    }
    Ok(cents as i64)
}
pub(crate) async fn load(
    pool: &SqlitePool,
    state: &AccessState,
    id: &str,
    legacy: Option<Repair>,
) -> Result<Snapshot, String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let user = state.authorize(&mut tx, "repairs.view").await?;
    active_order(&mut tx, id).await?;
    let snapshot = read(&mut tx, id).await?;
    if snapshot.revision > 0 {
        return Ok(snapshot);
    }
    let Some(legacy) = legacy else {
        return Ok(snapshot);
    };
    validate(&legacy)?;
    if legacy == Repair::default() {
        return Ok(snapshot);
    }
    state.authorize(&mut tx,"users.manage").await.map_err(|_|"Un administrador debe importar el seguimiento anterior antes de continuar. Los datos originales se conservan.")?;
    append_records(&mut tx, id, &user.id, &Repair::default(), &legacy).await?;
    sqlx::query("INSERT INTO repair_records(order_id,record_json,revision) VALUES (?,?,1)")
        .bind(id)
        .bind(serde_json::to_string(&legacy).map_err(|_| "No se pudo preparar el seguimiento.")?)
        .execute(&mut *tx)
        .await
        .map_err(|_| "No se pudo importar el seguimiento.")?;
    change(&mut tx, id, &user.id, "imported").await?;
    tx.commit()
        .await
        .map_err(|_| "No se pudo confirmar la importación.")?;
    Ok(Snapshot {
        repair: legacy,
        revision: 1,
    })
}
pub(crate) async fn save(
    pool: &SqlitePool,
    state: &AccessState,
    id: &str,
    repair: Repair,
    revision: i64,
) -> Result<Snapshot, String> {
    validate(&repair)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let user = state.authorize(&mut tx, "repairs.view").await?;
    let price = active_order(&mut tx, id).await?;
    let previous = read(&mut tx, id).await?;
    if previous.revision != revision {
        return Err("El seguimiento cambió desde que lo abriste. Conserva tu borrador y vuelve a consultar la orden.".into());
    }
    let old = &previous.repair;
    let mut operations = Vec::new();
    if repair.completed != old.completed {
        operations.push(if repair.completed == old.completed + 1 {
            "stages.advance"
        } else {
            "repairs.edit"
        });
    }
    if repair.technician != old.technician {
        operations.push("repairs.edit");
    }
    for (a, b) in [
        (&old.public_note, &repair.public_note),
        (&old.internal_note, &repair.internal_note),
    ] {
        if a != b {
            operations.push(
                if b.starts_with(a.as_str()) && state.authorize(&mut tx, "notes.add").await.is_ok()
                {
                    "notes.add"
                } else {
                    "repairs.edit"
                },
            );
        }
    }
    if repair.events.len() < old.events.len()
        || !repair.events.ends_with(&old.events)
        || !repair.warranties.starts_with(&old.warranties)
        || !repair.parts.starts_with(&old.parts)
        || !repair.payments.starts_with(&old.payments)
    {
        return Err("No se permite reemplazar o borrar avances, garantías, refacciones o pagos ya registrados.".into());
    }
    if repair.events != old.events {
        operations.push("notes.add");
    }
    if repair.warranties != old.warranties {
        operations.push("warranties.add");
    }
    if repair.parts != old.parts {
        operations.push("parts.add");
    }
    if repair.payments != old.payments {
        operations.push("payments.add");
        let paid: i64 = sqlx::query_scalar(
            "SELECT coalesce(sum(amount_minor),0) FROM payments WHERE order_id=?",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| "No se pudieron consultar los pagos.")?;
        let added = repair.payments[old.payments.len()..]
            .iter()
            .try_fold(0_i64, |sum, p| {
                minor(p.amount).and_then(|amount| {
                    sum.checked_add(amount)
                        .ok_or("Importe demasiado grande.".into())
                })
            })?;
        if added > price.saturating_sub(paid) {
            return Err("El abono no puede superar el saldo pendiente.".into());
        }
    }
    if repair.sync_mode != old.sync_mode || repair.visibility != old.visibility {
        operations.push("publication.edit");
    }
    operations.sort_unstable();
    operations.dedup();
    for operation in &operations {
        state.authorize(&mut tx, operation).await?;
    }
    if operations.is_empty() {
        return Ok(previous);
    }
    append_records(&mut tx, id, &user.id, old, &repair).await?;
    let revision = revision
        .checked_add(1)
        .ok_or("El seguimiento necesita revisión manual.")?;
    sqlx::query("INSERT INTO repair_records(order_id,record_json,revision) VALUES (?,?,?) ON CONFLICT(order_id) DO UPDATE SET record_json=excluded.record_json,revision=excluded.revision")
        .bind(id).bind(serde_json::to_string(&repair).map_err(|_|"No se pudo preparar el seguimiento.")?).bind(revision).execute(&mut *tx).await.map_err(|_|"No se pudo guardar el seguimiento.")?;
    for operation in operations {
        change(&mut tx, id, &user.id, operation).await?;
    }
    tx.commit()
        .await
        .map_err(|_| "No se pudo confirmar el seguimiento; conserva el borrador.")?;
    Ok(Snapshot { repair, revision })
}
#[tauri::command]
pub async fn repair_print_summary(
    order_id: String,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<PublicSummary, String> {
    public_summary(&access::pool(&instances).await?, &state, &order_id).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSummary {
    exists: bool,
    technician: String,
    public_note: String,
    warranties: Vec<Warranty>,
}
pub(crate) async fn public_summary(
    pool: &SqlitePool,
    state: &AccessState,
    id: &str,
) -> Result<PublicSummary, String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    state.authorize(&mut tx, "clients.view").await?;
    active_order(&mut tx, id).await?;
    let snapshot = read(&mut tx, id).await?;
    Ok(PublicSummary {
        exists: snapshot.revision > 0,
        technician: snapshot.repair.technician,
        public_note: snapshot.repair.public_note,
        warranties: snapshot.repair.warranties,
    })
}

#[tauri::command]
pub async fn load_repair(
    order_id: String,
    legacy: Option<Repair>,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<Snapshot, String> {
    load(&access::pool(&instances).await?, &state, &order_id, legacy).await
}
#[tauri::command]
pub async fn save_repair(
    order_id: String,
    repair: Repair,
    revision: i64,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<Snapshot, String> {
    save(
        &access::pool(&instances).await?,
        &state,
        &order_id,
        repair,
        revision,
    )
    .await
}
