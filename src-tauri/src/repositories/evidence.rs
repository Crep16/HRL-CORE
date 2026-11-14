use crate::{domain::evidence_contracts::*, error::AppError};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
pub struct EvidenceRepository;
pub struct StoredEvidence {
    pub value: Evidence,
    pub entity_id: i64,
    pub initial_note: String,
    pub author: i64,
}
const SELECT: &str =
    "SELECT e.*,u.nombre autor FROM evidencias e JOIN usuarios u ON u.id=e.id_autor";
fn value(r: &SqliteRow) -> Evidence {
    Evidence {
        id: r.get("codigo_archivo"),
        order_id: r.get::<i64, _>("id_orden").to_string(),
        original_name: r.get("nombre_original"),
        mime: r.get("mime"),
        size_bytes: r.get("tamano_bytes"),
        source: r.get("origen"),
        note: r.get("descripcion"),
        author_name: r.get("autor"),
        created_at: r.get("creado_en"),
        deleted: r.get("retirada"),
        revision: r.get("revision"),
    }
}
impl EvidenceRepository {
    pub async fn list(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        deleted: bool,
    ) -> Result<Vec<Evidence>, AppError> {
        Ok(sqlx::query(&format!(
            "{SELECT} WHERE e.id_orden=? AND e.retirada=? ORDER BY e.id"
        ))
        .bind(id)
        .bind(deleted)
        .fetch_all(db)
        .await?
        .iter()
        .map(value)
        .collect())
    }
    pub async fn get(
        &self,
        db: &mut SqliteConnection,
        id: &str,
    ) -> Result<Option<StoredEvidence>, AppError> {
        Ok(sqlx::query(&format!("{SELECT} WHERE e.codigo_archivo=?"))
            .bind(id)
            .fetch_optional(db)
            .await?
            .map(|r| StoredEvidence {
                value: value(&r),
                entity_id: r.get("id"),
                initial_note: r.get("descripcion_inicial"),
                author: r.get("id_autor"),
            }))
    }
    pub async fn insert(
        &self,
        db: &mut SqliteConnection,
        p: &NewEvidence,
        author: i64,
        path: &str,
    ) -> Result<i64, AppError> {
        Ok(sqlx::query("INSERT INTO evidencias(codigo_archivo,id_orden,id_autor,nombre_original,ruta_relativa,tipo_archivo,mime,tamano_bytes,origen,descripcion,descripcion_inicial) VALUES (?,?,?,?,?,'image',?,?,?,?,?)").bind(&p.id).bind(p.order_id).bind(author).bind(&p.original_name).bind(path).bind(&p.mime).bind(p.bytes.len() as i64).bind(&p.source).bind(&p.note).bind(&p.note).execute(db).await?.last_insert_rowid())
    }
    pub async fn update(
        &self,
        db: &mut SqliteConnection,
        p: &UpdateEvidence,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE evidencias SET descripcion=?,retirada=?,revision=revision+1 WHERE codigo_archivo=? AND revision=?").bind(&p.note).bind(p.deleted).bind(&p.id).bind(p.expected_revision).execute(db).await?;
        Ok(())
    }
}
