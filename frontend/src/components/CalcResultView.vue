<script setup>
import { computed, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { formatFixed, formatInt } from "../utils/number";

const props = defineProps({
  result: {
    type: Object,
    required: true
  }
});

const copied = ref(false);

function fmt(value) {
  return formatInt(value);
}

const judgementRows = computed(() => {
  const j = props.result.judgement || {};
  return [
    { label: "非Slide Perfect+ (P+)", value: j.non_slide_perfect_plus },
    { label: "非Slide Perfect (P)", value: j.non_slide_perfect },
    { label: "非Slide Good (G)", value: j.non_slide_good },
    { label: "非Slide Miss", value: j.non_slide_miss },
    { label: "Slide Miss", value: j.slide_miss }
  ];
});

async function copyCalcResult() {
  const r = props.result;
  const j = r.judgement || {};
  const text = [
    `【正算结果】分数 ${fmt(r.display_score)}`,
    `原始分数: ${formatFixed(r.raw_score)} · 判定基本分: ${formatFixed(r.base_score)} · Score Factor: ${formatFixed(r.score_factor)}`,
    `判定分布: P+${j.non_slide_perfect_plus} P${j.non_slide_perfect} G${j.non_slide_good} ` +
      `非Miss${j.non_slide_miss} SlideMiss${j.slide_miss}`
  ].join("\n");
  try {
    await writeText(text);
    copied.value = true;
  } catch {
    try {
      await navigator.clipboard.writeText(text);
      copied.value = true;
    } catch (err) {
      console.error("复制失败:", err);
      return;
    }
  }
  setTimeout(() => {
    copied.value = false;
  }, 1500);
}
</script>

<template>
  <div class="calc-result">
    <div class="calc-header">
      <div class="calc-score">
        <span class="calc-score-label">显示分数</span>
        <span class="calc-score-value">{{ fmt(result.display_score) }}</span>
      </div>
      <button class="mini-btn" @click="copyCalcResult">
        {{ copied ? "已复制" : "复制结果" }}
      </button>
    </div>

    <div class="calc-meta">
      <span>原始分数 {{ formatFixed(result.raw_score) }}</span>
      <span>判定基本分 {{ formatFixed(result.base_score) }}</span>
      <span>Score Factor {{ formatFixed(result.score_factor) }}</span>
    </div>

    <table class="calc-table">
      <thead>
        <tr>
          <th>判定</th>
          <th>数量</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, i) in judgementRows" :key="i">
          <td>{{ row.label }}</td>
          <td>{{ fmt(row.value) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.calc-result {
  font-size: 14px;
}

.calc-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  background: #eef2fb;
  border: 1px solid #d7ddea;
  border-radius: 10px;
  padding: 12px 16px;
  margin-bottom: 10px;
}

.calc-score {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.calc-score-label {
  color: #8a97ae;
  font-size: 12px;
}

.calc-score-value {
  color: #33415c;
  font-size: 30px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: 1px;
}

.calc-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
  font-size: 12px;
  color: #8a97ae;
  margin-bottom: 10px;
}

.calc-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.calc-table th,
.calc-table td {
  border: 1px solid #e8ebf3;
  padding: 5px 10px;
  text-align: center;
}

.calc-table thead th {
  background: #f4f6fb;
  color: #51607a;
  font-weight: 600;
}

.mini-btn {
  border: 1px solid #c8d0df;
  background: #fff;
  border-radius: 6px;
  padding: 4px 12px;
  font-size: 12px;
  color: #33415c;
  cursor: pointer;
  white-space: nowrap;
}

.mini-btn:hover {
  border-color: #5b7cfa;
  color: #5b7cfa;
}
</style>
