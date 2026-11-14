use crate::{access::{self, AccessState}, orders::{self, PaymentState}};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;
use tauri_plugin_sql::DbInstances;

#[cfg(test)]
#[path = "payments_tests.rs"]
mod tests;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MoneyRequest {
    pub request_id: String,
    pub order_id: String,
    pub kind: String,
    pub payment_id: Option<String>,
    pub amount_minor: i64,
    pub expected_minor: i64,
    pub method: String,
    pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Movement {
    id: String,
    kind: String,
    payment_id: Option<String>,
    amount_minor: i64,
    effective_minor: Option<i64>,
    method: Option<String>,
    reason: Option<String>,
    occurred_at: Option<String>,
}

#[derive(Serialize)]
pub struct Ledger { movements:Vec<Movement>, payment:PaymentState }

pub(crate) async fn ledger(pool: &SqlitePool, state: &AccessState, order: &str) -> Result<Ledger, String> {
    let mut tx = pool.begin().await.map_err(|_| "No se pudo abrir la base local.")?;
    if state.check_scoped(&mut tx, "repairs.view",Some(order)).await.is_err() {
        state.check_scoped(&mut tx, "logbook.view",None).await?;
    }
    let rows = sqlx::query("SELECT p.id,'receipt' AS tipo,NULL AS id_abono,p.importe_centavos,p.importe_centavos+coalesce((SELECT sum(importe_centavos) FROM Movimientos_abonos WHERE id_abono=p.id AND tipo='adjustment'),0) AS effective_minor,p.metodo_pago,NULL AS motivo,nullif(p.fecha_evento,'') AS fecha_evento FROM Abonos p WHERE id_orden=? UNION ALL SELECT id,tipo,id_abono,importe_centavos,NULL,metodo_pago,motivo,fecha_evento FROM Movimientos_abonos WHERE id_orden=? ORDER BY fecha_evento,id")
        .bind(order).bind(order).fetch_all(&mut *tx).await.map_err(|_| "No se pudo consultar el libro de pagos.")?;
    let movements=rows.into_iter().map(|r| Movement {
        id:r.get("id"), kind:r.get("tipo"), payment_id:r.get("id_abono"), amount_minor:r.get("importe_centavos"),
        effective_minor:r.get("effective_minor"), method:r.get("metodo_pago"), reason:r.get("motivo"), occurred_at:r.get("fecha_evento"),
    }).collect();
    let payment=orders::payment_state(&mut tx,order).await?;
    Ok(Ledger{movements,payment})
}

pub(crate) async fn record(pool: &SqlitePool, state: &AccessState, request: MoneyRequest) -> Result<PaymentState, String> {
    let operation = match request.kind.as_str() {
        "receipt" => "payments.add", "refund" => "payments.refund", "adjustment" => "payments.adjust",
        _ => return Err("Operación de dinero desconocida.".into()),
    };
    if request.request_id.len() < 16 || request.request_id.len() > 100 || request.order_id.len() > 100
        || request.amount_minor == 0 || !(-9_000_000_000_000..=9_000_000_000_000).contains(&request.amount_minor)
        || request.method.len() > 200 || request.reason.len() > 2000 {
        return Err("Revisa el importe y los datos de la operación.".into());
    }
    if request.kind != "adjustment" && (request.amount_minor <= 0 || request.method.trim().is_empty() || request.payment_id.is_some()) {
        return Err("Captura un importe positivo y el método de pago o devolución.".into());
    }
    if request.kind != "receipt" && request.reason.trim().is_empty() {
        return Err("Escribe el motivo del ajuste o devolución.".into());
    }
    if request.kind == "adjustment" && (request.payment_id.is_none() || !request.method.is_empty()) {
        return Err("Selecciona el abono original para corregirlo.".into());
    }
    let mut tx = pool.begin().await.map_err(|_| "No se pudo abrir la base local.")?;
    let author = state.check_scoped(&mut tx, operation,Some(&request.order_id)).await?;
    // Take the SQLite write lock before reading balances or checking a retry.
    let locked = sqlx::query("UPDATE Ordenes SET precio_centavos=precio_centavos WHERE id=?")
        .bind(&request.order_id).execute(&mut *tx).await.map_err(|_| "La orden está ocupada. Reintenta la misma operación.")?;
    if locked.rows_affected() != 1 { return Err("La orden ya no existe.".into()); }
    let payload = serde_json::to_string(&request).map_err(|_| "No se pudo preparar la operación.")?;
    if let Some(row) = sqlx::query("SELECT id_autor,solicitud_json FROM Solicitudes_abonos WHERE id=?")
        .bind(&request.request_id).fetch_optional(&mut *tx).await.map_err(|_| "No se pudo comprobar el reintento.")? {
        if row.get::<String,_>("id_autor") != author.id || row.get::<String,_>("solicitud_json") != payload {
            return Err("Este identificador ya pertenece a otra operación.".into());
        }
        let result = orders::payment_state(&mut tx, &request.order_id).await?;
        tx.commit().await.map_err(|_| "No se pudo confirmar la consulta.")?;
        return Ok(result);
    }
    state.authorize_scoped(&mut tx,operation,Some(&request.order_id)).await?;
    let current = orders::payment_state(&mut tx, &request.order_id).await?;
    if request.kind == "receipt" {
        let deleted:bool=sqlx::query_scalar("SELECT coalesce((SELECT accion FROM Historial_papelera WHERE id_orden=? ORDER BY secuencia DESC LIMIT 1),'restored')='deleted'")
            .bind(&request.order_id).fetch_one(&mut *tx).await.map_err(|_| "No se pudo comprobar la papelera.")?;
        if deleted { return Err("Restaura la orden antes de registrar un abono.".into()); }
        if request.expected_minor != current.collected_minor || request.amount_minor > (current.total_minor-current.collected_minor).max(0) {
            return Err("El saldo cambió o el abono supera lo pendiente. Actualiza la orden.".into());
        }
        sqlx::query("INSERT INTO Abonos(id,id_orden,id_autor,importe_centavos,fecha_evento,metodo_pago,motivo) VALUES (?,?,?,?,strftime('%Y-%m-%dT%H:%M:%fZ','now'),?,?)")
            .bind(&request.request_id).bind(&request.order_id).bind(&author.id).bind(request.amount_minor).bind(request.method.trim()).bind(request.reason.trim())
            .execute(&mut *tx).await.map_err(|_| "No se pudo registrar el abono y su historial.")?;
    } else {
        if request.kind == "refund" {
            if current.credit_minor <= 0 || request.amount_minor != current.credit_minor || request.expected_minor != current.collected_minor {
                return Err("Actualiza la orden: la devolución debe cubrir todo el saldo a favor vigente.".into());
            }
        } else {
            let effective:Option<i64>=sqlx::query_scalar("SELECT p.importe_centavos+coalesce((SELECT sum(importe_centavos) FROM Movimientos_abonos WHERE id_abono=p.id AND tipo='adjustment'),0) FROM Abonos p WHERE p.id=? AND p.id_orden=?")
                .bind(&request.payment_id).bind(&request.order_id).fetch_optional(&mut *tx).await.map_err(|_| "No se pudo consultar el abono original.")?;
            let effective=effective.ok_or("El abono no pertenece a esta orden.")?;
            if effective != request.expected_minor || effective.checked_add(request.amount_minor).is_none_or(|n| !(0..=9_000_000_000_000).contains(&n))
                || current.collected_minor.checked_add(request.amount_minor).is_none_or(|n| !(0..=9_000_000_000_000).contains(&n)) {
                return Err("El ajuste está desactualizado o dejaría un cobro negativo o fuera del límite. Revisa también las devoluciones anteriores.".into());
            }
        }
        sqlx::query("INSERT INTO Movimientos_abonos(id,id_orden,id_autor,tipo,id_abono,importe_centavos,importe_esperado_centavos,metodo_pago,motivo) VALUES (?,?,?,?,?,?,?,?,?)")
            .bind(&request.request_id).bind(&request.order_id).bind(&author.id).bind(&request.kind).bind(&request.payment_id)
            .bind(request.amount_minor).bind(request.expected_minor).bind(if request.kind=="refund" {Some(request.method.trim())} else {None})
            .bind(request.reason.trim()).execute(&mut *tx).await.map_err(|_| "No se pudo confirmar el ajuste o devolución con su historial.")?;
    }
    sqlx::query("INSERT INTO Solicitudes_abonos(id,id_autor,solicitud_json) VALUES (?,?,?)")
        .bind(&request.request_id).bind(&author.id).bind(payload).execute(&mut *tx).await.map_err(|_| "No se pudo proteger la operación contra duplicados.")?;
    let result=orders::payment_state(&mut tx,&request.order_id).await?;
    tx.commit().await.map_err(|_| "No se pudo confirmar la operación. Reintenta sin cambiar sus datos.")?;
    Ok(result)
}

#[tauri::command]
pub async fn list_payment_ledger(order_id:String, instances:State<'_,DbInstances>, state:State<'_,AccessState>) -> Result<Ledger,String> {
    ledger(&access::pool(&instances).await?,&state,&order_id).await
}
#[tauri::command]
pub async fn record_payment_operation(request:MoneyRequest, instances:State<'_,DbInstances>, state:State<'_,AccessState>) -> Result<PaymentState,String> {
    record(&access::pool(&instances).await?,&state,request).await.map_err(|message| {
        if message.starts_with("No se pudo confirmar la operación.") { message }
        else { format!("Operación no aplicada: {message}") }
    })
}
