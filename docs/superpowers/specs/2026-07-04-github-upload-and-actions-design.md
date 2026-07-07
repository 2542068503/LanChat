# LanChat 项目 GitHub 上传与 Actions 自动化打包设计文档

本项目是一个基于 Tauri (v2) + Vue 3 + TypeScript 开发的局域网 P2P 聊天和文件传输工具（LanChat）。
本设计旨在将项目上传至 GitHub 仓库 `git@github.com:2542068503/LanChat.git`，配置 GitHub Actions 自动化流水线，在推送版本标签时自动构建 Windows、macOS（无签名）和 Linux 安装包，并生成 GitHub Release 1.0.0 草稿，同时完善 README 文件。

## 1. 拟进行的修改与配置

### 1.1 版本信息同步
* **修改文件**：`src-tauri/Cargo.toml`
* **变更**：将 `version` 从 `0.1.0` 修改为 `1.0.0`，使其与 `package.json` 及 `src-tauri/tauri.conf.json` 保持一致。

### 1.2 Git 初始化与上传
* **操作**：
  1. 初始化本地 Git 仓库 (`git init`)。
  2. 确认 `.gitignore` 确保包含 `node_modules`、`dist`、`target` 等构建缓存目录，排查无敏感信息。
  3. 添加所有本地代码文件并提交第一个 Commit。
  4. 添加远程仓库地址：`git remote add origin git@github.com:2542068503/LanChat.git`。
  5. 推送代码至 GitHub `main` 分支。

### 1.3 GitHub Actions 配置 (方案 A)
* **新建文件**：`.github/workflows/publish.yml`
* **触发机制**：当推送符合 `v*` 规则的 Tag（例如 `v1.0.0`）时触发。
* **构建矩阵**：
  * **Windows**：使用 `windows-latest` 运行器，输出 `.msi` 或 `.exe` 安装包。
  * **macOS**：使用 `macos-latest` 运行器，无签名构建，输出 `.dmg` 或 `.app` 安装包。由于是无签名编译，支持编译为通用二进制文件 (`--target universal-apple-darwin`)。
  * **Linux**：使用 `ubuntu-22.04` 运行器，安装 `libwebkit2gtk-4.1-dev`、`libappindicator3-dev` 等依赖，输出 `.deb` 和 `.AppImage`。
* **发布步骤**：使用官方的 `tauri-apps/tauri-action@v0` 自动创建包含三大平台安装包的 GitHub Release 草稿。

### 1.4 完善 README 结构
* **修改文件**：`README.md`
* **内容**：
  * **中文标题**与项目定位。
  * **核心功能点**展示（用户发现、私聊/群聊、图片与大文件传输、进度速度 ETA 跟踪、安全校验、Markdown/LaTeX 渲染等）。
  * **技术栈说明**。
  * **快速开发与本地构建指南**（包含依赖安装、开发模式运行、本地打包命令）。
  * **GitHub Actions 自动构建与发布说明**（指导如何推送 tag 来触发云端自动打包）。

---

## 2. 确认设计方案

请您确认此设计。如果确认无误，我将把该设计文档写入 `docs/superpowers/specs/` 目录，然后创建实施计划。
