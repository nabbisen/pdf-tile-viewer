use serde_json::Value;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, Size};

mod utils;
use utils::app_window::{window_default_title, window_width_height};
use utils::file::run_file_manager;
use utils::pdf::{read, search, SearchResult};
use utils::user_settings::{KeyValue, UserSettings};

const WIDTH_RESOLUTION_RATIO: f32 = 0.80;
const HEIGHT_RESOLUTION_RATIO: f32 = 0.84;

#[tauri::command]
fn pdf_read(filepath: &str) -> Result<Vec<u8>, String> {
    read(filepath)
}

#[tauri::command]
fn pdf_search(search_term: &str, filepath: &str) -> Result<SearchResult, String> {
    search(search_term, filepath)
}

#[tauri::command]
fn file_manager_open(filepath: &str) -> Result<(), String> {
    run_file_manager(filepath)
}

#[tauri::command]
fn window_title_set(app: AppHandle, window: tauri::Window, filepath: &str) -> Result<(), String> {
    let default_title = window_default_title(&app);
    window
        .set_title(format!("{} [{}]", filepath, default_title).as_str())
        .expect("Failed to set window title");
    Ok(())
}

#[tauri::command]
fn window_title_reset(app: AppHandle, window: tauri::Window) -> Result<(), String> {
    let default_title = window_default_title(&app);
    window
        .set_title(default_title.as_str())
        .expect("Failed to set window title");
    Ok(())
}

#[tauri::command]
fn settings_read_by_key(key: &str) -> Result<KeyValue, String> {
    UserSettings::read_by_key(key).map_err(|err| err.to_string())
}

#[tauri::command]
fn settings_write_by_key(key: &str, value: Value) -> Result<(), String> {
    UserSettings::write_by_key(key, &value).map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // app window size
            match window.current_monitor() {
                Ok(Some(current_monitor)) => {
                    let resolution = current_monitor.size();

                    let width = if let Some(x) = window_width_height("windowWidth") {
                        if resolution.width < x {
                            resolution.width
                        } else {
                            x
                        }
                    } else {
                        (resolution.width as f32 * WIDTH_RESOLUTION_RATIO).round() as u32
                    };
                    let height = if let Some(x) = window_width_height("windowHeight") {
                        if resolution.height < x {
                            resolution.height
                        } else {
                            x
                        }
                    } else {
                        (resolution.height as f32 * HEIGHT_RESOLUTION_RATIO).round() as u32
                    };

                    let size = Size::Physical(PhysicalSize { width, height });
                    let position = Position::Physical(PhysicalPosition {
                        x: ((resolution.width - width) / 2) as i32,
                        y: ((resolution.height - height) / 2) as i32,
                    });

                    window.set_size(size).unwrap();
                    window.set_position(position).unwrap();
                }
                _ => {}
            };

            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            pdf_read,
            pdf_search,
            file_manager_open,
            window_title_set,
            window_title_reset,
            settings_read_by_key,
            settings_write_by_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
