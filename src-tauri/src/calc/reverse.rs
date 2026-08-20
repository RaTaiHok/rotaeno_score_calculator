use std::cmp::Ordering;

use crate::calc::math::{exact_p_range, nearest_p_probes, ScoreMath};
use crate::model::{
    JudgementBreakdown, MissVariant, NoteStats, ReverseCandidateResult, ReverseInput,
    ReverseJudgementFilter, ReverseResult,
};

const DEFAULT_REVERSE_SOLUTIONS: usize = 3;
/// 每个反算方案最多展示的 Miss 分配变体数。
/// 同一分数下 Good/SlideHit 可拆分的变体可能成百上千（尤其低分数 + 大量 Slide），
/// 全部列出会导致结果爆炸（序列化慢、前端渲染卡），只保留 Miss 最少的几种即可。
const MAX_MISS_VARIANTS: usize = 12;

pub fn from_target(
    stats: &NoteStats,
    input: &ReverseInput,
    include_all: bool,
    min_played_ratio: f64,
    mut on_progress: impl FnMut(u8),
) -> Result<ReverseResult, String> {
    let non_slide_total = stats.non_slide_total();
    let slide_total = stats.slide;
    let math = ScoreMath::new(stats);
    let filter = input.judgement_filter();

    // 预计算总外层迭代量，用于进度上报
    let mut total_iters: u64 = 0;
    for p_plus in 0..=non_slide_total {
        total_iters += ((non_slide_total - p_plus) + slide_total + 1) as u64;
    }
    let mut done_iters: u64 = 0;

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

        done_iters += (max_h + 1) as u64;
        // 封顶 99%：最后的 100% 在 build_result（排序/序列化）完成后才发，
        // 避免“进度已 100% 但结果迟迟不出现”的错觉
        let percent = (((done_iters * 100) / total_iters.max(1)) as u8).min(99);
        on_progress(percent);
    }

    let result = if exact.has_candidates() {
        let exact_count = exact.count;
        Ok(build_result(stats, input, exact, exact_count, include_all))
    } else if nearest.has_candidates() {
        Ok(build_result(stats, input, nearest, 0, include_all))
    } else {
        Err("未找到符合当前判定筛选条件的可行分布，请放宽筛选后重试。".to_string())
    };

    // 100% 与真正完成同步：build_result（排序、构造输出、序列化）之后才上报
    on_progress(100);
    result
}

fn build_result(
    stats: &NoteStats,
    input: &ReverseInput,
    mut bucket: CandidateBucket,
    exact_candidate_count: usize,
    include_all: bool,
) -> ReverseResult {
    // 展示全部模式：收集阶段未排序，此处统一排序
    if include_all {
        bucket.top.sort_by(compare_candidates);
    }
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
        let Some(candidate) = candidates_from_eval(eval, filter, min_played_ratio) else {
            return;
        };

        self.count += 1;
        push_candidate(&mut self.top, candidate, self.include_all);
    }

    fn push_nearest_eval(&mut self, eval: &TupleEval, filter: ReverseJudgementFilter, min_played_ratio: f64) {
        let Some(candidate) = candidates_from_eval(eval, filter, min_played_ratio) else {
            return;
        };

        let abs_diff = eval.difference.abs();

        match self.nearest_abs_diff {
            None => {
                self.nearest_abs_diff = Some(abs_diff);
                self.count = 1;
                self.top.clear();
                push_candidate(&mut self.top, candidate, self.include_all);
            }
            Some(current_best) if abs_diff < current_best => {
                self.nearest_abs_diff = Some(abs_diff);
                self.count = 1;
                self.top.clear();
                push_candidate(&mut self.top, candidate, self.include_all);
            }
            Some(current_best) if abs_diff == current_best => {
                self.count += 1;
                push_candidate(&mut self.top, candidate, self.include_all);
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
    g_max: u32,
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
            g_max,
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
            miss_variants: Vec::new(),
            miss_variant_total: 0,
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
    /// 该方案下展示的 Miss 分配变体（最多 MAX_MISS_VARIANTS 个）
    miss_variants: Vec<MissVariant>,
    /// 该方案实际的总 Miss 分配变体数
    miss_variant_total: usize,
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
            miss_variants: self.miss_variants.clone(),
            miss_variant_total: self.miss_variant_total,
        }
    }
}

/// 将一个 (P+, P, h) 元组转换为一个反算方案：
/// 枚举 h 在 Good / SlideHit 之间的所有拆分（g ∈ [g_min, g_max]），
/// 每种拆分对应一种 Miss 分配（non_slide_miss / slide_miss）。
/// 分数相同，但 Miss 在 Slide/非Slide 之间分配不同，因此全部保留为变体。
fn candidates_from_eval(
    eval: &TupleEval,
    filter: ReverseJudgementFilter,
    min_played_ratio: f64,
) -> Option<ReverseCandidate> {
    let non_slide_total =
        eval.remaining_non_slide + eval.non_slide_perfect_plus + eval.non_slide_perfect;

    let mut miss_variants: Vec<MissVariant> = Vec::new();
    let mut miss_variant_total: usize = 0;
    let mut primary: Option<ReverseCandidate> = None;

    for g in eval.g_min..=eval.g_max {
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

        if !filter.allows(&candidate.judgement) {
            continue;
        }

        miss_variant_total += 1;
        if primary.is_none() {
            primary = Some(candidate.clone());
        }
        // 只保留 Miss 最少的变体（g 从小到大 = slide_miss 递增），避免结果爆炸
        if miss_variants.len() < MAX_MISS_VARIANTS {
            miss_variants.push(MissVariant {
                non_slide_good: candidate.judgement.non_slide_good,
                non_slide_miss: candidate.judgement.non_slide_miss,
                slide_hit: candidate.judgement.slide_hit,
                slide_miss: candidate.judgement.slide_miss,
            });
        }
    }

    let mut candidate = primary?;
    candidate.miss_variant_total = miss_variant_total;
    candidate.miss_variants = miss_variants;
    Some(candidate)
}

fn push_candidate(
    candidates: &mut Vec<ReverseCandidate>,
    candidate: ReverseCandidate,
    include_all: bool,
) {
    // 展示全部模式：只收集，最后统一排序（避免 Vec::insert O(n²)）
    if include_all {
        candidates.push(candidate);
        return;
    }

    let idx = candidates
        .binary_search_by(|current| compare_candidates(current, &candidate))
        .unwrap_or_else(|i| i);
    candidates.insert(idx, candidate);

    if candidates.len() > DEFAULT_REVERSE_SOLUTIONS {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NoteStats;

    fn test_stats() -> NoteStats {
        NoteStats {
            tap: 50,
            flick: 20,
            slide: 10,
            catch: 10,
            rotate: 0,
            total: 90,
        }
    }

    fn reverse_input(target_score: u32) -> ReverseInput {
        ReverseInput {
            song_id: "test".into(),
            difficulty: "IV".into(),
            target_score,
            allow_perfect_plus: true,
            allow_perfect: true,
            allow_good: true,
            allow_miss: true,
            min_played_ratio: 0.0,
        }
    }

    #[test]
    fn miss_variants_share_score_and_conserve_counts() {
        let stats = test_stats();
        let result = from_target(&stats, &reverse_input(950_000), true, 0.0, |_| {}).unwrap();

        assert!(!result.candidates.is_empty(), "应至少有一个方案");

        let math = ScoreMath::new(&stats);
        for c in &result.candidates {
            assert!(!c.miss_variants.is_empty(), "每个方案都应有 Miss 变体");
            assert!(
                c.miss_variants.len() >= 1,
                "Good/SlideHit 可拆分时应有多个 Miss 分配"
            );

            let mut saw_different_miss_split = false;
            for v in &c.miss_variants {
                // 总数守恒
                let non_slide_sum = c.judgement.non_slide_perfect_plus
                    + c.judgement.non_slide_perfect
                    + v.non_slide_good
                    + v.non_slide_miss;
                assert_eq!(non_slide_sum, stats.non_slide_total(), "非Slide 总数守恒");
                assert_eq!(v.slide_hit + v.slide_miss, stats.slide, "Slide 总数守恒");

                // 重建分数：变体分数必须等于方案分数
                let jb = JudgementBreakdown {
                    non_slide_perfect_plus: c.judgement.non_slide_perfect_plus,
                    non_slide_perfect: c.judgement.non_slide_perfect,
                    non_slide_good: v.non_slide_good,
                    non_slide_miss: v.non_slide_miss,
                    slide_hit: v.slide_hit,
                    slide_miss: v.slide_miss,
                    non_slide_unplayed: 0,
                };
                assert_eq!(math.from_judgement(&jb).display_score, c.matched_score);

                // 总 Miss 数也应一致（Miss 不计分）
                let total_miss = v.non_slide_miss + v.slide_miss;
                assert_eq!(
                    total_miss,
                    c.judgement.non_slide_miss + c.judgement.slide_miss,
                    "总 Miss 数不变"
                );

                if c.miss_variants.len() > 1 {
                    let first = &c.miss_variants[0];
                    if v.non_slide_miss != first.non_slide_miss
                        || v.slide_miss != first.slide_miss
                    {
                        saw_different_miss_split = true;
                    }
                }
            }

            if c.miss_variants.len() > 1 {
                assert!(
                    saw_different_miss_split,
                    "存在多个变体时应包含不同的 Miss 分配"
                );
            }
        }
    }
}
