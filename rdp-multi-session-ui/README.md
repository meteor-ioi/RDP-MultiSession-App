<div align="center">
  <img src="src-tauri/icons/128x128.png" width="80" alt="logo" />
  <h1>RDP-MultiSession-App</h1>
  <p><b>一个极简设计的 Windows RDP 多用户并发管理工具</b></p>
</div>

## 📖 项目简介

`RDP-MultiSession-App` 是一款专为 Windows 系统打造的轻量级桌面应用，旨在提供一个直观、现代化的极简 GUI 界面（Vercel 设计风格）来替代繁杂的命令行或不稳定的第三方内存补丁。本应用基于 **Tauri 2.0 + React** 构建，前端轻量且极速，底部分离预留了与 PowerShell 脚本集成的强大后端潜力。

通过一键操作，您可以轻松解除 Windows 家庭版/专业版的原生远程桌面（RDP）的单用户登录限制，实现：
- **多人同时登录同一台电脑**而不被互相踢出。
- **后台极简管理**：随时控制并发状态，实时获取当前活跃的 RDP 会话数。

## ✨ 核心特性

- **极简 Vercel 风格设计**: 黑白纯色、自然呼吸灯状态环、平滑的过渡动画，提供极致的视觉体验。
- **一键切换开关**: 告别晦涩的注册表和文件修改，只需一键点击，智能触发底层环境配置与系统文件备份/修补。
- **原生只读信息流**: 直观展示您的 Windows 版本（包含 Build 号）、并发补丁状态以及系统级文件的自我保护禁用状态。
- **跨代平台兼容**: 本应用专注针对 Windows 操作系统生成原生的轻量 `.exe` 和 `.msi` 可执行文件。
- **超低资源占用**: 借助 Tauri 框架 (基于 Rust)，相比传统的 Electron 客户端，它拥有更快的启动速度和只有几兆字节的超小内存开销。

## 🚀 下载与安装

由于此修改涉及到 Windows 系统的核心文件（`termsrv.dll`），目前仅支持 **Windows 操作系统**。

请访问本仓库的 **[Releases 页面](https://github.com/meteor-ioi/RDP-MultiSession-App/releases)**，下载对应您系统架构的最新可执行文件进行安装。

## 🛠 开发与构建

本项目采用 Tauri 脚手架构建。如果需要自行修改或运行前端预览，需要安装 Node.js 和 Rust 编译环境。

```bash
# 安装依赖
npm install

# 仅作为前端 UI 启动开发服务器（支持热重载）
npm run dev

# 构建用于生产的 Tauri 安装包 (仅支持 Windows 下构建 Win 版包)
npm run tauri build
```

## ⚠️ 安全与免责声明

> [!WARNING]
> 本软件及其底层机制涉及对 Windows 核心服务（`termsrv.dll`）的修改及所有权的变更。
> 请注意：此类操作可能违反部分微软软件许可协议，并使系统更容易受到潜在的安全风险攻击。
> 
> 本工具仅供**个人学习、网络测试以及受控的实验环境**内使用，请勿用于关键生产环境。对于因使用本软件造成的任何系统崩溃、安全漏洞或数据丢失，开发者不承担任何责任。

---
*Developed with Tauri, React & Tailwind CSS v4.*
