use std::fs::File;
use std::io::Read;
use tauri::Manager;

#[tauri::command]
fn read_pdf(filepath: &str) -> Vec<u8> {
    let mut file = match File::open(filepath).map_err(|e| e.to_string()) {
        Ok(x) => x,
        Err(err) => {
            // todo file not found
            panic!("{}", err);
        }
    };
    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer).map_err(|e| e.to_string()) {
        Ok(_) => buffer,
        Err(err) => {
            // todo reading failed
            panic!("{}", err) ;
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![read_pdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
