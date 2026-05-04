#!/usr/bin/env bash
# deploy-pwa.sh
# 编译 PWA 并通过 SSH 部署到树莓派
# nginx 配置请单独运行 config-nginx.sh
# 用法：./deploy-pwa.sh

set -euo pipefail

# ── 配置 ──────────────────────────────────────────────────────────────────────
REMOTE_USER="flannian"
REMOTE_HOST="192.168.1.6"
REMOTE_DIR="/home/${REMOTE_USER}/vscode-copilot-agent-remote"
NGINX_PORT="2654"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PWA_DIR="${SCRIPT_DIR}/pwa"
DIST_DIR="${PWA_DIR}/dist"

# ── Step 1: 编译 ──────────────────────────────────────────────────────────────
echo "▶ 编译 PWA..."
cd "${PWA_DIR}"
npm run build
echo "✓ 编译完成：${DIST_DIR}"

# ── Step 2: 在树莓派创建目录 ──────────────────────────────────────────────────
echo "▶ 在树莓派创建目录 ${REMOTE_DIR}/dist ..."
ssh "${REMOTE_USER}@${REMOTE_HOST}" "mkdir -p '${REMOTE_DIR}/dist'"

# ── Step 3: 同步 dist 到树莓派 ────────────────────────────────────────────────
echo "▶ 同步文件到树莓派..."
rsync -avz --delete \
  -e ssh \
  "${DIST_DIR}/" \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/dist/"
echo "✓ 文件同步完成"

# ── Step 4: 修正权限（nginx www-data 需要可穿越 home 目录）──────────────────
echo "▶ 修正文件权限..."
ssh "${REMOTE_USER}@${REMOTE_HOST}" \
  "chmod o+x /home/${REMOTE_USER} && chmod -R o+rX '${REMOTE_DIR}/dist'"
echo "✓ 权限设置完成"

echo ""
echo "🎉 部署完成！访问地址："
echo "   http://${REMOTE_HOST}:${NGINX_PORT}/"

