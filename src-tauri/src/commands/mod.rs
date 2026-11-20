//! Contratos nuevos de la base normalizada; no comparten sesión ni datos con el IPC anterior.
//! Las entradas usan camelCase. Las claves de equipos solo salen de `get_order` cuando
//! se solicita `includeDevicePassword` y el servicio comprueba acceso a la orden.
use crate::{db::Database, domain::contracts::*, error::AppError, services::BackendServices};
use tauri::{Manager, State};
use tokio::sync::OnceCell;
pub mod portal;

#[tauri::command]
pub async fn backup_status(
    payload: crate::domain::backup::BackupAuth,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::backup::BackupStatus, AppError> {
    state.services(&app).await?.backups.status(&payload).await
}
#[tauri::command]
pub async fn configure_backup(
    payload: crate::domain::backup::BackupConfiguration,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state
        .services(&app)
        .await?
        .backups
        .configure(&payload)
        .await
}
#[tauri::command]
pub async fn create_backup(
    payload: crate::domain::backup::BackupAuth,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<String, AppError> {
    state.services(&app).await?.backups.create(&payload).await
}
#[tauri::command]
pub async fn automatic_backup(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Option<String>, AppError> {
    state.services(&app).await?.backups.automatic().await
}
#[tauri::command]
pub async fn open_backup_folder(
    payload: crate::domain::backup::BackupAuth,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    use tauri_plugin_opener::OpenerExt;
    let status = state.services(&app).await?.backups.status(&payload).await?;
    app.opener()
        .open_path(status.folder, None::<&str>)
        .map_err(|_| AppError::Validation("No se pudo abrir la carpeta Respaldos.".into()))
}
#[tauri::command]
pub async fn restore_backup(
    payload: crate::domain::backup::RestoreBackup,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<bool, AppError> {
    state
        .services(&app)
        .await?
        .backups
        .authorize_restore(&payload)
        .await?;
    #[cfg(windows)]
    let owner = window.hwnd().map_err(|_| AppError::Window)?.0 as usize;
    #[cfg(not(windows))]
    let owner = {
        let _ = window;
        0
    };
    let path = tauri::async_runtime::spawn_blocking(move || {
        crate::services::backups::choose_backup(owner)
    })
    .await
    .map_err(|_| AppError::Crypto)??;
    let Some(path) = path else {
        return Ok(false);
    };
    state
        .services(&app)
        .await?
        .backups
        .restore(&path, &payload)
        .await?;
    Ok(true)
}
#[tauri::command]
pub async fn restart_after_restore(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    if !state.services(&app).await?.backups.needs_restart() {
        return Err(AppError::Forbidden);
    }
    app.restart()
}

#[tauri::command]
pub async fn refresh_normalized_session(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<SessionResponse, AppError> {
    state.services(&app).await?.access.refresh_session().await
}
#[tauri::command]
pub async fn list_login_users(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Vec<String>, AppError> {
    state.services(&app).await?.access.login_names().await
}

#[tauri::command]
pub async fn list_normalized_users(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Vec<crate::domain::user_contracts::Account>, AppError> {
    state.services(&app).await?.users.list().await
}
#[tauri::command]
pub async fn save_normalized_user(
    payload: crate::domain::user_contracts::SaveUserPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.users.save(payload).await
}

#[tauri::command]
pub async fn load_normalized_company(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Option<crate::domain::company_contracts::CompanySettings>, AppError> {
    state.services(&app).await?.company.load().await
}
#[tauri::command]
pub async fn save_normalized_company(
    payload: crate::domain::company_contracts::SaveCompanyPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::company_contracts::CompanySettings, AppError> {
    state.services(&app).await?.company.save(payload).await
}

#[derive(Default)]
pub struct BackendState {
    services: OnceCell<BackendServices>,
}
impl BackendState {
    async fn services(&self, app: &tauri::AppHandle) -> Result<&BackendServices, AppError> {
        self.services
            .get_or_try_init(|| async {
                let directory = app.path().app_config_dir().map_err(|_| AppError::Crypto)?;
                let database = Database::open_file(&directory.join("hrl_core_3fn.db")).await?;
                let services =
                    BackendServices::with_os_key_store(database, &directory.join("keys")).await?;
                services.portal.start_worker();
                Ok(services)
            })
            .await
    }
}

#[tauri::command]
pub async fn sync_normalized_window_profile(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
    layout: State<'_, crate::windowing::WindowProfileState>,
) -> Result<(), AppError> {
    state
        .services(&app)
        .await?
        .access
        .synchronize_window(|authenticated, configured| {
            layout.apply(
                &window,
                crate::windowing::Profile::for_session(authenticated, configured),
            )
        })
        .await
}

#[tauri::command]
pub async fn generate_recovery_challenge(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<RecoveryChallengeResponse, AppError> {
    state
        .services(&app)
        .await?
        .access
        .generate_recovery_challenge()
        .await
}

#[tauri::command]
pub async fn verify_recovery_signature(
    challenge: String,
    signature: String,
    new_master_password: String,
    new_admin_password: String,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<RecoveryResponse, AppError> {
    let payload = VerifyRecoveryPayload {
        challenge,
        signature,
        new_master_password,
        new_admin_password,
    };
    state
        .services(&app)
        .await?
        .access
        .verify_recovery_signature(payload)
        .await
}

#[tauri::command]
pub async fn check_setup_status(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<SetupStatus, AppError> {
    state.services(&app).await?.access.setup_status().await
}
#[tauri::command]
pub async fn initialize_setup(
    payload: SetupPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<SessionResponse, AppError> {
    state.services(&app).await?.access.initialize(payload).await
}
#[tauri::command]
pub async fn login_user(
    payload: LoginPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<SessionResponse, AppError> {
    state.services(&app).await?.access.login(payload).await
}
#[tauri::command]
pub async fn logout_user(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.access.logout().await
}
#[tauri::command]
pub async fn request_temporary_access(
    payload: TemporaryAccessPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<TemporaryAccessResponse, AppError> {
    state.services(&app).await?.access.temporary(payload).await
}
#[tauri::command]
pub async fn finish_temporary_access(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.access.finish_temporary().await
}
#[tauri::command]
pub async fn create_order(
    payload: CreateOrderPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<OrderResponse, AppError> {
    state.services(&app).await?.orders.create(payload).await
}
#[tauri::command]
pub async fn get_order(
    payload: GetOrderPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<OrderResponse, AppError> {
    state.services(&app).await?.orders.get(payload).await
}

#[tauri::command]
pub async fn list_reception_orders(
    payload: ListOrdersPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<OrderListResponse, AppError> {
    state.services(&app).await?.orders.list(payload).await
}

#[tauri::command]
pub async fn find_reception_order(
    payload: FindOrderPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<OrderResponse, AppError> {
    state.services(&app).await?.orders.find(payload).await
}

#[tauri::command]
pub async fn get_reception_order_history(
    payload: OrderHistoryPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<OrderHistoryResponse, AppError> {
    state.services(&app).await?.orders.history(payload).await
}

#[tauri::command]
pub async fn load_normalized_repair(
    order_id: i64,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::repair_contracts::Snapshot, AppError> {
    state.services(&app).await?.repairs.load(order_id).await
}

#[tauri::command]
pub async fn save_normalized_repair(
    payload: crate::domain::repair_contracts::SaveRepairPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::repair_contracts::Snapshot, AppError> {
    state.services(&app).await?.repairs.save(payload).await
}

#[tauri::command]
pub async fn list_normalized_stages(
    order_id: i64,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Vec<String>, AppError> {
    state.services(&app).await?.repairs.stages(order_id).await
}

#[tauri::command]
pub async fn hide_normalized_stage_name(
    order_id: i64,
    name: String,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Vec<String>, AppError> {
    state
        .services(&app)
        .await?
        .repairs
        .hide_stage_name(order_id, name)
        .await
}

#[tauri::command]
pub async fn normalized_repair_summary(
    order_id: i64,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::services::repairs::PublicSummary, AppError> {
    state.services(&app).await?.repairs.summary(order_id).await
}

#[tauri::command]
pub async fn list_normalized_payments(
    order_id: i64,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::services::ledger::Ledger, AppError> {
    state.services(&app).await?.ledger.list(order_id).await
}

#[tauri::command]
pub async fn record_normalized_payment(
    payload: crate::domain::ledger_contracts::MoneyRequest,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::ledger_contracts::PaymentState, AppError> {
    state.services(&app).await?.ledger.record(payload).await
}

#[tauri::command]
pub async fn list_normalized_logbook(
    filters: crate::domain::report_contracts::LogbookFilters,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::report_contracts::LogbookPage, AppError> {
    state.services(&app).await?.reports.logbook(filters).await
}

#[tauri::command]
pub async fn list_normalized_finances(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Vec<crate::domain::report_contracts::FinancialOrder>, AppError> {
    state.services(&app).await?.reports.finances().await
}

#[tauri::command]
pub async fn add_normalized_expense(
    payload: crate::domain::report_contracts::ExpensePayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.reports.expense(payload).await
}

#[tauri::command]
pub async fn edit_normalized_order(
    payload: EditOrderPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<OrderResponse, AppError> {
    state.services(&app).await?.orders.edit(payload).await
}

#[tauri::command]
pub async fn trash_normalized_order(
    payload: TrashOrderPayload,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.orders.trash(payload).await
}

#[tauri::command]
pub async fn list_normalized_evidence(
    order_id: i64,
    deleted: bool,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Vec<crate::domain::evidence_contracts::Evidence>, AppError> {
    state
        .services(&app)
        .await?
        .evidence
        .list(order_id, deleted)
        .await
}

#[tauri::command]
pub async fn add_normalized_evidence(
    input: crate::domain::evidence_contracts::NewEvidence,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::evidence_contracts::Evidence, AppError> {
    state.services(&app).await?.evidence.add(input).await
}

#[tauri::command]
pub async fn update_normalized_evidence(
    payload: crate::domain::evidence_contracts::UpdateEvidence,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<crate::domain::evidence_contracts::Evidence, AppError> {
    state.services(&app).await?.evidence.update(payload).await
}

#[tauri::command]
pub async fn read_normalized_evidence(
    order_id: i64,
    id: String,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Vec<u8>, AppError> {
    state
        .services(&app)
        .await?
        .evidence
        .read(order_id, &id)
        .await
}
