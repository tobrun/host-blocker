pub mod app;
pub mod baseline;
pub mod commands;
pub mod domain;
pub mod error;
pub mod hosts_file;
pub mod store;

use std::fs;
use std::sync::Arc;

use tauri::Manager;

use app::App;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|handle| {
            let state_dir = handle.path().app_config_dir()?;
            let cache_dir = handle.path().app_cache_dir()?;
            fs::create_dir_all(&state_dir)?;
            fs::create_dir_all(&cache_dir)?;
            let app = App::production(state_dir, cache_dir, hosts_file::constants::HOSTS_PATH);
            handle.manage(Arc::new(app));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::add_site,
            commands::remove_site,
            commands::import_baseline,
            commands::update_baseline,
            commands::remove_baseline,
            commands::set_mode,
            commands::remove_all_entries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running HostBlocker");
}
