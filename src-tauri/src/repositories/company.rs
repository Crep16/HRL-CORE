use crate::{domain::company_contracts::*, error::AppError};
use sqlx::{Row, SqliteConnection};
#[derive(Default)]
pub struct CompanyRepository;
impl CompanyRepository {
    pub async fn load(
        &self,
        db: &mut SqliteConnection,
    ) -> Result<Option<(CompanySettings, Option<String>)>, AppError> {
        let Some(c) = sqlx::query("SELECT * FROM empresa WHERE id=1")
            .fetch_optional(&mut *db)
            .await?
        else {
            return Ok(None);
        };
        let d = sqlx::query("SELECT * FROM configuracion_documentos WHERE id=1")
            .fetch_one(&mut *db)
            .await?;
        let prefix: String = d.try_get("prefijo_folio")?;
        let next_number =
            sqlx::query_scalar("SELECT siguiente_numero FROM secuencias_folio WHERE prefijo=?")
                .bind(&prefix)
                .fetch_optional(&mut *db)
                .await?
                .unwrap_or(1);
        let clauses = sqlx::query("SELECT * FROM clausulas ORDER BY posicion,id")
            .fetch_all(&mut *db)
            .await?
            .into_iter()
            .map(|r| {
                Ok(CompanyClause {
                    id: r.try_get::<i64, _>("id")?.to_string(),
                    title: r.try_get("titulo")?,
                    text: r.try_get("texto")?,
                    entry: r.try_get("aplica_entrada")?,
                    exit: r.try_get("aplica_salida")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;
        Ok(Some((
            CompanySettings {
                name: c.try_get("nombre")?,
                tax_id: c.try_get("rfc")?,
                phone: c.try_get("telefono")?,
                whatsapp: c.try_get("whatsapp")?,
                email: c.try_get("correo")?,
                website: c.try_get("sitio_web")?,
                address: c.try_get("direccion")?,
                hours: c.try_get("horario")?,
                country: c.try_get("pais")?,
                locale: c.try_get("locale")?,
                timezone: c.try_get("zona_horaria")?,
                currency: c.try_get("codigo_moneda")?,
                show_tax: c.try_get("mostrar_impuesto")?,
                tax_rate_thousandths: c.try_get("tasa_impuesto_milesimas")?,
                tax_name: c.try_get("nombre_impuesto")?,
                labor_days: c.try_get("dias_garantia_mano_obra")?,
                parts_days: c.try_get("dias_garantia_refacciones")?,
                logo_name: d
                    .try_get::<Option<String>, _>("logo_nombre")?
                    .unwrap_or_default(),
                logo_print: d.try_get("usar_logo_impresion")?,
                logo_web: d.try_get("usar_logo_web")?,
                entry_footer: d.try_get("pie_entrada")?,
                exit_footer: d.try_get("pie_salida")?,
                print_address: d.try_get("imprimir_direccion")?,
                signature: d.try_get("pedir_firma")?,
                web_reference: d.try_get("referencia_web")?,
                clause_position: d.try_get("posicion_clausulas")?,
                number_clauses: d.try_get("numerar_clausulas")?,
                paper: d.try_get("papel")?,
                orientation: d.try_get("orientacion")?,
                copies: d.try_get("copias")?,
                prefix: d.try_get("prefijo_folio")?,
                logo: String::new(),
                next_number,
                clauses,
                revision: c.try_get("revision")?,
            },
            d.try_get("logo_ruta")?,
        )))
    }
    pub async fn save(
        &self,
        db: &mut SqliteConnection,
        value: &CompanySettings,
        logo: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO empresa(id,nombre,rfc,telefono,whatsapp,correo,sitio_web,direccion,horario,pais,locale,zona_horaria,codigo_moneda,mostrar_impuesto,tasa_impuesto_milesimas,nombre_impuesto,dias_garantia_mano_obra,dias_garantia_refacciones) VALUES(1,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET nombre=excluded.nombre,rfc=excluded.rfc,telefono=excluded.telefono,whatsapp=excluded.whatsapp,correo=excluded.correo,sitio_web=excluded.sitio_web,direccion=excluded.direccion,horario=excluded.horario,pais=excluded.pais,locale=excluded.locale,zona_horaria=excluded.zona_horaria,codigo_moneda=excluded.codigo_moneda,mostrar_impuesto=excluded.mostrar_impuesto,tasa_impuesto_milesimas=excluded.tasa_impuesto_milesimas,nombre_impuesto=excluded.nombre_impuesto,dias_garantia_mano_obra=excluded.dias_garantia_mano_obra,dias_garantia_refacciones=excluded.dias_garantia_refacciones,revision=empresa.revision+1").bind(&value.name).bind(&value.tax_id).bind(&value.phone).bind(&value.whatsapp).bind(&value.email).bind(&value.website).bind(&value.address).bind(&value.hours).bind(&value.country).bind(&value.locale).bind(&value.timezone).bind(&value.currency).bind(&value.show_tax).bind(&value.tax_rate_thousandths).bind(&value.tax_name).bind(&value.labor_days).bind(&value.parts_days).execute(&mut *db).await?;
        sqlx::query("INSERT INTO configuracion_documentos(id,logo_nombre,usar_logo_impresion,usar_logo_web,pie_entrada,pie_salida,imprimir_direccion,pedir_firma,referencia_web,posicion_clausulas,numerar_clausulas,papel,orientacion,copias,prefijo_folio,logo_ruta) VALUES(1,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET logo_nombre=excluded.logo_nombre,usar_logo_impresion=excluded.usar_logo_impresion,usar_logo_web=excluded.usar_logo_web,pie_entrada=excluded.pie_entrada,pie_salida=excluded.pie_salida,imprimir_direccion=excluded.imprimir_direccion,pedir_firma=excluded.pedir_firma,referencia_web=excluded.referencia_web,posicion_clausulas=excluded.posicion_clausulas,numerar_clausulas=excluded.numerar_clausulas,papel=excluded.papel,orientacion=excluded.orientacion,copias=excluded.copias,prefijo_folio=excluded.prefijo_folio,logo_ruta=excluded.logo_ruta,revision=configuracion_documentos.revision+1").bind(&value.logo_name).bind(&value.logo_print).bind(&value.logo_web).bind(&value.entry_footer).bind(&value.exit_footer).bind(&value.print_address).bind(&value.signature).bind(&value.web_reference).bind(&value.clause_position).bind(&value.number_clauses).bind(&value.paper).bind(&value.orientation).bind(&value.copies).bind(&value.prefix).bind(logo).execute(&mut *db).await?;
        sqlx::query("INSERT INTO secuencias_folio(prefijo,siguiente_numero) VALUES(?,?) ON CONFLICT(prefijo) DO UPDATE SET siguiente_numero=max(siguiente_numero,excluded.siguiente_numero)").bind(&value.prefix).bind(value.next_number).execute(&mut *db).await?;
        sqlx::query("DELETE FROM clausulas")
            .execute(&mut *db)
            .await?;
        for (position, clause) in value.clauses.iter().enumerate() {
            sqlx::query("INSERT INTO clausulas(titulo,texto,aplica_entrada,aplica_salida,posicion) VALUES(?,?,?,?,?)").bind(&clause.title).bind(&clause.text).bind(clause.entry).bind(clause.exit).bind(position as i64).execute(&mut *db).await?;
        }
        Ok(())
    }
}
