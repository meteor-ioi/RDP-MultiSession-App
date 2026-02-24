# RDP-MultiSession-App

[![Release](https://img.shields.io/github/v/release/meteor-ioi/RDP-MultiSession-App?style=flat-square)](https://github.com/meteor-ioi/RDP-MultiSession-App/releases)
[![Platform](https://img.shields.io/badge/platform-Windows-blue?style=flat-square)](#)

> [!IMPORTANT]
> **本应用专为 Windows 操作系统设计。** 不支持 macOS 或 Linux。
> 本仓库为 `RDP-MultiSession-App` 项目的同步镜像仓库。

RDP-MultiSession-App 是一款基于 Tauri + React 架构开发的现代化 Windows 远程桌面并发管理工具。它旨在为用户提供一个直观、美观且高效的界面，用于一键开启或恢复 Windows 的多用户同时登陆（RDP 并发）功能。

## 🌟 关键特性

- **现代化 UI/UX**：采用 Vercel 风格设计语言，支持实时状态监听和流畅的动画交互。
- **内核驱动补丁**：自动化获取 `termsrv.dll` 权限并应用最新的多会话特征码补丁。
- **持久化守护**：内置系统级任务计划，即便系统更新或重启也能自动尝试修复补丁。
- **安全中心兼容**：一键将关键系统组件加入 Windows Defender 排除列表。
- **特征码云同步**：支持通过高可用代理链在线拉取并同步最新的 RDP 偏移量规则。
- **实时日志系统**：详细记录每一次内核操作与桥接通讯，确保操作可追溯。

## 🛠️ 安装与运行

### 系统要求
- Windows 10 / 11 (build 10240+)
- 管理员权限

### 下载使用
您可以前往 [Releases](https://github.com/meteor-ioi/RDP-MultiSession-App/releases) 页面下载最新编译好的 `.exe` 安装程序或绿色版。

### 开发者编译
1. 安装 [Rust](https://www.rust-lang.org/tools/install) 环境。
2. 安装 [Node.js](https://nodejs.org/)。
3. 克隆仓库并安装依赖：
   ```bash
   cd rdp-multi-session-ui
   npm install
   ```
4. 开发模式运行：
   ```bash
   npm run tauri dev
   ```
5. 构建发布版本：
   ```bash
   npm run tauri build
   ```

## 📖 关于项目

本项目是基于对 `RDPWrap` 相关技术及偏移量自动化查找逻辑的现代化重构和封装。

- 核心偏移查找逻辑参考了本项目内的 `RDPWrapOffsetFinder` 模块。
- 本仓库作为 **meteor-ioi/RDP-MultiSession-App** 的同步镜像，致力于提供稳定且美观的多用户并发解决方案。

## ⚖️ 免责声明

本工具仅供学习研究使用。修改系统文件存在一定风险，请在使用前务必通过应用内置功能进行备份。因使用本工具导致的任何系统不稳定或法律风险由用户自行承担。
