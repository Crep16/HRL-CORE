use crate::{
    domain::{contracts::OrderHistoryChange, report_contracts::*},
    error::AppError,
};
use sqlx::{Row, SqliteConnection};
pub struct ReportRepository;
impl ReportRepository {
    pub async fn logbook(
        &self,
        db: &mut SqliteConnection,
        f: &LogbookFilters,
    ) -> Result<LogbookPage, AppError> {
        let filter=" FROM auditoria a JOIN usuarios u ON u.id=a.id_autor JOIN operaciones_auditoria p ON p.id=a.id_operacion LEFT JOIN ordenes o ON o.id=a.id_orden WHERE (?1='' OR instr(lower(coalesce(o.folio,'')||' '||u.nombre),lower(?1))>0) AND (?2='' OR p.codigo=?2) AND (?3='' OR cast(u.id AS TEXT)=?3) AND (?4='' OR o.folio=?4) AND (?5='' OR a.creado_en>=?5) AND (?6='' OR a.creado_en<?6)";
        let bind = |sql: String| sql;
        let count = bind(format!("SELECT count(*){filter}"));
        let total = sqlx::query_scalar(&count)
            .bind(&f.query)
            .bind(&f.operation)
            .bind(&f.author_id)
            .bind(&f.order_id)
            .bind(&f.from_utc)
            .bind(&f.to_utc)
            .fetch_one(&mut *db)
            .await?;
        let sql=format!("SELECT a.*,u.nombre,p.codigo,coalesce(o.folio,'') folio,coalesce(o.codigo_moneda,'MXN') moneda{filter} ORDER BY a.id DESC LIMIT 50 OFFSET ?7");
        let rows = sqlx::query(&sql)
            .bind(&f.query)
            .bind(&f.operation)
            .bind(&f.author_id)
            .bind(&f.order_id)
            .bind(&f.from_utc)
            .bind(&f.to_utc)
            .bind(f.page * 50)
            .fetch_all(&mut *db)
            .await?;
        let mut entries = vec![];
        for r in rows {
            let id: i64 = r.get("id");
            let details=sqlx::query("SELECT campo,valor_anterior,valor_nuevo FROM auditoria_detalle WHERE id_auditoria=? ORDER BY id").bind(id).fetch_all(&mut *db).await?.iter().map(|d|OrderHistoryChange{field:d.get("campo"),old_value:d.get("valor_anterior"),new_value:d.get("valor_nuevo")}).collect();
            entries.push(LogbookEntry {
                id,
                entity: r.try_get("entidad")?,
                entity_id: r.try_get("id_entidad")?,
                order_id: r.get("folio"),
                author_id: r.get::<i64, _>("id_autor").to_string(),
                author_name: r.get("nombre"),
                operation: r.get("codigo"),
                details,
                occurred_at: r.get("creado_en"),
                event_date: None,
                currency: r.get("moneda"),
                reason: r.get("motivo"),
                result: r.get("resultado"),
            });
        }
        let authors=sqlx::query("SELECT DISTINCT u.id,u.nombre FROM usuarios u JOIN auditoria a ON a.id_autor=u.id ORDER BY u.nombre").fetch_all(db).await?.iter().map(|r|Author{id:r.get::<i64,_>("id").to_string(),name:r.get("nombre")}).collect();
        Ok(LogbookPage {
            entries,
            total,
            page: f.page,
            page_size: 50,
            authors,
        })
    }
    pub async fn finances(
        &self,
        db: &mut SqliteConnection,
    ) -> Result<Vec<FinancialOrder>, AppError> {
        let rows=sqlx::query("SELECT o.*,c.nombre cliente,e.articulo||' '||e.marca||' '||e.modelo equipo,coalesce(s.nombre,o.estado_personalizado) estado FROM ordenes o JOIN equipos e ON e.id=o.id_equipo JOIN clientes c ON c.id=e.id_cliente LEFT JOIN estados_orden s ON s.id=o.id_estado ORDER BY o.id DESC").fetch_all(&mut *db).await?;
        let mut result = vec![];
        for row in rows {
            let id: i64 = row.get("id");
            let state = super::ledger::LedgerRepository.state(db, id).await?;
            let mut movements = super::ledger::LedgerRepository.movements(db, id).await?;
            movements.reverse();
            let payments = movements
                .into_iter()
                .map(|p| Payment {
                    amount_minor: if p.kind == "refund" {
                        -p.amount_minor
                    } else {
                        p.amount_minor
                    },
                    occurred_at: p.occurred_at,
                    kind: p.kind,
                })
                .collect();
            let expenses=sqlx::query("SELECT descripcion,importe_centavos importe,fecha_evento fecha FROM gastos_directos WHERE id_orden=?1 UNION ALL SELECT nombre,cantidad*precio_centavos,'' FROM refacciones WHERE id_orden=?1").bind(id).fetch_all(&mut *db).await?.iter().map(|r|Expense{description:r.get("descripcion"),amount_minor:r.get("importe"),occurred_at:r.get("fecha")}).collect();
            result.push(FinancialOrder {
                database_id: id,
                id: row.get("folio"),
                client: row.get("cliente"),
                equipment: row.get::<String, _>("equipo").trim().into(),
                status: row.get("estado"),
                currency: row.get("codigo_moneda"),
                total_minor: state.total_minor,
                collected_minor: state.collected_minor,
                balance_minor: (state.total_minor - state.collected_minor).max(0),
                costs_minor: 0,
                profit_minor: 0,
                received_minor: state.received_minor,
                adjusted_minor: state.adjusted_minor,
                refunded_minor: state.refunded_minor,
                credit_minor: state.credit_minor,
                delivered: row.get("entregado"),
                paid: state.paid,
                classification: String::new(),
                excluded: row.get::<Option<String>, _>("eliminada_en").is_some(),
                received: row.get("fecha_recepcion"),
                delivered_at: row.get("fecha_entrega"),
                settled_at: None,
                payments,
                expenses,
            });
        }
        Ok(result)
    }
    pub async fn expense(
        &self,
        db: &mut SqliteConnection,
        p: &ExpensePayload,
        author: i64,
    ) -> Result<i64, AppError> {
        Ok(sqlx::query("INSERT INTO gastos_directos(id_orden,id_autor,descripcion,importe_centavos,motivo) VALUES (?,?,?,?,?)").bind(p.order_id).bind(author).bind(p.description.trim()).bind(p.amount_minor).bind(p.reason.trim()).execute(db).await?.last_insert_rowid())
    }
}
