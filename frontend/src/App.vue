<script setup>
import { onMounted, ref } from "vue";
import { checkDataUpdate, downloadLatestData, getDataStatus, resetData } from "./api/scoreApi";
import AboutDialog from "./components/AboutDialog.vue";
import AppHeader from "./components/AppHeader.vue";
import ChartSelector from "./components/ChartSelector.vue";
import HistoryPanel from "./components/HistoryPanel.vue";
import JudgementInput from "./components/JudgementInput.vue";
import ScoreActions from "./components/ScoreActions.vue";
import { useHistory } from "./composables/useHistory";
import { useScoreCalculator } from "./composables/useScoreCalculator";

const { history, addEntry, removeEntry, clearHistory } = useHistory();
const showHistory = ref(false);
const showAbout = ref(false);

// --- Data update ---
const updateInfo = ref(null);
const showUpdateDialog = ref(false);
const updateDialogMode = ref("update"); // "first-launch" | "update" | "first-launch-retry"
const updateLoading = ref(false);
const updateError = ref("");
const dataVersion = ref(""); // for About dialog

async function initData() {
  // Get current data status
  try {
    const status = await getDataStatus();
    dataVersion.value = status.version;

    if (status.is_bundled) {
      // First launch — must download data from server
      updateDialogMode.value = "first-launch";
      updateInfo.value = { local_version: status.version, remote_version: "" };
      showUpdateDialog.value = true;
      // Try to auto-download
      await tryDownload();
      return;
    }

    // Subsequent launch — check for updates
    const info = await checkDataUpdate();
    if (info.has_update) {
      updateInfo.value = info;
      updateDialogMode.value = "update";
      showUpdateDialog.value = true;
    } else if (info.error) {
      // Server check failed — show the error so user knows what's wrong
      updateInfo.value = info;
      updateError.value = info.error;
      updateDialogMode.value = "update";
      showUpdateDialog.value = true;
    }
  } catch (e) {
    // Truly offline — no network at all, silently use local data
    console.log("Update check skipped (offline):", e);
  }
}

async function tryDownload() {
  updateLoading.value = true;
  updateError.value = "";
  try {
    const newVersion = await downloadLatestData();
    dataVersion.value = newVersion;
    showUpdateDialog.value = false;
  } catch (e) {
    updateError.value = typeof e === "string" ? e : "无法连接服务器，请检查网络后重试";
    if (updateDialogMode.value === "first-launch") {
      updateDialogMode.value = "first-launch-retry";
    }
  } finally {
    updateLoading.value = false;
  }
}

function handleSkipUpdate() {
  showUpdateDialog.value = false;
}

async function handleResetData() {
  try {
    await resetData();
    // Full reload to clear all cached state and restart the init flow
    window.location.reload();
  } catch (e) {
    console.error("Reset failed:", e);
  }
}

onMounted(initData);

const {
  applyJudgement,
  calcResult,
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
  reverseLoading,
  reverseProgress,
  reverseResult,
  reverseResultText,
  selectedChart,
  selectedDifficulty,
  selectedSongId,
  slideLimit,
  slideSum,
  targetScore,
  updateJudgementField,
  updateReverseFilterField
} = useScoreCalculator(addEntry);

function toggleHistory() {
  showHistory.value = !showHistory.value;
}

function handleClear() {
  clearHistory();
}
</script>

<template>
  <AppHeader @toggle-history="toggleHistory" @toggle-about="showAbout = !showAbout" />

  <main class="container">
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
      :calc-result="calcResult"
      :calc-result-text="calcResultText"
      :can-operate="canOperate"
      :error-message="errorMessage"
      :reverse-filter="reverseFilter"
      :reverse-result="reverseResult"
      :reverse-result-text="reverseResultText"
      @calculate="calculateScore"
      @reverse="reverseFromScore"
      @reverse-all="reverseAllFromScore"
      @update-filter="updateReverseFilterField"
      @apply-candidate="applyJudgement"
    />
  </main>

  <HistoryPanel
    :visible="showHistory"
    :history="history"
    @close="showHistory = false"
    @clear="handleClear"
    @remove="removeEntry"
  />

  <!-- 反算计算中遮罩：转圈 + 进度百分比 -->
  <Transition name="modal">
    <div v-if="reverseLoading" class="reverse-overlay">
      <div class="reverse-spinner"></div>
      <div class="reverse-text">正在反算...</div>
      <div class="reverse-progress-track">
        <div class="reverse-progress-fill" :style="{ width: reverseProgress + '%' }"></div>
      </div>
      <div class="reverse-percent">{{ reverseProgress }}%</div>
    </div>
  </Transition>

  <AboutDialog
    :visible="showAbout"
    :data-version="dataVersion"
    @close="showAbout = false"
    @reset-data="handleResetData"
  />

  <!-- Data Update / First Launch Dialog -->
  <Transition name="modal">
    <div v-if="showUpdateDialog" class="about-overlay">
      <div class="about-dialog">
        <!-- First launch: must download -->
        <template v-if="updateDialogMode === 'first-launch'">
          <h3 style="margin:0 0 8px">正在下载谱面数据...</h3>
          <p style="margin:0;font-size:13px;color:#8a97ae">首次使用需从服务器获取最新数据</p>
        </template>

        <!-- First launch retry: download failed -->
        <template v-else-if="updateDialogMode === 'first-launch-retry'">
          <h3 style="margin:0 0 8px">需要联网下载数据</h3>
          <p style="margin:0 0 12px;font-size:13px;color:#b45309">{{ updateError }}</p>
          <p style="margin:0 0 16px;font-size:13px;color:#51607a">
            应用首次启动需要联网下载谱面数据后才能使用
          </p>
          <div style="display:flex;gap:10px;justify-content:center">
            <el-button type="primary" :loading="updateLoading" @click="tryDownload">
              {{ updateLoading ? '下载中...' : '重试' }}
            </el-button>
          </div>
        </template>

        <!-- Update available -->
        <template v-else-if="updateInfo?.has_update">
          <h3 style="margin:0 0 12px">谱面数据更新可用</h3>
          <p style="margin:0 0 8px;font-size:14px;color:#51607a">
            本地版本: <strong>{{ updateInfo?.local_version }}</strong>
          </p>
          <p style="margin:0 0 8px;font-size:14px;color:#51607a">
            最新版本: <strong>{{ updateInfo?.remote_version }}</strong>
          </p>
          <p v-if="updateError" style="margin:0 0 12px;font-size:13px;color:#b45309">{{ updateError }}</p>
          <div style="display:flex;gap:10px;justify-content:center">
            <el-button :disabled="updateLoading" @click="handleSkipUpdate">暂不更新</el-button>
            <el-button type="primary" :loading="updateLoading" @click="tryDownload">
              {{ updateLoading ? '下载中...' : '立即更新' }}
            </el-button>
          </div>
        </template>

        <!-- Server unreachable or misconfigured -->
        <template v-else>
          <h3 style="margin:0 0 12px">无法检查更新</h3>
          <p style="margin:0 0 16px;font-size:13px;color:#b45309;text-align:left;word-break:break-all">
            {{ updateError }}
          </p>
          <p style="margin:0 0 16px;font-size:13px;color:#51607a">
            当前使用本地版本 <strong>{{ updateInfo?.local_version }}</strong>，你可以继续使用或稍后重试
          </p>
          <div style="display:flex;gap:10px;justify-content:center">
            <el-button @click="handleSkipUpdate">继续使用</el-button>
            <el-button type="primary" :loading="updateLoading" @click="tryDownload">重试</el-button>
          </div>
        </template>
      </div>
    </div>
  </Transition>
</template>
