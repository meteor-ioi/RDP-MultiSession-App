// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod commands;
mod patcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_status,
            commands::set_defender_exclusion,
            commands::set_persistence,
            commands::check_updates,
            commands::patch_rdp,
            commands::restore_rdp
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
