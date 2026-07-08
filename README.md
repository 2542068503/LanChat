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

## 📝 版本号修改指南

在发布新版本前，您需要同步更新项目中的版本号。由于这是一个 Tauri 项目，需要同时更新前端和后端的版本号。

主要需要修改以下 **3** 个文件：
1. `package.json` (前端及 Node.js 依赖配置)
2. `src-tauri/tauri.conf.json` (Tauri 构建配置)
3. `src-tauri/Cargo.toml` (Rust 后端配置)

**修改方法：手动修改**
打开上述三个文件，将其中的 `"version": "x.x.x"` (或 `version = "x.x.x"`) 手动统一修改为您需要的新版本号。

*(注：Tauri 目前没有官方内置的一键修改多文件版本号的命令，因此最稳妥的方法是手动或使用全局替换搜索进行更改)*

## 🤖 自动化构建与发布 (GitHub Actions)

项目已配置 GitHub Actions 自动构建工作流。在您需要发布新版本时，只需执行以下操作：

1. **更新版本号**：按照上方《版本号修改指南》修改并提交代码。
2. **推送版本标签**：
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
3. **获取安装包**：GitHub Actions 将自动拉起 Windows、macOS 和 Linux 的构建虚拟环境，完成构建后会在仓库的 **Releases** 页面自动创建一个 **Release Draft (草稿)**，其中已挂载好所有平台的安装包（`.msi`, `.dmg`, `.deb`, `.AppImage` 等）。您只需在 GitHub 上将其点击发布即可。
