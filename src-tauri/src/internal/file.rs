use std::process::Command;

pub fn run_file_manager(filepath: &str) -> Result<(), String> {
    let command = file_manager_command();

    Command::new(command)
        .arg(filepath)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn file_manager_command() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "explorer"
    }
    #[cfg(target_os = "macos")]
    {
        "open"
    }
    #[cfg(target_os = "linux")]
    {
        if Command::new("nautilus").arg("--version").output().is_ok() {
            "nautilus"
        } else if Command::new("dolphin").arg("--version").output().is_ok() {
            "dolphin"
        } else if Command::new("nemo").arg("--version").output().is_ok() {
            "nemo"
        } else if Command::new("thunar").arg("--version").output().is_ok() {
            "thunar"
        } else {
            "xdg-open"
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        compile_error!("Unsupported operating system")
    }
}
