use tauri::AppHandle;

pub fn window_default_title(app: &AppHandle) -> String {
    app.config().app.windows.as_slice()[0].title.clone()
}
