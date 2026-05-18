<script setup>
const targetScore = defineModel("targetScore", { default: 1_000_000 });

defineProps({
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
  reverseResultText: {
    type: String,
    default: "-"
  }
});

const emit = defineEmits(["calculate", "reverse", "reverse-all", "update-filter"]);
</script>

<template>
  <section class="panel">
    <h2>3. 计算 / 反算</h2>
    <div class="action-row">
      <el-button type="primary" :disabled="!canOperate" @click="emit('calculate')">
        计算分数
      </el-button>
      <div class="target-box">
        <span>目标分数（反算）</span>
        <el-input-number v-model="targetScore" :min="0" :disabled="!canOperate" />
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
        <small>只选 Perfect+ 时，会自动允许 Miss/未判定用于表示未打到的 note。</small>
      </div>
      <el-button :disabled="!canOperate" @click="emit('reverse')">
        反算前三方案
      </el-button>
      <el-button :disabled="!canOperate" @click="emit('reverse-all')">
        展示全部
      </el-button>
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
</template>
