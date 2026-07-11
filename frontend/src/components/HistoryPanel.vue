<script setup>
import { computed, ref } from "vue";

const TYPE_LABELS = {
  calc: "正算",
  reverse: "反算",
  "reverse-all": "反算(全部)"
};

const props = defineProps({
  history: {
    type: Array,
    default: () => []
  },
  visible: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(["close", "clear", "remove"]);

const sortedHistory = computed(() => [...props.history].sort((a, b) => b.timestamp - a.timestamp));

const expandedId = ref(null);

function toggleExpand(id) {
  expandedId.value = expandedId.value === id ? null : id;
}

function formatTime(ts) {
  const d = new Date(ts);
  const pad = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function typeBadge(type) {
  return TYPE_LABELS[type] || type;
}

function summaryText(entry) {
  if (entry.type === "calc" && entry.calcResult) {
    // Extract just the display score line
    const match = entry.calcResult.match(/显示分数:\s*([\d,]+)/);
    return match ? `得分: ${match[1]}` : entry.calcResult.split("\n")[0];
  }
  if (entry.type !== "calc" && entry.targetScore) {
    return `目标分数: ${entry.targetScore.toLocaleString()}`;
  }
  return "";
}

function detailText(entry) {
  if (entry.type === "calc") {
    return entry.calcResult || "";
  }
  return entry.reverseResult || "";
}
</script>

<template>
  <Transition name="drawer">
    <div v-if="visible" class="history-overlay" @click.self="emit('close')">
      <div class="history-drawer">
        <div class="history-drawer__header">
          <h3>计算历史</h3>
          <div class="history-drawer__header-actions">
            <button
              v-if="history.length > 0"
              class="history-drawer__clear-btn"
              @click="emit('clear')"
            >
              清空
            </button>
            <button class="history-drawer__close-btn" @click="emit('close')">
              ✕
            </button>
          </div>
        </div>

        <div class="history-drawer__body">
          <div v-if="history.length === 0" class="history-empty">
            暂无计算记录
          </div>

          <div
            v-for="entry in sortedHistory"
            :key="entry.id"
            class="history-entry"
            :class="{ 'history-entry--expanded': expandedId === entry.id }"
            @click="toggleExpand(entry.id)"
          >
            <div class="history-entry__header">
              <span class="history-entry__type" :class="'history-entry__type--' + entry.type">{{ typeBadge(entry.type) }}</span>
              <span class="history-entry__song">{{ entry.songName }}</span>
              <span class="history-entry__diff">{{ entry.difficulty }}</span>
              <span class="history-entry__time">{{ formatTime(entry.timestamp) }}</span>
              <span class="history-entry__expand-icon">{{ expandedId === entry.id ? '▾' : '▸' }}</span>
            </div>

            <div class="history-entry__summary">
              {{ summaryText(entry) }}
            </div>

            <div v-if="expandedId === entry.id" class="history-entry__detail">
              <pre>{{ detailText(entry) }}</pre>
            </div>

            <button
              class="history-entry__remove"
              @click.stop="emit('remove', entry.id)"
              title="删除此条"
            >
              删除
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
/* styles are in style.css for global breakpoint control */
</style>
