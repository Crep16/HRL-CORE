use sqlx::SqlitePool;

const LEGACY_MIGRATIONS: [&str; 13] = [
    include_str!("../migrations/legacy/001_access.sql"),
    include_str!("../migrations/legacy/002_workshop.sql"),
    include_str!("../migrations/legacy/003_authentication.sql"),
    include_str!("../migrations/legacy/004_order_records.sql"),
    include_str!("../migrations/legacy/005_order_visibility.sql"),
    include_str!("../migrations/legacy/006_repair_records.sql"),
    include_str!("../migrations/legacy/007_logbook.sql"),
    include_str!("../migrations/legacy/008_financial_logbook.sql"),
    include_str!("../migrations/legacy/009_company_settings.sql"),
    include_str!("../migrations/legacy/010_payment_adjustments.sql"),
    include_str!("../migrations/legacy/011_temporary_access.sql"),
    include_str!("../migrations/legacy/012_order_evidence.sql"),
    include_str!("../migrations/legacy/013_spanish_schema_names.sql"),
];

pub(crate) async fn migrate_legacy(pool: &SqlitePool) {
    migrate_legacy_range(pool, 1, 13).await;
}

pub(crate) async fn migrate_legacy_range(pool: &SqlitePool, first: usize, last: usize) {
    for migration in &LEGACY_MIGRATIONS[first - 1..last] {
        sqlx::raw_sql(migration).execute(pool).await.unwrap();
    }
}
