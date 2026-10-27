mod access;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(access::AccessState::default())
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
                    ],
                )
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            access::access_status,
            access::first_setup,
            access::login,
            access::logout
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
