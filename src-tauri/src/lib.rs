pub mod commands;
pub mod core;
mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(core::launcher::LauncherState(std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()))))
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::save_settings,
            core::launcher::get_instances,
            core::launcher::create_instance,
            core::launcher::rename_instance,
            core::launcher::delete_instance,
            core::launcher::open_instance_folder,
            core::launcher::launch_instance,
            core::launcher::kill_instance,
            core::launcher::get_running_instances,
            core::minecraft::fetch_forge_versions,
            commands::accounts::get_accounts,
            commands::accounts::add_account,
            commands::accounts::add_microsoft_account,
            commands::accounts::select_account,
            commands::accounts::delete_account,
            commands::auth::start_microsoft_login,
            commands::auth::cancel_microsoft_login,
            commands::auth::refresh_account_token,
            commands::instances::get_instance_servers,
            commands::instances::add_instance_server,
            commands::instances::update_instance_server,
            commands::instances::remove_instance_server,
            commands::instances::update_server_icon,
            commands::instances::get_instance_worlds,
            commands::instances::ping_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
