import { invoke } from "@tauri-apps/api/core";

export function listSongs() {
  return invoke("list_songs");
}

export function getSongDifficulties(songId) {
  return invoke("get_song_difficulties", { songId });
}

export function calculateScore(input) {
  return invoke("calculate_score", { input });
}

export function reverseFromScore(input) {
  return invoke("reverse_from_score", { input });
}

export function reverseAllFromScore(input) {
  return invoke("reverse_all_from_score", { input });
}
