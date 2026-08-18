<script setup>
import { openUrl } from "@tauri-apps/plugin-opener";

defineEmits(["toggle-history", "toggle-about"]);

const GITHUB_URL = "https://github.com/RaTaiHok/rotaeno_score_calculator";

function openGithub() {
  // Tauri 环境：用系统默认浏览器打开；浏览器调试环境回退到 window.open
  openUrl(GITHUB_URL).catch(() => {
    window.open(GITHUB_URL, "_blank", "noopener,noreferrer");
  });
}
</script>

<template>
  <header class="app-header">
    <div class="app-header__inner">
      <!-- Brand -->
      <div class="app-header__brand">
        <img
          class="app-header__logo"
          src="/app-icon.png"
          width="28"
          height="28"
          alt="App Icon"
        />
        <div class="app-header__titles">
          <span class="app-header__title">Rotaeno 分数计算器</span>
          <span class="app-header__version">v0.1.0</span>
        </div>
      </div>

      <!-- Actions -->
      <div class="app-header__actions">
        <button
          class="app-header__icon-btn"
          title="关于"
          @click="$emit('toggle-about')"
        >
          <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z" />
          </svg>
        </button>
        <button
          class="app-header__icon-btn"
          title="计算历史"
          @click="$emit('toggle-history')"
        >
          <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 2C6.486 2 2 6.486 2 12s4.486 10 10 10 10-4.486 10-10S17.514 2 12 2zm0 18c-4.411 0-8-3.589-8-8s3.589-8 8-8 8 3.589 8 8-3.589 8-8 8z" />
            <path d="M13 7h-2v5.414l3.293 3.293 1.414-1.414L13 11.586z" />
          </svg>
        </button>
        <button
          class="app-header__icon-btn"
          title="GitHub"
          @click="openGithub"
        >
          <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
/* styles are in style.css for global breakpoint control */
</style>
