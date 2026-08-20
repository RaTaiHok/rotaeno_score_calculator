<p align="center">
  <img src="frontend/public/app-icon-128.png" width="80" height="80" alt="Rotaeno 分数计算器" style="border-radius: 16px" />
</p>
<p align="center"><em>选了一只笨鸟做图标（雾）</em></p>

<h1 align="center">Rotaeno 分数计算器</h1>

<p align="center">
  <strong>App v0.3.0</strong> &nbsp;|&nbsp;
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

App 启动时自动从服务器下载谱面数据并**逐字节比对本地缓存**，内容不同才更新。
服务器只维护一个固定文件，内置数据仅作为全新安装且无法联网时的离线兜底，**数据更新无需重新构建/发布 App**。

```text
https://rth.srv-selena.lookatthesky.cn/Rotaeno/data/
└── all_song_note_stats.json   ← 唯一需要维护的文件
```

### 更新数据的步骤（服务器端）

1. 用新的 `all_song_note_stats.json` **覆盖上传**到服务器
2. 完成 —— App 下次启动会自动检测并下载

> 若服务器支持 ETag（nginx / 对象存储 / CDN 通常支持），App 会自动带上 `If-None-Match` 头，
> 数据未变化时命中 304，几乎不消耗流量。

> 旧方案（`latest_version.txt` + `all_song_note_stats_{version}.json`）已废弃，可删除。

## License

MIT
