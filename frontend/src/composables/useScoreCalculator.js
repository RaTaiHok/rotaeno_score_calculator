import { computed, onMounted, reactive, ref, watch } from "vue";
import {
  calculateScore as requestCalculateScore,
  getSongDifficulties,
  listSongs,
  reverseAllFromScore as requestReverseAllFromScore,
  reverseFromScore as requestReverseFromScore
} from "../api/scoreApi";
import { stringifyError } from "../utils/error";
import { toInt } from "../utils/number";
import { formatCalcResult, formatReverseResult } from "../utils/resultFormat";
import { fuzzyMatch, normalize } from "../utils/text";

const EMPTY_RESULT = "-";

function createEmptyJudgement() {
  return {
    non_slide_perfect_plus: 0,
    non_slide_perfect: 0,
    non_slide_good: 0,
    non_slide_miss: 0,
    slide_hit: 0,
    slide_miss: 0
  };
}

export function useScoreCalculator(onResult = null) {
  const songs = ref([]);
  const difficulties = ref([]);
  const selectedSongId = ref("");
  const selectedDifficulty = ref("");
  const songQuery = ref("");
  const difficultyQuery = ref("");
  const loadingSongs = ref(false);
  const loadingDifficulties = ref(false);

  const errorMessage = ref("");
  const calcResultText = ref(EMPTY_RESULT);
  const reverseResultText = ref(EMPTY_RESULT);
  const judgement = reactive(createEmptyJudgement());
  const reverseFilter = reactive({
    allow_perfect_plus: true,
    allow_perfect: true,
    allow_good: true,
    allow_miss: true
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
  const slideSum = computed(() => toInt(judgement.slide_hit) + toInt(judgement.slide_miss));
  const canOperate = computed(() => !!selectedChart.value);

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

    return difficulties.value.filter((item) => fuzzyMatch(normalize(item.difficulty), q));
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

  watch(selectedSongId, async (songId) => {
    resetChartSelection();

    if (!songId) {
      difficulties.value = [];
      return;
    }

    loadingDifficulties.value = true;
    try {
      clearError();
      difficulties.value = await getSongDifficulties(songId);
    } catch (error) {
      setError(error);
    } finally {
      loadingDifficulties.value = false;
    }
  });

  watch(selectedDifficulty, () => {
    clearJudgement();
    clearResults();
    clearError();
  });

  onMounted(loadSongs);

  async function loadSongs() {
    loadingSongs.value = true;
    try {
      songs.value = await listSongs();
    } catch (error) {
      setError(error);
    } finally {
      loadingSongs.value = false;
    }
  }

  async function calculateScore() {
    if (!ensureChartSelected()) {
      return;
    }

    if (inputWarning.value) {
      setError(inputWarning.value);
      return;
    }

    try {
      clearError();
      const result = await requestCalculateScore(createScoreInput());
      calcResultText.value = formatCalcResult(result);
      saveEntry("calc", calcResultText.value);
    } catch (error) {
      setError(error);
    }
  }

  async function reverseFromScore() {
    await runReverse(false);
  }

  async function reverseAllFromScore() {
    await runReverse(true);
  }

  async function runReverse(includeAll) {
    if (!ensureChartSelected()) {
      return;
    }

    try {
      clearError();
      const request = includeAll ? requestReverseAllFromScore : requestReverseFromScore;
      const result = await request({
        song_id: selectedSongId.value,
        difficulty: selectedDifficulty.value,
        target_score: Math.max(0, toInt(targetScore.value)),
        ...createReverseFilterInput()
      });

      reverseResultText.value = formatReverseResult(result);
      applyJudgement(result.judgement);
      saveEntry(includeAll ? "reverse-all" : "reverse", reverseResultText.value);
    } catch (error) {
      setError(error);
    }
  }

  function saveEntry(type, resultText) {
    if (!onResult) {
      return;
    }

    const song = songs.value.find((s) => s.song_id === selectedSongId.value);
    onResult({
      type,
      songId: selectedSongId.value,
      songName: song?.song_name ?? selectedSongId.value,
      difficulty: selectedDifficulty.value,
      targetScore: type !== "calc" ? Math.max(0, toInt(targetScore.value)) : null,
      calcResult: type === "calc" ? resultText : null,
      reverseResult: type !== "calc" ? resultText : null
    });
  }

  function createScoreInput() {
    return {
      song_id: selectedSongId.value,
      difficulty: selectedDifficulty.value,
      non_slide_perfect_plus: toInt(judgement.non_slide_perfect_plus),
      non_slide_perfect: toInt(judgement.non_slide_perfect),
      non_slide_good: toInt(judgement.non_slide_good),
      non_slide_miss: toInt(judgement.non_slide_miss),
      slide_hit: toInt(judgement.slide_hit),
      slide_miss: toInt(judgement.slide_miss)
    };
  }

  function createReverseFilterInput() {
    return {
      allow_perfect_plus: Boolean(reverseFilter.allow_perfect_plus),
      allow_perfect: Boolean(reverseFilter.allow_perfect),
      allow_good: Boolean(reverseFilter.allow_good),
      allow_miss: Boolean(reverseFilter.allow_miss),
      min_played_ratio: 0.0
    };
  }

  function applyJudgement(nextJudgement = {}) {
    for (const field of Object.keys(createEmptyJudgement())) {
      updateJudgementField(field, nextJudgement[field]);
    }
  }

  function updateJudgementField(field, value) {
    if (!Object.hasOwn(judgement, field)) {
      return;
    }

    judgement[field] = toInt(value);
  }

  function updateReverseFilterField(field, value) {
    if (!Object.hasOwn(reverseFilter, field)) {
      return;
    }

    reverseFilter[field] = Boolean(value);
  }

  function clearJudgement() {
    applyJudgement(createEmptyJudgement());
  }

  function resetChartSelection() {
    selectedDifficulty.value = "";
    difficultyQuery.value = "";
    clearJudgement();
    clearResults();
  }

  function clearResults() {
    calcResultText.value = EMPTY_RESULT;
    reverseResultText.value = EMPTY_RESULT;
  }

  function ensureChartSelected() {
    if (selectedChart.value) {
      return true;
    }

    setError("请先选择歌曲和难度");
    return false;
  }

  function onSongFilter(query) {
    songQuery.value = query;
  }

  function onDifficultyFilter(query) {
    difficultyQuery.value = query;
  }

  function setError(error) {
    errorMessage.value = stringifyError(error);
  }

  function clearError() {
    errorMessage.value = "";
  }

  return {
    calcResultText,
    calculateScore,
    canOperate,
    difficulties,
    difficultyQuery,
    errorMessage,
    filteredDifficulties,
    filteredSongs,
    inputWarning,
    judgement,
    loadingDifficulties,
    loadingSongs,
    nonSlideLimit,
    nonSlideSum,
    onDifficultyFilter,
    onSongFilter,
    reverseFilter,
    reverseAllFromScore,
    reverseFromScore,
    reverseResultText,
    selectedChart,
    selectedDifficulty,
    selectedSongId,
    slideLimit,
    slideSum,
    songQuery,
    songs,
    targetScore,
    updateJudgementField,
    updateReverseFilterField
  };
}
