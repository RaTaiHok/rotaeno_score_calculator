<script setup>
const selectedSongId = defineModel("selectedSongId", { default: "" });
const selectedDifficulty = defineModel("selectedDifficulty", { default: "" });

defineProps({
  filteredSongs: {
    type: Array,
    default: () => []
  },
  filteredDifficulties: {
    type: Array,
    default: () => []
  },
  loadingSongs: {
    type: Boolean,
    default: false
  },
  loadingDifficulties: {
    type: Boolean,
    default: false
  },
  selectedChart: {
    type: Object,
    default: null
  }
});

const emit = defineEmits(["song-filter", "difficulty-filter"]);
</script>

<template>
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
          :filter-method="(query) => emit('song-filter', query)"
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
          :filter-method="(query) => emit('difficulty-filter', query)"
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
      <p>总物量: <strong>{{ selectedChart?.stats.total ?? "-" }}</strong></p>
      <p>Slide 数量 (S): <strong>{{ selectedChart?.stats.slide ?? "-" }}</strong></p>
      <p>非Slide数量 (O): <strong>{{ selectedChart?.non_slide_total ?? "-" }}</strong></p>
      <p>
        有效物量 (0.25S + O):
        <strong>{{ selectedChart ? Number(selectedChart.effective_notes).toFixed(4) : "-" }}</strong>
      </p>
      <p>
        判定基本分 x:
        <strong>{{ selectedChart ? Number(selectedChart.base_score).toFixed(4) : "-" }}</strong>
      </p>
    </div>
  </section>
</template>
