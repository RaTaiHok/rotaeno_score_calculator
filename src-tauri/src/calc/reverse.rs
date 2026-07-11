use std::cmp::Ordering;

use crate::calc::math::{exact_p_range, nearest_p_probes, ScoreMath};
use crate::model::{
    JudgementBreakdown, NoteStats, ReverseCandidateResult, ReverseInput, ReverseJudgementFilter,
    ReverseResult,
};

const DEFAULT_REVERSE_SOLUTIONS: usize = 3;

pub fn from_target(
    stats: &NoteStats,
    input: &ReverseInput,
    include_all: bool,
    min_played_ratio: f64,
) -> Result<ReverseResult, String> {
    let non_slide_total = stats.non_slide_total();
    let slide_total = stats.slide;
    let math = ScoreMath::new(stats);
    let filter = input.judgement_filter();

    let mut exact = CandidateBucket::new(include_all);
    let mut nearest = CandidateBucket::new(include_all);

    for non_slide_perfect_plus in 0..=non_slide_total {
        let max_h = (non_slide_total - non_slide_perfect_plus) + slide_total;

        for h in 0..=max_h {
            let fixed_num = math.fixed_term(non_slide_perfect_plus, h);
            let max_p = non_slide_total - non_slide_perfect_plus;

            let exact_range = exact_p_range(input.target_score, math.denominator, fixed_num, max_p);

            if let Some((exact_min, exact_max)) = exact_range {
                for non_slide_perfect in exact_min..=exact_max {
                    if let Some(eval) = TupleEval::new(
                        stats,
                        math,
                        non_slide_perfect_plus,
                        non_slide_perfect,
                        h,
                        input.target_score,
                    ) {
                        exact.push_eval(&eval, filter, min_played_ratio);
                    }
                }
            }

            if exact.has_candidates() {
                continue;
            }

            let probe_ps = nearest_p_probes(input.target_score, math.denominator, fixed_num, max_p);

            for &non_slide_perfect in &probe_ps {
                if let Some((exact_min, exact_max)) = exact_range {
                    if (exact_min..=exact_max).contains(&non_slide_perfect) {
                        continue;
                    }
                }

                if let Some(eval) = TupleEval::new(
                    stats,
                    math,
                    non_slide_perfect_plus,
                    non_slide_perfect,
                    h,
                    input.target_score,
                ) {
                    if eval.exact_match {
                        exact.push_eval(&eval, filter, min_played_ratio);
                    } else {
                        nearest.push_nearest_eval(&eval, filter, min_played_ratio);
                    }
                }
            }
        }
    }

    if exact.has_candidates() {
        let exact_count = exact.count;
        return Ok(build_result(stats, input, exact, exact_count, include_all));
    }

    if nearest.has_candidates() {
        return Ok(build_result(stats, input, nearest, 0, include_all));
    }

    Err("未找到符合当前判定筛选条件的可行分布，请放宽筛选后重试。".to_string())
}

fn build_result(
    stats: &NoteStats,
    input: &ReverseInput,
    bucket: CandidateBucket,
    exact_candidate_count: usize,
    include_all: bool,
) -> ReverseResult {
    let candidates = bucket.top;
    let top = candidates
        .first()
        .expect("build_result requires non-empty candidates");

    let message = if top.exact_match {
        if include_all {
            format!(
                "已找到 {} 个精确命中目标分数的方案，按 Miss数少、P+比率高、G数少 的优先级排序，当前展示全部方案。",
                exact_candidate_count
            )
        } else {
            format!(
                "已找到 {} 个精确命中目标分数的方案，按 Miss数少、P+比率高、G数少 的优先级排序，仅展示前 {} 个。",
                exact_candidate_count,
                candidates.len()
            )
        }
    } else {
        if include_all {
            format!(
                "未找到精确命中方案，找到 {} 个与目标分数最接近的方案（绝对差值 {}），按 Miss数少、P+比率高、G数少 的优先级排序，当前展示全部方案。",
                bucket.count,
                top.difference.abs()
            )
        } else {
            format!(
                "未找到精确命中方案，找到 {} 个与目标分数最接近的方案（绝对差值 {}），按 Miss数少、P+比率高、G数少 的优先级排序，仅展示前 {} 个。",
                bucket.count,
                top.difference.abs(),
                candidates.len()
            )
        }
    };

    let candidate_results = candidates.iter().map(ReverseCandidate::to_output).collect();

    ReverseResult {
        song_id: input.song_id.clone(),
        difficulty: input.difficulty.clone(),
        target_score: input.target_score,
        matched_score: top.matched_score,
        difference: top.difference,
        exact_match: top.exact_match,
        total: stats.total,
        slide_total: stats.slide,
        non_slide_total: stats.non_slide_total(),
        effective_notes: stats.effective_notes(),
        base_score: stats.base_score(),
        score_factor: top.score_factor,
        raw_score: top.raw_score,
        judgement: top.judgement.clone(),
        candidate_count: bucket.count,
        exact_candidate_count,
        candidates: candidate_results,
        message,
    }
}

#[derive(Debug, Default)]
struct CandidateBucket {
    top: Vec<ReverseCandidate>,
    count: usize,
    include_all: bool,
    nearest_abs_diff: Option<i64>,
}

impl CandidateBucket {
    fn new(include_all: bool) -> Self {
        Self {
            include_all,
            ..Self::default()
        }
    }

    fn has_candidates(&self) -> bool {
        !self.top.is_empty()
    }

    fn push_eval(&mut self, eval: &TupleEval, filter: ReverseJudgementFilter, min_played_ratio: f64) {
        let accepted = candidates_from_eval(eval, self.include_all, filter, min_played_ratio);
        if accepted.is_empty() {
            return;
        }

        self.count += accepted.len();
        for candidate in accepted {
            push_candidate(&mut self.top, candidate, self.include_all);
        }
    }

    fn push_nearest_eval(&mut self, eval: &TupleEval, filter: ReverseJudgementFilter, min_played_ratio: f64) {
        let accepted = candidates_from_eval(eval, self.include_all, filter, min_played_ratio);
        if accepted.is_empty() {
            return;
        }

        let abs_diff = eval.difference.abs();

        match self.nearest_abs_diff {
            None => {
                self.nearest_abs_diff = Some(abs_diff);
                self.count = accepted.len();
                self.top.clear();
                for candidate in accepted {
                    push_candidate(&mut self.top, candidate, self.include_all);
                }
            }
            Some(current_best) if abs_diff < current_best => {
                self.nearest_abs_diff = Some(abs_diff);
                self.count = accepted.len();
                self.top.clear();
                for candidate in accepted {
                    push_candidate(&mut self.top, candidate, self.include_all);
                }
            }
            Some(current_best) if abs_diff == current_best => {
                self.count += accepted.len();
                for candidate in accepted {
                    push_candidate(&mut self.top, candidate, self.include_all);
                }
            }
            Some(_) => {}
        }
    }
}

#[derive(Debug, Clone)]
struct TupleEval {
    non_slide_perfect_plus: u32,
    non_slide_perfect: u32,
    h: u32,
    remaining_non_slide: u32,
    slide_total: u32,
    g_min: u32,
    g_count: usize,
    score_factor: f64,
    raw_score: f64,
    matched_score: u32,
    difference: i64,
    exact_match: bool,
}

impl TupleEval {
    fn new(
        stats: &NoteStats,
        math: ScoreMath,
        non_slide_perfect_plus: u32,
        non_slide_perfect: u32,
        h: u32,
        target_score: u32,
    ) -> Option<Self> {
        let non_slide_total = stats.non_slide_total();
        if non_slide_perfect_plus + non_slide_perfect > non_slide_total {
            return None;
        }

        let remaining_non_slide = non_slide_total - non_slide_perfect_plus - non_slide_perfect;
        let slide_total = stats.slide;

        let g_min = h.saturating_sub(slide_total);
        let g_max = h.min(remaining_non_slide);
        if g_min > g_max {
            return None;
        }

        let score_parts = math.from_terms(non_slide_perfect_plus, non_slide_perfect, h);
        let difference = score_parts.display_score as i64 - target_score as i64;

        Some(Self {
            non_slide_perfect_plus,
            non_slide_perfect,
            h,
            remaining_non_slide,
            slide_total,
            g_min,
            g_count: (g_max - g_min + 1) as usize,
            score_factor: score_parts.score_factor,
            raw_score: score_parts.raw_score,
            matched_score: score_parts.display_score,
            difference,
            exact_match: difference == 0,
        })
    }

    fn to_candidate(&self, non_slide_good: u32, non_slide_unplayed: u32) -> ReverseCandidate {
        let slide_hit = self.h - non_slide_good;
        let non_slide_miss = self.remaining_non_slide - non_slide_good - non_slide_unplayed;
        let slide_miss = self.slide_total - slide_hit;

        ReverseCandidate {
            judgement: JudgementBreakdown {
                non_slide_perfect_plus: self.non_slide_perfect_plus,
                non_slide_perfect: self.non_slide_perfect,
                non_slide_good,
                non_slide_miss,
                slide_hit,
                slide_miss,
                non_slide_unplayed,
            },
            score_factor: self.score_factor,
            raw_score: self.raw_score,
            matched_score: self.matched_score,
            difference: self.difference,
            exact_match: self.exact_match,
        }
    }
}

#[derive(Debug, Clone)]
struct ReverseCandidate {
    judgement: JudgementBreakdown,
    score_factor: f64,
    raw_score: f64,
    matched_score: u32,
    difference: i64,
    exact_match: bool,
}

impl ReverseCandidate {
    fn to_output(&self) -> ReverseCandidateResult {
        ReverseCandidateResult {
            matched_score: self.matched_score,
            difference: self.difference,
            exact_match: self.exact_match,
            score_factor: self.score_factor,
            raw_score: self.raw_score,
            judgement: self.judgement.clone(),
        }
    }
}

fn candidates_from_eval(
    eval: &TupleEval,
    include_all: bool,
    filter: ReverseJudgementFilter,
    min_played_ratio: f64,
) -> Vec<ReverseCandidate> {
    let non_slide_total = eval.remaining_non_slide + eval.non_slide_perfect_plus + eval.non_slide_perfect;
    let local_keep = if include_all {
        eval.g_count
    } else {
        eval.g_count.min(DEFAULT_REVERSE_SOLUTIONS)
    };

    let mut candidates = Vec::with_capacity(local_keep);
    for offset in 0..local_keep {
        // Iterate from g_min upward: prefer fewer G (more realistic)
        let g = eval.g_min + offset as u32;
        let candidate = eval.to_candidate(g, 0);

        // Filter by min_played_ratio: skip candidates with too few played notes
        if min_played_ratio > 0.0 {
            let played = candidate.judgement.non_slide_played();
            let ratio = if non_slide_total > 0 {
                played as f64 / non_slide_total as f64
            } else {
                0.0
            };
            if ratio < min_played_ratio {
                continue;
            }
        }

        if filter.allows(&candidate.judgement) {
            candidates.push(candidate);
        }
    }

    candidates
}

fn push_candidate(
    candidates: &mut Vec<ReverseCandidate>,
    candidate: ReverseCandidate,
    include_all: bool,
) {
    let idx = candidates
        .binary_search_by(|current| compare_candidates(current, &candidate))
        .unwrap_or_else(|i| i);
    candidates.insert(idx, candidate);

    if !include_all && candidates.len() > DEFAULT_REVERSE_SOLUTIONS {
        candidates.pop();
    }
}

fn compare_candidates(a: &ReverseCandidate, b: &ReverseCandidate) -> Ordering {
    // 1. Closest score match always wins
    a.difference
        .abs()
        .cmp(&b.difference.abs())
    // 2. Fewer total misses first (key: prefer actually playing notes)
        .then_with(|| a.judgement.total_miss().cmp(&b.judgement.total_miss()))
    // 3. Higher P+ ratio on played non-slide notes (more realistic accuracy)
        .then_with(|| {
            let ar = ratio_on_played(&a.judgement);
            let br = ratio_on_played(&b.judgement);
            br.partial_cmp(&ar).unwrap_or(Ordering::Equal)
        })
    // 4. Higher raw P+ count (within same ratio)
        .then_with(|| {
            b.judgement
                .non_slide_perfect_plus
                .cmp(&a.judgement.non_slide_perfect_plus)
        })
    // 5. Fewer Good (lower G is more realistic)
        .then_with(|| a.judgement.non_slide_good.cmp(&b.judgement.non_slide_good))
    // 6. Higher SlideHit (more slides hit = better play)
        .then_with(|| b.judgement.slide_hit.cmp(&a.judgement.slide_hit))
    // 7. Higher matched score as final tiebreaker
        .then_with(|| b.matched_score.cmp(&a.matched_score))
}

fn ratio_on_played(j: &JudgementBreakdown) -> f64 {
    let played = j.non_slide_played();
    if played == 0 {
        return 0.0;
    }
    j.non_slide_perfect_plus as f64 / played as f64
}
