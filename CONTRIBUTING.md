# Contributing to LanChat / 贡献指南

[English](#english) | [简体中文](#简体中文)

---

<a name="english"></a>
## English

Thank you for your interest in contributing to **LanChat**! As an open-source decentralized P2P LAN communication project, we welcome contributions of all kinds, including bug fixes, feature proposals, documentation updates, translations, and architectural improvements.

### 📋 Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please treat all members with respect and kindness.

### 🛠️ Development Setup

#### Prerequisites
- **Node.js**: `v18.0.0` or higher
- **pnpm**: `v9.0.0` or higher (recommended package manager)
- **Rust Toolchain**: `stable` (1.77+) with `cargo` installed via [rustup.rs](https://rustup.rs)
- **OS-specific dependencies for Tauri v2**:
  - **Windows**: Microsoft Visual Studio C++ Build Tools & WebView2 (built-in on Win 10/11)
  - **Linux**: `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
  - **macOS**: Xcode Command Line Tools

#### Getting Started
1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/<your-username>/LanChat.git
   cd LanChat
   ```
2. **Install frontend dependencies**:
   ```bash
   pnpm install
   ```
3. **Run local development mode**:
   ```bash
   pnpm tauri dev
   ```
4. **Run test suites**:
   ```bash
   # Rust backend unit tests
   cd src-tauri && cargo test && cd ..

   # Frontend type check & build verification
   pnpm run build
   ```

### 🌿 Git Workflow & Commit Guidelines

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. Follow **Conventional Commits**:
   - `feat:` A new user-facing feature
   - `fix:` A bug fix
   - `docs:` Documentation changes only
   - `refactor:` Code refactoring without changing behavior
   - `perf:` Performance improvements
   - `test:` Adding or updating tests
   - `ci:` CI/CD pipeline or workflow changes
   - `chore:` Maintenance tasks, dependency updates, version bumps
3. Keep pull requests focused on a single responsibility.
4. Verify tests pass and no linter warnings remain before opening a PR.

---

<a name="简体中文"></a>
## 简体中文

感谢关注并参与 **LanChat** 开源项目！作为一款去中心化、免服务器的局域网 P2P 通讯工具，我们欢迎任何形式的贡献，包括修复 Bug、新增特性、完善文档、改进 UI/UX 或重构优化。

### 📋 行为准则

参与本项目即表示您同意遵守 [行为准则 (Code of Conduct)](CODE_OF_CONDUCT.md)。请尊重社区的每位参与者。

### 🛠️ 本地开发环境准备

#### 前置要求
- **Node.js**：`v18.0.0` 及以上
- **pnpm**：`v9.0.0` 及以上（官方推荐包管理器）
- **Rust 工具链**：`stable` (1.77+) ，推荐使用 [rustup.rs](https://rustup.rs) 安装
- **操作系统底层支持**：
  - **Windows**：Visual Studio C++ 生成工具以及 WebView2 Runtime（Win10/11 默认已内置）
  - **Linux**：安装 GTK3/WebKit2GTK 依赖项（如 `libwebkit2gtk-4.1-dev` 等）
  - **macOS**：Xcode Command Line Tools

#### 启动开发流程
1. **克隆代码库**：
   ```bash
   git clone https://github.com/<your-username>/LanChat.git
   cd LanChat
   ```
2. **安装前端依赖**：
   ```bash
   pnpm install
   ```
3. **拉起本地开发调试**：
   ```bash
   pnpm tauri dev
   ```
4. **运行自动化测试与编译校验**：
   ```bash
   # 验证后端单元测试
   cd src-tauri && cargo test && cd ..

   # 验证前端类型推导与产物构建
   pnpm run build
   ```

### 🌿 分支与提交规范

1. 基于 `main` 分支拉取新的工作分支：
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. 遵循 **Conventional Commits** 提交规范：
   - `feat:` 新增功能
   - `fix:` 修复缺陷
   - `docs:` 仅文档变更
   - `refactor:` 代码重构（不改变外部行为）
   - `perf:` 性能调优
   - `test:` 补充或更新测试用例
   - `ci:` 工作流与构建配置调整
   - `chore:` 日常构建、依赖升级、辅助工具调整
3. 保证提交粒度清晰，并在发起 Pull Request 前确保 `cargo test` 和 `pnpm run build` 全部成功通过。
