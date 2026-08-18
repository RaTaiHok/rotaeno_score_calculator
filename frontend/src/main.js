import { createApp } from "vue";
import { ElButton, ElCheckbox, ElInputNumber, ElOption, ElSelect } from "element-plus";
import "element-plus/es/components/button/style/css";
import "element-plus/es/components/checkbox/style/css";
import "element-plus/es/components/input-number/style/css";
import "element-plus/es/components/option/style/css";
import "element-plus/es/components/select/style/css";
import App from "./App.vue";
import "./style.css";

// Prevent pinch-zoom on mobile — viewport meta alone is unreliable in Android WebView
document.addEventListener(
  "touchmove",
  (e) => {
    if (e.touches.length > 1) {
      e.preventDefault();
    }
  },
  { passive: false }
);

// Also block gesture events (iOS)
document.addEventListener("gesturestart", (e) => e.preventDefault());
document.addEventListener("gesturechange", (e) => e.preventDefault());
document.addEventListener("gestureend", (e) => e.preventDefault());

// Dynamically measure safe-area-inset-top and set --status-bar-height CSS variable.
// On PC: env() returns 0 → no padding needed.
// On iOS (with viewport-fit=cover): env() returns the actual inset → use it.
// On Android WebView: env() may return 0 despite the status bar → fall back to 24dp * DPR.
function applyMobileAdaptations() {
  const isAndroid = /android/i.test(navigator.userAgent);
  const isIOS = /iphone|ipad|ipod/i.test(navigator.userAgent);

  // --- Status bar height ---
  const probe = document.createElement("div");
  probe.style.cssText =
    "position:fixed;top:0;left:0;height:env(safe-area-inset-top,0px);width:1px;pointer-events:none;";
  document.body.appendChild(probe);
  const envHeight = probe.getBoundingClientRect().height;
  probe.remove();

  if (envHeight > 0) {
    document.documentElement.style.setProperty("--status-bar-height", `${envHeight}px`);
  } else if (isAndroid) {
    const dpr = window.devicePixelRatio || 1;
    document.documentElement.style.setProperty("--status-bar-height", `${Math.round(24 * dpr)}px`);
  } else if (isIOS) {
    // env() 未生效时的兑底（如旧版 WebView / 缺失 viewport-fit=cover）。
    // 无刘海机型约 20pt，刘海机型约 47pt，取 44 折中（多余部分为深色 header 背景）。
    document.documentElement.style.setProperty("--status-bar-height", "44px");
  }
  // PC: leave --status-bar-height unset (defaults to 0px in CSS)

  // --- 90% scale on Android WebView ---
  // Android WebView ignores viewport meta initial-scale, so we use CSS zoom instead.
  // zoom is natively supported by Chromium (which powers Android WebView).
  if (isAndroid) {
    document.documentElement.style.zoom = "0.9";
  }

  // --- Fixed header offset ---
  // Measure actual header height and set body padding-top so content isn't hidden.
  measureHeaderHeight();
  window.addEventListener("resize", measureHeaderHeight);
}

function measureHeaderHeight() {
  const header = document.querySelector(".app-header");
  if (header) {
    const height = header.offsetHeight;
    document.documentElement.style.setProperty("--header-total-height", `${height}px`);
  }
}

document.addEventListener("DOMContentLoaded", applyMobileAdaptations);
// Also run now in case DOMContentLoaded already fired
if (document.readyState !== "loading") {
  applyMobileAdaptations();
}

createApp(App)
  .use(ElButton)
  .use(ElCheckbox)
  .use(ElInputNumber)
  .use(ElOption)
  .use(ElSelect)
  .mount("#app");
