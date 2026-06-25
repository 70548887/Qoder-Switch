use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let start = MenuItem::with_id(app, "start_proxy", "启动代理", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop_proxy", "停止代理", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &sep, &start, &stop, &sep, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main");
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .menu(&menu)
        .tooltip("Qoder Proxy")
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
                "start_proxy" => {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("tray-start-proxy", ());
                        }
                    });
                }
                "stop_proxy" => {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("tray-stop-proxy", ());
                        }
                    });
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
        })
        .build(app)?;

    Ok(())
}
