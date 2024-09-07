use app_json_settings::JsonSettings;
use tauri::AppHandle;

pub fn window_width_height(key: &str) -> Option<u32> {
    let read = if let Ok(x) = JsonSettings::exe_dir().read_by_key(key) {
        x
    } else {
        return None;
    };
    let value = if let Some(x) = read.value {
        x
    } else {
        return None;
    };
    let ret = if let Some(x) = value.as_u64() {
        Some(x as u32)
    } else {
        return None;
    };
    ret
}

pub fn window_default_title(app: &AppHandle) -> String {
    app.config().app.windows.as_slice()[0].title.clone()
}
