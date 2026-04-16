#[path = "../../src/model.rs"]
mod model;
#[path = "../../src/calc.rs"]
mod calc;
#[path = "../../src/app.rs"]
mod app;

use app::{calculate_score, get_song_difficulties, list_songs, reverse_from_score};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = app::load_app_state().expect("Failed to load chart data");

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            list_songs,
            get_song_difficulties,
            calculate_score,
            reverse_from_score
        ])
        .run(tauri::generate_context!())
        .expect("Failed to launch Tauri app");
}
