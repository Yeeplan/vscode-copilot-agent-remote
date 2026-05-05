# vscode-remote-control

一个运行在 macOS 上的轻量 HTTP 服务（Rust 编写），允许通过局域网 REST API 远程控制 VS Code：聚焦指定窗口、打开 GitHub Copilot Chat 面板，并自动输入聊天内容触发 Agent 响应。

配套的 [PWA 应用](./pwa/) 提供 iPhone 友好的 UI，部署在树莓派上，扫码即可在手机上使用。

---

## 架构概览

```
iPhone (PWA)
    │  HTTP (局域网)
    ▼
树莓派 nginx :2654
    │  静态文件
    └─► PWA (Vue 3 + Vite)
            │  HTTP (局域网)
            ▼
        Mac :3030
        vscode-remote-control (Rust / axum)
            │  AppleScript (osascript)
            ▼
        VS Code Insiders + Copilot Chat
```

---

## Rust 服务端

### 工作原理

服务通过 `osascript` 执行 AppleScript 自动化：

1. 根据窗口标题的部分匹配字符串，激活并置顶目标 VS Code 窗口。
2. 按下 **Cmd+Shift+I** 打开 Copilot Chat 面板。
3. 通过 osascript 将聊天内容写入系统剪贴板（兼容 LaunchAgent 环境）。
4. 等待 `step_delay_ms` 毫秒后粘贴内容（Cmd+V）。
5. 再等待 3 秒后按下 **Enter** 触发 Agent 响应。

### 依赖

- macOS（需要 AppleScript / osascript）
- [Rust toolchain](https://rustup.rs/)
- VS Code 或 VS Code Insiders，已安装 GitHub Copilot Chat
- 在 **系统设置 → 隐私与安全性 → 辅助功能** 中授予二进制文件权限

> **注意（LaunchAgent）**：从终端直接运行时，Terminal.app 的辅助功能权限会被继承，无需额外配置。但部署为 LaunchAgent 服务后，需要将 **二进制文件本身** 加入辅助功能白名单，否则 AppleScript 调用会报错 `-25211`：
> 1. 打开 **系统设置 → 隐私与安全性 → 辅助功能**
> 2. 点击 `+`，添加 `~/tools/vscode-copilot-agent-remote/vscode-remote-control`
> 3. 重启服务：`launchctl unload ~/Library/LaunchAgents/com.vscode-copilot-agent-remote.plist && launchctl load ~/Library/LaunchAgents/com.vscode-copilot-agent-remote.plist`

### 手动构建与运行

```bash
cargo build --release

# 默认端口 3030，监听 0.0.0.0
./target/release/vscode-remote-control

# 自定义端口
PORT=8080 ./target/release/vscode-remote-control
```

### 一键部署（推荐）

`deploy-rust.sh` 完成编译、部署到 `~/tools/vscode-copilot-agent-remote/`，并自动配置为 macOS LaunchAgent（开机自启、崩溃自动重启）：

```bash
./deploy-rust.sh
```

- 服务标识：`com.vscode-copilot-agent-remote`
- 日志路径：`~/Library/Logs/com.vscode-copilot-agent-remote.log`
- 停止服务：`launchctl unload ~/Library/LaunchAgents/com.vscode-copilot-agent-remote.plist`
- 启动服务：`launchctl load ~/Library/LaunchAgents/com.vscode-copilot-agent-remote.plist`

### 生成 DMG 安装器

如果希望给普通 macOS 用户一个“双击即可安装”的交付物，可以在项目根目录执行：

```bash
./build-dmg.sh
```

脚本会完成以下工作：

- 编译 release 版 Rust 二进制
- 生成一个 `VSCode Remote Control Installer.app`
- 打包为 `./vscode-remote-control-installer.dmg`

用户使用方式：

1. 双击挂载 `vscode-remote-control-installer.dmg`
2. 双击其中的 `VSCode Remote Control Installer.app`
3. 安装完成后，到 **系统设置 → 隐私与安全性 → 辅助功能** 中允许 `~/tools/vscode-copilot-agent-remote/vscode-remote-control`

> 说明：这里生成的是本地 unsigned DMG，适合内部分发和局域网环境使用；如果要面向更广泛用户分发，还需要补充 Developer ID 签名和 notarization。

---

## REST API

### `GET /health`

健康检查。

```json
{ "status": "ok" }
```

---

### `GET /api/windows`

获取指定应用所有窗口的标题列表。

| 查询参数 | 默认值 | 说明 |
|---|---|---|
| `app_name` | `Code - Insiders` | 进程名（Activity Monitor 中显示的名称） |

**示例**

```bash
curl "http://127.0.0.1:3030/api/windows"
curl "http://127.0.0.1:3030/api/windows?app_name=Code%20-%20Insiders"
```

**响应**

```json
{
  "success": true,
  "windows": ["my-project — Visual Studio Code - Insiders", "..."]
}
```

---

### `POST /api/focus`

激活窗口，打开 Copilot Chat，并发送聊天内容。

**请求体（JSON）**

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `app_name` | string | `"Code - Insiders"` | 进程名 |
| `window_name` | string | *(必填)* | 窗口标题的部分匹配字符串 |
| `open_chat` | bool | `true` | 聚焦后是否打开 Copilot Chat 面板 |
| `chat_content` | string \| null | `null` | 要粘贴到 Chat 输入框的文本 |
| `step_delay_ms` | number | `400` | 各自动化步骤之间的间隔（毫秒） |

**示例**

```bash
curl -X POST http://127.0.0.1:3030/api/focus \
  -H "Content-Type: application/json" \
  -d '{
    "window_name": "my-project",
    "open_chat": true,
    "chat_content": "帮我 review 这段代码的安全性"
  }'
```

**响应**

```json
{ "success": true, "message": "OK" }
```

---

## CORS

服务包含宽松的 CORS 策略（`Access-Control-Allow-Origin: *`），允许 PWA 及任意来源的浏览器客户端直接调用。

## 安全说明

- `window_name` 和 `app_name` 会校验，拒绝含 `"` 或 `\` 的值，防止 AppleScript 注入。
- `chat_content` 通过 osascript 写入剪贴板，不嵌入 AppleScript 字符串中。
- 服务监听 `0.0.0.0`，局域网内可访问，请确保在防火墙后运行或限制访问来源。

---

## PWA 移动端

### 功能

- **iOS App 风格**，面向 iPhone，竖屏、standalone 模式（可添加到主屏幕）
- **窗口列表页**：自动调用 `/api/windows` 获取所有 VS Code 窗口，支持离线缓存（localStorage）
- **聊天页**：选中窗口后进入，填写内容后点击发送，调用 `/api/focus` 接口
- **离线可用**：Service Worker（Workbox）缓存所有构建产物，断网时显示上次缓存的窗口列表

### 配置

API 地址通过环境变量 `VITE_API_BASE` 配置，默认为 `http://127.0.0.1:3030`。

在 `pwa/` 目录下创建 `.env.local` 文件覆盖：

```
VITE_API_BASE=http://192.168.1.14:3030
```

### 本地开发

```bash
cd pwa
npm install
npm run dev          # 开发服务器，监听 0.0.0.0（--host）
npm run preview      # 预览构建产物，自动在终端打印 Network 地址二维码
```

### 构建

```bash
npm run build        # 标准构建，输出到 pwa/dist/
npm run build:single # 单文件 HTML 构建，输出到 pwa/dist-single/index.html
```

### 部署到树莓派

`deploy-pwa.sh` 自动完成构建并通过 rsync 同步到树莓派，由 nginx 在 **2654 端口**提供服务：

```bash
./deploy-pwa.sh
```

- 树莓派地址：`192.168.1.6`，用户：`flannian`
- 文件部署路径：`~/vscode-copilot-agent-remote/dist/`
- 访问地址：`http://192.168.1.6:2654/`

nginx 配置单独由 `config-nginx.sh` 完成（仅首次部署时需要执行）。

---

## 项目结构

```
.
├── src/main.rs            # Rust HTTP 服务端（axum）
├── Cargo.toml
├── build-dmg.sh           # 生成 macOS DMG 安装器
├── deploy-rust.sh         # 编译并部署为 macOS LaunchAgent
├── pwa/
│   ├── src/
│   │   ├── views/
│   │   │   ├── WindowsView.vue   # 窗口列表页
│   │   │   └── ChatView.vue      # 聊天发送页
│   │   └── router/index.js
│   ├── scripts/install-common.sh # CLI / GUI 共用安装逻辑
│   ├── scripts/preview.mjs       # 预览时打印二维码
│   ├── scripts/gui-install.sh    # DMG 中 Installer.app 的入口脚本
│   ├── vite.config.js            # PWA 配置
│   ├── vite.config.singlefile.js # 单文件构建配置
│   └── package.json
├── deploy-pwa.sh          # 编译并部署 PWA 到树莓派
└── config-nginx.sh        # 树莓派 nginx 配置
```

---

## License

MIT
