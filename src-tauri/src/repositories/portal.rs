use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Row, SqliteConnection};

pub struct PortalRepository;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortalConfig {
    pub project_id: String,
    pub api_key: String,
    pub publisher_uid: String,
    pub portal_url: String,
    #[serde(default)]
    pub paused: bool,
}

pub struct PublicationRecord {
    pub id: i64,
    pub order_id: i64,
    pub code: String,
    pub revision: i64,
    pub attempts: i64,
    pub document: Value,
}

impl PortalRepository {
    pub async fn config(&self, db: &mut SqliteConnection) -> Result<PortalConfig, AppError> {
        let r = sqlx::query("SELECT * FROM portal_configuracion WHERE id=1")
            .fetch_one(db)
            .await?;
        Ok(PortalConfig {
            project_id: r.get("proyecto"),
            api_key: r.get("clave_api"),
            publisher_uid: r.get("uid_emisor"),
            portal_url: r.get("url_portal"),
            paused: r.get("pausado"),
        })
    }
    pub async fn configure(
        &self,
        db: &mut SqliteConnection,
        c: &PortalConfig,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_configuracion SET proyecto=?,clave_api=?,uid_emisor=?,url_portal=?,pausado=? WHERE id=1").bind(&c.project_id).bind(&c.api_key).bind(&c.publisher_uid).bind(&c.portal_url).bind(c.paused).execute(db).await?;
        Ok(())
    }
    pub async fn pause(&self, db: &mut SqliteConnection) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_configuracion SET pausado=1 WHERE id=1")
            .execute(db)
            .await?;
        Ok(())
    }
    pub async fn ensure(&self, db: &mut SqliteConnection, order: i64) -> Result<(), AppError> {
        sqlx::query("INSERT INTO portal_ordenes(id_orden) VALUES(?) ON CONFLICT DO NOTHING")
            .bind(order)
            .execute(db)
            .await?;
        Ok(())
    }
    pub async fn code(
        &self,
        db: &mut SqliteConnection,
        order: i64,
    ) -> Result<Option<String>, AppError> {
        Ok(
            sqlx::query_scalar("SELECT codigo FROM portal_codigos WHERE id_orden=? AND activo=1")
                .bind(order)
                .fetch_optional(db)
                .await?,
        )
    }
    pub async fn add_code(
        &self,
        db: &mut SqliteConnection,
        order: i64,
        code: &str,
    ) -> Result<bool, AppError> {
        Ok(sqlx::query("INSERT INTO portal_codigos(codigo,id_orden,activo) VALUES(?,?,1) ON CONFLICT(codigo) DO NOTHING").bind(code).bind(order).execute(db).await?.rows_affected()==1)
    }
    pub async fn retire_code(&self, db: &mut SqliteConnection, order: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_codigos SET activo=0 WHERE id_orden=?")
            .bind(order)
            .execute(db)
            .await?;
        Ok(())
    }
    pub async fn policy(
        &self,
        db: &mut SqliteConnection,
        order: i64,
    ) -> Result<(bool, String, i64, Option<String>), AppError> {
        Ok(sqlx::query_as("SELECT p.habilitado,coalesce(s.modo_publicacion,'auto'),p.revision,p.entrega_instante FROM portal_ordenes p LEFT JOIN seguimientos s ON s.id_orden=p.id_orden WHERE p.id_orden=?").bind(order).fetch_one(db).await?)
    }
    pub async fn enabled(
        &self,
        db: &mut SqliteConnection,
        order: i64,
        enabled: bool,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_ordenes SET habilitado=? WHERE id_orden=?")
            .bind(enabled)
            .bind(order)
            .execute(db)
            .await?;
        Ok(())
    }
    pub async fn delivery(
        &self,
        db: &mut SqliteConnection,
        order: i64,
        delivered: bool,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_ordenes SET entrega_instante=CASE WHEN ? THEN coalesce(entrega_instante,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ELSE NULL END WHERE id_orden=?").bind(delivered).bind(order).execute(db).await?;
        Ok(())
    }
    pub async fn stage_dates(
        &self,
        db: &mut SqliteConnection,
        order: i64,
    ) -> Result<Vec<Option<String>>, AppError> {
        Ok(sqlx::query_scalar("SELECT completada_en FROM orden_etapas WHERE id_orden=? AND retirada_en IS NULL ORDER BY posicion").bind(order).fetch_all(db).await?)
    }
    pub async fn watermarks(&self, db: &mut SqliteConnection) -> Result<(i64, i64, i64), AppError> {
        Ok(sqlx::query_as("SELECT coalesce((SELECT max(id) FROM abonos),0),coalesce((SELECT max(id) FROM ajustes_abono),0),coalesce((SELECT max(id) FROM devoluciones),0)").fetch_one(db).await?)
    }
    pub async fn issue(
        &self,
        db: &mut SqliteConnection,
        order: i64,
        error: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_ordenes SET error_publicacion=? WHERE id_orden=?")
            .bind(error)
            .bind(order)
            .execute(&mut *db)
            .await?;
        sqlx::query("UPDATE portal_envios SET estado='superseded' WHERE estado IN('pending','error','conflict') AND id_publicacion IN(SELECT id FROM portal_publicaciones WHERE id_orden=? AND codigo IN(SELECT codigo FROM portal_codigos WHERE id_orden=? AND activo=1))").bind(order).bind(order).execute(db).await?;
        Ok(())
    }
    pub async fn save(
        &self,
        db: &mut SqliteConnection,
        order: i64,
        d: &Value,
        m: (i64, i64, i64),
    ) -> Result<i64, AppError> {
        let s = |path: &str| d.pointer(path).and_then(Value::as_str);
        let id=sqlx::query("INSERT INTO portal_publicaciones(id_orden,codigo,revision,publicado,huella,tipo,marca,modelo,estado_tecnico,recibido_en,entrega_estimada,entregado_en,acceso_hasta,nota_publica,eventos_visibles,garantias_visibles,moneda,precio_centavos,ultimo_abono,ultimo_ajuste,ultima_devolucion) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(order).bind(s("/idPublico")).bind(d["revision"].as_i64()).bind(d["publicado"].as_bool()).bind(s("/huella"))
            .bind(s("/equipo/tipo")).bind(s("/equipo/marca")).bind(s("/equipo/modelo")).bind(s("/estadoTecnico"))
            .bind(s("/recibidoEn")).bind(s("/entregaEstimada")).bind(s("/entregadoEn")).bind(s("/accesoHasta")).bind(s("/notaPublica"))
            .bind(d.get("eventos").is_some()).bind(d.get("garantias").is_some()).bind(s("/pago/moneda")).bind(d.pointer("/pago/totalCentavos").and_then(Value::as_i64))
            .bind(m.0).bind(m.1).bind(m.2).execute(&mut *db).await?.last_insert_rowid();
        if let Some(items) = d["etapas"].as_array() {
            for (i, v) in items.iter().enumerate() {
                sqlx::query("INSERT INTO portal_publicacion_etapas VALUES(?,?,?,?,?)")
                    .bind(id)
                    .bind(i as i64)
                    .bind(v["nombre"].as_str())
                    .bind(v["completada"].as_bool())
                    .bind(v["completadaEn"].as_str())
                    .execute(&mut *db)
                    .await?;
            }
        }
        if let Some(items) = d["eventos"].as_array() {
            for (i, v) in items.iter().enumerate() {
                sqlx::query("INSERT INTO portal_publicacion_eventos VALUES(?,?,?,?,?)")
                    .bind(id)
                    .bind(i as i64)
                    .bind(v["fecha"].as_str())
                    .bind(v["titulo"].as_str())
                    .bind(v["detalle"].as_str())
                    .execute(&mut *db)
                    .await?;
            }
        }
        if let Some(items) = d["garantias"].as_array() {
            for (i, v) in items.iter().enumerate() {
                sqlx::query("INSERT INTO portal_publicacion_garantias VALUES(?,?,?,?)")
                    .bind(id)
                    .bind(i as i64)
                    .bind(v["cobertura"].as_str())
                    .bind(v["dias"].as_i64())
                    .execute(&mut *db)
                    .await?;
            }
        }
        sqlx::query("UPDATE portal_ordenes SET revision=?,error_publicacion=NULL WHERE id_orden=?")
            .bind(d["revision"].as_i64())
            .bind(order)
            .execute(&mut *db)
            .await?;
        sqlx::query("INSERT INTO portal_envios(id_publicacion) VALUES(?)")
            .bind(id)
            .execute(db)
            .await?;
        Ok(id)
    }
    pub async fn load(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<PublicationRecord, AppError> {
        let r=sqlx::query("SELECT p.*,e.intentos FROM portal_publicaciones p JOIN portal_envios e ON e.id_publicacion=p.id WHERE p.id=?").bind(id).fetch_one(&mut *db).await?;
        let order: i64 = r.get("id_orden");
        let mut d = json!({"versionContrato":1,"idPublico":r.get::<String,_>("codigo"),"revision":r.get::<i64,_>("revision"),"publicado":r.get::<bool,_>("publicado"),"huella":r.get::<String,_>("huella")});
        for (column, key) in [
            ("entregado_en", "entregadoEn"),
            ("acceso_hasta", "accesoHasta"),
        ] {
            if let Some(v) = r.get::<Option<String>, _>(column) {
                d[key] = json!(v);
            }
        }
        if r.get::<bool, _>("publicado") {
            d["equipo"] = json!({"tipo":r.get::<String,_>("tipo"),"marca":r.get::<String,_>("marca"),"modelo":r.get::<String,_>("modelo")});
            d["estadoTecnico"] = json!(r.get::<String, _>("estado_tecnico"));
            for (column, key) in [
                ("recibido_en", "recibidoEn"),
                ("entrega_estimada", "entregaEstimada"),
                ("nota_publica", "notaPublica"),
            ] {
                if let Some(v) = r.get::<Option<String>, _>(column) {
                    d[key] = json!(v);
                }
            }
            let mut stages = vec![];
            for s in sqlx::query(
                "SELECT * FROM portal_publicacion_etapas WHERE id_publicacion=? ORDER BY posicion",
            )
            .bind(id)
            .fetch_all(&mut *db)
            .await?
            {
                let mut v = json!({"nombre":s.get::<String,_>("nombre"),"completada":s.get::<bool,_>("completada")});
                if let Some(at) = s.get::<Option<String>, _>("completada_en") {
                    v["completadaEn"] = json!(at);
                }
                stages.push(v);
            }
            d["etapas"] = json!(stages);
            if r.get::<bool, _>("eventos_visibles") {
                let rows=sqlx::query("SELECT * FROM portal_publicacion_eventos WHERE id_publicacion=? ORDER BY posicion").bind(id).fetch_all(&mut *db).await?;
                d["eventos"]=json!(rows.iter().map(|v|json!({"fecha":v.get::<String,_>("fecha"),"titulo":v.get::<String,_>("titulo"),"detalle":v.get::<String,_>("detalle")})).collect::<Vec<_>>());
            }
            if r.get::<bool, _>("garantias_visibles") {
                let rows=sqlx::query("SELECT * FROM portal_publicacion_garantias WHERE id_publicacion=? ORDER BY posicion").bind(id).fetch_all(&mut *db).await?;
                d["garantias"]=json!(rows.iter().map(|v|json!({"cobertura":v.get::<String,_>("cobertura"),"dias":v.get::<i64,_>("dias")})).collect::<Vec<_>>());
            }
            if let Some(currency) = r.get::<Option<String>, _>("moneda") {
                let net:i64=sqlx::query_scalar("SELECT coalesce((SELECT sum(importe_centavos) FROM abonos WHERE id_orden=?1 AND id<=?2),0)+coalesce((SELECT sum(a.importe_centavos) FROM ajustes_abono a JOIN abonos b ON b.id=a.id_abono WHERE b.id_orden=?1 AND b.id<=?2 AND a.id<=?3),0)-coalesce((SELECT sum(importe_centavos) FROM devoluciones WHERE id_orden=?1 AND id<=?4),0)")
                    .bind(order).bind(r.get::<i64,_>("ultimo_abono")).bind(r.get::<i64,_>("ultimo_ajuste")).bind(r.get::<i64,_>("ultima_devolucion")).fetch_one(&mut *db).await?;
                let total: i64 = r.get("precio_centavos");
                d["pago"] = json!({"moneda":currency,"totalCentavos":total,"cobradoCentavos":net,"pendienteCentavos":(total-net).max(0)});
            }
        }
        Ok(PublicationRecord {
            id,
            order_id: order,
            code: r.get("codigo"),
            revision: r.get("revision"),
            attempts: r.get("intentos"),
            document: d,
        })
    }
    pub async fn next(&self, db: &mut SqliteConnection, now: i64) -> Result<Option<i64>, AppError> {
        Ok(sqlx::query_scalar("SELECT id_publicacion FROM portal_envios WHERE estado='pending' AND siguiente_intento<=? ORDER BY id_publicacion LIMIT 1").bind(now).fetch_optional(db).await?)
    }
    pub async fn result(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        state: &str,
        error: Option<&str>,
        next: i64,
        confirmed: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_envios SET estado=?,error_envio=?,siguiente_intento=?,confirmado_en=?,intentos=intentos+1 WHERE id_publicacion=?").bind(state).bind(error).bind(next).bind(confirmed).bind(id).execute(db).await?;
        Ok(())
    }
    pub async fn recover(&self, db: &mut SqliteConnection) -> Result<(), AppError> {
        sqlx::query("UPDATE portal_envios SET estado='pending' WHERE estado='sending'")
            .execute(db)
            .await?;
        Ok(())
    }
    pub async fn status(&self, db: &mut SqliteConnection, order: i64) -> Result<Value, AppError> {
        let r=sqlx::query("SELECT p.*,c.codigo,(SELECT estado FROM portal_envios e JOIN portal_publicaciones q ON q.id=e.id_publicacion WHERE q.id_orden=p.id_orden ORDER BY q.id DESC LIMIT 1) estado_envio,(SELECT error_envio FROM portal_envios e JOIN portal_publicaciones q ON q.id=e.id_publicacion WHERE q.id_orden=p.id_orden ORDER BY q.id DESC LIMIT 1) error_envio,(SELECT confirmado_en FROM portal_envios e JOIN portal_publicaciones q ON q.id=e.id_publicacion WHERE q.id_orden=p.id_orden AND e.estado='confirmed' ORDER BY q.id DESC LIMIT 1) confirmado FROM portal_ordenes p LEFT JOIN portal_codigos c ON c.id_orden=p.id_orden AND c.activo=1 WHERE p.id_orden=?").bind(order).fetch_optional(db).await?;
        Ok(match r {
            None => {
                json!({"code":null,"revision":0,"enabled":true,"state":"unpublished","error":null,"confirmedAt":null})
            }
            Some(r) => {
                json!({"code":r.get::<Option<String>,_>("codigo"),"revision":r.get::<i64,_>("revision"),"enabled":r.get::<bool,_>("habilitado"),"state":r.get::<Option<String>,_>("estado_envio").unwrap_or("unpublished".into()),"error":r.get::<Option<String>,_>("error_publicacion").or(r.get::<Option<String>,_>("error_envio")),"confirmedAt":r.get::<Option<String>,_>("confirmado")})
            }
        })
    }
}
