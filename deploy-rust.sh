#!/usr/bin/env bash
# deploy-rust.sh
# 编译 Rust 程序并部署到本机 ~/tools/vscode-copilot-agent-remote
# 部署后自动检测并配置 macOS launchd 服务（LaunchAgent）
# 用法：./deploy-rust.sh

set -euo pipefail

# ── 配置 ──────────────────────────────────────────────────────────────────────
BINARY_NAME="vscode-remote-control"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEPLOY_DIR="${HOME}/tools/vscode-copilot-agent-remote"
PLIST_LABEL="com.vscode-copilot-agent-remote"
PLIST_FILE="${HOME}/Library/LaunchAgents/${PLIST_LABEL}.plist"

# ── Step 1: 编译 ──────────────────────────────────────────────────────────────
echo "▶ 编译 Rust 程序（release 模式）..."
cd "${SCRIPT_DIR}"
cargo build --release
echo "✓ 编译完成"

# ── Step 2: 部署到本机 ────────────────────────────────────────────────────────
echo "▶ 部署到 ${DEPLOY_DIR}..."
mkdir -p "${DEPLOY_DIR}"
cp "target/release/${BINARY_NAME}" "${DEPLOY_DIR}/${BINARY_NAME}"
chmod +x "${DEPLOY_DIR}/${BINARY_NAME}"
echo "✓ 文件部署完成：${DEPLOY_DIR}/${BINARY_NAME}"

# ── Step 3: 检查并配置 launchd 服务 ──────────────────────────────────────────
if launchctl list | grep -q "${PLIST_LABEL}"; then
  echo "▶ 服务 ${PLIST_LABEL} 已配置，重新加载..."
  launchctl unload "${PLIST_FILE}" 2>/dev/null || true
  launchctl load "${PLIST_FILE}"
  echo "✓ 服务重新加载完成"
else
  echo "▶ 服务未配置，正在创建 LaunchAgent..."
  mkdir -p "${HOME}/Library/LaunchAgents"
  cat > "${PLIST_FILE}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${PLIST_LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>${DEPLOY_DIR}/${BINARY_NAME}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>${HOME}/Library/Logs/${PLIST_LABEL}.log</string>
  <key>StandardErrorPath</key>
  <string>${HOME}/Library/Logs/${PLIST_LABEL}.error.log</string>
</dict>
</plist>
EOF
  launchctl load "${PLIST_FILE}"
  echo "✓ LaunchAgent 已创建并启动：${PLIST_FILE}"
fi

echo ""
echo "🎉 部署完成！"
echo "   二进制：${DEPLOY_DIR}/${BINARY_NAME}"
echo "   服务：${PLIST_LABEL}"
echo "   日志：${HOME}/Library/Logs/${PLIST_LABEL}.log"
echo ""
echo "   停止服务：launchctl unload ${PLIST_FILE}"
echo "   启动服务：launchctl load ${PLIST_FILE}"
