use crate::model::{
    JudgementBreakdown, NoteStats, ReverseInput, ReverseResult, ScoreInput, ScoreResult,
};

mod math;
mod reverse;

pub fn calculate_score(stats: &NoteStats, input: &ScoreInput) -> Result<ScoreResult, String> {
    validate_input_counts(stats, input)?;

    let judgement = JudgementBreakdown {
        non_slide_perfect_plus: input.non_slide_perfect_plus,
        non_slide_perfect: input.non_slide_perfect,
        non_slide_good: input.non_slide_good,
        non_slide_miss: input.non_slide_miss,
        slide_hit: input.slide_hit,
        slide_miss: input.slide_miss,
        non_slide_unplayed: 0,
    };

    let score_parts = math::ScoreMath::new(stats).from_judgement(&judgement);

    Ok(ScoreResult {
        song_id: input.song_id.clone(),
        difficulty: input.difficulty.clone(),
        total: stats.total,
        slide_total: stats.slide,
        non_slide_total: stats.non_slide_total(),
        effective_notes: stats.effective_notes(),
        base_score: stats.base_score(),
        score_factor: score_parts.score_factor,
        raw_score: score_parts.raw_score,
        display_score: score_parts.display_score,
        judgement,
    })
}

pub fn reverse_from_target(
    stats: &NoteStats,
    input: &ReverseInput,
    on_progress: impl FnMut(u8),
) -> Result<ReverseResult, String> {
    reverse::from_target(stats, input, false, input.min_played_ratio, on_progress)
}

pub fn reverse_all_from_target(
    stats: &NoteStats,
    input: &ReverseInput,
    on_progress: impl FnMut(u8),
) -> Result<ReverseResult, String> {
    reverse::from_target(stats, input, true, input.min_played_ratio, on_progress)
}

pub fn validate_input_counts(stats: &NoteStats, input: &ScoreInput) -> Result<(), String> {
    let judgement = JudgementBreakdown {
        non_slide_perfect_plus: input.non_slide_perfect_plus,
        non_slide_perfect: input.non_slide_perfect,
        non_slide_good: input.non_slide_good,
        non_slide_miss: input.non_slide_miss,
        slide_hit: input.slide_hit,
        slide_miss: input.slide_miss,
        non_slide_unplayed: 0,
    };

    if judgement.non_slide_sum() > stats.non_slide_total() {
        return Err(format!(
            "非Slide判定总和 {} 超过上限 {}。",
            judgement.non_slide_sum(),
            stats.non_slide_total()
        ));
    }

    if judgement.slide_sum() > stats.slide {
        return Err(format!(
            "Slide判定总和 {} 超过上限 {}。",
            judgement.slide_sum(),
            stats.slide
        ));
    }

    Ok(())
}
