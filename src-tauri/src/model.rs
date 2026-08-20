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

impl SongEntry {
    pub fn summary(&self) -> SongSummary {
        SongSummary {
            song_id: self.song_id.clone(),
            song_name: self.song_name.clone(),
        }
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

impl DifficultyOption {
    pub fn from_stats(difficulty: &str, stats: &NoteStats) -> Self {
        Self {
            difficulty: difficulty.to_string(),
            stats: stats.clone(),
            non_slide_total: stats.non_slide_total(),
            effective_notes: stats.effective_notes(),
            base_score: stats.base_score(),
        }
    }
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
    #[serde(default)]
    pub non_slide_unplayed: u32,
}

impl JudgementBreakdown {
    pub fn non_slide_sum(&self) -> u32 {
        self.non_slide_perfect_plus
            + self.non_slide_perfect
            + self.non_slide_good
            + self.non_slide_miss
            + self.non_slide_unplayed
    }

    pub fn non_slide_played(&self) -> u32 {
        self.non_slide_perfect_plus + self.non_slide_perfect + self.non_slide_good
    }

    pub fn slide_sum(&self) -> u32 {
        self.slide_hit + self.slide_miss
    }

    pub fn total_miss(&self) -> u32 {
        self.non_slide_miss + self.slide_miss
    }
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
    #[serde(default = "default_true")]
    pub allow_perfect_plus: bool,
    #[serde(default = "default_true")]
    pub allow_perfect: bool,
    #[serde(default = "default_true")]
    pub allow_good: bool,
    #[serde(default = "default_true")]
    pub allow_miss: bool,
    #[serde(default)]
    pub min_played_ratio: f64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy)]
pub struct ReverseJudgementFilter {
    pub allow_perfect_plus: bool,
    pub allow_perfect: bool,
    pub allow_good: bool,
    pub allow_miss: bool,
}

impl ReverseInput {
    pub fn judgement_filter(&self) -> ReverseJudgementFilter {
        let mut filter = ReverseJudgementFilter {
            allow_perfect_plus: self.allow_perfect_plus,
            allow_perfect: self.allow_perfect,
            allow_good: self.allow_good,
            allow_miss: self.allow_miss,
        };

        if !filter.has_any_scoring_judgement() {
            filter.allow_perfect_plus = true;
            filter.allow_perfect = true;
            filter.allow_good = true;
        }

        // “只选 P+”用于表达阶段全 P+，剩余未打到的 note 需要归到 Miss/未判定。
        if filter.allow_perfect_plus && !filter.allow_perfect && !filter.allow_good {
            filter.allow_miss = true;
        }

        filter
    }
}

impl ReverseJudgementFilter {
    fn has_any_scoring_judgement(self) -> bool {
        self.allow_perfect_plus || self.allow_perfect || self.allow_good
    }

    pub fn allows(self, judgement: &JudgementBreakdown) -> bool {
        (self.allow_perfect_plus || judgement.non_slide_perfect_plus == 0)
            && (self.allow_perfect || judgement.non_slide_perfect == 0)
            && (self.allow_good || judgement.non_slide_good == 0)
            && (self.allow_miss || judgement.total_miss() == 0)
    }
}

/// 同一个反算方案（同一分数）下，Miss 在 Slide / 非Slide 之间的一种分配情况。
/// 反算时 Good 与 Slide Hit 权重相同（均 101），无法区分，因此需要分情况展示。
#[derive(Debug, Clone, Serialize)]
pub struct MissVariant {
    pub non_slide_good: u32,
    pub non_slide_miss: u32,
    pub slide_hit: u32,
    pub slide_miss: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReverseCandidateResult {
    pub matched_score: u32,
    pub difference: i64,
    pub exact_match: bool,
    pub score_factor: f64,
    pub raw_score: f64,
    /// 主判定分布（取 Good 最少 / Slide Hit 最多的代表变体）
    pub judgement: JudgementBreakdown,
    /// 分情况：该方案的部分 Miss 分配变体（最多 MAX_MISS_VARIANTS 个，取 Miss 少的）
    pub miss_variants: Vec<MissVariant>,
    /// 该方案实际的总 Miss 分配变体数（可能远大于 miss_variants.len()）
    pub miss_variant_total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataUpdateInfo {
    pub has_update: bool,
    pub local_version: String,
    pub remote_version: String,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 反算进度消息：由 Rust 计算线程通过 Channel 推送给前端
#[derive(Debug, Clone, Serialize)]
pub struct ReverseProgress {
    pub percent: u8,
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
    pub candidate_count: usize,
    pub exact_candidate_count: usize,
    pub candidates: Vec<ReverseCandidateResult>,
    pub message: String,
}
