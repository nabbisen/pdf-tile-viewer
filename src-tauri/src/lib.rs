use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, Size};

mod internal;
use internal::file::run_file_manager;
use internal::pdf::{read, search, SearchResult};
use internal::tauri::window_default_title;

const WIDTH_RESOLUTION_RATIO: f32 = 0.80;
const HEIGHT_RESOLUTION_RATIO: f32 = 0.87;

#[tauri::command]
fn read_pdf(filepath: &str) -> Result<Vec<u8>, String> {
    read(filepath)
}

#[tauri::command]
fn search_pdf(search_term: &str, filepath: &str) -> Result<SearchResult, String> {
    search(search_term, filepath)
}

#[tauri::command]
fn open_file_manager(filepath: &str) -> Result<(), String> {
    run_file_manager(filepath)
}

#[tauri::command]
fn set_window_title(app: AppHandle, window: tauri::Window, filepath: &str) -> Result<(), String> {
    let default_title = window_default_title(&app);
    window
        .set_title(format!("{} [{}]", filepath, default_title).as_str())
        .expect("Failed to set window title");
    Ok(())
}

#[tauri::command]
fn reset_window_title(app: AppHandle, window: tauri::Window) -> Result<(), String> {
    let default_title = window_default_title(&app);
    window
        .set_title(default_title.as_str())
        .expect("Failed to set window title");
    Ok(())
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
                    let width = (resolution.width as f32 * WIDTH_RESOLUTION_RATIO).round() as u32;
                    let height =
                        (resolution.height as f32 * HEIGHT_RESOLUTION_RATIO).round() as u32;
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
            read_pdf,
            search_pdf,
            open_file_manager,
            set_window_title,
            reset_window_title
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
