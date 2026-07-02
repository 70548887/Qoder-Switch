mod account_pool;
mod auth;
mod chat;
mod commands;
mod config;
mod error;
mod logger;
mod metrics;
mod proxy;
mod quota;
mod state;
mod tray;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let data_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("Qoder");
            std::fs::create_dir_all(&data_dir).ok();

            let app_state = AppState::new(&data_dir, 5888);

            // 加载已保存的 tokens
            let tm = app_state.token_manager.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = tm.load().await {
                    log::error!("加载账号失败: {}", e);
                }
            });

            app.manage(app_state);
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().ok();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::proxy_cmd::start_proxy,
            commands::proxy_cmd::stop_proxy,
            commands::proxy_cmd::get_proxy_status,
            commands::proxy_cmd::set_auto_rotate,
            commands::proxy_cmd::get_metrics,
            commands::proxy_cmd::reset_metrics,
            commands::proxy_cmd::get_dashboard_stats,
            commands::account_cmd::list_accounts,
            commands::account_cmd::add_accounts,
            commands::account_cmd::remove_account,
            commands::account_cmd::switch_account,
            commands::account_cmd::fetch_pool_account,
            commands::cert_cmd::install_cert,
            commands::cert_cmd::uninstall_cert,
            commands::cert_cmd::get_cert_status,
            commands::quota_cmd::check_quota,
            commands::quota_cmd::check_all_quotas,
            commands::proxy_cmd::get_request_logs,
            commands::proxy_cmd::get_discovered_domains,
            commands::proxy_cmd::set_target_domains,
            commands::config_cmd::get_config,
            commands::config_cmd::update_config,
            commands::chat_cmd::list_chat_workspaces,
            commands::chat_cmd::get_workspace_chats,
            commands::chat_cmd::search_workspace_chats,
            commands::chat_cmd::delete_workspace_chats,
            commands::chat_cmd::backup_workspace_session,
            commands::chat_cmd::restore_workspace_session,
            commands::chat_cmd::list_session_backups,
            commands::chat_cmd::backup_all_workspaces,
            commands::chat_cmd::export_chats_markdown,
            commands::chat_cmd::delete_backup_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
