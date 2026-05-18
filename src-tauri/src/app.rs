use crate::calc;
use crate::model::{
    DifficultyOption, NoteStats, ReverseInput, ReverseResult, ScoreInput, ScoreResult,
    SongDataFile, SongEntry, SongSummary,
};
use tauri::State;

pub struct AppState {
    songs: Vec<SongEntry>,
}

pub fn load_app_state() -> Result<AppState, String> {
    let raw = include_str!("../../data/all_song_note_stats.json");
    let parsed: SongDataFile =
        serde_json::from_str(raw).map_err(|err| format!("解析谱面JSON失败: {err}"))?;

    Ok(AppState {
        songs: parsed.songs,
    })
}

#[tauri::command]
pub fn list_songs(state: State<'_, AppState>) -> Vec<SongSummary> {
    let mut songs: Vec<SongSummary> = state.songs.iter().map(SongEntry::summary).collect();

    songs.sort_by(|a, b| a.song_name.cmp(&b.song_name));
    songs
}

#[tauri::command]
pub fn get_song_difficulties(
    song_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DifficultyOption>, String> {
    let song = find_song(&state, &song_id)
        .ok_or_else(|| format!("未找到歌曲ID: {song_id}，请检查歌曲数据。"))?;

    let mut options: Vec<DifficultyOption> = song
        .difficulties
        .iter()
        .map(|(difficulty, stats)| DifficultyOption::from_stats(difficulty, stats))
        .collect();

    options.sort_by(|a, b| difficulty_rank(&a.difficulty).cmp(&difficulty_rank(&b.difficulty)));
    Ok(options)
}

#[tauri::command]
pub fn calculate_score(
    input: ScoreInput,
    state: State<'_, AppState>,
) -> Result<ScoreResult, String> {
    let stats = find_stats(&state, &input.song_id, &input.difficulty)?;
    calc::calculate_score(stats, &input)
}

#[tauri::command]
pub fn reverse_from_score(
    input: ReverseInput,
    state: State<'_, AppState>,
) -> Result<ReverseResult, String> {
    let stats = find_stats(&state, &input.song_id, &input.difficulty)?;
    calc::reverse_from_target(stats, &input)
}

#[tauri::command]
pub fn reverse_all_from_score(
    input: ReverseInput,
    state: State<'_, AppState>,
) -> Result<ReverseResult, String> {
    let stats = find_stats(&state, &input.song_id, &input.difficulty)?;
    calc::reverse_all_from_target(stats, &input)
}

fn find_song<'a>(state: &'a AppState, song_id: &str) -> Option<&'a SongEntry> {
    state.songs.iter().find(|song| song.song_id == song_id)
}

fn find_stats<'a>(
    state: &'a AppState,
    song_id: &str,
    difficulty: &str,
) -> Result<&'a NoteStats, String> {
    let song = find_song(state, song_id)
        .ok_or_else(|| format!("未找到歌曲ID: {song_id}，请检查输入。"))?;

    song.difficulties
        .get(difficulty)
        .ok_or_else(|| format!("歌曲 {song_id} 不存在难度 {difficulty}。"))
}

fn difficulty_rank(name: &str) -> (u8, &str) {
    match name {
        "I" => (1, name),
        "II" => (2, name),
        "III" => (3, name),
        "IV" => (4, name),
        "IV_Alpha" => (5, name),
        "Sp" => (6, name),
        _ => (7, name),
    }
}
