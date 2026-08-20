<script setup>
import { computed, onMounted, onUnmounted, ref } from "vue";

defineProps({
  canOperate: {
    type: Boolean,
    default: false
  },
  inputWarning: {
    type: String,
    default: ""
  },
  judgement: {
    type: Object,
    required: true
  },
  nonSlideLimit: {
    type: Number,
    default: 0
  },
  nonSlideSum: {
    type: Number,
    default: 0
  },
  selectedChart: {
    type: Object,
    default: null
  },
  slideLimit: {
    type: Number,
    default: 0
  },
  slideSum: {
    type: Number,
    default: 0
  }
});

const emit = defineEmits(["update-field"]);

const windowWidth = ref(window.innerWidth);
const onResize = () => { windowWidth.value = window.innerWidth; };
const componentSize = computed(() => windowWidth.value <= 480 ? "large" : "default");
onMounted(() => window.addEventListener("resize", onResize));
onUnmounted(() => window.removeEventListener("resize", onResize));
</script>

<template>
  <section class="panel">
    <h2>2. 输入判定</h2>

    <div class="grid grid-three judge-grid">
      <label class="field judge-field">
        <span class="field-label">Perfect+</span>
        <el-input-number
          :controls="false"
          :model-value="judgement.non_slide_perfect_plus"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_perfect_plus', value)"
        />
      </label>
      <label class="field judge-field">
        <span class="field-label">Perfect</span>
        <el-input-number
          :controls="false"
          :model-value="judgement.non_slide_perfect"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_perfect', value)"
        />
      </label>
      <label class="field judge-field">
        <span class="field-label">Good</span>
        <el-input-number
          :controls="false"
          :model-value="judgement.non_slide_good"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_good', value)"
        />
      </label>
      <label class="field judge-field">
        <span class="field-label">Miss</span>
        <el-input-number
          :controls="false"
          :model-value="judgement.non_slide_miss"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_miss', value)"
        />
      </label>
      <label class="field judge-field">
        <span class="field-label">Slide Miss</span>
        <el-input-number
          :controls="false"
          :model-value="judgement.slide_miss"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'slide_miss', value)"
        />
      </label>
    </div>

    <div class="limit-box">
      <p>非Slide: <strong>{{ nonSlideSum }}/{{ selectedChart ? nonSlideLimit : "-" }}</strong></p>
      <p>Slide Miss: <strong>{{ slideSum }}/{{ selectedChart ? slideLimit : "-" }}</strong>（Hit 自动 = 上限 − Miss）</p>
      <p v-if="selectedChart">
        未分配: 非Slide {{ Math.max(0, nonSlideLimit - nonSlideSum) }}
      </p>
    </div>

    <p v-if="inputWarning" class="warning">{{ inputWarning }}</p>
  </section>
</template>
