use crate::state::AppState;
use crate::error::AppResult;
use tauri::State;

#[tauri::command]
pub async fn install_cert(state: State<'_, AppState>) -> AppResult<()> {
    state.cert_manager.install_ca()
        .map_err(|e| crate::error::AppError::Cert(e))
}

#[tauri::command]
pub async fn uninstall_cert(state: State<'_, AppState>) -> AppResult<()> {
    state.cert_manager.uninstall_ca()
        .map_err(|e| crate::error::AppError::Cert(e))
}

#[tauri::command]
pub async fn get_cert_status(state: State<'_, AppState>) -> AppResult<bool> {
    Ok(state.cert_manager.is_installed())
}
