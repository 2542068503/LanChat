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
