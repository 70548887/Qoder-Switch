use crate::config::ProxyConfig;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> AppResult<ProxyConfig> {
    Ok(state.config.read().await.clone())
}

#[tauri::command]
pub async fn update_config(state: State<'_, AppState>, config: ProxyConfig) -> AppResult<()> {
    config.save(&state.config_path)
        .map_err(|e| AppError::Config(e))?;
    *state.config.write().await = config;
    Ok(())
}
