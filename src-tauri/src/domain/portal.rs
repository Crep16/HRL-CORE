use chrono::{
    DateTime, LocalResult, NaiveDate, NaiveDateTime, SecondsFormat, TimeDelta, TimeZone, Utc,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionError(pub &'static str);

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ProjectionError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct PublicTimestamp(String);

impl PublicTimestamp {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_workshop_time(value: &str, timezone: &str) -> Result<Self, ProjectionError> {
        if DateTime::parse_from_rfc3339(value).is_ok() {
            return Self::parse(value);
        }
        if value.len() == 10 {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|_| ProjectionError("Fecha de calendario inválida."))?;
            return Self::parse(&format!("{value}T00:00:00.000Z"));
        }
        let local = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
            .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f"))
            .map_err(|_| ProjectionError("Revisa la fecha y hora del avance."))?;
        let zone: chrono_tz::Tz = timezone
            .parse()
            .map_err(|_| ProjectionError("Revisa la zona horaria configurada en Empresa."))?;
        match zone.from_local_datetime(&local) {
            LocalResult::Single(date) => Self::parse(&date.to_rfc3339()),
            LocalResult::Ambiguous(_, _) => Err(ProjectionError(
                "La hora del avance se repite por el cambio de horario. Elige una hora fuera de ese cambio.",
            )),
            LocalResult::None => Err(ProjectionError(
                "La hora del avance no existe por el cambio de horario. Revisa la fecha y hora.",
            )),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProjectionError> {
        let date = DateTime::parse_from_rfc3339(value)
            .map_err(|_| {
                ProjectionError("La fecha pública requiere hora y zona horaria explícitas.")
            })?
            .with_timezone(&Utc);
        if date.timestamp_subsec_nanos() % 1_000_000 != 0
            || !(-62_135_596_800_000..=253_402_300_799_999).contains(&date.timestamp_millis())
        {
            return Err(ProjectionError(
                "La fecha pública debe estar entre los años 1 y 9999 y usar precisión de milisegundos.",
            ));
        }
        Ok(Self(date.to_rfc3339_opts(SecondsFormat::Millis, true)))
    }

    fn expires_after_delivery(&self) -> Result<Self, ProjectionError> {
        let delivery = DateTime::parse_from_rfc3339(&self.0)
            .map_err(|_| ProjectionError("Fecha de entrega inválida."))?;
        let expiration = delivery
            .checked_add_signed(TimeDelta::days(7))
            .ok_or(ProjectionError("El vencimiento supera el rango de fechas."))?;
        Self::parse(&expiration.to_rfc3339())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicEquipment {
    tipo: String,
    marca: String,
    modelo: String,
}
impl PublicEquipment {
    pub fn new(kind: String, brand: String, model: String) -> Result<Self, ProjectionError> {
        text(&kind, 60, true)?;
        text(&brand, 60, false)?;
        text(&model, 100, false)?;
        Ok(Self {
            tipo: kind,
            marca: brand,
            modelo: model,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicStage {
    nombre: String,
    completada: bool,
    #[serde(rename = "completadaEn", skip_serializing_if = "Option::is_none")]
    completada_en: Option<PublicTimestamp>,
}
impl PublicStage {
    pub fn new(
        name: String,
        completed: bool,
        at: Option<PublicTimestamp>,
    ) -> Result<Self, ProjectionError> {
        text(&name, 100, true)?;
        if !completed && at.is_some() {
            return Err(ProjectionError(
                "Una etapa pendiente no puede tener fecha de completado.",
            ));
        }
        Ok(Self {
            nombre: name,
            completada: completed,
            completada_en: at,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct PublicEvent {
    fecha: PublicTimestamp,
    titulo: String,
    detalle: String,
}

#[derive(Debug, Clone, Serialize)]
struct PublicWarranty {
    cobertura: String,
    dias: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicPayment {
    moneda: String,
    total_centavos: i64,
    cobrado_centavos: i64,
    pendiente_centavos: i64,
}
impl PublicPayment {
    pub fn new(
        currency: String,
        total: i64,
        collected: i64,
        pending: i64,
    ) -> Result<Self, ProjectionError> {
        if currency.len() != 3
            || !currency.bytes().all(|b| b.is_ascii_uppercase())
            || [total, collected, pending]
                .iter()
                .any(|n| !(0..=MAX_SAFE_INTEGER).contains(n))
            || pending != (total - collected).max(0)
        {
            return Err(ProjectionError(
                "El resumen de pago público no coincide con el saldo de la orden.",
            ));
        }
        Ok(Self {
            moneda: currency,
            total_centavos: total,
            cobrado_centavos: collected,
            pendiente_centavos: pending,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PublicSections {
    pub events: bool,
    pub note: bool,
    pub warranties: bool,
    pub payment: bool,
}

pub struct ProjectionInput {
    pub equipment: PublicEquipment,
    pub technical_status: String,
    pub stages: Vec<PublicStage>,
    pub events: Vec<super::repair_contracts::Event>,
    pub public_note: String,
    pub warranties: Vec<super::repair_contracts::Warranty>,
    pub payment: Option<PublicPayment>,
    pub sections: PublicSections,
    pub received_at: Option<PublicTimestamp>,
    pub estimated_delivery_at: Option<PublicTimestamp>,
    pub delivered_at: Option<PublicTimestamp>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackingContent {
    equipo: PublicEquipment,
    estado_tecnico: String,
    etapas: Vec<PublicStage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eventos: Option<Vec<PublicEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nota_publica: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    garantias: Option<Vec<PublicWarranty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pago: Option<PublicPayment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recibido_en: Option<PublicTimestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entrega_estimada: Option<PublicTimestamp>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicProjection {
    version_contrato: u8,
    id_publico: String,
    revision: i64,
    publicado: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    entregado_en: Option<PublicTimestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    acceso_hasta: Option<PublicTimestamp>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    content: Option<TrackingContent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignedProjection {
    #[serde(flatten)]
    projection: PublicProjection,
    huella: String,
}

impl PublicProjection {
    pub fn publish(
        id: String,
        revision: i64,
        input: ProjectionInput,
    ) -> Result<Self, ProjectionError> {
        let mut projection = Self::withdraw(id, revision, input.delivered_at)?;
        text(&input.technical_status, 100, true)?;
        if input.stages.len() > 30 {
            return Err(ProjectionError(
                "El portal admite hasta 30 etapas; la orden local se conserva.",
            ));
        }
        let events = if input.sections.events {
            let mut events = Vec::new();
            for event in input.events.into_iter().filter(|event| event.public) {
                text(&event.title, 200, true)?;
                text(&event.detail, 2000, false)?;
                events.push(PublicEvent {
                    fecha: PublicTimestamp::parse(&event.date)?,
                    titulo: event.title,
                    detalle: event.detail,
                });
            }
            if events.len() > 200 {
                return Err(ProjectionError(
                    "El portal admite hasta 200 avances públicos; la orden local se conserva.",
                ));
            }
            Some(events)
        } else {
            None
        };
        let note = if input.sections.note {
            text(&input.public_note, 4000, false)?;
            Some(input.public_note)
        } else {
            None
        };
        let warranties = if input.sections.warranties {
            if input.warranties.len() > 20 {
                return Err(ProjectionError(
                    "El portal admite hasta 20 garantías; la orden local se conserva.",
                ));
            }
            let mut result = Vec::new();
            for warranty in input.warranties {
                text(&warranty.name, 200, true)?;
                if !(0..=MAX_SAFE_INTEGER).contains(&warranty.days) {
                    return Err(ProjectionError(
                        "Los días de garantía pública no son válidos.",
                    ));
                }
                result.push(PublicWarranty {
                    cobertura: warranty.name,
                    dias: warranty.days,
                });
            }
            Some(result)
        } else {
            None
        };
        let payment = if input.sections.payment {
            Some(input.payment.ok_or(ProjectionError(
                "Falta el saldo para publicar el resumen de pago.",
            ))?)
        } else {
            None
        };
        projection.publicado = true;
        projection.content = Some(TrackingContent {
            equipo: input.equipment,
            estado_tecnico: input.technical_status,
            etapas: input.stages,
            eventos: events,
            nota_publica: note,
            garantias: warranties,
            pago: payment,
            recibido_en: input.received_at,
            entrega_estimada: input.estimated_delivery_at,
        });
        Ok(projection)
    }

    pub fn withdraw(
        id: String,
        revision: i64,
        delivered_at: Option<PublicTimestamp>,
    ) -> Result<Self, ProjectionError> {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || !(1..=MAX_SAFE_INTEGER).contains(&revision)
        {
            return Err(ProjectionError("Código público o revisión inválidos."));
        }
        let expires = delivered_at
            .as_ref()
            .map(PublicTimestamp::expires_after_delivery)
            .transpose()?;
        Ok(Self {
            version_contrato: 1,
            id_publico: id,
            revision,
            publicado: false,
            entregado_en: delivered_at,
            acceso_hasta: expires,
            content: None,
        })
    }

    pub fn sign(self) -> Result<SignedProjection, ProjectionError> {
        let value = serde_json::to_value(&self)
            .map_err(|_| ProjectionError("No se pudo preparar el documento público."))?;
        let mut canonical = String::new();
        canonical_json(&value, &mut canonical)?;
        let huella = format!("{:x}", Sha256::digest(canonical.as_bytes()));
        Ok(SignedProjection {
            projection: self,
            huella,
        })
    }
}

impl SignedProjection {
    pub fn document(&self) -> Result<serde_json::Value, ProjectionError> {
        serde_json::to_value(self)
            .map_err(|_| ProjectionError("No se pudo preparar el documento público."))
    }
    pub fn fingerprint(&self) -> &str {
        &self.huella
    }
    pub fn revision(&self) -> i64 {
        self.projection.revision
    }
    pub fn public_id(&self) -> &str {
        &self.projection.id_publico
    }
}

pub fn fingerprint_document(document: &serde_json::Value) -> Result<String, ProjectionError> {
    let mut content = document
        .as_object()
        .cloned()
        .ok_or(ProjectionError("Documento público inválido."))?;
    content.remove("huella");
    content.remove("actualizadoEn");
    let mut canonical = String::new();
    canonical_json(&serde_json::Value::Object(content), &mut canonical)?;
    Ok(format!("{:x}", Sha256::digest(canonical.as_bytes())))
}

fn text(value: &str, max: usize, required: bool) -> Result<(), ProjectionError> {
    if value.chars().count() > max || (required && value.trim().is_empty()) {
        return Err(ProjectionError(
            "Un texto público está vacío o supera el límite del contrato.",
        ));
    }
    Ok(())
}

fn canonical_json(value: &serde_json::Value, out: &mut String) -> Result<(), ProjectionError> {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (index, (key, item)) in entries.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(
                    &serde_json::to_string(key)
                        .map_err(|_| ProjectionError("Clave pública inválida."))?,
                );
                out.push(':');
                canonical_json(item, out)?;
            }
            out.push('}');
        }
        serde_json::Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                canonical_json(item, out)?;
            }
            out.push(']');
        }
        _ => out.push_str(
            &serde_json::to_string(value)
                .map_err(|_| ProjectionError("Valor público inválido."))?,
        ),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
