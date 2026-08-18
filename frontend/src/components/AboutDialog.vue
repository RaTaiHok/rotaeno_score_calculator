<script setup>
defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  dataVersion: {
    type: String,
    default: ""
  }
});

defineEmits(["close", "reset-data"]);
</script>

<template>
  <Transition name="modal">
    <div v-if="visible" class="about-overlay" @click.self="$emit('close')">
      <div class="about-dialog">
        <button class="about-dialog__close" @click="$emit('close')">✕</button>

        <img
          class="about-dialog__icon"
          src="/app-icon-128.png"
          width="80"
          height="80"
          alt="App Icon"
        />
        <p class="about-dialog__icon-caption">选了一只笨鸟做图标（雾）</p>

        <h2 class="about-dialog__name">Rotaeno 分数计算器</h2>

        <div class="about-dialog__info">
          <div class="about-dialog__row">
            <span class="about-dialog__label">App 版本</span>
            <span class="about-dialog__value">v0.2.0</span>
          </div>
          <div class="about-dialog__row">
            <span class="about-dialog__label">Rotaeno Data</span>
            <span class="about-dialog__value">{{ dataVersion || '-' }}</span>
          </div>
          <div class="about-dialog__row">
            <span class="about-dialog__label">作者</span>
            <span class="about-dialog__value">RTHsama</span>
          </div>
        </div>

        <button
          class="about-dialog__reset-btn"
          title="清除本地数据，恢复内置版本"
          @click="$emit('reset-data')"
        >
          重置谱面数据
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
/* styles are in style.css */
</style>
