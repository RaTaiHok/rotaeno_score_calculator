use crate::model::{JudgementBreakdown, NoteStats, ReverseInput, ReverseResult, ScoreInput, ScoreResult};

pub fn calculate_score(stats: &NoteStats, input: &ScoreInput) -> Result<ScoreResult, String> {
    validate_input_counts(stats, input)?;

    let judgement = JudgementBreakdown {
        non_slide_perfect_plus: input.non_slide_perfect_plus,
        non_slide_perfect: input.non_slide_perfect,
        non_slide_good: input.non_slide_good,
        non_slide_miss: input.non_slide_miss,
        slide_hit: input.slide_hit,
        slide_miss: input.slide_miss,
    };

    let (score_factor, raw_score, display_score) = score_from_judgement(stats, &judgement);

    Ok(ScoreResult {
        song_id: input.song_id.clone(),
        difficulty: input.difficulty.clone(),
        total: stats.total,
        slide_total: stats.slide,
        non_slide_total: stats.non_slide_total(),
        effective_notes: stats.effective_notes(),
        base_score: stats.base_score(),
        score_factor,
        raw_score,
        display_score,
        judgement,
    })
}

pub fn reverse_from_target(stats: &NoteStats, input: &ReverseInput) -> Result<ReverseResult, String> {
    let non_slide_total = stats.non_slide_total();
    let slide_total = stats.slide;
    let base_score = stats.base_score();
    let target = input.target_score;

    let lower = target as f64 / base_score;
    let upper = (target as f64 + 1.0) / base_score;

    let mut best: Option<ReverseCandidate> = None;

    for non_slide_perfect_plus in 0..=non_slide_total {
        let max_h = (non_slide_total - non_slide_perfect_plus) + slide_total;
        for h in 0..=max_h {
            let fixed = 1.01 * non_slide_perfect_plus as f64 + 0.2525 * h as f64;

            let mut p_min = (lower - fixed).ceil() as i64;
            let mut p_max = ((upper - fixed).ceil() as i64) - 1;
            let max_p = (non_slide_total - non_slide_perfect_plus) as i64;

            if p_max < 0 || p_min > max_p {
                continue;
            }

            p_min = p_min.max(0);
            p_max = p_max.min(max_p);
            if p_min > p_max {
                continue;
            }

            for non_slide_perfect in p_min as u32..=p_max as u32 {
                if let Some(candidate) = build_candidate(
                    stats,
                    non_slide_perfect_plus,
                    non_slide_perfect,
                    h,
                    target,
                    &input.song_id,
                    &input.difficulty,
                ) {
                    if candidate.exact_match {
                        return Ok(candidate.into_result(stats, input.target_score));
                    }

                    if should_replace_best(best.as_ref(), &candidate) {
                        best = Some(candidate);
                    }
                }
            }
        }
    }

    match best {
        Some(candidate) => Ok(candidate.into_result(stats, input.target_score)),
        None => Err("未找到可行分布，请检查谱面数据。".to_string()),
    }
}

pub fn validate_input_counts(stats: &NoteStats, input: &ScoreInput) -> Result<(), String> {
    let non_slide_total = input.non_slide_perfect_plus
        + input.non_slide_perfect
        + input.non_slide_good
        + input.non_slide_miss;
    let slide_total = input.slide_hit + input.slide_miss;

    if non_slide_total > stats.non_slide_total() {
        return Err(format!(
            "非Slide判定总和 {} 超过上限 {}。",
            non_slide_total,
            stats.non_slide_total()
        ));
    }

    if slide_total > stats.slide {
        return Err(format!("Slide判定总和 {} 超过上限 {}。", slide_total, stats.slide));
    }

    Ok(())
}

fn score_from_terms(base_score: f64, p_plus: u32, p: u32, g_plus_slide_hit: u32) -> (f64, f64, u32) {
    let score_factor = 1.01 * p_plus as f64 + p as f64 + 0.2525 * g_plus_slide_hit as f64;
    let raw_score = base_score * score_factor;
    let display_score = raw_score.floor() as u32;
    (score_factor, raw_score, display_score)
}

fn score_from_judgement(stats: &NoteStats, judgement: &JudgementBreakdown) -> (f64, f64, u32) {
    score_from_terms(
        stats.base_score(),
        judgement.non_slide_perfect_plus,
        judgement.non_slide_perfect,
        judgement.non_slide_good + judgement.slide_hit,
    )
}

fn build_candidate(
    stats: &NoteStats,
    non_slide_perfect_plus: u32,
    non_slide_perfect: u32,
    h: u32,
    target_score: u32,
    song_id: &str,
    difficulty: &str,
) -> Option<ReverseCandidate> {
    let non_slide_total = stats.non_slide_total();
    let slide_total = stats.slide;

    if non_slide_perfect_plus + non_slide_perfect > non_slide_total {
        return None;
    }

    let remaining_non_slide = non_slide_total - non_slide_perfect_plus - non_slide_perfect;
    let non_slide_good_min = h.saturating_sub(slide_total);
    let non_slide_good_max = h.min(remaining_non_slide);

    if non_slide_good_min > non_slide_good_max {
        return None;
    }

    let non_slide_good = non_slide_good_min;
    let slide_hit = h - non_slide_good;

    if slide_hit > slide_total {
        return None;
    }

    let non_slide_miss = remaining_non_slide - non_slide_good;
    let slide_miss = slide_total - slide_hit;

    let judgement = JudgementBreakdown {
        non_slide_perfect_plus,
        non_slide_perfect,
        non_slide_good,
        non_slide_miss,
        slide_hit,
        slide_miss,
    };

    let (score_factor, raw_score, matched_score) = score_from_judgement(stats, &judgement);
    let difference = matched_score as i64 - target_score as i64;
    let exact_match = difference == 0;

    Some(ReverseCandidate {
        song_id: song_id.to_string(),
        difficulty: difficulty.to_string(),
        judgement,
        score_factor,
        raw_score,
        matched_score,
        difference,
        exact_match,
    })
}

fn should_replace_best(best: Option<&ReverseCandidate>, candidate: &ReverseCandidate) -> bool {
    match best {
        None => true,
        Some(current_best) => {
            let best_diff = current_best.difference.abs();
            let candidate_diff = candidate.difference.abs();

            candidate_diff < best_diff
                || (candidate_diff == best_diff && candidate.matched_score > current_best.matched_score)
        }
    }
}

#[derive(Debug, Clone)]
struct ReverseCandidate {
    song_id: String,
    difficulty: String,
    judgement: JudgementBreakdown,
    score_factor: f64,
    raw_score: f64,
    matched_score: u32,
    difference: i64,
    exact_match: bool,
}

impl ReverseCandidate {
    fn into_result(self, stats: &NoteStats, target_score: u32) -> ReverseResult {
        let message = if self.exact_match {
            "已找到可达到目标分数的判定分布。".to_string()
        } else {
            format!(
                "未找到完全命中目标分数的分布，返回最接近方案（差值 {}）。",
                self.difference
            )
        };

        ReverseResult {
            song_id: self.song_id,
            difficulty: self.difficulty,
            target_score,
            matched_score: self.matched_score,
            difference: self.difference,
            exact_match: self.exact_match,
            total: stats.total,
            slide_total: stats.slide,
            non_slide_total: stats.non_slide_total(),
            effective_notes: stats.effective_notes(),
            base_score: stats.base_score(),
            score_factor: self.score_factor,
            raw_score: self.raw_score,
            judgement: self.judgement,
            message,
        }
    }
}
