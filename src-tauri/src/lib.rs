mod config;
mod input;

use config::{read_config, sanitize_config, write_config, AppConfig};
use input::InputController;
use std::sync::{Arc, RwLock};
use tauri::{
    menu::{CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    ActivationPolicy, AppHandle, Emitter, Manager, State, WindowEvent, Wry,
};
use tauri_plugin_autostart::ManagerExt;

pub struct SharedState {
    config: Arc<RwLock<AppConfig>>,
    toggle_menu: Arc<RwLock<Option<CheckMenuItem<Wry>>>>,
}

#[tauri::command]
fn get_config(state: State<'_, SharedState>) -> Result<AppConfig, String> {
    state
        .config
        .read()
        .map(|config| config.clone())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn save_config(
    app: AppHandle,
    state: State<'_, SharedState>,
    config: AppConfig,
) -> Result<(), String> {
    apply_autostart(&app, config.launch_at_startup)?;
    let config = sanitize_config(config);
    write_config(&config).map_err(|error| error.to_string())?;
    *state.config.write().map_err(|error| error.to_string())? = config;
    Ok(())
}

#[tauri::command]
fn toggle_mouse_mode(app: AppHandle, state: State<'_, SharedState>) -> Result<bool, String> {
    let enabled = {
        let config = state.config.read().map_err(|error| error.to_string())?;
        !config.mouse_mode_enabled
    };
    set_mouse_mode(&app, &state, enabled)?;
    Ok(enabled)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(ActivationPolicy::Accessory);
                let _ = app.set_dock_visibility(false);
            }

            let mut config = read_config().unwrap_or_default();
            config.mouse_mode_enabled = false;
            let _ = write_config(&config);
            apply_autostart(app.handle(), config.launch_at_startup)?;

            let shared_config = Arc::new(RwLock::new(config));
            app.manage(SharedState {
                config: Arc::clone(&shared_config),
                toggle_menu: Arc::new(RwLock::new(None)),
            });

            InputController::start(shared_config, app.handle().clone());
            build_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            toggle_mouse_mode
        ])
        .run(tauri::generate_context!())
        .expect("failed to run KeyPoint");
}

fn apply_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|error| error.to_string())
    } else {
        manager.disable().map_err(|error| error.to_string())
    }
}

fn set_mouse_mode(app: &AppHandle, state: &SharedState, enabled: bool) -> Result<(), String> {
    {
        let mut config = state.config.write().map_err(|error| error.to_string())?;
        config.mouse_mode_enabled = enabled;
        write_config(&config).map_err(|error| error.to_string())?;
    }

    if let Ok(toggle_menu) = state.toggle_menu.read() {
        if let Some(toggle_menu) = toggle_menu.as_ref() {
            let _ = toggle_menu.set_checked(enabled);
        }
    }

    update_mouse_mode_indicators(app, enabled);
    Ok(())
}

pub(crate) fn update_mouse_mode_indicators(app: &AppHandle, enabled: bool) {
    let label = if enabled {
        "KeyPoint ON"
    } else {
        "KeyPoint OFF"
    };
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(label));
        let _ = tray.set_title(Some(label));
    }

    if let Some(state) = app.try_state::<SharedState>() {
        if let Ok(toggle_menu) = state.toggle_menu.read() {
            if let Some(toggle_menu) = toggle_menu.as_ref() {
                let _ = toggle_menu.set_checked(enabled);
            }
        }
    }

    let _ = app.emit("mouse-mode-changed", enabled);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_settings = MenuItemBuilder::with_id("open_settings", "설정 열기").build(app)?;
    let toggle_mode =
        CheckMenuItemBuilder::with_id("toggle_mode", "마우스 모드 ON/OFF").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "종료").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&open_settings, &toggle_mode, &separator, &quit])
        .build()?;

    TrayIconBuilder::with_id("main")
        .title("KeyPoint OFF")
        .tooltip("KeyPoint OFF")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open_settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "toggle_mode" => {
                if let Some(state) = app.try_state::<SharedState>() {
                    let enabled = state
                        .config
                        .read()
                        .map(|config| !config.mouse_mode_enabled)
                        .unwrap_or(false);
                    let _ = set_mouse_mode(app, &state, enabled);
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    if let Some(state) = app.try_state::<SharedState>() {
        if let Ok(mut item) = state.toggle_menu.write() {
            *item = Some(toggle_mode);
        }
        if let Ok(config) = state.config.read() {
            if let Ok(item) = state.toggle_menu.read() {
                if let Some(item) = item.as_ref() {
                    let _ = item.set_checked(config.mouse_mode_enabled);
                }
            }
            update_mouse_mode_indicators(app, config.mouse_mode_enabled);
        }
    }

    Ok(())
}
