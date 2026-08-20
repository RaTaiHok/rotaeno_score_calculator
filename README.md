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

一个基于 Tauri 2、Vue 3 和 Rust 的 Rotaeno 分数计算器，支持 PC、Android 和 iOS。

## 功能

- 歌曲搜索与难度选择
- 谱面物量信息展示（总物量 / Slide / 非Slide / 有效物量 / 判定基本分）
- 正向分数计算（输入判定 → 显示分数）
- 目标分数反算（输入目标分数 → 可能的判定分布）
  - 前三方案 / 展示全部 两种模式
  - 可筛选允许判定：Perfect+ / Perfect / Good / Miss/未判定
  - 反算时显示计算进度
  - 同一分数下 Miss 的多种分配可能，全部列出供参考
- 反算结果一键复制（电脑、手机都支持），也可直接选中文本复制
- 计算历史记录（本地存储，可展开查看详情）
- 谱面数据自动更新（联网时自动检测，无需手动操作）

## 支持平台

| 平台 | 架构 |
|------|------|
| Windows | x64 / x86 / ARM64 |
| Android | ARM64 |
| iOS | 通用 |
| macOS | Apple Silicon |
| Linux | x64 |

## 下载

从 [GitHub Actions](https://github.com/RaTaiHok/rotaeno_score_calculator/actions) 的构建产物中下载：

- `Rotaeno_Score_Calc-All.zip` — 全平台汇总包
- `Rotaeno_Score_Calc-Windows64x.zip` / `-Windows32x.zip` / `-Windowsarm64.zip`
- `Rotaeno_Score_Calc-Android.zip`
- `Rotaeno_Score_Calc-iOS.zip`
- `Rotaeno_Score_Calc-Mac.zip`
- `Rotaeno_Score_Calc-Linux.zip`

> 从 zip 中选择对应自己设备的文件进行安装。

## 使用说明

- **Windows / macOS / Linux**：解压后直接运行
- **Android**：安装 APK 即可（新版可直接覆盖安装旧版）
- **iOS**：无签名 IPA，需自行使用 AltStore / SideStore / TrollStore 等工具签名安装

## License

MIT
