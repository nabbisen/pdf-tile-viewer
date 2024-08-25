use tauri::{Manager, PhysicalPosition, PhysicalSize, Position, Size};

mod pdf;
use pdf::{read, search, SearchResult};

const WIDTH_RESOLUTION_RATIO: f32 = 0.8;
const HEIGHT_RESOLUTION_RATIO: f32 = 0.9;

#[tauri::command]
fn read_pdf(filepath: &str) -> Result<Vec<Vec<u8>>, String> {
    read(filepath)
}

#[tauri::command]
fn search_pdf(search_term: &str, filepath: &str) -> Result<SearchResult, String> {
    search(search_term, filepath)
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
        .invoke_handler(tauri::generate_handler![read_pdf, search_pdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
