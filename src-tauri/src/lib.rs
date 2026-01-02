mod claude;
mod constants;
mod credentials;
mod settings;

use credentials::CredentialsManager;
use settings::{AppSettings, SettingsManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageData {
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub utilization: f64,
    pub resets_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub org_id: String,
    pub session_key: String,
}

pub struct AppState {
    credentials_manager: CredentialsManager,
    settings_manager: SettingsManager,
    http_client: reqwest::Client,
    usage: Mutex<Option<UsageData>>,
    settings: Mutex<AppSettings>,
    last_notified_session: Mutex<Option<u32>>,
    last_notified_weekly: Mutex<Option<u32>>,
}

#[tauri::command]
async fn get_credentials(state: State<'_, Arc<AppState>>) -> Result<Credentials, String> {
    state.credentials_manager.load().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_credentials(
    state: State<'_, Arc<AppState>>,
    org_id: String,
    session_key: String,
) -> Result<(), String> {
    state
        .credentials_manager
        .save(&org_id, &session_key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
async fn save_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    new_settings: AppSettings,
) -> Result<(), String> {
    state.settings_manager.save(&new_settings).map_err(|e| e.to_string())?;
    
    {
        let mut settings = state.settings.lock().await;
        *settings = new_settings;
    }
    
    let usage = state.usage.lock().await;
    let settings = state.settings.lock().await;
    if let Some(ref usage_data) = *usage {
        update_tray(&app, usage_data, &settings);
    }
    
    Ok(())
}

#[tauri::command]
async fn test_notification(app: AppHandle) -> Result<(), String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i32)
        .unwrap_or(0);
    
    app.notification()
        .builder()
        .id(id)
        .title("Seekers")
        .body("This is a test notification!")
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn refresh_usage(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let creds = state.credentials_manager.load().map_err(|e| e.to_string())?;

    if creds.org_id.is_empty() || creds.session_key.is_empty() {
        return Err("Credentials not configured".to_string());
    }

    let usage = claude::fetch_usage(&creds.org_id, &creds.session_key, &state.http_client)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut stored = state.usage.lock().await;
        *stored = Some(usage.clone());
    }

    let _ = app.emit("usage-updated", &usage);

    let settings = state.settings.lock().await;
    update_tray(&app, &usage, &settings);
    
    check_and_notify(&app, &state, &usage, &settings).await;

    Ok(())
}

async fn check_and_notify(app: &AppHandle, state: &Arc<AppState>, usage: &UsageData, settings: &AppSettings) {
    if settings.notify_session > 0 {
        if let Some(ref five_hour) = usage.five_hour {
            let pct = five_hour.utilization.round() as u32;
            if pct >= settings.notify_session {
                let mut last = state.last_notified_session.lock().await;
                if *last != Some(settings.notify_session) {
                    *last = Some(settings.notify_session);
                    let _ = app.notification()
                        .builder()
                        .title("Claude Session Limit")
                        .body(format!("Session usage at {pct}%"))
                        .show();
                }
            } else {
                let mut last = state.last_notified_session.lock().await;
                *last = None;
            }
        }
    }
    
    if settings.notify_weekly > 0 {
        if let Some(ref seven_day) = usage.seven_day {
            let pct = seven_day.utilization.round() as u32;
            if pct >= settings.notify_weekly {
                let mut last = state.last_notified_weekly.lock().await;
                if *last != Some(settings.notify_weekly) {
                    *last = Some(settings.notify_weekly);
                    let _ = app.notification()
                        .builder()
                        .title("Claude Weekly Limit")
                        .body(format!("Weekly usage at {pct}%"))
                        .show();
                }
            } else {
                let mut last = state.last_notified_weekly.lock().await;
                *last = None;
            }
        }
    }
}

fn update_tray(app: &AppHandle, usage: &UsageData, settings: &AppSettings) {
    if let Some(tray) = app.tray_by_id(constants::TRAY_ID) {
        let title = format_tray_title(usage, settings);
        let _ = tray.set_title(Some(&title));
        
        if let Ok(menu) = create_tray_menu(app, Some(usage), settings) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn format_tray_title(usage: &UsageData, settings: &AppSettings) -> String {
    let five = usage.five_hour.as_ref().map(|w| w.utilization.round() as i32);
    let seven = usage.seven_day.as_ref().map(|w| w.utilization.round() as i32);
    
    let value = match settings.menu_bar_display.as_str() {
        "session" => five.map(|v| v.to_string()),
        "weekly" => seven.map(|v| v.to_string()),
        "both" => match (five, seven) {
            (Some(f), Some(s)) => Some(format!("{f}/{s}")),
            (Some(f), None) => Some(f.to_string()),
            (None, Some(s)) => Some(s.to_string()),
            _ => None,
        },
        "higher" => match (five, seven) {
            (Some(f), Some(s)) => Some(f.max(s).to_string()),
            (Some(f), None) => Some(f.to_string()),
            (None, Some(s)) => Some(s.to_string()),
            _ => None,
        },
        _ => five.map(|v| v.to_string()),
    };
    
    match value {
        Some(v) if settings.show_percent_symbol => format!("{v}%"),
        Some(v) => v,
        None => "--".to_string(),
    }
}

fn make_progress_bar(pct: f64, settings: &AppSettings) -> String {
    let len = settings.progress_length as usize;
    let filled = ((pct / 100.0) * len as f64).round() as usize;
    let empty = len - filled.min(len);
    
    let (filled_char, empty_char) = match settings.progress_style.as_str() {
        "blocks" => constants::progress::BLOCKS,
        "bar" => constants::progress::BAR,
        "dots" => constants::progress::DOTS,
        _ => constants::progress::CIRCLES,
    };
    
    format!("{}{}", filled_char.repeat(filled.min(len)), empty_char.repeat(empty))
}

fn create_tray_menu(app: &AppHandle, usage: Option<&UsageData>, settings: &AppSettings) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let mut builder = MenuBuilder::new(app);

    if let Some(usage) = usage {
        if let Some(ref five_hour) = usage.five_hour {
            let pct = five_hour.utilization.round() as i32;
            let bar = make_progress_bar(five_hour.utilization, settings);
            let item = MenuItemBuilder::new(format!(
                "Session  {bar} {pct:>3}%"
            ))
            .enabled(false)
            .build(app)?;
            builder = builder.item(&item);
            
            let reset = MenuItemBuilder::new(format!(
                "         ↻ {}",
                format_reset_time(&five_hour.resets_at)
            ))
            .enabled(false)
            .build(app)?;
            builder = builder.item(&reset);
        }

        if let Some(ref seven_day) = usage.seven_day {
            let pct = seven_day.utilization.round() as i32;
            let bar = make_progress_bar(seven_day.utilization, settings);
            let item = MenuItemBuilder::new(format!(
                "Weekly   {bar} {pct:>3}%"
            ))
            .enabled(false)
            .build(app)?;
            builder = builder.item(&item);
            
            let reset = MenuItemBuilder::new(format!(
                "         ↻ {}",
                format_reset_time(&seven_day.resets_at)
            ))
            .enabled(false)
            .build(app)?;
            builder = builder.item(&reset);
        }

        builder = builder.separator();
    } else {
        let item = MenuItemBuilder::new("Not configured")
            .enabled(false)
            .build(app)?;
        builder = builder.item(&item).separator();
    }

    let open_claude = MenuItemBuilder::with_id(constants::menu::OPEN_CLAUDE, "Open Claude").build(app)?;
    let refresh = MenuItemBuilder::with_id(constants::menu::REFRESH, "Refresh").build(app)?;
    let settings_item = MenuItemBuilder::with_id(constants::menu::SETTINGS, "Settings...").build(app)?;
    let quit = MenuItemBuilder::with_id(constants::menu::QUIT, "Quit").build(app)?;

    builder
        .item(&open_claude)
        .item(&refresh)
        .separator()
        .item(&settings_item)
        .item(&quit)
        .build()
}

fn format_reset_time(iso_string: &str) -> String {
    use chrono::{DateTime, Local, Utc};

    if let Ok(date) = iso_string.parse::<DateTime<Utc>>() {
        let now = Utc::now();
        let diff = date.signed_duration_since(now);
        let local = date.with_timezone(&Local);

        if diff.num_seconds() <= 0 {
            "any moment".to_string()
        } else if diff.num_minutes() < constants::time::MINUTES_PER_HOUR {
            format!("in {}m", diff.num_minutes())
        } else if diff.num_hours() < constants::time::HOURS_PER_DAY {
            let hours = diff.num_hours();
            let mins = diff.num_minutes() % constants::time::MINUTES_PER_HOUR;
            if mins > 0 {
                format!("in {hours}h {mins}m")
            } else {
                format!("in {hours}h")
            }
        } else if diff.num_hours() < constants::time::HOURS_TOMORROW_THRESHOLD {
            format!("tomorrow {}", local.format("%-I:%M %p"))
        } else {
            local.format("%a %-I:%M %p").to_string()
        }
    } else {
        "unknown".to_string()
    }
}

async fn do_refresh(app: &AppHandle, state: &Arc<AppState>) {
    let Ok(creds) = state.credentials_manager.load() else {
        return;
    };
    if creds.org_id.is_empty() || creds.session_key.is_empty() {
        return;
    }
    if let Ok(usage) = claude::fetch_usage(&creds.org_id, &creds.session_key, &state.http_client).await {
        let mut stored = state.usage.lock().await;
        *stored = Some(usage.clone());
        drop(stored);
        
        let settings = state.settings.lock().await;
        update_tray(app, &usage, &settings);
        check_and_notify(app, state, &usage, &settings).await;
        drop(settings);
        
        let _ = app.emit("usage-updated", &usage);
    }
}

fn start_auto_refresh(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let interval = {
                let settings = state.settings.lock().await;
                settings.refresh_interval
            };
            
            if interval == 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(constants::time::DISABLED_REFRESH_CHECK_SECS)).await;
                continue;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(u64::from(interval) * constants::time::SECONDS_PER_MINUTE)).await;
            do_refresh(&app, &state).await;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let settings_manager = SettingsManager::new();
            let initial_settings = settings_manager.load().unwrap_or_default();
            
            let state = Arc::new(AppState {
                credentials_manager: CredentialsManager::new(),
                settings_manager,
                http_client: reqwest::Client::new(),
                usage: Mutex::new(None),
                settings: Mutex::new(initial_settings.clone()),
                last_notified_session: Mutex::new(None),
                last_notified_weekly: Mutex::new(None),
            });

            app.manage(state.clone());

            let menu = create_tray_menu(app.handle(), None, &initial_settings)?;

            let _tray = TrayIconBuilder::with_id(constants::TRAY_ID)
                .title(constants::TRAY_TITLE_DEFAULT)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        constants::menu::OPEN_CLAUDE => {
                            let _ = open::that(constants::CLAUDE_URL);
                        }
                        constants::menu::REFRESH => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app.state::<Arc<AppState>>();
                                do_refresh(&app, &state).await;
                            });
                        }
                        constants::menu::SETTINGS => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        constants::menu::QUIT => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                    }
                })
                .build(app)?;

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let app_handle = app.handle().clone();
            let state_clone = state.clone();
            tauri::async_runtime::spawn(async move {
                do_refresh(&app_handle, &state_clone).await;
            });
            
            start_auto_refresh(app.handle().clone(), state.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_credentials,
            save_credentials,
            get_settings,
            save_settings,
            refresh_usage,
            test_notification
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
