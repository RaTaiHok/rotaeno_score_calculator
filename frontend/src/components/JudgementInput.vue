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
    <p class="muted">允许输入总和小于上限，未填部分可理解为未得分项</p>

    <div class="grid grid-three">
      <label class="field">
        <span>非Slide Perfect+ (P+)</span>
        <el-input-number
          :model-value="judgement.non_slide_perfect_plus"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_perfect_plus', value)"
        />
      </label>
      <label class="field">
        <span>非Slide Perfect (P)</span>
        <el-input-number
          :model-value="judgement.non_slide_perfect"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_perfect', value)"
        />
      </label>
      <label class="field">
        <span>非Slide Good (G)</span>
        <el-input-number
          :model-value="judgement.non_slide_good"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_good', value)"
        />
      </label>
      <label class="field">
        <span>非Slide Miss</span>
        <el-input-number
          :model-value="judgement.non_slide_miss"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'non_slide_miss', value)"
        />
      </label>
      <label class="field">
        <span>Slide Hit (P')</span>
        <el-input-number
          :model-value="judgement.slide_hit"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'slide_hit', value)"
        />
      </label>
      <label class="field">
        <span>Slide Miss</span>
        <el-input-number
          :model-value="judgement.slide_miss"
          :min="0"
          :size="componentSize"
          :disabled="!canOperate"
          @update:model-value="(value) => emit('update-field', 'slide_miss', value)"
        />
      </label>
    </div>

    <div class="limit-box">
      <p>非Slide上限: <strong>{{ selectedChart ? nonSlideLimit : "-" }}</strong></p>
      <p>Slide上限: <strong>{{ selectedChart ? slideLimit : "-" }}</strong></p>
      <p>
        当前输入: 非Slide {{ nonSlideSum }}/{{ selectedChart ? nonSlideLimit : "-" }}，
        Slide {{ slideSum }}/{{ selectedChart ? slideLimit : "-" }}
      </p>
    </div>

    <p v-if="inputWarning" class="warning">{{ inputWarning }}</p>
  </section>
</template>
