use crate::model::{JudgementBreakdown, NoteStats};

// Exact integer scoring:
// x = 4_000_000 / (4 * O + S)
// factor = (404 * P+ + 400 * p + 101 * (g + slide_hit)) / 400
// raw = x * factor = 10_000 * score_num / (4 * O + S)
pub const SCORE_NUM_MULT: i128 = 10_000;
pub const PPLUS_TERM_NUM: i128 = 404;
pub const P_TERM_NUM: i128 = 400;
pub const H_TERM_NUM: i128 = 101;
pub const SCORE_FACTOR_DEN: f64 = 400.0;

#[derive(Debug, Clone, Copy)]
pub struct ScoreMath {
    pub denominator: i128,
}

impl ScoreMath {
    pub fn new(stats: &NoteStats) -> Self {
        Self {
            denominator: score_denominator(stats),
        }
    }

    pub fn fixed_term(self, non_slide_perfect_plus: u32, h: u32) -> i128 {
        PPLUS_TERM_NUM * non_slide_perfect_plus as i128 + H_TERM_NUM * h as i128
    }

    pub fn score_term(self, non_slide_perfect_plus: u32, non_slide_perfect: u32, h: u32) -> i128 {
        self.fixed_term(non_slide_perfect_plus, h) + P_TERM_NUM * non_slide_perfect as i128
    }

    pub fn from_judgement(self, judgement: &JudgementBreakdown) -> ScoreParts {
        self.from_terms(
            judgement.non_slide_perfect_plus,
            judgement.non_slide_perfect,
            judgement.non_slide_good + judgement.slide_hit,
        )
    }

    pub fn from_terms(
        self,
        non_slide_perfect_plus: u32,
        non_slide_perfect: u32,
        h: u32,
    ) -> ScoreParts {
        self.from_score_term(self.score_term(non_slide_perfect_plus, non_slide_perfect, h))
    }

    pub fn from_score_term(self, score_term_num: i128) -> ScoreParts {
        let raw_num = SCORE_NUM_MULT * score_term_num;
        let display_score = div_floor(raw_num, self.denominator) as u32;

        ScoreParts {
            score_factor: score_term_num as f64 / SCORE_FACTOR_DEN,
            raw_score: raw_num as f64 / self.denominator as f64,
            display_score,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScoreParts {
    pub score_factor: f64,
    pub raw_score: f64,
    pub display_score: u32,
}

pub fn score_denominator(stats: &NoteStats) -> i128 {
    (4 * stats.non_slide_total() + stats.slide) as i128
}

pub fn exact_p_range(
    target_score: u32,
    score_den: i128,
    fixed_num: i128,
    max_p: u32,
) -> Option<(u32, u32)> {
    let p_den = SCORE_NUM_MULT * P_TERM_NUM;

    let low_num = target_score as i128 * score_den - SCORE_NUM_MULT * fixed_num;
    let high_num = (target_score as i128 + 1) * score_den - 1 - SCORE_NUM_MULT * fixed_num;

    let mut p_min = div_ceil(low_num, p_den);
    let mut p_max = div_floor(high_num, p_den);

    if p_max < 0 || p_min > max_p as i128 {
        return None;
    }

    p_min = p_min.max(0);
    p_max = p_max.min(max_p as i128);

    if p_min > p_max {
        None
    } else {
        Some((p_min as u32, p_max as u32))
    }
}

pub fn nearest_p_probes(
    target_score: u32,
    score_den: i128,
    fixed_num: i128,
    max_p: u32,
) -> Vec<u32> {
    let p_den = SCORE_NUM_MULT * P_TERM_NUM;

    let low_num = target_score as i128 * score_den - SCORE_NUM_MULT * fixed_num;
    let high_num = (target_score as i128 + 1) * score_den - 1 - SCORE_NUM_MULT * fixed_num;

    let q_low = div_floor(low_num, p_den);
    let q_high = div_floor(high_num, p_den);

    let probes = [
        q_low - 1,
        q_low,
        q_low + 1,
        q_high,
        q_high + 1,
        0,
        max_p as i128,
    ];

    let mut out: Vec<u32> = Vec::with_capacity(probes.len());
    for p in probes {
        if p < 0 || p > max_p as i128 {
            continue;
        }

        let p_u32 = p as u32;
        if !out.contains(&p_u32) {
            out.push(p_u32);
        }
    }

    out
}

pub fn div_floor(a: i128, b: i128) -> i128 {
    let mut q = a / b;
    let r = a % b;
    if r != 0 && ((r > 0) != (b > 0)) {
        q -= 1;
    }
    q
}

fn div_ceil(a: i128, b: i128) -> i128 {
    let mut q = a / b;
    let r = a % b;
    if r != 0 && ((r > 0) == (b > 0)) {
        q += 1;
    }
    q
}
