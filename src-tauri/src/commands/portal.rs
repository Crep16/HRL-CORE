use super::BackendState;
use crate::{
    domain::portal::PortalConfig, error::AppError, services::portal_publisher::PublisherLogin,
};
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn portal_settings(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Value, AppError> {
    state.services(&app).await?.portal.settings().await
}
#[tauri::command]
pub async fn configure_portal(
    config: PortalConfig,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.portal.configure(config).await
}
#[tauri::command]
pub async fn connect_portal(
    input: PublisherLogin,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.portal.connect(input).await
}
#[tauri::command]
pub async fn pause_portal(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.portal.pause().await
}
#[tauri::command]
pub async fn resume_portal(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.portal.resume().await
}
#[tauri::command]
pub async fn disconnect_portal(
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<(), AppError> {
    state.services(&app).await?.portal.disconnect().await
}
#[tauri::command]
pub async fn portal_order_status(
    order_id: i64,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Value, AppError> {
    state.services(&app).await?.portal.status(order_id).await
}
#[tauri::command]
pub async fn portal_order_action(
    order_id: i64,
    action: String,
    app: tauri::AppHandle,
    state: State<'_, BackendState>,
) -> Result<Value, AppError> {
    let services = state.services(&app).await?;
    if action == "reconcile" {
        services.portal.reconcile_order(order_id).await
    } else {
        services.portal.action(order_id, &action).await
    }
}
