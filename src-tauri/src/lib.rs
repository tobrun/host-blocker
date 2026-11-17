pub mod baseline;
pub mod domain;
pub mod error;
pub mod hosts_file;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running HostBlocker");
}
