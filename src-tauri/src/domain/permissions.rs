pub const OPERATIONS: &[(&str, &str, &str)] = &[
    ("clients.view", "clients", "view"),
    ("orders.create", "clients", "create"),
    ("orders.edit", "clients", "edit"),
    ("orders.delete", "clients", "delete"),
    ("evidence.view", "clients", "view"),
    ("evidence.add", "clients", "create"),
    ("evidence.edit", "clients", "edit"),
    ("evidence.delete", "clients", "delete"),
    ("repairs.view", "repairs", "view"),
    ("repairs.edit", "repairs", "edit"),
    ("publication.edit", "repairs", "edit"),
    ("notes.add", "repairs", "create"),
    ("stages.advance", "repairs", "create"),
    ("warranties.add", "repairs", "create"),
    ("parts.add", "repairs", "create"),
    ("payments.add", "payments", "add"),
    ("payments.refund", "payments", "refund"),
    ("payments.adjust", "payments", "adjust"),
    ("company.view", "company", "view"),
    ("company.edit", "company", "edit"),
    ("logbook.view", "logbook", "view"),
    ("users.manage", "users", "manage"),
];

pub fn limited_allows(operation: &str) -> bool {
    matches!(
        operation,
        "clients.view"
            | "evidence.view"
            | "evidence.add"
            | "orders.create"
            | "repairs.view"
            | "notes.add"
            | "stages.advance"
            | "warranties.add"
            | "parts.add"
    )
}
