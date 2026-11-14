use super::{access::AccessService, audit::AuditService, crypto::CryptoService};
use crate::{
    domain::contracts::*,
    error::AppError,
    repositories::{audit::AuditRecord, orders::OrderRepository},
};
use sqlx::SqlitePool;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

pub struct OrderService {
    pool: SqlitePool,
    repository: Arc<OrderRepository>,
    access: Arc<AccessService>,
    audit: Arc<AuditService>,
    crypto: Arc<CryptoService>,
}

fn invalid(message: &str) -> AppError {
    AppError::Validation(message.into())
}
fn validate(payload: &CreateOrderPayload) -> Result<(), AppError> {
    if payload.client.full_name.trim().is_empty()
        || payload.client.phone.trim().is_empty()
        || payload.equipment.item.trim().is_empty()
        || payload.reported_failure.trim().is_empty()
        || payload.status.trim().is_empty()
        || payload.received_at.trim().is_empty()
    {
        return Err(invalid(
            "Completa cliente, teléfono, artículo, fecha, estado y falla reportada.",
        ));
    }
    if payload.folio_prefix.is_empty()
        || payload.folio_prefix.chars().count() > 12
        || !payload
            .folio_prefix
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        || !(1..=999999999).contains(&payload.sequence_start)
    {
        return Err(invalid(
            "Revisa el prefijo y el siguiente número del folio.",
        ));
    }
    if !(0..=9_000_000_000_000).contains(&payload.price_minor) {
        return Err(invalid(
            "El precio debe ser un entero de centavos dentro del rango permitido.",
        ));
    }
    if payload.client.id.is_some_and(|id| id <= 0) || payload.equipment.id.is_some_and(|id| id <= 0)
    {
        return Err(invalid(
            "Selecciona identificadores válidos de cliente y equipo.",
        ));
    }
    if payload
        .history
        .notes
        .as_ref()
        .is_some_and(|text| text.chars().count() > 16000)
    {
        return Err(invalid("Los antecedentes admiten hasta 16000 caracteres."));
    }
    if [
        &payload.client.full_name,
        &payload.client.phone,
        &payload.equipment.item,
        &payload.equipment.brand,
        &payload.equipment.model,
        &payload.equipment.serial,
        &payload.status,
        &payload.planned_work,
        &payload.reception_notes,
        &payload.history.previous_workshop_detail,
        &payload.history.impact_detail,
        &payload.history.liquid_detail,
    ]
    .iter()
    .any(|s| s.len() > 16000)
        || payload.reported_failure.chars().count() > 16000
        || payload
            .device_password
            .as_ref()
            .is_some_and(|s| s.len() > 16000)
        || payload
            .client
            .email
            .as_ref()
            .is_some_and(|s| s.trim().is_empty() || s.len() > 254)
        || payload.received_at.len() > 100
        || payload
            .estimated_delivery_at
            .as_ref()
            .is_some_and(|s| s.trim().is_empty() || s.len() > 100)
    {
        return Err(invalid(
            "Un campo supera el tamaño permitido o contiene un valor vacío no admitido.",
        ));
    }
    if let Some(payment) = &payload.initial_payment {
        if payment.amount_minor <= 0
            || payment.amount_minor > payload.price_minor
            || payment.method.trim().is_empty()
            || payment.method.len() > 200
            || !(16..=100).contains(&payment.request_id.len())
        {
            return Err(invalid(
                "El abono inicial requiere método, identificador único e importe positivo que no supere el precio.",
            ));
        }
    }
    Ok(())
}

impl OrderService {
    pub async fn edit(&self, payload: EditOrderPayload) -> Result<OrderResponse, AppError> {
        validate(&payload.value)?;
        let p = &payload.value;
        if p.initial_payment.is_some() {
            return Err(invalid("Registra los abonos desde Reparaciones."));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let author = self
            .access
            .require(
                &mut tx,
                "orders.edit",
                AccessScope::Clients,
                Some(payload.order_id),
                true,
            )
            .await?;
        let old = self
            .repository
            .load(&mut tx, payload.order_id)
            .await?
            .ok_or(AppError::NotFound)?;
        if old.revision != payload.revision {
            return Err(AppError::Conflict(
                "La orden cambió. Conserva tus cambios y vuelve a consultarla.".into(),
            ));
        }
        if old.currency != p.currency {
            return Err(invalid(
                "La moneda de una orden guardada no se puede cambiar.",
            ));
        }
        let client = if old.client.full_name == p.client.full_name
            && old.client.phone == p.client.phone
            && old.client.email == p.client.email
        {
            old.client_id
        } else {
            self.repository.insert_client(&mut tx, &p.client).await?
        };
        let equipment = if client == old.client_id
            && old.equipment.item == p.equipment.item
            && old.equipment.brand == p.equipment.brand
            && old.equipment.model == p.equipment.model
            && old.equipment.serial == p.equipment.serial
        {
            old.equipment_id
        } else {
            self.repository
                .insert_equipment(&mut tx, client, &p.equipment)
                .await?
        };
        let status = self.repository.status(&mut tx, &p.status).await?;
        self.repository
            .update_order(&mut tx, old.id, equipment, status, p, payload.delivered)
            .await?;
        if payload.change_password {
            self.repository.remove_credential(&mut tx, old.id).await?;
            if let Some(password) = p.device_password.as_ref().filter(|s| !s.is_empty()) {
                self.repository
                    .insert_credential(&mut tx, &self.crypto.encrypt(old.id, password)?)
                    .await?;
            }
        }
        let mut changes = vec![];
        for (field, old_value, new_value) in [
            ("email", &old.client.email, &p.client.email),
            ("equipmentHistory", &old.history.notes, &p.history.notes),
            (
                "estimatedDelivery",
                &old.estimated_delivery_at,
                &p.estimated_delivery_at,
            ),
        ] {
            if old_value != new_value {
                changes.push((field, old_value.clone(), new_value.clone()));
            }
        }
        for (field, a, b) in [
            ("client", &old.client.full_name, &p.client.full_name),
            ("phone", &old.client.phone, &p.client.phone),
            ("item", &old.equipment.item, &p.equipment.item),
            ("brand", &old.equipment.brand, &p.equipment.brand),
            ("model", &old.equipment.model, &p.equipment.model),
            ("serial", &old.equipment.serial, &p.equipment.serial),
            ("status", &old.status, &p.status),
            ("received", &old.received_at, &p.received_at),
            ("failure", &old.reported_failure, &p.reported_failure),
            ("work", &old.planned_work, &p.planned_work),
            ("notes", &old.reception_notes, &p.reception_notes),
            (
                "other_shop_reason",
                &old.history.previous_workshop_detail,
                &p.history.previous_workshop_detail,
            ),
            (
                "impactDetail",
                &old.history.impact_detail,
                &p.history.impact_detail,
            ),
            (
                "liquidDetail",
                &old.history.liquid_detail,
                &p.history.liquid_detail,
            ),
        ] {
            if a != b {
                changes.push((field, Some(a.clone()), Some(b.clone())));
            }
        }
        if old.price_minor != p.price_minor {
            changes.push((
                "price_minor",
                Some(old.price_minor.to_string()),
                Some(p.price_minor.to_string()),
            ));
        }
        for (field, a, b) in [
            ("delivered", old.delivered, payload.delivered),
            (
                "other_shop",
                old.history.previous_workshop,
                p.history.previous_workshop,
            ),
            (
                "sufferedImpact",
                old.history.suffered_impact,
                p.history.suffered_impact,
            ),
            (
                "liquidSpill",
                old.history.liquid_spill,
                p.history.liquid_spill,
            ),
        ] {
            if a != b {
                changes.push((field, Some(a.to_string()), Some(b.to_string())));
            }
        }
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: author.id,
                    operation: "orders.edit",
                    entity: "ordenes",
                    entity_id: Some(old.id),
                    order_id: Some(old.id),
                    reason: None,
                },
                &changes,
            )
            .await?;
        let result = self
            .repository
            .load(&mut tx, old.id)
            .await?
            .ok_or(AppError::NotFound)?;
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(result)
    }
    pub async fn trash(&self, p: TrashOrderPayload) -> Result<(), AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let user = self
            .access
            .require(
                &mut tx,
                "orders.delete",
                AccessScope::Clients,
                Some(p.order_id),
                true,
            )
            .await?;
        let old = self
            .repository
            .load_any(&mut tx, p.order_id, true)
            .await?
            .ok_or(AppError::NotFound)?;
        if old.revision != p.revision {
            return Err(AppError::Conflict(
                "La orden cambió. Actualiza la ficha antes de enviarla a papelera o restaurarla."
                    .into(),
            ));
        }
        self.repository
            .trash(&mut tx, p.order_id, p.deleted)
            .await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: user.id,
                    operation: if p.deleted {
                        "orders.delete"
                    } else {
                        "orders.restore"
                    },
                    entity: "ordenes",
                    entity_id: Some(p.order_id),
                    order_id: Some(p.order_id),
                    reason: None,
                },
                &[(
                    "deleted",
                    Some((!p.deleted).to_string()),
                    Some(p.deleted.to_string()),
                )],
            )
            .await?;
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(())
    }
    pub async fn list(
        &self,
        mut payload: ListOrdersPayload,
    ) -> Result<OrderListResponse, AppError> {
        if payload.query.chars().count() > 100
            || payload.offset < 0
            || !(1..=100).contains(&payload.limit)
        {
            return Err(invalid(
                "Revisa la búsqueda y la página de órdenes solicitada.",
            ));
        }
        payload.query = payload.query.trim().to_lowercase();
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(&mut tx, "clients.view", AccessScope::Clients, None, false)
            .await?;
        let result = self.repository.list(&mut tx, &payload).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn find(&self, payload: FindOrderPayload) -> Result<OrderResponse, AppError> {
        let reference = payload.reference.trim();
        if reference.is_empty() || reference.chars().count() > 100 {
            return Err(invalid("Escribe un folio o identificador de orden válido."));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(&mut tx, "clients.view", AccessScope::Clients, None, false)
            .await?;
        let id = self
            .repository
            .find_id(&mut tx, reference)
            .await?
            .ok_or(AppError::NotFound)?;
        let result = self
            .repository
            .load(&mut tx, id)
            .await?
            .ok_or(AppError::NotFound)?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn history(
        &self,
        payload: OrderHistoryPayload,
    ) -> Result<OrderHistoryResponse, AppError> {
        if payload.order_id <= 0 || payload.offset < 0 || !(1..=50).contains(&payload.limit) {
            return Err(invalid(
                "Selecciona una orden y una página de historial válidas.",
            ));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(
                &mut tx,
                "logbook.view",
                AccessScope::Logbook,
                Some(payload.order_id),
                false,
            )
            .await?;
        self.repository
            .load_any(&mut tx, payload.order_id, true)
            .await?
            .ok_or(AppError::NotFound)?;
        let mut result = self.repository.history(&mut tx, &payload).await?;
        for event in &mut result.items {
            event.changes.retain(|change| {
                let field = change.field.to_lowercase();
                ![
                    "password", "contras", "clave", "hash", "nonce", "cifrado", "token",
                ]
                .iter()
                .any(|secret| field.contains(secret))
            });
        }
        tx.commit().await?;
        Ok(result)
    }

    pub fn new(
        pool: SqlitePool,
        repository: Arc<OrderRepository>,
        access: Arc<AccessService>,
        audit: Arc<AuditService>,
        crypto: Arc<CryptoService>,
    ) -> Self {
        Self {
            pool,
            repository,
            access,
            audit,
            crypto,
        }
    }
    pub async fn create(&self, payload: CreateOrderPayload) -> Result<OrderResponse, AppError> {
        validate(&payload)?;
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let author = self
            .access
            .require(&mut tx, "orders.create", AccessScope::Clients, None, false)
            .await?;
        if payload.initial_payment.is_some() {
            self.access
                .require(&mut tx, "payments.add", AccessScope::Clients, None, false)
                .await?;
        }
        let fingerprint = if payload.initial_payment.is_some() {
            let bytes = Zeroizing::new(
                serde_json::to_vec(&(author.id, &payload))
                    .map_err(|_| invalid("No se pudo preparar la solicitud."))?,
            );
            Some(self.crypto.request_fingerprint(&bytes)?)
        } else {
            None
        };
        if let Some(payment) = &payload.initial_payment {
            if let Some(previous) = self
                .repository
                .request(&mut tx, &payment.request_id)
                .await?
            {
                if previous.author != author.id
                    || previous.kind != "receipt"
                    || !bool::from(
                        previous
                            .fingerprint
                            .as_slice()
                            .ct_eq(fingerprint.as_ref().ok_or(AppError::Crypto)?.as_slice()),
                    )
                {
                    return Err(AppError::Conflict(
                        "El identificador del abono ya corresponde a otra solicitud.".into(),
                    ));
                }
                let result = self
                    .repository
                    .load(
                        &mut tx,
                        previous.order_id.ok_or_else(|| {
                            AppError::Conflict("La solicitud previa requiere revisión.".into())
                        })?,
                    )
                    .await?
                    .ok_or(AppError::NotFound)?;
                tx.commit().await.map_err(AppError::Commit)?;
                return Ok(result);
            }
        }
        self.access
            .require(&mut tx, "orders.create", AccessScope::Clients, None, true)
            .await?;
        if payload.initial_payment.is_some() {
            self.access
                .require(&mut tx, "payments.add", AccessScope::Clients, None, true)
                .await?;
        }
        if !self
            .repository
            .currency_exists(&mut tx, &payload.currency)
            .await?
        {
            return Err(invalid("Selecciona una moneda disponible."));
        }
        let client_input = ClientInput {
            id: payload.client.id,
            full_name: payload.client.full_name.trim().into(),
            phone: payload.client.phone.trim().into(),
            email: payload.client.email.as_ref().map(|s| s.trim().into()),
        };
        let client =
            if let Some(id) = client_input.id {
                let existing = self
                    .repository
                    .client(&mut tx, id)
                    .await?
                    .ok_or(AppError::NotFound)?;
                if existing.full_name != client_input.full_name
                    || existing.phone != client_input.phone
                    || existing.email != client_input.email
                {
                    return Err(AppError::Conflict(
                        "Los datos no coinciden con el cliente seleccionado; vuelve a consultarlo."
                            .into(),
                    ));
                }
                id
            } else {
                let matches = self
                    .repository
                    .clients_matching(&mut tx, &client_input)
                    .await?;
                match matches.as_slice() {
                    [id] => *id,
                    [] => {
                        self.repository
                            .insert_client(&mut tx, &client_input)
                            .await?
                    }
                    _ => return Err(AppError::Conflict(
                        "Hay varios clientes coincidentes; selecciona uno por su identificador."
                            .into(),
                    )),
                }
            };
        let equipment_input = EquipmentInput {
            id: payload.equipment.id,
            item: payload.equipment.item.trim().into(),
            brand: payload.equipment.brand.clone(),
            model: payload.equipment.model.clone(),
            serial: payload.equipment.serial.clone(),
        };
        let equipment = if let Some(id) = equipment_input.id {
            let (owner, existing) = self
                .repository
                .equipment(&mut tx, id)
                .await?
                .ok_or(AppError::NotFound)?;
            if owner != client
                || existing.item != equipment_input.item
                || existing.brand != equipment_input.brand
                || existing.model != equipment_input.model
                || existing.serial != equipment_input.serial
            {
                return Err(AppError::Conflict(
                    "El equipo no pertenece al cliente o sus datos cambiaron.".into(),
                ));
            }
            id
        } else if !equipment_input.serial.trim().is_empty() {
            let matches = self
                .repository
                .equipment_matching(&mut tx, client, &equipment_input)
                .await?;
            match matches.as_slice() {
                [id] => *id,
                [] => {
                    self.repository
                        .insert_equipment(&mut tx, client, &equipment_input)
                        .await?
                }
                _ => {
                    return Err(AppError::Conflict(
                        "Hay varios equipos coincidentes; selecciona uno por su identificador."
                            .into(),
                    ));
                }
            }
        } else {
            self.repository
                .insert_equipment(&mut tx, client, &equipment_input)
                .await?
        };
        let folio = self
            .repository
            .reserve_folio(&mut tx, &payload.folio_prefix, payload.sequence_start)
            .await?;
        let status = self.repository.status(&mut tx, &payload.status).await?;
        let id = self
            .repository
            .insert_order(&mut tx, &folio, equipment, author.id, status, &payload)
            .await?;
        if let Some(password) = payload.device_password.as_ref().filter(|p| !p.is_empty()) {
            let encrypted = self.crypto.encrypt(id, password)?;
            self.repository
                .insert_credential(&mut tx, &encrypted)
                .await?;
        }
        if let Some(payment) = &payload.initial_payment {
            let receipt = self
                .repository
                .initial_payment(
                    &mut tx,
                    id,
                    author.id,
                    payment,
                    fingerprint.as_ref().ok_or(AppError::Crypto)?,
                )
                .await?;
            self.audit
                .record(
                    &mut tx,
                    AuditRecord {
                        author_id: author.id,
                        operation: "payments.add",
                        entity: "abonos",
                        entity_id: Some(receipt),
                        order_id: Some(id),
                        reason: Some("Abono inicial de recepción"),
                    },
                    &[
                        ("amountMinor", None, Some(payment.amount_minor.to_string())),
                        ("method", None, Some(payment.method.clone())),
                    ],
                )
                .await?;
        }
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: author.id,
                    operation: "orders.create",
                    entity: "ordenes",
                    entity_id: Some(id),
                    order_id: Some(id),
                    reason: None,
                },
                &[
                    ("folio", None, Some(folio)),
                    ("clientId", None, Some(client.to_string())),
                    ("equipmentId", None, Some(equipment.to_string())),
                    ("priceMinor", None, Some(payload.price_minor.to_string())),
                    ("equipmentHistory", None, payload.history.notes.clone()),
                ],
            )
            .await?;
        let result = self
            .repository
            .load(&mut tx, id)
            .await?
            .ok_or(AppError::NotFound)?;
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(result)
    }
    pub async fn get(&self, payload: GetOrderPayload) -> Result<OrderResponse, AppError> {
        if payload.order_id <= 0
            || !matches!(payload.scope, AccessScope::Clients | AccessScope::Repairs)
            || (payload.include_deleted && payload.scope != AccessScope::Clients)
        {
            return Err(invalid("Selecciona una orden y un apartado válidos."));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let operation = if payload.scope == AccessScope::Clients {
            "clients.view"
        } else {
            "repairs.view"
        };
        self.access
            .require(
                &mut tx,
                operation,
                payload.scope,
                Some(payload.order_id),
                false,
            )
            .await?;
        let mut result = self
            .repository
            .load_any(&mut tx, payload.order_id, payload.include_deleted)
            .await?
            .ok_or(AppError::NotFound)?;
        if payload.include_device_password {
            if let Some(encrypted) = self
                .repository
                .credential(&mut tx, payload.order_id)
                .await?
            {
                result.device_password = Some(self.crypto.decrypt(&encrypted)?);
            }
        }
        tx.commit().await?;
        Ok(result)
    }
}
