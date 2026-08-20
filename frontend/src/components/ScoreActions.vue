<script setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import ReverseResultView from "./ReverseResultView.vue";
import CalcResultView from "./CalcResultView.vue";

const targetScore = defineModel("targetScore", { default: 1_000_000 });

const props = defineProps({
  calcResult: {
    type: Object,
    default: null
  },
  calcResultText: {
    type: String,
    default: "-"
  },
  canOperate: {
    type: Boolean,
    default: false
  },
  errorMessage: {
    type: String,
    default: ""
  },
  reverseFilter: {
    type: Object,
    required: true
  },
  reverseResult: {
    type: Object,
    default: null
  },
  reverseResultText: {
    type: String,
    default: "-"
  }
});

const emit = defineEmits(["calculate", "reverse", "reverse-all", "update-filter", "apply-candidate"]);

const windowWidth = ref(window.innerWidth);
const onResize = () => { windowWidth.value = window.innerWidth; };
const componentSize = computed(() => windowWidth.value <= 480 ? "large" : "default");
onMounted(() => window.addEventListener("resize", onResize));
onUnmounted(() => window.removeEventListener("resize", onResize));
</script>

<template>
  <section class="panel">
    <h2>3. 计算 / 反算</h2>

    <!-- ===== Forward Calculation ===== -->
    <div class="calc-section">
      <div class="calc-row">
        <el-button type="primary" :size="componentSize" :disabled="!canOperate" @click="emit('calculate')">计算分数</el-button>
      </div>
      <div v-if="calcResult" class="result">
        <div class="result-header">
          <h3>计算结果</h3>
        </div>
        <CalcResultView :result="calcResult" />
      </div>
    </div>

    <hr class="section-divider" />

    <!-- ===== Reverse Calculation ===== -->
    <div class="reverse-section">
      <div class="action-row">
        <div class="target-box">
          <span>目标分数（反算）</span>
          <el-input-number v-model="targetScore" :min="0" :size="componentSize" :disabled="!canOperate" />
        </div>
        <div class="reverse-filter">
          <span>反算允许判定</span>
          <div class="filter-options">
            <el-checkbox
              :model-value="reverseFilter.allow_perfect_plus"
              :disabled="!canOperate"
              @update:model-value="(value) => emit('update-filter', 'allow_perfect_plus', value)"
            >
              Perfect+
            </el-checkbox>
            <el-checkbox
              :model-value="reverseFilter.allow_perfect"
              :disabled="!canOperate"
              @update:model-value="(value) => emit('update-filter', 'allow_perfect', value)"
            >
              Perfect
            </el-checkbox>
            <el-checkbox
              :model-value="reverseFilter.allow_good"
              :disabled="!canOperate"
              @update:model-value="(value) => emit('update-filter', 'allow_good', value)"
            >
              Good
            </el-checkbox>
            <el-checkbox
              :model-value="reverseFilter.allow_miss"
              :disabled="!canOperate"
              @update:model-value="(value) => emit('update-filter', 'allow_miss', value)"
            >
              Miss/未判定
            </el-checkbox>
          </div>
          <small>只选 Perfect+ 时，会自动允许 Miss/未判定用于表示未打到的 note</small>
        </div>
      </div>
      <div class="reverse-actions">
        <el-button :size="componentSize" :disabled="!canOperate" @click="emit('reverse')">反算前三方案</el-button>
        <el-button :size="componentSize" :disabled="!canOperate" @click="emit('reverse-all')">展示全部</el-button>
      </div>
      <div v-if="reverseResult" class="result">
        <div class="result-header">
          <h3>反算结果</h3>
        </div>
        <ReverseResultView
          :result="reverseResult"
          @apply-candidate="emit('apply-candidate', $event)"
        />
      </div>
    </div>

    <p v-if="errorMessage" class="error">{{ errorMessage }}</p>
  </section>
</template>
