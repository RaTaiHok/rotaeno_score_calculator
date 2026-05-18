# 构建和运行

本文档说明本项目在 PC 和 Android 上的开发、运行与打包方式

## 环境要求

基础环境：

- Node.js
- npm
- Rust stable
- Tauri CLI 2
- 对应平台的 WebView 运行环境

安装 Tauri CLI：

```powershell
cargo install tauri-cli
```

安装前端依赖：

```powershell
npm --prefix frontend install
```

检查 Rust 工程：

```powershell
cargo check -p rotaeno_score_calc
```

检查前端构建：

```powershell
npm --prefix frontend run build
```

## PC 开发运行

推荐使用 Tauri CLI：

```powershell
cargo tauri dev
```

该命令会读取 `src-tauri/tauri.conf.json`，并执行：

```json
"beforeDevCommand": "npm --prefix ../frontend run dev"
```

因此 Vite 前端服务会被自动启动

不要直接用下面这个命令作为日常开发入口：

```powershell
cargo run
```

`cargo run` 只会启动 Rust/Tauri 程序，不会自动启动 Vite

如果直接运行它，窗口可能会访问 `http://127.0.0.1:5173`，但前端服务不存在，从而出现：

```text
ERR_CONNECTION_REFUSED
```

如果确实要用 `cargo run`，需要两个终端：

终端 1：

```powershell
npm --prefix frontend run dev
```

终端 2：

```powershell
cargo run -p rotaeno_score_calc
```

## PC 打包

执行：

```powershell
cargo tauri build
```

该命令会自动执行：

```json
"beforeBuildCommand": "npm --prefix ../frontend run build"
```

构建产物通常位于：

```text
src-tauri/target/release/
src-tauri/target/release/bundle/
```

实际文件类型取决于当前操作系统和 Tauri bundle 配置

## Android 环境要求

Android 构建需要额外安装：

- Android Studio
- Android SDK
- Android SDK Command-line Tools
- Android SDK Platform Tools
- Android SDK Build Tools
- JDK
- Android NDK
- Rust Android targets

常见 Rust Android targets：

```powershell
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

确保环境变量能被终端读取，例如：

```powershell
$env:ANDROID_HOME
$env:ANDROID_NDK_HOME
$env:JAVA_HOME
```

## Android 初始化

首次为项目启用 Android：

```powershell
cargo tauri android init
```

该命令会生成 Android 工程文件，通常位于：

```text
src-tauri/gen/android/
```

如果已经初始化过，不要反复执行，避免覆盖本地 Android 配置

## Android 开发运行

连接手机或启动模拟器后，查看设备：

```powershell
adb devices
```

开发模式运行：

```powershell
cargo tauri android dev
```

如果需要打开 Android Studio：

```powershell
cargo tauri android dev --open
```

## Android 生产运行

构建前端并以生产模式运行到设备：

```powershell
cargo tauri android run
```

Release 模式：

```powershell
cargo tauri android run --release
```

## Android 打包 APK/AAB

Debug 包：

```powershell
cargo tauri android build --debug
```

Release 包：

```powershell
cargo tauri android build
```

Tauri 2 的 Android CLI 支持 `init`、`dev`、`run` 和 `build` 子命令

`android build` 会构建 release 包并生成 APK/AAB

常见输出位置在：

```text
src-tauri/gen/android/app/build/outputs/apk/
src-tauri/gen/android/app/build/outputs/bundle/
```

具体路径可能因 Gradle、构建类型和 ABI 配置不同而变化

## Android 签名

如果需要发布 APK/AAB，需要配置 Android 签名

生成 keystore 示例：

```powershell
keytool -genkey -v -keystore $env:USERPROFILE\upload-keystore.jks -storetype JKS -keyalg RSA -keysize 2048 -validity 10000 -alias upload
```

之后需要在 Android Gradle 配置中引用该 keystore

相关文件通常位于：

```text
src-tauri/gen/android/app/build.gradle.kts
```

签名文件和密码不要提交到仓库

## 数据打包说明

谱面数据位于：

```text
data/all_song_note_stats.json
```

当前后端通过：

```rust
include_str!("../../data/all_song_note_stats.json")
```

将 JSON 编译进程序

因此 PC 和 Android 构建都会包含这份数据

`src-tauri/tauri.conf.json` 中也保留了 resources 配置：

```json
"resources": [
  "../data/all_song_note_stats.json"
]
```

当前主要读取方式仍是 Rust 编译期内嵌

## 常见问题

### `cargo run` 后页面拒绝连接

原因：`cargo run` 不会启动 Vite

解决：使用：

```powershell
cargo tauri dev
```

或者手动先运行：

```powershell
npm --prefix frontend run dev
```

### `cargo tauri dev` 提示没有 tauri 子命令

安装 Tauri CLI：

```powershell
cargo install tauri-cli
```

然后确认：

```powershell
cargo tauri --version
```

### Android 找不到设备

检查：

```powershell
adb devices
```

如果是真机，需要打开 USB 调试并授权当前电脑

### Android 构建找不到 SDK/NDK/JDK

检查环境变量：

```powershell
echo $env:ANDROID_HOME
echo $env:ANDROID_NDK_HOME
echo $env:JAVA_HOME
```

也可以通过 Android Studio 的 SDK Manager 确认 SDK、NDK 和 Build Tools 已安装
