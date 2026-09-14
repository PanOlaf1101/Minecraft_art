use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
mod mc_art;

#[tauri::command]
async fn choose_file(app: AppHandle) -> Option<String> {
    match app.dialog().file().blocking_pick_file() {
        Some(x) => Some(x.to_string()),
        None => None,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![choose_file, mc_art::get_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
