use crate::{domain::repair_contracts::*, error::AppError};
use sqlx::{Row, SqliteConnection};
pub struct RepairRepository;
impl RepairRepository {
    pub async fn load(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<(Repair, i64), AppError> {
        let mut repair = Repair::default();
        let row=sqlx::query("SELECT tecnico,nota_publica,nota_interna,modo_publicacion,revision FROM seguimientos WHERE id_orden=?").bind(id).fetch_optional(&mut *db).await?;
        let Some(row) = row else {
            return Ok((repair, 0));
        };
        repair.technician = row.try_get("tecnico")?;
        repair.public_note = row.try_get("nota_publica")?;
        repair.internal_note = row.try_get("nota_interna")?;
        repair.sync_mode = row.try_get("modo_publicacion")?;
        let stages=sqlx::query("SELECT coalesce(e.nombre,o.nombre_personalizado) nombre,o.completada_en IS NOT NULL completa FROM orden_etapas o LEFT JOIN etapas e ON e.id=o.id_etapa WHERE o.id_orden=? ORDER BY o.posicion").bind(id).fetch_all(&mut *db).await?;
        repair.completed = stages
            .iter()
            .take_while(|s| s.get::<bool, _>("completa"))
            .count();
        repair.stages = stages.iter().map(|s| s.get("nombre")).collect();
        repair.events=sqlx::query("SELECT a.*,coalesce(e.nombre,o.nombre_personalizado) etapa FROM avances a JOIN orden_etapas o ON o.id=a.id_etapa_orden LEFT JOIN etapas e ON e.id=o.id_etapa WHERE o.id_orden=? ORDER BY a.id DESC").bind(id).fetch_all(&mut *db).await?.iter().map(|r|Event{id:r.get("id"),date:r.get("fecha_evento"),title:r.get("titulo"),detail:r.get("detalle"),stage:r.get("etapa"),public:r.get("visible_cliente")}).collect();
        repair.warranties =
            sqlx::query("SELECT cobertura,dias FROM garantias WHERE id_orden=? ORDER BY id")
                .bind(id)
                .fetch_all(&mut *db)
                .await?
                .iter()
                .map(|r| Warranty {
                    name: r.get("cobertura"),
                    days: r.get("dias"),
                })
                .collect();
        repair.parts = sqlx::query(
            "SELECT nombre,precio_centavos FROM refacciones WHERE id_orden=? ORDER BY id",
        )
        .bind(id)
        .fetch_all(&mut *db)
        .await?
        .iter()
        .map(|r| Part {
            name: r.get("nombre"),
            price: r.get("precio_centavos"),
        })
        .collect();
        repair.visibility = sqlx::query(
            "SELECT clave,habilitado FROM seguimiento_publicacion WHERE id_orden=? ORDER BY clave",
        )
        .bind(id)
        .fetch_all(&mut *db)
        .await?
        .iter()
        .map(|r| Visibility {
            label: r.get("clave"),
            enabled: r.get("habilitado"),
        })
        .collect();
        Ok((repair, row.try_get("revision")?))
    }
    pub async fn stages(&self, db: &mut SqliteConnection) -> Result<Vec<String>, AppError> {
        Ok(sqlx::query_scalar("SELECT nombre FROM etapas UNION SELECT nombre_personalizado FROM orden_etapas WHERE nombre_personalizado IS NOT NULL AND NOT EXISTS(SELECT 1 FROM nombres_etapas_ocultos h WHERE h.nombre=nombre_personalizado COLLATE NOCASE) ORDER BY 1 COLLATE NOCASE").fetch_all(db).await?)
    }
    pub async fn hide_stage_name(
        &self,
        db: &mut SqliteConnection,
        name: &str,
    ) -> Result<bool, AppError> {
        let custom: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM orden_etapas WHERE nombre_personalizado=? COLLATE NOCASE) AND NOT EXISTS(SELECT 1 FROM etapas WHERE nombre=? COLLATE NOCASE)")
            .bind(name).bind(name).fetch_one(&mut *db).await?;
        if !custom {
            return Ok(false);
        }
        sqlx::query("INSERT INTO nombres_etapas_ocultos(nombre) VALUES (?) ON CONFLICT DO NOTHING")
            .bind(name)
            .execute(db)
            .await?;
        Ok(true)
    }
    pub async fn save(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        author: i64,
        old: &Repair,
        next: &Repair,
        revision: i64,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO seguimientos(id_orden,tecnico,nota_publica,nota_interna,modo_publicacion,revision) VALUES (?,?,?,?,?,?) ON CONFLICT(id_orden) DO UPDATE SET tecnico=excluded.tecnico,nota_publica=excluded.nota_publica,nota_interna=excluded.nota_interna,modo_publicacion=excluded.modo_publicacion,revision=excluded.revision").bind(id).bind(&next.technician).bind(&next.public_note).bind(&next.internal_note).bind(&next.sync_mode).bind(revision+1).execute(&mut *db).await?;
        // Desplaza posiciones sin borrar las etapas a las que apunta el historial.
        sqlx::query("UPDATE orden_etapas SET posicion=posicion+1000 WHERE id_orden=?")
            .bind(id)
            .execute(&mut *db)
            .await?;
        for (position, name) in next.stages.iter().enumerate() {
            let existing:Option<i64>=sqlx::query_scalar("SELECT o.id FROM orden_etapas o LEFT JOIN etapas e ON e.id=o.id_etapa WHERE o.id_orden=? AND coalesce(e.nombre,o.nombre_personalizado)=?").bind(id).bind(name).fetch_optional(&mut *db).await?;
            if let Some(stage) = existing {
                sqlx::query("UPDATE orden_etapas SET posicion=?,completada_en=CASE WHEN ? THEN coalesce(completada_en,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ELSE NULL END WHERE id=?").bind(position as i64).bind(position<next.completed).bind(stage).execute(&mut *db).await?;
            } else {
                let catalog: Option<i64> =
                    sqlx::query_scalar("SELECT id FROM etapas WHERE nombre=? COLLATE NOCASE")
                        .bind(name)
                        .fetch_optional(&mut *db)
                        .await?;
                sqlx::query("INSERT INTO orden_etapas(id_orden,id_etapa,nombre_personalizado,posicion,completada_en) VALUES (?,?,?,?,CASE WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END)").bind(id).bind(catalog).bind(if catalog.is_none(){Some(name)}else{None}).bind(position as i64).bind(position<next.completed).execute(&mut *db).await?;
            }
        }
        for event in next.events[..next.events.len() - old.events.len()]
            .iter()
            .rev()
        {
            sqlx::query("INSERT INTO avances(id_etapa_orden,id_autor,titulo,detalle,visible_cliente,fecha_evento) SELECT o.id,?,?,?,?,? FROM orden_etapas o LEFT JOIN etapas e ON e.id=o.id_etapa WHERE o.id_orden=? AND coalesce(e.nombre,o.nombre_personalizado)=?").bind(author).bind(&event.title).bind(&event.detail).bind(event.public).bind(&event.date).bind(id).bind(&event.stage).execute(&mut *db).await?;
        }
        for (previous, next, public) in [
            (&old.public_note, &next.public_note, true),
            (&old.internal_note, &next.internal_note, false),
        ] {
            if previous != next {
                let text = next.strip_prefix(previous).unwrap_or(next).trim();
                if !text.is_empty() {
                    sqlx::query("INSERT INTO notas(id_orden,id_autor,texto,visible_cliente) VALUES (?,?,?,?)").bind(id).bind(author).bind(text).bind(public).execute(&mut *db).await?;
                }
            }
        }
        for w in &next.warranties[old.warranties.len()..] {
            sqlx::query("INSERT INTO garantias(id_orden,id_autor,cobertura,dias) VALUES (?,?,?,?)")
                .bind(id)
                .bind(author)
                .bind(&w.name)
                .bind(w.days)
                .execute(&mut *db)
                .await?;
        }
        for p in &next.parts[old.parts.len()..] {
            sqlx::query("INSERT INTO refacciones(id_orden,id_autor,nombre,precio_centavos) VALUES (?,?,?,?)").bind(id).bind(author).bind(&p.name).bind(p.price).execute(&mut *db).await?;
        }
        sqlx::query("DELETE FROM seguimiento_publicacion WHERE id_orden=?")
            .bind(id)
            .execute(&mut *db)
            .await?;
        for v in &next.visibility {
            sqlx::query(
                "INSERT INTO seguimiento_publicacion(id_orden,clave,habilitado) VALUES (?,?,?)",
            )
            .bind(id)
            .bind(&v.label)
            .bind(v.enabled)
            .execute(&mut *db)
            .await?;
        }
        Ok(())
    }
}
