<p align="center">
  <img src="frontend/public/app-icon-128.png" width="80" height="80" alt="Rotaeno 分数计算器" style="border-radius: 16px" />
</p>
<p align="center"><em>选了一只笨鸟做图标（雾）</em></p>

<h1 align="center">Rotaeno 分数计算器</h1>

<p align="center">
  <strong>App v0.1.0</strong> &nbsp;|&nbsp;
  <strong>作者</strong> RTHsama &nbsp;|&nbsp;
  <a href="https://github.com/RaTaiHok/rotaeno_score_calculator">GitHub</a>
</p>

---

一个基于 Tauri 2、Vue 3 和 Rust 的 Rotaeno 分数计算器，支持 PC 和 Android

## 功能

- 歌曲搜索与难度选择
- 谱面物量信息展示（总物量 / Slide / 非Slide / 有效物量 / 判定基本分）
- 正向分数计算（输入判定 → 显示分数）
- 目标分数反算（输入目标分数 → 可能的判定分布）
  - 前三方案 / 展示全部 两种模式
  - 可筛选允许判定：Perfect+ / Perfect / Good / Miss/未判定
- 计算历史记录（本地存储，可展开查看详情）
- 谱面数据在线更新（首次启动自动下载，后续检测版本更新）

## 数据

谱面数据首次启动时从服务器下载最新版本，后续自动检测更新。内置 v2.24.0 作为离线备用

```text
https://rth.srv-selena.lookatthesky.cn/Rotaeno/data/
├── latest_version.txt
└── all_song_note_stats_{version}.json
```

## License

MIT
