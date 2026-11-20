#[cfg(test)]
#[path = "../legacy/access.rs"]
mod access;
pub mod commands;
pub mod db;
pub mod domain;
pub mod error;
pub mod repositories;
pub mod services;
#[cfg(test)]
mod test_support;

#[cfg(test)]
#[path = "../legacy/company.rs"]
mod company;
#[cfg(test)]
#[path = "../legacy/evidence.rs"]
mod evidence;
#[cfg(test)]
#[path = "../legacy/finances.rs"]
mod finances;
#[cfg(test)]
#[path = "../legacy/logbook.rs"]
mod logbook;
#[cfg(test)]
#[path = "../legacy/orders.rs"]
mod orders;
#[cfg(test)]
#[path = "../legacy/payments.rs"]
mod payments;
#[cfg(test)]
use domain::permissions;
#[cfg(test)]
#[path = "../legacy/repairs.rs"]
mod repairs;
#[cfg(test)]
#[path = "../legacy/users.rs"]
mod users;
mod windowing;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::BackendState::default())
        .manage(windowing::WindowProfileState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::portal::portal_settings,
            commands::portal::configure_portal,
            commands::portal::connect_portal,
            commands::portal::pause_portal,
            commands::portal::resume_portal,
            commands::portal::disconnect_portal,
            commands::portal::portal_order_status,
            commands::portal::portal_order_action,
            commands::backup_status,
            commands::configure_backup,
            commands::create_backup,
            commands::automatic_backup,
            commands::open_backup_folder,
            commands::restore_backup,
            commands::restart_after_restore,
            commands::check_setup_status,
            commands::list_login_users,
            commands::sync_normalized_window_profile,
            commands::load_normalized_company,
            commands::refresh_normalized_session,
            commands::list_normalized_users,
            commands::save_normalized_user,
            commands::edit_normalized_order,
            commands::trash_normalized_order,
            commands::list_normalized_evidence,
            commands::add_normalized_evidence,
            commands::update_normalized_evidence,
            commands::read_normalized_evidence,
            commands::list_normalized_logbook,
            commands::list_normalized_finances,
            commands::add_normalized_expense,
            commands::load_normalized_repair,
            commands::save_normalized_repair,
            commands::list_normalized_stages,
            commands::hide_normalized_stage_name,
            commands::normalized_repair_summary,
            commands::list_normalized_payments,
            commands::record_normalized_payment,
            commands::save_normalized_company,
            commands::generate_recovery_challenge,
            commands::verify_recovery_signature,
            commands::initialize_setup,
            commands::login_user,
            commands::logout_user,
            commands::request_temporary_access,
            commands::finish_temporary_access,
            commands::create_order,
            commands::get_order,
            commands::list_reception_orders,
            commands::find_reception_order,
            commands::get_reception_order_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
