# Rotaeno 分数计算器

一个基于 Tauri 2、Vue 3 和 Rust 的 Rotaeno 分数计算器桌面/移动应用

项目使用 `data/all_song_note_stats.json` 中的谱面物量数据，根据 Rotaeno 的计分规则计算分数，并提供目标分数反算功能

## 功能

- 按歌曲名搜索并选择歌曲
- 根据歌曲数据选择可用难度
- 展示谱面总物量、Slide 数量、非 Slide 数量、有效物量和判定基本分
- 输入非 Slide 判定和 Slide 判定后正向计算显示分数
- 根据目标分数反算可能的判定分布
- 反算支持默认前三方案和展示全部方案
- 反算支持筛选允许判定：`Perfect+`、`Perfect`、`Good`、`Miss/未判定`
- 适配中途分数场景：`Miss/未判定` 可以理解为尚未得分的 note

## 计分规则

Slide 音符权重为 `1`，其余音符权重为 `4`

设：

- `S` 为 Slide 数量
- `O` 为非 Slide 数量
- `P+` 为非 Slide Perfect+ 数量
- `P` 为非 Slide Perfect 数量
- `G` 为非 Slide Good 数量
- `SlideHit` 为 Slide 命中数量

判定基本分：

```text
x = 1,000,000 / (0.25S + O)
```

总分：

```text
score = x * (1.01 * P+ + P + 0.2525 * (G + SlideHit))
```

游戏内显示分数为向下取整后的整数

后端反算使用整数形式避免浮点误差：

```text
display_score = floor(10000 * (404 * P+ + 400 * P + 101 * (G + SlideHit)) / (4O + S))
```

## 项目结构

```text
.
├── Cargo.toml
├── Cargo.lock
├── README.md
├── BUILD.md
├── data/
│   └── all_song_note_stats.json
├── frontend/
│   ├── index.html
│   ├── package.json
│   ├── vite.config.js
│   └── src/
│       ├── App.vue
│       ├── main.js
│       ├── style.css
│       ├── api/
│       │   └── scoreApi.js
│       ├── components/
│       │   ├── ChartSelector.vue
│       │   ├── JudgementInput.vue
│       │   └── ScoreActions.vue
│       ├── composables/
│       │   └── useScoreCalculator.js
│       └── utils/
│           ├── error.js
│           ├── number.js
│           ├── resultFormat.js
│           └── text.js
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json
    ├── icons/
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── app.rs
        ├── calc.rs
        ├── model.rs
        └── calc/
            ├── math.rs
            └── reverse.rs
```

## 主要模块

- `frontend/src/App.vue`：页面组合入口
- `frontend/src/components/ChartSelector.vue`：歌曲和难度选择
- `frontend/src/components/JudgementInput.vue`：判定输入
- `frontend/src/components/ScoreActions.vue`：计算、反算和结果展示
- `frontend/src/composables/useScoreCalculator.js`：前端状态和业务动作
- `frontend/src/api/scoreApi.js`：Tauri 命令调用封装
- `src-tauri/src/app.rs`：Tauri 命令、谱面数据加载和查询
- `src-tauri/src/model.rs`：数据结构和前后端传输模型
- `src-tauri/src/calc.rs`：计分模块入口
- `src-tauri/src/calc/math.rs`：精确整数计分公式
- `src-tauri/src/calc/reverse.rs`：目标分数反算

## 数据文件

谱面数据位于：

```text
data/all_song_note_stats.json
```

结构大致为：

```json
{
  "songs": [
    {
      "song_id": "example",
      "song_name": "Example Song",
      "difficulties": {
        "IV": {
          "tap": 0,
          "flick": 0,
          "slide": 0,
          "catch": 0,
          "rotate": 0,
          "total": 0
        }
      },
      "song_total": 0
    }
  ]
}
```

应用启动时会通过 Rust 的 `include_str!` 将该 JSON 编译进程序

## 构建和运行

请看 [BUILD.md](./BUILD.md)
