use crate::access::{self, AccessState};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;
use tauri_plugin_sql::DbInstances;

#[cfg(test)]
#[path = "logbook_tests.rs"]
mod tests;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Filters {
    query: String,
    operation: String,
    author_id: String,
    page: u32,
    order_id: String,
    from_utc: String,
    to_utc: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    id: i64,
    order_id: String,
    author_id: String,
    author_name: String,
    operation: String,
    field: Option<String>,
    old_value: Option<String>,
    new_value: Option<String>,
    details: Option<serde_json::Value>,
    occurred_at: Option<String>,
    event_date: Option<String>,
    currency: String,
    reason: Option<String>,
    result: String,
}
#[derive(Serialize)]
pub struct Author { id: String, name: String }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    entries: Vec<Entry>,
    total: i64,
    page: u32,
    page_size: u32,
    authors: Vec<Author>,
}

pub(crate) async fn list(pool: &SqlitePool, state: &AccessState, filters: Filters) -> Result<Page, String> {
    if filters.query.len() > 200 || filters.operation.len() > 80 || filters.author_id.len() > 100 {
        return Err("Los filtros de la bitácora son demasiado largos.".into());
    }
    let mut tx = pool.begin().await.map_err(|_| "No se pudo abrir la base local.")?;
    state.check_scoped(&mut tx, "logbook.view",None).await?;
    let query = filters.query.trim().to_lowercase();
    if filters.order_id.len()>100 || filters.from_utc.len()>30 || filters.to_utc.len()>30 {return Err("Revisa los filtros de fecha y orden.".into());}
    let condition = "FROM Vista_bitacora_completa l LEFT JOIN Ordenes o ON o.id=l.id_orden WHERE (?='' OR instr(lower(l.id_orden || ' ' || l.nombre_autor || ' ' || coalesce(l.campo,'')),?)>0) AND (?='' OR l.operacion=?) AND (?='' OR l.id_autor=?) AND (?='' OR l.id_orden=?) AND (?='' OR l.fecha_evento>=?) AND (?='' OR l.fecha_evento<?)";
    let count_sql = format!("SELECT count(*) {condition}");
    let total: i64 = sqlx::query_scalar(&count_sql)
        .bind(&query).bind(&query).bind(&filters.operation).bind(&filters.operation).bind(&filters.author_id).bind(&filters.author_id)
        .bind(&filters.order_id).bind(&filters.order_id).bind(&filters.from_utc).bind(&filters.from_utc).bind(&filters.to_utc).bind(&filters.to_utc)
        .fetch_one(&mut *tx).await.map_err(|_| "No se pudo contar la bitácora.")?;
    let page_size = 50;
    let page = filters.page.min(((total.max(1) - 1) / page_size as i64) as u32);
    let sql = format!("SELECT l.*,coalesce(o.moneda,'MXN') AS moneda {condition} ORDER BY l.fecha_evento IS NULL,l.fecha_evento DESC,l.id DESC LIMIT ? OFFSET ?");
    let rows = sqlx::query(&sql)
        .bind(&query).bind(&query).bind(&filters.operation).bind(&filters.operation).bind(&filters.author_id).bind(&filters.author_id)
        .bind(&filters.order_id).bind(&filters.order_id).bind(&filters.from_utc).bind(&filters.from_utc).bind(&filters.to_utc).bind(&filters.to_utc)
        .bind(page_size).bind(i64::from(page) * i64::from(page_size))
        .fetch_all(&mut *tx).await.map_err(|_| "No se pudo recuperar la bitácora.")?;
    let mut entries = Vec::new();
    for row in rows {
        let raw: Option<String> = row.get("detalles_json");
        let details = raw.map(|value| serde_json::from_str(&value)).transpose().map_err(|_| "Un registro de la bitácora necesita revisión.")?;
        entries.push(Entry {
            id:row.get("id"), order_id:row.get("id_orden"), author_id:row.get("id_autor"), author_name:row.get("nombre_autor"),
            operation:row.get("operacion"), field:row.get("campo"), old_value:row.get("valor_anterior"), new_value:row.get("valor_nuevo"),
            details, occurred_at:row.get("fecha_evento"), event_date:row.get("fecha_declarada"), currency:row.get("moneda"),reason:row.get("motivo"),result:row.get("resultado"),
        });
    }
    let authors = sqlx::query("SELECT id_autor,nombre_autor FROM (SELECT id_autor,nombre_autor,row_number() OVER(PARTITION BY id_autor ORDER BY fecha_evento DESC,id DESC) AS position FROM Vista_bitacora_completa) WHERE position=1 ORDER BY nombre_autor COLLATE NOCASE")
        .fetch_all(&mut *tx).await.map_err(|_| "No se pudieron recuperar los usuarios de la bitácora.")?
        .into_iter().map(|row| Author { id:row.get("id_autor"),name:row.get("nombre_autor") }).collect();
    Ok(Page { entries, total, page, page_size:page_size as u32, authors })
}

pub(crate) async fn order_notes(db: &mut sqlx::SqliteConnection, order: &str, author: &str, old: &str, new: &str) -> Result<(), String> {
    if old == new { return Ok(()); }
    sqlx::query("INSERT INTO Bitacora_ordenes(origen,id_origen,id_orden,id_autor,nombre_autor,operacion,campo,valor_anterior,valor_nuevo,fecha_evento) VALUES ('order-note',?,?,?,(SELECT nombre FROM Usuarios WHERE id=?),'orders.edit','notes',?,?,strftime('%Y-%m-%dT%H:%M:%fZ','now'))")
        .bind(access::random_id()).bind(order).bind(author).bind(author).bind(old).bind(new)
        .execute(db).await.map_err(|_| "No se pudo registrar el cambio de notas; no se confirmó la orden.")?;
    Ok(())
}

pub(crate) async fn system_record(db:&mut sqlx::SqliteConnection,record:&str,author:&str,operation:&str,details:serde_json::Value)->Result<(),String>{
    sqlx::query("INSERT INTO Bitacora_sistema(id_registro,id_autor,nombre_autor,operacion,detalles_json) VALUES (?,?,(SELECT nombre FROM Usuarios WHERE id=?),?,?)")
        .bind(record).bind(author).bind(author).bind(operation).bind(details.to_string()).execute(db).await.map_err(|_|"No se pudo guardar el historial de la operación.")?;
    Ok(())
}

#[tauri::command]
pub async fn list_logbook(filters: Filters, instances: State<'_, DbInstances>, state: State<'_, AccessState>) -> Result<Page, String> {
    list(&access::pool(&instances).await?, &state, filters).await
}
