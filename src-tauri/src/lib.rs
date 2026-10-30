mod access;
mod schema_backup;
mod evidence;
mod logbook;
mod finances;
mod company;
mod payments;
mod orders;
mod permissions;
mod repairs;
mod users;
mod windowing;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(access::AccessState::default())
        .manage(evidence::EvidenceState::default())
        .manage(windowing::WindowProfileState::default())
        .plugin(schema_backup::init())
        .plugin(
            tauri_plugin_sql::Builder::new()
                .add_migrations(
                    "sqlite:hrl_datos.db",
                    vec![
                        tauri_plugin_sql::Migration {
                            version: 1,
                            description: "users_and_permissions",
                            sql: include_str!("../migrations/001_access.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 2,
                            description: "workshop_records",
                            sql: include_str!("../migrations/002_workshop.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 3,
                            description: "local_authentication",
                            sql: include_str!("../migrations/003_authentication.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 4,
                            description: "order_records_and_legacy_import",
                            sql: include_str!("../migrations/004_order_records.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 5,
                            description: "recoverable_order_deletion",
                            sql: include_str!("../migrations/005_order_visibility.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 6,
                            description: "authorized_repair_records",
                            sql: include_str!("../migrations/006_repair_records.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 7,
                            description: "internal_logbook",
                            sql: include_str!("../migrations/007_logbook.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 8,
                            description: "financial_logbook",
                            sql: include_str!("../migrations/008_financial_logbook.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 9,
                            description: "audited_company_settings",
                            sql: include_str!("../migrations/009_company_settings.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 10,
                            description: "payment_adjustments_and_refunds",
                            sql: include_str!("../migrations/010_payment_adjustments.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 11,
                            description: "audited_temporary_access",
                            sql: include_str!("../migrations/011_temporary_access.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 12,
                            description: "local_order_evidence",
                            sql: include_str!("../migrations/012_order_evidence.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                        tauri_plugin_sql::Migration {
                            version: 13,
                            description: "nombres_del_esquema_en_espanol",
                            sql: include_str!("../migrations/013_spanish_schema_names.sql"),
                            kind: tauri_plugin_sql::MigrationKind::Up,
                        },
                    ],
                )
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            evidence::list_evidence,
            evidence::read_evidence,
            evidence::add_evidence,
            evidence::update_evidence,
            access::access_status,
            access::sync_window_profile,
            access::first_setup,
            access::login,
            access::logout,
            access::access_permissions,
            access::temporary::grant_temporary_access,
            access::temporary::end_temporary_access,
            users::save_user,
            users::list_users,
            logbook::list_logbook,
            finances::list_order_finances,
            finances::add_direct_expense,
            company::save_company_settings,
            company::load_company_settings,
            payments::list_payment_ledger,
            payments::record_payment_operation,
            orders::list_orders,
            orders::load_authorized_repair_order,
            orders::save_order,
            orders::import_orders,
            orders::next_order_number,
            orders::set_order_deleted,
            repairs::load_repair,
            repairs::save_repair,
            repairs::list_repair_stages,
            repairs::repair_print_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
