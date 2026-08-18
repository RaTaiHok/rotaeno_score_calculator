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

    const pPlus = candidate.judgement.non_slide_perfect_plus;
    const p = candidate.judgement.non_slide_perfect;
    const g = candidate.judgement.non_slide_good;
    const played = pPlus + p + g;
    const pPlusRatio = played > 0 ? ((pPlus / played) * 100).toFixed(1) : "0.0";
    const unplayed = candidate.judgement.non_slide_unplayed || 0;
    const totalMiss = (candidate.judgement.non_slide_miss || 0) + (candidate.judgement.slide_miss || 0);

    // 分情况：同一方案（同一分数）下 Miss 在 Slide/非Slide 之间的所有分配可能
    const variants = Array.isArray(candidate.miss_variants) ? candidate.miss_variants : [];
    const variantLabels = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const variantLines = variants.map((v, i) => {
      const label = i < variantLabels.length ? variantLabels[i] : String(i + 1);
      return `      ${label}) 非Slide G ${formatInt(v.non_slide_good)} | Slide Hit ${formatInt(v.slide_hit)} | 非Slide Miss ${formatInt(v.non_slide_miss)} | Slide Miss ${formatInt(v.slide_miss)}`;
    });

    const lines = [
      `[${index + 1}] 方案分数 ${formatInt(candidate.matched_score)} (差值 ${diffSign}${candidate.difference}) | P+比率: ${pPlusRatio}% | 总Miss: ${formatInt(totalMiss)}`,
      `  - 非Slide P+: ${formatInt(pPlus)}`,
      `  - 非Slide P: ${formatInt(p)}`,
      `  - 非Slide G: ${formatInt(g)}`,
      `  - 非Slide Miss: ${formatInt(candidate.judgement.non_slide_miss)}`,
      `  - Slide Hit: ${formatInt(candidate.judgement.slide_hit)}`,
      `  - Slide Miss: ${formatInt(candidate.judgement.slide_miss)}`
    ];

    if (variants.length > 1) {
      lines.push(`  - Miss 分配（${formatInt(variants.length)} 种可能）:`);
      lines.push(...variantLines);
    }

    if (unplayed > 0) {
      lines.push(`  - 未游玩: ${formatInt(unplayed)}`);
    }

    return lines.join("\n");
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
    "候选方案（已按 Miss数少、P+比率高、G数少 的优先级排序）:",
    candidateLines.length > 0 ? candidateLines.join("\n\n") : "-"
  ].join("\n");
}
