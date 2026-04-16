use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongDataFile {
    pub songs: Vec<SongEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongEntry {
    pub song_id: String,
    pub song_name: String,
    pub difficulties: BTreeMap<String, NoteStats>,
    pub song_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteStats {
    pub tap: u32,
    pub flick: u32,
    pub slide: u32,
    pub catch: u32,
    pub rotate: u32,
    pub total: u32,
}

impl NoteStats {
    pub fn non_slide_total(&self) -> u32 {
        self.total.saturating_sub(self.slide)
    }

    pub fn effective_notes(&self) -> f64 {
        0.25 * self.slide as f64 + self.non_slide_total() as f64
    }

    pub fn base_score(&self) -> f64 {
        1_000_000.0 / self.effective_notes()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SongSummary {
    pub song_id: String,
    pub song_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DifficultyOption {
    pub difficulty: String,
    pub stats: NoteStats,
    pub non_slide_total: u32,
    pub effective_notes: f64,
    pub base_score: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoreInput {
    pub song_id: String,
    pub difficulty: String,
    pub non_slide_perfect_plus: u32,
    pub non_slide_perfect: u32,
    pub non_slide_good: u32,
    pub non_slide_miss: u32,
    pub slide_hit: u32,
    pub slide_miss: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct JudgementBreakdown {
    pub non_slide_perfect_plus: u32,
    pub non_slide_perfect: u32,
    pub non_slide_good: u32,
    pub non_slide_miss: u32,
    pub slide_hit: u32,
    pub slide_miss: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreResult {
    pub song_id: String,
    pub difficulty: String,
    pub total: u32,
    pub slide_total: u32,
    pub non_slide_total: u32,
    pub effective_notes: f64,
    pub base_score: f64,
    pub score_factor: f64,
    pub raw_score: f64,
    pub display_score: u32,
    pub judgement: JudgementBreakdown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverseInput {
    pub song_id: String,
    pub difficulty: String,
    pub target_score: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReverseResult {
    pub song_id: String,
    pub difficulty: String,
    pub target_score: u32,
    pub matched_score: u32,
    pub difference: i64,
    pub exact_match: bool,
    pub total: u32,
    pub slide_total: u32,
    pub non_slide_total: u32,
    pub effective_notes: f64,
    pub base_score: f64,
    pub score_factor: f64,
    pub raw_score: f64,
    pub judgement: JudgementBreakdown,
    pub message: String,
}
