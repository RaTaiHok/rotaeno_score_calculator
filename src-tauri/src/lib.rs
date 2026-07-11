mod app;
mod calc;
mod model;

use tauri::Manager;
use app::{
    calculate_score, check_data_update, download_latest_data, get_data_status,
    get_song_difficulties, list_songs, reset_data, reverse_all_from_score,
    reverse_from_score,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = app::load_app_state(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_songs,
            get_song_difficulties,
            calculate_score,
            reverse_from_score,
            reverse_all_from_score,
            check_data_update,
            download_latest_data,
            get_data_status,
            reset_data,
        ])
        .run(tauri::generate_context!())
        .expect("Failed to launch Tauri app");
}
