<script setup>
import ChartSelector from "./components/ChartSelector.vue";
import JudgementInput from "./components/JudgementInput.vue";
import ScoreActions from "./components/ScoreActions.vue";
import { useScoreCalculator } from "./composables/useScoreCalculator";

const {
  calcResultText,
  calculateScore,
  canOperate,
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
  targetScore,
  updateJudgementField,
  updateReverseFilterField
} = useScoreCalculator();
</script>

<template>
  <main class="container">
    <h1>Rotaeno 分数计算器</h1>

    <ChartSelector
      v-model:selected-song-id="selectedSongId"
      v-model:selected-difficulty="selectedDifficulty"
      :filtered-songs="filteredSongs"
      :filtered-difficulties="filteredDifficulties"
      :loading-songs="loadingSongs"
      :loading-difficulties="loadingDifficulties"
      :selected-chart="selectedChart"
      @song-filter="onSongFilter"
      @difficulty-filter="onDifficultyFilter"
    />

    <JudgementInput
      :judgement="judgement"
      :can-operate="canOperate"
      :input-warning="inputWarning"
      :non-slide-limit="nonSlideLimit"
      :non-slide-sum="nonSlideSum"
      :selected-chart="selectedChart"
      :slide-limit="slideLimit"
      :slide-sum="slideSum"
      @update-field="updateJudgementField"
    />

    <ScoreActions
      v-model:target-score="targetScore"
      :calc-result-text="calcResultText"
      :can-operate="canOperate"
      :error-message="errorMessage"
      :reverse-filter="reverseFilter"
      :reverse-result-text="reverseResultText"
      @calculate="calculateScore"
      @reverse="reverseFromScore"
      @reverse-all="reverseAllFromScore"
      @update-filter="updateReverseFilterField"
    />
  </main>
</template>
