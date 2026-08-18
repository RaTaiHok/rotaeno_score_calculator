import { Channel, invoke } from "@tauri-apps/api/core";

export function listSongs() {
  return invoke("list_songs");
}

export function getSongDifficulties(songId) {
  return invoke("get_song_difficulties", { songId });
}

export function calculateScore(input) {
  return invoke("calculate_score", { input });
}

export function reverseFromScore(input, onProgress) {
  return invoke("reverse_from_score", {
    input,
    onProgress: createProgressChannel(onProgress)
  });
}

export function reverseAllFromScore(input, onProgress) {
  return invoke("reverse_all_from_score", {
    input,
    onProgress: createProgressChannel(onProgress)
  });
}

function createProgressChannel(onProgress) {
  const channel = new Channel();
  channel.onmessage = (msg) => {
    if (onProgress) {
      onProgress(msg);
    }
  };
  return channel;
}

export function checkDataUpdate() {
  return invoke("check_data_update");
}

export function downloadLatestData() {
  return invoke("download_latest_data");
}

export function getDataStatus() {
  return invoke("get_data_status");
}

export function resetData() {
  return invoke("reset_data");
}
