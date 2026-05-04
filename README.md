# Vibe Coding Voice Native

一个面向 Windows 的本地优先语音输入工具，目标是让你在 Cursor、VS Code Chat、任意输入框等场景里，用全局快捷键开始口述、结束录音，并把识别结果快速投送回当前输入位置。

当前版本基于 Rust + egui 构建，聚焦中文口述、全局快捷键、原生录音、轻量浮层和本地 SenseVoice 转写链路。

## 项目定位

- 本地优先：录音、转写、界面交互尽量在本地完成
- 面向开发者：不是通用会议转录器，而是偏向 `vibe coding` 的语音输入法
- 原生桌面：不使用 WebView，采用 Rust 原生桌面实现
- 快速投送：目标是把识别结果送回当前输入框，而不是停留在应用内部

## 当前功能

- 全局快捷键开始/结束录音
- Windows 原生主界面与设置页
- 中文字体自动注入，避免乱码
- 本地麦克风采集
- SenseVoice 兼容模型探测与转写
- 识别结果展示、复制、发送
- 自动粘贴开关
- 托盘常驻基础能力
- 录音/识别状态浮层

## 技术栈

- `Rust`
- `eframe` / `egui`
- `cpal`
- `global-hotkey`
- `tray-icon`
- `arboard`
- `enigo`
- `transcribe-rs` with `sense_voice`
- `windows-sys`

## 项目结构

```text
.
├─ src/
│  ├─ app.rs
│  ├─ hotkeys.rs
│  ├─ main.rs
│  ├─ sensevoice.rs
│  ├─ services.rs
│  ├─ state.rs
│  └─ tray.rs
├─ Cargo.toml
├─ Cargo.lock
├─ recording-overlay.ps1
├─ run-native.ps1
└─ README.md
```

## 运行环境

- Windows
- Rust 工具链
- 可用的麦克风输入设备
- PowerShell

## 快速开始

1. 安装 Rust 工具链
2. 准备本地 SenseVoice 模型目录
3. 在项目根目录执行

```powershell
cargo run --bin vibe-coding-voice-native
```

如果你本地已经在项目目录维护了 `CARGO_HOME` / `RUSTUP_HOME`，也可以使用：

```powershell
.\run-native.ps1
```

## 模型说明

当前代码使用 `transcribe-rs` 的 `sense_voice` 能力，因此更适合使用兼容的 SenseVoice 导出模型。

注意：

- 仓库本身不包含模型文件
- 首次运行前需要你自己准备模型目录
- 如果模型目录结构不兼容，程序会在状态提示里给出错误信息

## 开发说明

常用检查：

```powershell
cargo check
```

主要入口：

- `src/main.rs`：窗口、字体与应用初始化
- `src/app.rs`：主界面、设置页、录音区与结果区 UI
- `src/services.rs`：录音、转写、投送、浮层联动
- `src/hotkeys.rs`：全局快捷键注册与事件处理
- `src/sensevoice.rs`：本地模型探测与兼容层
- `src/state.rs`：应用状态与默认值

## 当前状态

- 项目仍在持续迭代
- 当前主要面向 Windows
- 投送目标 UI 已收敛为“当前输入框”，底层接口仍保留扩展空间
- 浮层动效、输入焦点恢复、模型准备说明仍会继续打磨

## 免责声明

本项目仍在迭代中，请不要直接用于高风险生产场景。  
如果你准备公开发布，建议补充许可证、第三方依赖说明和模型来源说明。
