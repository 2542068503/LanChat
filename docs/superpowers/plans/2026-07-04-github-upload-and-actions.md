# LanChat GitHub 上传与 Actions 自动构建实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 同步版本信息、初始化本地 Git 仓库并推送至 GitHub，然后配置 GitHub Actions 实现基于 v* 标签触发的三平台自动打包和发布 Release 草稿，并完善项目 README 文件。

**Architecture:**
1. 同步 `src-tauri/Cargo.toml` 的包版本为 `1.0.0`。
2. 本地初始化 Git，添加并提交所有项目文件。
3. 添加 GitHub 远程源并推送至 `main` 分支。
4. 新建 GitHub Action 配置文件 `.github/workflows/publish.yml`，使用官方 `tauri-apps/tauri-action` 实现云端自动化打包构建。
5. 重写 README.md 文件。

**Tech Stack:** Git, GitHub Actions, Tauri CLI (v2), Node.js, pnpm, Rust

## Global Constraints
- 构建平台：Windows (MSI/NSIS), macOS (无签名dmg/app), Linux (deb/AppImage)。
- 应用版本：`1.0.0`。
- 交互语言：全程使用中文回答用户。

---

### Task 1: 版本信息同步与本地 Git 初始化

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: 当前项目文件
- Produces: 匹配 1.0.0 版本的 Cargo.toml、已初始化并拥有首个提交的本地 Git 仓库

- [ ] **Step 1: 修改 Cargo.toml 版本**
  将 `src-tauri/Cargo.toml` 第 3 行的 `version = "0.1.0"` 修改为 `version = "1.0.0"`。
  
- [ ] **Step 2: 本地 Git 初始化**
  在项目根目录下执行以下命令：
  ```powershell
  git init
  ```
  
- [ ] **Step 3: 添加并提交所有本地文件**
  在项目根目录下执行以下命令：
  ```powershell
  git add .
  git commit -m "chore: initial commit and sync version to 1.0.0"
  ```
  
- [ ] **Step 4: 验证版本与 Git 状态**
  执行命令验证 Git 状态已是 clean：
  ```powershell
  git status
  ```
  预期输出：`nothing to commit, working tree clean`

---

### Task 2: 关联远程 GitHub 仓库并推送

**Files:**
- 无 (仅执行 Git 远程关联与推送指令)

**Interfaces:**
- Consumes: Task 1 中初始化的本地 Git 仓库
- Produces: GitHub 远程仓库 `git@github.com:2542068503/LanChat.git` 的 `main` 分支中包含本地所有文件

- [ ] **Step 1: 添加 GitHub 远程仓库**
  在项目根目录下执行：
  ```powershell
  git remote add origin git@github.com:2542068503/LanChat.git
  ```

- [ ] **Step 2: 推送代码到 GitHub**
  在项目根目录下执行推送指令：
  ```powershell
  git push -u origin main
  ```

- [ ] **Step 3: 验证推送是否成功**
  推送无报错，终端显示分支跟踪成功。

---

### Task 3: 配置 GitHub Actions 自动构建工作流

**Files:**
- Create: `.github/workflows/publish.yml`

**Interfaces:**
- Consumes: 项目代码与 pnpm 依赖配置
- Produces: 可在推送 `v*` 标签时自动运行并编译三大平台安装包的 GitHub Actions 工作流文件

- [ ] **Step 1: 创建 GitHub Actions 工作流文件**
  新建 `.github/workflows/publish.yml` 文件，填入以下内容：
  ```yaml
  name: "publish"

  on:
    push:
      tags:
        - 'v*'

  jobs:
    publish-tauri:
      permissions:
        contents: write
      strategy:
        fail-fast: false
        matrix:
          include:
            - platform: "macos-latest"
              args: "--target universal-apple-darwin"
            - platform: "ubuntu-22.04"
              args: ""
            - platform: "windows-latest"
              args: ""

      runs-on: ${{ matrix.platform }}
      steps:
        - uses: actions/checkout@v4

        - name: Setup Node
          uses: actions/setup-node@v4
          with:
            node-version: 20

        - name: Install pnpm
          uses: pnpm/action-setup@v4
          with:
            version: 11.9.0

        - name: Install Rust stable
          uses: dtolnay/rust-toolchain@stable
          with:
            targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

        - name: Install dependencies (Ubuntu only)
          if: matrix.platform == 'ubuntu-22.04'
          run: |
            sudo apt-get update
            sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

        - name: Install frontend dependencies
          run: pnpm install

        - name: Build and Publish Tauri App
          uses: tauri-apps/tauri-action@v0
          env:
            GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          with:
            tagName: v__VERSION__
            releaseName: "LanChat v__VERSION__"
            releaseBody: "See the assets below to download."
            releaseDraft: true
            prerelease: false
            args: ${{ matrix.args }}
  ```

- [ ] **Step 2: 提交并推送工作流配置**
  在项目根目录下执行：
  ```powershell
  git add .github/workflows/publish.yml
  git commit -m "ci: add GitHub Actions release workflow"
  git push origin main
  ```

- [ ] **Step 3: 验证工作流推送**
  查看本地 Git 状态是否干净。

---

### Task 4: 完善项目 README 文件

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: 原始模板 README.md
- Produces: 包含项目定位、核心功能点、技术栈、开发与构建指南的精美 README.md

- [ ] **Step 1: 重写 README.md**
  修改 `README.md` 文件为以下内容：
  ```markdown
  # LanChat

  LanChat 是一款基于 **Tauri (v2)** + **Vue 3** + **TypeScript** 构建的局域网免服务器 P2P 聊天与文件传输应用。致力于提供简单、快速且安全的局域网内通讯体验。

  ## 🚀 核心功能

  - **免服务器 & 自动发现**：基于 UDP 广播与组播技术，自动发现同一局域网内的在线用户，即开即用。
  - **私聊与大厅广播**：支持与指定好友的 1-to-1 私密聊天，以及面向整个局域网的大厅广播（群聊）。
  - **丰富内容传输**：支持文字聊天、图片发送以及任意大文件的传输。
  - **大文件断点/进度跟踪**：实时显示文件传输进度、速度、预计剩余时间（ETA），并在传输完成后自动进行 **SHA-256 安全校验**。
  - **高级渲染支持**：内置 Markdown 渲染器，并支持 **LaTeX 数学公式** 的完美渲染。
  - **极速本地存储**：自动持久化保存最近的聊天记录，基于本地存储，保护隐私。
  - **个性化设置**：支持自定义昵称、备注、修改预设头像或上传 Base64 自定义头像。

  ## 🛠️ 技术栈

  - **前端**：Vue 3 (Composition API), TypeScript, Vite, KaTeX (公式渲染), Marked (Markdown 渲染)
  - **后端 & 跨平台**：Tauri (v2), Rust, Tokio (异步运行时), Socket2 (UDP 套接字), AES-GCM (加密传输支持)

  ## 📦 本地快速开始

  ### 1. 安装依赖
  建议使用 `pnpm` 安装项目依赖：
  ```bash
  pnpm install
  ```

  ### 2. 运行开发模式
  启动前端 Dev Server 并拉起 Tauri 桌面应用窗口：
  ```bash
  pnpm tauri dev
  ```

  ### 3. 本地构建打包
  打包当前操作系统的生产版本：
  ```bash
  pnpm tauri build
  ```

  ## 🤖 自动化构建与发布 (GitHub Actions)

  项目已配置 GitHub Actions 自动构建工作流。在您需要发布新版本时，只需执行以下操作：

  1. **修改版本号**：在 `package.json`、`src-tauri/tauri.conf.json` 以及 `src-tauri/Cargo.toml` 中修改版本。
  2. **推送版本标签**：
     ```bash
     git tag v1.0.0
     git push origin v1.0.0
     ```
  3. **获取安装包**：GitHub Actions 将自动拉起 Windows、macOS 和 Linux 的构建虚拟环境，完成构建后会在仓库的 **Releases** 页面自动创建一个 **Release Draft (草稿)**，其中已挂载好所有平台的安装包（`.msi`, `.dmg`, `.deb`, `.AppImage` 等）。您只需在 GitHub 上将其点击发布即可。
  ```

- [ ] **Step 2: 提交并推送 README.md**
  在项目根目录下运行：
  ```powershell
  git add README.md
  git commit -m "docs: update README.md with features and build guide"
  git push origin main
  ```

- [ ] **Step 3: 验证最终状态**
  运行 `git status` 确保本地工作区完全干净。
