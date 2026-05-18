import { formatFixed, formatInt } from "./number";

export function formatCalcResult(result) {
  return [
    `显示分数: ${formatInt(result.display_score)}`,
    `原始分数: ${formatFixed(result.raw_score)}`,
    `判定基本分 x: ${formatFixed(result.base_score)}`,
    `Score Factor: ${formatFixed(result.score_factor)}`,
    "",
    "判定分布:",
    `- 非Slide P+: ${formatInt(result.judgement.non_slide_perfect_plus)}`,
    `- 非Slide P: ${formatInt(result.judgement.non_slide_perfect)}`,
    `- 非Slide G: ${formatInt(result.judgement.non_slide_good)}`,
    `- 非Slide Miss: ${formatInt(result.judgement.non_slide_miss)}`,
    `- Slide Hit: ${formatInt(result.judgement.slide_hit)}`,
    `- Slide Miss: ${formatInt(result.judgement.slide_miss)}`
  ].join("\n");
}

export function formatReverseResult(result) {
  const candidates = Array.isArray(result.candidates) ? result.candidates : [];

  const candidateLines = candidates.map((candidate, index) => {
    const diffSign = candidate.difference >= 0 ? "+" : "";

    return [
      `[${index + 1}] 方案分数 ${formatInt(candidate.matched_score)} (差值 ${diffSign}${candidate.difference})`,
      `  - 非Slide P+: ${formatInt(candidate.judgement.non_slide_perfect_plus)}`,
      `  - 非Slide P: ${formatInt(candidate.judgement.non_slide_perfect)}`,
      `  - 非Slide G: ${formatInt(candidate.judgement.non_slide_good)}`,
      `  - 非Slide Miss: ${formatInt(candidate.judgement.non_slide_miss)}`,
      `  - Slide Hit: ${formatInt(candidate.judgement.slide_hit)}`,
      `  - Slide Miss: ${formatInt(candidate.judgement.slide_miss)}`
    ].join("\n");
  });

  return [
    String(result.message ?? ""),
    "",
    `目标分数: ${formatInt(result.target_score)}`,
    `推荐方案分数: ${formatInt(result.matched_score)} (差值 ${result.difference >= 0 ? "+" : ""}${result.difference})`,
    `是否精确命中: ${result.exact_match ? "是" : "否"}`,
    `总候选数: ${formatInt(result.candidate_count ?? candidates.length)}`,
    `展示方案数: ${formatInt(candidates.length)}`,
    `精确命中数: ${formatInt(result.exact_candidate_count ?? 0)}`,
    "",
    "候选方案（已按 Perfect+、Perfect、Good、Miss 优先级排序）:",
    candidateLines.length > 0 ? candidateLines.join("\n\n") : "-"
  ].join("\n");
}
