<script setup>
import { computed, onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

const songs = ref([]);
const difficulties = ref([]);
const selectedSongId = ref("");
const selectedDifficulty = ref("");
const songQuery = ref("");
const difficultyQuery = ref("");
const loadingSongs = ref(false);
const loadingDifficulties = ref(false);

const errorMessage = ref("");
const calcResultText = ref("-");
const reverseResultText = ref("-");

const judgement = reactive({
  non_slide_perfect_plus: 0,
  non_slide_perfect: 0,
  non_slide_good: 0,
  non_slide_miss: 0,
  slide_hit: 0,
  slide_miss: 0
});

const targetScore = ref(1_000_000);

const selectedChart = computed(
  () => difficulties.value.find((item) => item.difficulty === selectedDifficulty.value) ?? null
);

const nonSlideLimit = computed(() => Number(selectedChart.value?.non_slide_total ?? 0));
const slideLimit = computed(() => Number(selectedChart.value?.stats?.slide ?? 0));

const nonSlideSum = computed(
  () =>
    toInt(judgement.non_slide_perfect_plus) +
    toInt(judgement.non_slide_perfect) +
    toInt(judgement.non_slide_good) +
    toInt(judgement.non_slide_miss)
);

const slideSum = computed(
  () => toInt(judgement.slide_hit) + toInt(judgement.slide_miss)
);

const filteredSongs = computed(() => {
  const q = normalize(songQuery.value);
  if (!q) {
    return songs.value;
  }
  return songs.value.filter((song) => {
    const name = normalize(song.song_name);
    const id = normalize(song.song_id);
    return fuzzyMatch(name, q) || fuzzyMatch(id, q);
  });
});

const filteredDifficulties = computed(() => {
  const q = normalize(difficultyQuery.value);
  if (!q) {
    return difficulties.value;
  }
  return difficulties.value.filter((item) =>
    fuzzyMatch(normalize(item.difficulty), q)
  );
});

const inputWarning = computed(() => {
  if (!selectedChart.value) {
    return "";
  }
  if (nonSlideSum.value > nonSlideLimit.value) {
    return `非Slide输入总和 ${nonSlideSum.value} 超过上限 ${nonSlideLimit.value}`;
  }
  if (slideSum.value > slideLimit.value) {
    return `Slide输入总和 ${slideSum.value} 超过上限 ${slideLimit.value}`;
  }
  return "";
});

const canOperate = computed(() => !!selectedChart.value);

watch(selectedSongId, async (songId) => {
  resetChartSelection();
  if (!songId) {
    difficulties.value = [];
    return;
  }

  loadingDifficulties.value = true;
  try {
    clearError();
    difficulties.value = await invoke("get_song_difficulties", { songId });
  } catch (error) {
    setError(error);
  } finally {
    loadingDifficulties.value = false;
  }
});

watch(selectedDifficulty, () => {
  clearJudgement();
  calcResultText.value = "-";
  reverseResultText.value = "-";
  clearError();
});

onMounted(async () => {
  loadingSongs.value = true;
  try {
    songs.value = await invoke("list_songs");
  } catch (error) {
    setError(error);
  } finally {
    loadingSongs.value = false;
  }
});

function onSongFilter(query) {
  songQuery.value = query;
}

function onDifficultyFilter(query) {
  difficultyQuery.value = query;
}

function clearJudgement() {
  judgement.non_slide_perfect_plus = 0;
  judgement.non_slide_perfect = 0;
  judgement.non_slide_good = 0;
  judgement.non_slide_miss = 0;
  judgement.slide_hit = 0;
  judgement.slide_miss = 0;
}

function resetChartSelection() {
  selectedDifficulty.value = "";
  difficultyQuery.value = "";
  clearJudgement();
  calcResultText.value = "-";
  reverseResultText.value = "-";
}

async function calculateScore() {
  if (!selectedChart.value) {
    setError("请先选择歌曲和难度");
    return;
  }
  if (inputWarning.value) {
    setError(inputWarning.value);
    return;
  }

  try {
    clearError();
    const input = {
      song_id: selectedSongId.value,
      difficulty: selectedDifficulty.value,
      non_slide_perfect_plus: toInt(judgement.non_slide_perfect_plus),
      non_slide_perfect: toInt(judgement.non_slide_perfect),
      non_slide_good: toInt(judgement.non_slide_good),
      non_slide_miss: toInt(judgement.non_slide_miss),
      slide_hit: toInt(judgement.slide_hit),
      slide_miss: toInt(judgement.slide_miss)
    };

    const result = await invoke("calculate_score", { input });
    calcResultText.value = formatCalcResult(result);
  } catch (error) {
    setError(error);
  }
}

async function reverseFromScore() {
  if (!selectedChart.value) {
    setError("请先选择歌曲和难度");
    return;
  }
  const safeTargetScore = Math.max(0, toInt(targetScore.value));

  try {
    clearError();
    const input = {
      song_id: selectedSongId.value,
      difficulty: selectedDifficulty.value,
      target_score: safeTargetScore
    };
    const result = await invoke("reverse_from_score", { input });
    reverseResultText.value = formatReverseResult(result);

    judgement.non_slide_perfect_plus = toInt(result.judgement.non_slide_perfect_plus);
    judgement.non_slide_perfect = toInt(result.judgement.non_slide_perfect);
    judgement.non_slide_good = toInt(result.judgement.non_slide_good);
    judgement.non_slide_miss = toInt(result.judgement.non_slide_miss);
    judgement.slide_hit = toInt(result.judgement.slide_hit);
    judgement.slide_miss = toInt(result.judgement.slide_miss);
  } catch (error) {
    setError(error);
  }
}

function formatCalcResult(result) {
  return [
    `显示分数：${formatInt(result.display_score)}`,
    `原始分数：${Number(result.raw_score).toFixed(6)}`,
    `判定基本分 x：${Number(result.base_score).toFixed(6)}`,
    `Score Factor：${Number(result.score_factor).toFixed(6)}`,
    "",
    "判定分布：",
    `- 非Slide P+：${formatInt(result.judgement.non_slide_perfect_plus)}`,
    `- 非Slide P：${formatInt(result.judgement.non_slide_perfect)}`,
    `- 非Slide G：${formatInt(result.judgement.non_slide_good)}`,
    `- 非Slide Miss：${formatInt(result.judgement.non_slide_miss)}`,
    `- Slide Hit：${formatInt(result.judgement.slide_hit)}`,
    `- Slide Miss：${formatInt(result.judgement.slide_miss)}`
  ].join("\n");
}

function formatReverseResult(result) {
  const diffSign = result.difference >= 0 ? "+" : "";
  return [
    result.message,
    "",
    `目标分数：${formatInt(result.target_score)}`,
    `方案分数：${formatInt(result.matched_score)}（差值 ${diffSign}${result.difference}）`,
    `是否精确命中：${result.exact_match ? "是" : "否"}`,
    `原始分数：${Number(result.raw_score).toFixed(6)}`,
    "",
    "建议判定分布：",
    `- 非Slide P+：${formatInt(result.judgement.non_slide_perfect_plus)}`,
    `- 非Slide P：${formatInt(result.judgement.non_slide_perfect)}`,
    `- 非Slide G：${formatInt(result.judgement.non_slide_good)}`,
    `- 非Slide Miss：${formatInt(result.judgement.non_slide_miss)}`,
    `- Slide Hit：${formatInt(result.judgement.slide_hit)}`,
    `- Slide Miss：${formatInt(result.judgement.slide_miss)}`
  ].join("\n");
}

function setError(error) {
  if (typeof error === "string") {
    errorMessage.value = error;
    return;
  }
  if (error instanceof Error) {
    errorMessage.value = error.message;
    return;
  }
  try {
    errorMessage.value = JSON.stringify(error);
  } catch {
    errorMessage.value = "发生未知错误";
  }
}

function clearError() {
  errorMessage.value = "";
}

function normalize(text) {
  return String(text ?? "")
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "");
}

function fuzzyMatch(text, query) {
  if (!query) {
    return true;
  }
  if (text.includes(query)) {
    return true;
  }
  return isSubsequence(query, text);
}

function isSubsequence(pattern, text) {
  let p = 0;
  let t = 0;
  while (p < pattern.length && t < text.length) {
    if (pattern[p] === text[t]) {
      p += 1;
    }
    t += 1;
  }
  return p === pattern.length;
}

function formatInt(value) {
  return Number(value ?? 0).toLocaleString("en-US");
}

function toInt(value) {
  const n = Number.parseInt(String(value ?? "0"), 10);
  if (!Number.isFinite(n) || Number.isNaN(n) || n < 0) {
    return 0;
  }
  return n;
}
</script>

<template>
  <main class="container">
    <h1>Rotaeno 分数计算器</h1>

    <section class="panel">
      <h2>1. 选择谱面</h2>
      <div class="grid grid-two">
        <label class="field">
          <span>歌曲</span>
          <el-select
            v-model="selectedSongId"
            placeholder="请选择歌曲"
            filterable
            clearable
            :loading="loadingSongs"
            :filter-method="onSongFilter"
            class="full-width"
          >
            <el-option
              v-for="song in filteredSongs"
              :key="song.song_id"
              :label="song.song_name"
              :value="song.song_id"
            >
              <div class="option-row">
                <span>{{ song.song_name }}</span>
                <small>{{ song.song_id }}</small>
              </div>
            </el-option>
          </el-select>
        </label>

        <label class="field">
          <span>难度</span>
          <el-select
            v-model="selectedDifficulty"
            placeholder="请选择难度"
            filterable
            clearable
            :loading="loadingDifficulties"
            :disabled="!selectedSongId"
            :filter-method="onDifficultyFilter"
            class="full-width"
          >
            <el-option
              v-for="item in filteredDifficulties"
              :key="item.difficulty"
              :label="item.difficulty"
              :value="item.difficulty"
            />
          </el-select>
        </label>
      </div>

      <div class="info">
        <p>总物量：<strong>{{ selectedChart?.stats.total ?? "-" }}</strong></p>
        <p>Slide 数量 (S)：<strong>{{ selectedChart?.stats.slide ?? "-" }}</strong></p>
        <p>非Slide数量 (O)：<strong>{{ selectedChart?.non_slide_total ?? "-" }}</strong></p>
        <p>
          有效物量 (0.25S + O)：<strong>{{ selectedChart ? Number(selectedChart.effective_notes).toFixed(4) : "-" }}</strong>
        </p>
        <p>
          判定基本分 x：<strong>{{ selectedChart ? Number(selectedChart.base_score).toFixed(4) : "-" }}</strong>
        </p>
      </div>
    </section>

    <section class="panel">
      <h2>2. 输入判定</h2>
      <p class="muted">允许输入总和小于物量上限，未填部分等价视为 Miss。</p>

      <div class="grid grid-three">
        <label class="field">
          <span>非Slide Perfect+ (P)</span>
          <el-input-number v-model="judgement.non_slide_perfect_plus" :min="0" :disabled="!canOperate" />
        </label>
        <label class="field">
          <span>非Slide Perfect (p)</span>
          <el-input-number v-model="judgement.non_slide_perfect" :min="0" :disabled="!canOperate" />
        </label>
        <label class="field">
          <span>非Slide Good (g)</span>
          <el-input-number v-model="judgement.non_slide_good" :min="0" :disabled="!canOperate" />
        </label>
        <label class="field">
          <span>非Slide Miss</span>
          <el-input-number v-model="judgement.non_slide_miss" :min="0" :disabled="!canOperate" />
        </label>
        <label class="field">
          <span>Slide Hit (P')</span>
          <el-input-number v-model="judgement.slide_hit" :min="0" :disabled="!canOperate" />
        </label>
        <label class="field">
          <span>Slide Miss</span>
          <el-input-number v-model="judgement.slide_miss" :min="0" :disabled="!canOperate" />
        </label>
      </div>

      <div class="limit-box">
        <p>非Slide上限：<strong>{{ selectedChart ? nonSlideLimit : "-" }}</strong></p>
        <p>Slide上限：<strong>{{ selectedChart ? slideLimit : "-" }}</strong></p>
        <p>当前输入：非Slide {{ nonSlideSum }}/{{ selectedChart ? nonSlideLimit : "-" }}，Slide {{ slideSum }}/{{ selectedChart ? slideLimit : "-" }}</p>
      </div>
      <p v-if="inputWarning" class="warning">{{ inputWarning }}</p>
    </section>

    <section class="panel">
      <h2>3. 计算 / 反算</h2>
      <div class="action-row">
        <el-button type="primary" :disabled="!canOperate" @click="calculateScore">计算分数</el-button>
        <div class="target-box">
          <span>目标分数（反算）</span>
          <el-input-number v-model="targetScore" :min="0" :disabled="!canOperate" />
        </div>
        <el-button :disabled="!canOperate" @click="reverseFromScore">尝试反算</el-button>
      </div>

      <div class="result">
        <h3>计算结果</h3>
        <pre>{{ calcResultText }}</pre>
      </div>

      <div class="result">
        <h3>反算结果</h3>
        <pre>{{ reverseResultText }}</pre>
      </div>

      <p v-if="errorMessage" class="error">{{ errorMessage }}</p>
    </section>
  </main>
</template>
