<script setup>
import { ref, watch } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { formatInt } from "../utils/number";

const props = defineProps({
  result: {
    type: Object,
    required: true
  },
  showApply: {
    type: Boolean,
    default: true
  }
});

const emit = defineEmits(["apply-candidate"]);

// 每个方案判定组合默认展示条数
const COMBO_PREVIEW = 5;
// “展示全部”模式：方案分页加载步长
const VISIBLE_STEP = 10;
// 方案内展开超出的组合
const expandedCombos = ref({});
const copiedIdx = ref(-1);
const copyTimer = ref(null);
const copiedAll = ref(false);
const visibleCount = ref(VISIBLE_STEP);

// 结果变化时重置分页
watch(
  () => props.result,
  () => {
    visibleCount.value = VISIBLE_STEP;
  }
);

function fmt(value) {
  return formatInt(value);
}

function diffOf(candidate) {
  const d = candidate.difference ?? 0;
  return `${d >= 0 ? "+" : ""}${formatInt(d)}`;
}

function totalMiss(judgement) {
  return (judgement.non_slide_miss || 0) + (judgement.slide_miss || 0);
}

function candidates() {
  return Array.isArray(props.result.candidates) ? props.result.candidates : [];
}

function visibleCandidates() {
  return candidates().slice(0, visibleCount.value);
}

function hasMore() {
  return visibleCount.value < candidates().length;
}

function loadMore() {
  visibleCount.value += VISIBLE_STEP;
}

function combosOf(candidate) {
  return Array.isArray(candidate.miss_variants) ? candidate.miss_variants : [];
}

function comboTotal(candidate) {
  return candidate.miss_variant_total ?? combosOf(candidate).length;
}

function shownCombos(candidate) {
  const all = combosOf(candidate);
  if (expandedCombos.value[candidateIdx(candidate)] || all.length <= COMBO_PREVIEW) {
    return all;
  }
  return all.slice(0, COMBO_PREVIEW);
}

function hasMoreCombos(candidate) {
  return combosOf(candidate).length > COMBO_PREVIEW;
}

function candidateIdx(candidate) {
  return candidates().indexOf(candidate);
}

function toggleCombos(candidate) {
  const idx = candidateIdx(candidate);
  expandedCombos.value = {
    ...expandedCombos.value,
    [idx]: !expandedCombos.value[idx]
  };
}

function applyCandidate(candidate) {
  emit("apply-candidate", candidate.judgement);
}

function flashCopied(i) {
  copiedIdx.value = i;
  if (copyTimer.value) {
    clearTimeout(copyTimer.value);
  }
  copyTimer.value = setTimeout(() => {
    copiedIdx.value = -1;
  }, 1500);
}

async function copyCandidate(candidate, i) {
  const j = candidate.judgement;
  const combos = combosOf(candidate);
  const text = [
    `【方案 ${i + 1}】P+ ${j.non_slide_perfect_plus} · P ${j.non_slide_perfect}`,
    `分数: ${fmt(candidate.matched_score)} (${diffOf(candidate)}) · 总Miss: ${fmt(totalMiss(j))}`,
    `判定组合:`
  ];
  combos.forEach((v, vi) => {
    const label = vi < variantLabels.length ? variantLabels[vi] : vi + 1;
    text.push(`  ${label}) G${v.non_slide_good} 非SlideMiss${v.non_slide_miss} SlideMiss${v.slide_miss}`);
  });
  if (comboTotal(candidate) > combos.length) {
    text.push(`  …共 ${fmt(comboTotal(candidate))} 种`);
  }
  const copyText = text.join("\n");
  try {
    await writeText(copyText);
    flashCopied(i);
  } catch {
    try {
      await navigator.clipboard.writeText(copyText);
      flashCopied(i);
    } catch (err) {
      console.error("复制失败:", err);
    }
  }
}

async function copyAll() {
  const text = allResultText();
  try {
    await writeText(text);
    copiedAll.value = true;
  } catch {
    try {
      await navigator.clipboard.writeText(text);
      copiedAll.value = true;
    } catch (err) {
      console.error("复制失败:", err);
      return;
    }
  }
  setTimeout(() => {
    copiedAll.value = false;
  }, 1500);
}

function allResultText() {
  const r = props.result;
  const list = candidates();
  if (list.length === 0) {
    return `【反算结果】目标 ${fmt(r.target_score)}，未找到可行方案`;
  }
  const hitInfo = r.exact_match
    ? `精确命中 ${fmt(r.exact_candidate_count ?? 0)} 个方案`
    : `未精确命中（差值 ${diffOf(list[0])}）`;
  const lines = [`【反算结果】目标 ${fmt(r.target_score)}，${hitInfo}，共 ${fmt(list.length)} 个方案`];
  list.forEach((c, i) => {
    const j = c.judgement;
    lines.push(
      `【方案 ${i + 1}】P+ ${j.non_slide_perfect_plus} · P ${j.non_slide_perfect} · ` +
        `分数 ${fmt(c.matched_score)} (${diffOf(c)}) · 总Miss ${fmt(totalMiss(j))}`
    );
    const combos = combosOf(c).slice(0, COMBO_PREVIEW);
    combos.forEach((v) => {
      lines.push(`  G${v.non_slide_good} 非SlideMiss${v.non_slide_miss} SlideMiss${v.slide_miss}`);
    });
    if (comboTotal(c) > combos.length) {
      lines.push(`  …共 ${fmt(comboTotal(c))} 种`);
    }
  });
  return lines.join("\n");
}

const overview = () => {
  const r = props.result;
  const list = candidates();
  if (list.length === 0) {
    return `目标 ${fmt(r.target_score)} │ 未找到可行方案`;
  }
  const top = list[0];
  const j = top.judgement;
  const hitInfo = r.exact_match
    ? `精确命中 ${fmt(r.exact_candidate_count ?? 0)} 个方案`
    : `未精确命中（差值 ${diffOf(top)}）`;
  return `目标 ${fmt(r.target_score)} │ ${hitInfo} │ 共 ${fmt(list.length)} 个方案`;
};

const variantLabels = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
</script>

<template>
  <div class="reverse-result">
    <div class="reverse-overview">
      <span class="overview-text">{{ overview() }}</span>
      <button class="mini-btn copy-all-btn" @click="copyAll">
        {{ copiedAll ? "已复制全部" : "复制全部" }}
      </button>
    </div>

    <div
      v-for="(c, i) in visibleCandidates()"
      :key="i"
      class="scheme-block"
    >
      <div class="scheme-header">
        <div class="scheme-info">
          <span class="scheme-title">方案 {{ i + 1 }}</span>
          <span class="judge-badge badge-pplus">P+ {{ c.judgement.non_slide_perfect_plus }}</span>
          <span class="judge-badge badge-p">P {{ c.judgement.non_slide_perfect }}</span>
          <span class="judge-badge badge-miss">总Miss {{ fmt(totalMiss(c.judgement)) }}</span>
          <span class="scheme-meta">
            分数 {{ fmt(c.matched_score) }} ({{ diffOf(c) }})
          </span>
        </div>
        <div class="scheme-ops">
          <button
            v-if="showApply"
            class="mini-btn"
            title="把该方案判定回填到上方输入框"
            @click="applyCandidate(c)"
          >
            应用到输入
          </button>
          <button class="mini-btn" title="复制该方案" @click="copyCandidate(c, i)">
            {{ copiedIdx === i ? "已复制" : "复制" }}
          </button>
        </div>
      </div>

      <div class="combo-wrap">
        <table class="combo-table">
          <thead>
            <tr>
              <th>组合</th>
              <th>G</th>
              <th>非Slide Miss</th>
              <th>Slide Miss</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(v, vi) in shownCombos(c)" :key="vi">
              <td class="combo-idx">{{ vi < variantLabels.length ? variantLabels[vi] : vi + 1 }}</td>
              <td>{{ v.non_slide_good }}</td>
              <td>{{ fmt(v.non_slide_miss) }}</td>
              <td>{{ fmt(v.slide_miss) }}</td>
            </tr>
          </tbody>
        </table>
        <button
          v-if="hasMoreCombos(c)"
          class="combo-more"
          @click="toggleCombos(c)"
        >
          {{ expandedCombos[candidateIdx(c)] ? "收起" : `…共 ${fmt(comboTotal(c))} 种，展开` }}
        </button>
      </div>
    </div>

    <div v-if="hasMore()" class="load-more-wrap">
      <button class="load-more-btn" @click="loadMore">
        加载更多（已显示 {{ visibleCandidates().length }} / 共 {{ fmt(candidates().length) }} 个方案）
      </button>
    </div>

    <div class="reverse-legend">
      P+ = 非Slide Perfect+ 数 · P = 非Slide Perfect 数 · G = 非Slide Good 数 ·
      Slide Hit 隐含（Slide 总数 − Slide Miss）
    </div>
  </div>
</template>

<style scoped>
.reverse-result {
  font-size: 14px;
}

.reverse-overview {
  background: #eef2fb;
  border: 1px solid #d7ddea;
  border-radius: 8px;
  padding: 8px 12px;
  margin-bottom: 12px;
  color: #33415c;
  word-break: break-all;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.overview-text {
  flex: 1;
  min-width: 0;
}

.copy-all-btn {
  flex-shrink: 0;
}

.scheme-block {
  border: 1px solid #e2e6ef;
  border-radius: 10px;
  margin-bottom: 10px;
  overflow: hidden;
}

.scheme-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 12px;
  background: #f7f9fe;
  flex-wrap: wrap;
}

.scheme-info {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  min-width: 0;
}

.scheme-title {
  font-weight: 700;
  color: #33415c;
}

.judge-badge {
  padding: 2px 10px;
  border-radius: 999px;
  font-weight: 700;
  font-size: 14px;
  white-space: nowrap;
}

.badge-pplus {
  background: #fff3d6;
  color: #a87906;
  border: 1px solid #e8c96a;
}

.badge-p {
  background: #e3edff;
  color: #3a6fd8;
  border: 1px solid #b9d0f5;
}

.badge-miss {
  background: #fde8e8;
  color: #c0392b;
  border: 1px solid #f0b9b4;
}

.scheme-meta {
  color: #51607a;
  font-size: 13px;
  word-break: break-all;
}

.scheme-ops {
  display: flex;
  gap: 6px;
  white-space: nowrap;
}

.mini-btn {
  border: 1px solid #c8d0df;
  background: #fff;
  border-radius: 6px;
  padding: 3px 10px;
  font-size: 12px;
  color: #33415c;
  cursor: pointer;
}

.mini-btn:hover {
  border-color: #5b7cfa;
  color: #5b7cfa;
}

.combo-wrap {
  padding: 8px 12px 10px;
}

.combo-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.combo-table th,
.combo-table td {
  border: 1px solid #e8ebf3;
  padding: 5px 10px;
  text-align: center;
}

.combo-table thead th {
  background: #f4f6fb;
  color: #51607a;
  font-weight: 600;
}

.combo-idx {
  color: #8a97ae;
  font-weight: 600;
}

.combo-more {
  margin-top: 6px;
  border: none;
  background: none;
  color: #5b7cfa;
  font-size: 12px;
  cursor: pointer;
  padding: 2px 4px;
}

.combo-more:hover {
  text-decoration: underline;
}

.load-more-wrap {
  text-align: center;
  margin: 4px 0 10px;
}

.load-more-btn {
  border: 1px solid #c8d0df;
  background: #fff;
  border-radius: 999px;
  padding: 6px 22px;
  font-size: 13px;
  color: #5b7cfa;
  cursor: pointer;
}

.load-more-btn:hover {
  border-color: #5b7cfa;
  background: #f4f7ff;
}

.reverse-legend {
  margin-top: 4px;
  font-size: 12px;
  color: #8a97ae;
  line-height: 1.6;
}
</style>
