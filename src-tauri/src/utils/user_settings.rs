use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct KeyValue {
    key: Option<String>,
    value: Option<Value>,
    file_exists: bool,
    key_exists: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pages_tile_viewer: PagesTileViewer,
    zoomed_page_viewer: ZoomedPageViewer,
}

#[derive(Serialize, Deserialize)]
struct PagesTileViewer {
    scale: f32,
    page_num_visible: bool,
    fix_pages_per_row: bool,
    pages_per_row: u16,
}

#[derive(Serialize, Deserialize)]
struct ZoomedPageViewer {
    zoom_view_background_locked: bool,
    zoom_view_scale: f32,
    zoom_view_transparency: f32,
}

impl UserSettings {
    pub fn read_by_key(key: &str) -> Result<KeyValue, Box<dyn std::error::Error>> {
        let filepath = settings_filepath();

        if !filepath.exists() {
            return Ok(KeyValue {
                key: None,
                value: None,
                file_exists: false,
                key_exists: false,
            });
        }

        let json = json_load(&filepath)?;
        if let Some(value) = json.get(key) {
            Ok(KeyValue {
                key: Some(key.to_owned()),
                value: Some(value.to_owned()),
                file_exists: true,
                key_exists: true,
            })
        } else {
            return Ok(KeyValue {
                key: Some(key.to_owned()),
                value: None,
                file_exists: true,
                key_exists: false,
            });
        }
    }

    pub fn write_by_key(key: &str, value: &Value) -> Result<(), std::io::Error> {
        let filepath = settings_filepath();

        let mut current_settings = if filepath.exists() {
            let file_text = read_to_string(&filepath)?;
            serde_json::from_str(&file_text).unwrap_or_default()
        } else {
            Value::Object(serde_json::Map::new())
        };

        let map = current_settings.as_object_mut().unwrap();
        map.insert(key.to_owned(), value.to_owned());

        let updated_settings = serde_json::to_string_pretty(&current_settings)?;

        let mut file = File::create(&filepath)?;
        file.write_all(updated_settings.as_bytes())?;

        Ok(())
    }
}

const SETTINGS_FILENAME: &str = "settings.json";

fn settings_filepath() -> PathBuf {
    let exec_filepath = std::env::current_exe().expect("Failed to get exec path");
    let dirpath = exec_filepath
        .parent()
        .expect("Failed to get exec parent dir path");
    dirpath.join(SETTINGS_FILENAME)
}

fn json_load(filepath: &PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
    let mut file =
        File::open(&filepath).map_err(|e| format!("Failed to open settings file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    let json: Value =
        from_str(&contents).map_err(|e| format!("Failed to deserialize settings: {}", e))?;
    Ok(json)
}
