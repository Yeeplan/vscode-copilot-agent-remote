#!/usr/bin/env bash
# deploy-rust.sh
# 编译 Rust 程序并部署到本机 ~/tools/vscode-copilot-agent-remote
# 部署后自动检测并配置 macOS launchd 服务（LaunchAgent）
# 用法：./deploy-rust.sh

set -euo pipefail

BINARY_NAME="vscode-remote-control"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEPLOY_DIR="${HOME}/tools/vscode-copilot-agent-remote"
source "${SCRIPT_DIR}/scripts/install-common.sh"

echo "▶ 编译 Rust 程序（release 模式）..."
cd "${SCRIPT_DIR}"
cargo build --release
echo "✓ 编译完成"

install_binary "${SCRIPT_DIR}/target/release/${BINARY_NAME}"
print_post_install_notes
