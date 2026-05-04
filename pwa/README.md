# VSCode Remote — PWA

基于 Vite + Vue 3 构建的 PWA 移动端控制界面，iOS APP 风格，面向 iPhone。

配合 [vscode-remote-control](../README.md) Rust 服务使用，可在手机上浏览所有 VS Code Insiders 窗口并向 Copilot Chat 发送消息。

## 功能

- 列出 Mac 上所有 VS Code Insiders 窗口（一级导航）
- 点击窗口进入聊天页面，填写内容后一键发送至 Copilot Chat（二级页面）
- 支持 PWA 安装到 iPhone 主屏幕，离线缓存

## 开发环境

```bash
npm install
npm run dev
```

## 构建

```bash
npm run build
# 预览构建产物（可加 --host 暴露给局域网）
npm run preview -- --host
```

## 配置 API 地址

复制 `.env.example` 为 `.env.local`，修改为 Mac 的局域网 IP：

```bash
cp .env.example .env.local
# 编辑 VITE_API_BASE=http://192.168.x.x:3030
```

> **注意**：Rust 服务默认只监听 `127.0.0.1`。若需从 iPhone 访问，需修改服务绑定地址为 `0.0.0.0` 或通过反向代理暴露。

## 安装到 iPhone 主屏幕

1. iPhone Safari 打开 `http://<Mac-IP>:<port>/`
2. 点击底部分享按钮 → **添加到主屏幕**
3. 即可以 APP 形式启动，支持全屏无地址栏

## 技术栈

- [Vite](https://vite.dev/) + [Vue 3](https://vuejs.org/)
- [Vue Router 4](https://router.vuejs.org/)
- [vite-plugin-pwa](https://vite-pwa-org.netlify.app/) + Workbox
