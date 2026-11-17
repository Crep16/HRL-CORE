use super::ledger_contracts::PaymentState;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Event {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub detail: String,
    pub stage: String,
    pub public: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Warranty {
    pub name: String,
    pub days: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Part {
    pub name: String,
    pub price: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Payment {
    pub date: String,
    pub amount: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Visibility {
    pub label: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Repair {
    pub stages: Vec<String>,
    pub completed: usize,
    pub technician: String,
    pub public_note: String,
    pub internal_note: String,
    pub sync_mode: String,
    pub events: Vec<Event>,
    pub warranties: Vec<Warranty>,
    pub parts: Vec<Part>,
    pub payments: Vec<Payment>,
    pub visibility: Vec<Visibility>,
}
impl Default for Repair {
    fn default() -> Self {
        Self {
            stages: vec!["Recibido".into(), "Entregado".into()],
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
    pub repair: Repair,
    pub revision: i64,
    pub payment: PaymentState,
}

pub fn validate(repair: &Repair) -> Result<(), String> {
    let stage_keys: std::collections::HashSet<_> = repair
        .stages
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect();
    if repair.stages.len() < 2
        || repair.stages.len() > 200
        || stage_keys.len() != repair.stages.len()
        || repair
            .stages
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 100)
        || repair.completed > repair.stages.len()
        || !["manual", "auto"].contains(&repair.sync_mode.as_str())
    {
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
            .any(|p| p.name.trim().is_empty() || !(0..=9_000_000_000_000).contains(&p.price))
        || repair
            .payments
            .iter()
            .any(|p| p.amount <= 0 || p.method.as_ref().is_some_and(|s| s.len() > 200))
    {
        return Err("Revisa los campos y los importes del seguimiento.".into());
    }
    Ok(())
}

pub fn operations(old: &Repair, repair: &Repair) -> Result<Vec<&'static str>, String> {
    let mut operations = Vec::new();
    if repair.stages != old.stages {
        if repair.stages.len() < old.stages.len() {
            let mut original = old.stages.iter();
            let only_removed_pending = repair.completed == old.completed
                && repair.stages.first() == old.stages.first()
                && repair.stages.last() == old.stages.last()
                && repair.stages.get(..old.completed) == old.stages.get(..old.completed)
                && repair
                    .stages
                    .iter()
                    .all(|stage| original.by_ref().any(|s| s == stage));
            if !only_removed_pending {
                return Err("Solo puedes eliminar etapas intermedias pendientes, sin alterar las completadas ni su orden.".into());
            }
            operations.push("repairs.edit");
        } else {
            let mut retained = repair.stages.iter();
            let preserves_order = old
                .stages
                .iter()
                .all(|stage| retained.by_ref().any(|s| s == stage));
            let added = &repair.events[..repair.events.len().saturating_sub(old.events.len())];
            let only_added_stages = preserves_order
                && repair.stages.first() == old.stages.first()
                && repair.stages.last() == old.stages.last()
                && repair.stages.get(..old.completed) == old.stages.get(..old.completed)
                && repair
                    .stages
                    .iter()
                    .filter(|s| !old.stages.contains(s))
                    .all(|s| added.iter().any(|e| &e.stage == s));
            if !only_added_stages {
                return Err("Las etapas nuevas deben corresponder a un avance y conservar las etapas anteriores.".into());
            }
            operations.push("notes.add");
        }
    }
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
            operations.push(if b.starts_with(a.as_str()) {
                "notes.add"
            } else {
                "repairs.edit"
            });
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
        return Err("Registra el cobro desde el libro de pagos.".into());
    }
    if repair.sync_mode != old.sync_mode || repair.visibility != old.visibility {
        operations.push("publication.edit");
    }

    operations.sort_unstable();
    operations.dedup();
    Ok(operations)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRepairPayload {
    pub order_id: i64,
    pub repair: Repair,
    pub revision: i64,
}
