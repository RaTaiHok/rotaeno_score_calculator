use crate::calc;
use crate::model::{
    DataUpdateInfo, DifficultyOption, NoteStats, ReverseInput, ReverseProgress, ReverseResult,
    ScoreInput, ScoreResult, SongDataFile, SongEntry, SongSummary,
};
use std::fs;
use std::io::Read;
use std::sync::RwLock;
use tauri::Manager;
use tauri::State;

const DATA_BASE_URL: &str = "https://rth.srv-selena.lookatthesky.cn/Rotaeno/data";
/// 服务器上的固定数据文件名。更新数据时只需覆盖上传这一个文件。
const REMOTE_DATA_FILE: &str = "all_song_note_stats.json";
/// 本地缓存文件名（与服务器同名）。
const LOCAL_DATA_FILE: &str = "all_song_note_stats.json";
/// 本地保存的服务器 ETag，用于下次请求带 If-None-Match（304 跳过下载）。
const LOCAL_ETAG_FILE: &str = "data_etag.txt";
/// 内置数据：仅作为全新安装且无法联网时的兜底，不再跟随每次数据更新重新构建。
const BUNDLED_JSON: &str = include_str!("../../data/all_song_note_stats.json");
const BUNDLED_VERSION: &str = include_str!("../../data/version.txt");

pub struct AppState {
    pub songs: RwLock<Vec<SongEntry>>,
    pub data_version: RwLock<String>,
    pub is_bundled: RwLock<bool>, // true = using built-in data, needs network download
    // check_data_update 下载好的数据字节，供 download_latest_data 复用（省一次下载）
    pub pending_data: RwLock<Option<Vec<u8>>>,
    // 服务器返回的 ETag，下载成功后才写入本地文件
    pub pending_etag: RwLock<Option<String>>,
}

fn data_version_desc(song_count: usize) -> String {
    format!("{} 首歌曲", song_count)
}

fn bundled_version_desc() -> String {
    format!("内置 v{}", BUNDLED_VERSION.trim())
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(15))
        .build()
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

    let local_json_path = data_dir.join(LOCAL_DATA_FILE);

    // 优先使用本地缓存（有缓存说明之前已成功联网同步过）
    if let Ok(json) = fs::read_to_string(&local_json_path) {
        if let Ok(parsed) = serde_json::from_str::<SongDataFile>(&json) {
            let count = parsed.songs.len();
            return Ok(AppState {
                songs: RwLock::new(parsed.songs),
                data_version: RwLock::new(data_version_desc(count)),
                is_bundled: RwLock::new(false),
                pending_data: RwLock::new(None),
                pending_etag: RwLock::new(None),
            });
        }
    }

    // 全新安装（或本地缓存损坏）— 用内置数据兜底，等待联网同步
    let parsed: SongDataFile =
        serde_json::from_str(BUNDLED_JSON).map_err(|e| format!("内置数据解析失败: {e}"))?;

    Ok(AppState {
        songs: RwLock::new(parsed.songs),
        data_version: RwLock::new(bundled_version_desc()),
        is_bundled: RwLock::new(true),
        pending_data: RwLock::new(None),
        pending_etag: RwLock::new(None),
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
pub async fn reverse_from_score(
    input: ReverseInput,
    on_progress: tauri::ipc::Channel<ReverseProgress>,
    state: State<'_, AppState>,
) -> Result<ReverseResult, String> {
    let songs = state
        .songs
        .read()
        .map_err(|e| format!("{e}"))?
        .clone();
    let stats = find_stats_in(&songs, &input.song_id, &input.difficulty)?.clone();

    // 重计算放到 blocking 线程池，避免阻塞 UI 线程
    tauri::async_runtime::spawn_blocking(move || {
        let mut progress = |percent: u8| {
            let _ = on_progress.send(ReverseProgress { percent });
        };
        calc::reverse_from_target(&stats, &input, &mut progress)
    })
    .await
    .map_err(|e| format!("计算任务异常: {e}"))?
}

#[tauri::command]
pub async fn reverse_all_from_score(
    input: ReverseInput,
    on_progress: tauri::ipc::Channel<ReverseProgress>,
    state: State<'_, AppState>,
) -> Result<ReverseResult, String> {
    let songs = state
        .songs
        .read()
        .map_err(|e| format!("{e}"))?
        .clone();
    let stats = find_stats_in(&songs, &input.song_id, &input.difficulty)?.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let mut progress = |percent: u8| {
            let _ = on_progress.send(ReverseProgress { percent });
        };
        calc::reverse_all_from_target(&stats, &input, &mut progress)
    })
    .await
    .map_err(|e| format!("计算任务异常: {e}"))?
}

#[tauri::command]
pub fn check_data_update(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<DataUpdateInfo, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;

    let local_path = data_dir.join(LOCAL_DATA_FILE);
    let etag_path = data_dir.join(LOCAL_ETAG_FILE);
    let local_etag = fs::read_to_string(&etag_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let local_count = state.songs.read().map_err(|e| format!("{e}"))?.len();
    let local_version = data_version_desc(local_count);
    let url = format!("{DATA_BASE_URL}/{REMOTE_DATA_FILE}");

    let mut req = http_agent().get(&url);
    if let Some(etag) = &local_etag {
        req = req.set("If-None-Match", etag);
    }

    let resp = match req.call() {
        Ok(r) => r,
        Err(e) => {
            return Ok(DataUpdateInfo {
                has_update: false,
                local_version,
                remote_version: String::new(),
                download_url: url,
                error: Some(format!("无法连接更新服务器: {e}")),
            });
        }
    };

    // 服务器 ETag 与本地记录一致 → 数据没有变化，无需下载
    if resp.status() == 304 {
        let remote_version = format!("{}（无更新）", local_version);
        return Ok(DataUpdateInfo {
            has_update: false,
            local_version,
            remote_version,
            download_url: String::new(),
            error: None,
        });
    }

    if resp.status() != 200 {
        return Ok(DataUpdateInfo {
            has_update: false,
            local_version,
            remote_version: String::new(),
            download_url: url,
            error: Some(format!("服务器返回状态 {}", resp.status())),
        });
    }

    let server_etag = resp.header("etag").map(|s| s.to_string());
    let mut bytes = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("读取数据内容失败: {e}"))?;

    // 校验服务器数据格式
    let parsed: SongDataFile =
        serde_json::from_slice(&bytes).map_err(|e| format!("远端数据格式错误: {e}"))?;
    let remote_count = parsed.songs.len();

    // 与本地缓存逐字节比较，内容不同才算有更新
    let local_bytes = fs::read(&local_path).unwrap_or_default();
    let has_update = local_bytes.is_empty() || bytes != local_bytes;

    // 缓存字节供 download_latest_data 复用，避免二次下载
    {
        let mut pending = state.pending_data.write().map_err(|e| format!("{e}"))?;
        *pending = Some(bytes);
    }
    {
        let mut etag = state.pending_etag.write().map_err(|e| format!("{e}"))?;
        *etag = server_etag;
    }

    Ok(DataUpdateInfo {
        has_update,
        local_version,
        remote_version: data_version_desc(remote_count),
        download_url: url,
        error: None,
    })
}

#[tauri::command]
pub fn download_latest_data(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    fs::create_dir_all(&data_dir).map_err(|e| format!("无法创建数据目录: {e}"))?;

    // 优先复用 check_data_update 已下载的字节；否则重新下载
    let (bytes, server_etag) = {
        let pending = state.pending_data.read().map_err(|e| format!("{e}"))?;
        match pending.as_ref() {
            Some(b) => (
                b.clone(),
                state.pending_etag.read().map_err(|e| format!("{e}"))?.clone(),
            ),
            None => {
                let url = format!("{DATA_BASE_URL}/{REMOTE_DATA_FILE}");
                let resp = http_agent()
                    .get(&url)
                    .call()
                    .map_err(|e| format!("无法连接更新服务器: {e}"))?;
                if resp.status() != 200 {
                    return Err(format!("服务器返回状态 {}", resp.status()));
                }
                let etag = resp.header("etag").map(|s| s.to_string());
                let mut b = Vec::new();
                resp.into_reader()
                    .read_to_end(&mut b)
                    .map_err(|e| format!("读取数据内容失败: {e}"))?;
                (b, etag)
            }
        }
    };

    // 校验格式
    let parsed: SongDataFile =
        serde_json::from_slice(&bytes).map_err(|e| format!("远端数据格式错误: {e}"))?;
    let count = parsed.songs.len();

    // 保存本地缓存 + ETag（下次启动带 If-None-Match 可命中 304）
    fs::write(data_dir.join(LOCAL_DATA_FILE), &bytes)
        .map_err(|e| format!("保存数据失败: {e}"))?;
    if let Some(etag) = &server_etag {
        let _ = fs::write(data_dir.join(LOCAL_ETAG_FILE), etag);
    }

    // 更新内存状态
    {
        let mut songs = state.songs.write().map_err(|e| format!("{e}"))?;
        *songs = parsed.songs;
    }
    {
        let mut ver = state.data_version.write().map_err(|e| format!("{e}"))?;
        *ver = data_version_desc(count);
    }
    {
        let mut bundled = state.is_bundled.write().map_err(|e| format!("{e}"))?;
        *bundled = false;
    }

    Ok(data_version_desc(count))
}

#[tauri::command]
pub fn reset_data(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;

    // 删除本地缓存和 ETag
    let json_path = data_dir.join(LOCAL_DATA_FILE);
    let etag_path = data_dir.join(LOCAL_ETAG_FILE);
    let json_existed = json_path.exists();
    fs::remove_file(&json_path).ok();
    fs::remove_file(&etag_path).ok();

    // 清理旧版本遗留的版本化 JSON 文件
    if let Ok(entries) = fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("all_song_note_stats_") && name.ends_with(".json") {
                fs::remove_file(entry.path()).ok();
            }
        }
    }

    // 恢复内置数据
    let parsed: SongDataFile =
        serde_json::from_str(BUNDLED_JSON).map_err(|e| format!("内置数据解析失败: {e}"))?;
    let count = parsed.songs.len();

    {
        let mut songs = state.songs.write().map_err(|e| format!("{e}"))?;
        *songs = parsed.songs;
    }
    {
        let mut ver = state.data_version.write().map_err(|e| format!("{e}"))?;
        *ver = bundled_version_desc();
    }
    {
        let mut bundled = state.is_bundled.write().map_err(|e| format!("{e}"))?;
        *bundled = true;
    }

    Ok(format!(
        "已重置为内置数据（{} 首歌曲，删除: json={json_existed}）",
        count
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
