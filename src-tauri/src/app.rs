use crate::calc;
use crate::model::{
    DataUpdateInfo, DifficultyOption, NoteStats, ReverseInput, ReverseResult, ScoreInput, ScoreResult,
    SongDataFile, SongEntry, SongSummary,
};
use std::fs;
use std::sync::RwLock;
use tauri::Manager;
use tauri::State;

const DATA_BASE_URL: &str = "https://rth.srv-selena.lookatthesky.cn/Rotaeno/data";
const BUNDLED_JSON: &str = include_str!("../../data/all_song_note_stats.json");
const BUNDLED_VERSION: &str = include_str!("../../data/version.txt");

pub struct AppState {
    pub songs: RwLock<Vec<SongEntry>>,
    pub data_version: RwLock<String>,
    pub is_bundled: RwLock<bool>, // true = using built-in data, needs network download
}

#[derive(serde::Serialize)]
pub struct DataStatus {
    pub version: String,
    pub is_bundled: bool,
}

pub fn load_app_state(app: &tauri::AppHandle) -> Result<AppState, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;

    fs::create_dir_all(&data_dir).map_err(|e| format!("无法创建数据目录: {e}"))?;

    let local_json_path = data_dir.join("all_song_note_stats.json");
    let local_version_path = data_dir.join("data_version.txt");

    // Try local file first
    if let (Ok(json), Ok(ver)) = (
        fs::read_to_string(&local_json_path),
        fs::read_to_string(&local_version_path),
    ) {
        let parsed: SongDataFile =
            serde_json::from_str(&json).map_err(|e| format!("本地数据解析失败: {e}"))?;
        let version = ver.trim().to_string();
        return Ok(AppState {
            songs: RwLock::new(parsed.songs),
            data_version: RwLock::new(version),
            is_bundled: RwLock::new(false),
        });
    }

    // First launch — use bundled data as temporary fallback, mark as bundled
    let parsed: SongDataFile =
        serde_json::from_str(BUNDLED_JSON).map_err(|e| format!("内置数据解析失败: {e}"))?;
    let version = BUNDLED_VERSION.trim().to_string();

    Ok(AppState {
        songs: RwLock::new(parsed.songs),
        data_version: RwLock::new(version),
        is_bundled: RwLock::new(true),
    })
}

#[tauri::command]
pub fn get_data_status(state: State<'_, AppState>) -> Result<DataStatus, String> {
    let version = state.data_version.read().map_err(|e| format!("{e}"))?.clone();
    let is_bundled = *state.is_bundled.read().map_err(|e| format!("{e}"))?;
    Ok(DataStatus { version, is_bundled })
}

#[tauri::command]
pub fn list_songs(state: State<'_, AppState>) -> Result<Vec<SongSummary>, String> {
    let songs = state.songs.read().map_err(|e| format!("{e}"))?;
    let mut songs: Vec<SongSummary> = songs.iter().map(SongEntry::summary).collect();
    songs.sort_by(|a, b| a.song_name.cmp(&b.song_name));
    Ok(songs)
}

#[tauri::command]
pub fn get_song_difficulties(
    song_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DifficultyOption>, String> {
    let songs = state.songs.read().map_err(|e| format!("{e}"))?;
    let song = find_song_in(&songs, &song_id)
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
    let songs = state.songs.read().map_err(|e| format!("{e}"))?;
    let stats = find_stats_in(&songs, &input.song_id, &input.difficulty)?;
    calc::calculate_score(stats, &input)
}

#[tauri::command]
pub fn reverse_from_score(
    input: ReverseInput,
    state: State<'_, AppState>,
) -> Result<ReverseResult, String> {
    let songs = state.songs.read().map_err(|e| format!("{e}"))?;
    let stats = find_stats_in(&songs, &input.song_id, &input.difficulty)?;
    calc::reverse_from_target(stats, &input)
}

#[tauri::command]
pub fn reverse_all_from_score(
    input: ReverseInput,
    state: State<'_, AppState>,
) -> Result<ReverseResult, String> {
    let songs = state.songs.read().map_err(|e| format!("{e}"))?;
    let stats = find_stats_in(&songs, &input.song_id, &input.difficulty)?;
    calc::reverse_all_from_target(stats, &input)
}

#[tauri::command]
pub fn check_data_update(state: State<'_, AppState>) -> Result<DataUpdateInfo, String> {
    let local_version = state
        .data_version
        .read()
        .map_err(|e| format!("{e}"))?
        .clone();

    let version_url = format!("{DATA_BASE_URL}/latest_version.txt");

    match ureq::get(&version_url).call() {
        Ok(response) => {
            if response.status() != 200 {
                return Ok(DataUpdateInfo {
                    has_update: false,
                    local_version,
                    remote_version: String::new(),
                    download_url: String::new(),
                    error: Some(format!(
                        "服务器返回 {} —— {} 不存在，请确认服务端已放置 latest_version.txt",
                        response.status(),
                        version_url
                    )),
                });
            }

            let remote_version = match response.into_string() {
                Ok(s) => s.trim().to_string(),
                Err(e) => {
                    return Ok(DataUpdateInfo {
                        has_update: false,
                        local_version,
                        remote_version: String::new(),
                        download_url: String::new(),
                        error: Some(format!("读取版本内容失败: {e}")),
                    });
                }
            };

            let has_update = remote_version != local_version && !remote_version.is_empty();
            let download_url = if has_update {
                format!("{DATA_BASE_URL}/all_song_note_stats_{remote_version}.json")
            } else {
                String::new()
            };

            Ok(DataUpdateInfo {
                has_update,
                local_version,
                remote_version,
                download_url,
                error: None,
            })
        }
        Err(e) => Ok(DataUpdateInfo {
            has_update: false,
            local_version,
            remote_version: String::new(),
            download_url: String::new(),
            error: Some(format!("无法连接服务器: {e}")),
        }),
    }
}

#[tauri::command]
pub fn download_latest_data(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Fetch remote version
    let version_url = format!("{DATA_BASE_URL}/latest_version.txt");
    let response = ureq::get(&version_url)
        .call()
        .map_err(|e| format!("无法连接更新服务器: {e}"))?;

    if response.status() != 200 {
        return Err(format!("服务器返回状态 {}", response.status()));
    }

    let remote_version = response
        .into_string()
        .map_err(|e| format!("读取远端版本信息失败: {e}"))?
        .trim()
        .to_string();

    // Download data
    let data_url = format!("{DATA_BASE_URL}/all_song_note_stats_{remote_version}.json");
    let response = ureq::get(&data_url)
        .call()
        .map_err(|e| format!("下载数据失败: {e}"))?;

    if response.status() != 200 {
        return Err(format!("下载数据返回状态 {}", response.status()));
    }

    let json = response
        .into_string()
        .map_err(|e| format!("读取数据内容失败: {e}"))?;

    // Validate
    let parsed: SongDataFile =
        serde_json::from_str(&json).map_err(|e| format!("远端数据格式错误: {e}"))?;

    // Save locally
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    fs::create_dir_all(&data_dir).map_err(|e| format!("无法创建数据目录: {e}"))?;

    fs::write(data_dir.join("all_song_note_stats.json"), &json)
        .map_err(|e| format!("保存数据失败: {e}"))?;
    fs::write(data_dir.join("data_version.txt"), &remote_version)
        .map_err(|e| format!("保存版本信息失败: {e}"))?;

    // Update in-memory state
    {
        let mut songs = state.songs.write().map_err(|e| format!("{e}"))?;
        *songs = parsed.songs;
    }
    {
        let mut ver = state.data_version.write().map_err(|e| format!("{e}"))?;
        *ver = remote_version.clone();
    }
    {
        let mut bundled = state.is_bundled.write().map_err(|e| format!("{e}"))?;
        *bundled = false;
    }

    Ok(remote_version)
}

#[tauri::command]
pub fn reset_data(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;

    // Delete local data files — report what was deleted
    let json_path = data_dir.join("all_song_note_stats.json");
    let ver_path = data_dir.join("data_version.txt");
    let json_existed = json_path.exists();
    let ver_existed = ver_path.exists();
    fs::remove_file(&json_path).ok();
    fs::remove_file(&ver_path).ok();

    // Also clean up any versioned JSON files that were downloaded
    if let Ok(entries) = fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("all_song_note_stats_") && name.ends_with(".json") {
                fs::remove_file(entry.path()).ok();
            }
        }
    }

    // Revert to bundled data
    let parsed: SongDataFile =
        serde_json::from_str(BUNDLED_JSON).map_err(|e| format!("内置数据解析失败: {e}"))?;
    let version = BUNDLED_VERSION.trim().to_string();

    {
        let mut songs = state.songs.write().map_err(|e| format!("{e}"))?;
        *songs = parsed.songs;
    }
    {
        let mut ver = state.data_version.write().map_err(|e| format!("{e}"))?;
        *ver = version.clone();
    }
    {
        let mut bundled = state.is_bundled.write().map_err(|e| format!("{e}"))?;
        *bundled = true;
    }

    Ok(format!(
        "已重置为内置数据 v{version}（数据目录: {}，删除: json={json_existed} ver={ver_existed}）",
        data_dir.display()
    ))
}

fn find_song_in<'a>(songs: &'a [SongEntry], song_id: &str) -> Option<&'a SongEntry> {
    songs.iter().find(|song| song.song_id == song_id)
}

fn find_stats_in<'a>(
    songs: &'a [SongEntry],
    song_id: &str,
    difficulty: &str,
) -> Result<&'a NoteStats, String> {
    let song =
        find_song_in(songs, song_id).ok_or_else(|| format!("未找到歌曲ID: {song_id}，请检查输入。"))?;

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
