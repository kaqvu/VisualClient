mod settings;
mod launcher;
mod accounts;
mod minecraft;
mod auth;
mod crypto;
mod instances_data;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::save_settings,
            launcher::get_instances,
            launcher::create_instance,
            launcher::rename_instance,
            launcher::delete_instance,
            launcher::open_instance_folder,
            launcher::launch_instance,
            minecraft::fetch_forge_versions,
            accounts::get_accounts,
            accounts::add_account,
            accounts::add_microsoft_account,
            accounts::select_account,
            accounts::delete_account,
            auth::start_microsoft_login,
            auth::cancel_microsoft_login,
            instances_data::get_instance_servers,
            instances_data::add_instance_server,
            instances_data::update_instance_server,
            instances_data::remove_instance_server,
            instances_data::update_server_icon,
            instances_data::get_instance_worlds,
            instances_data::ping_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
