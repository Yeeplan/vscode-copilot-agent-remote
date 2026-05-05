#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESOURCE_DIR="$(cd "${SCRIPT_DIR}/../Resources" && pwd)"

export BINARY_NAME="vscode-remote-control"
export DEPLOY_DIR="${HOME}/tools/vscode-copilot-agent-remote"

source "${RESOURCE_DIR}/install-common.sh"

show_dialog() {
  local title="$1"
  local message="$2"
  /usr/bin/osascript <<EOF >/dev/null
display dialog "${message}" with title "${title}" buttons {"好"} default button "好"
EOF
}

show_failure() {
  local message="$1"
  /usr/bin/osascript <<EOF >/dev/null
display dialog "${message}" with title "安装失败" buttons {"好"} default button "好" with icon stop
EOF
}

trap 'show_failure "安装过程中出现错误。请在终端执行 deploy-rust.sh 查看详细日志。"' ERR

install_binary "${RESOURCE_DIR}/${BINARY_NAME}"
print_post_install_notes

show_dialog "安装完成" "vscode-remote-control 已安装并已尝试启动服务。\n\n首次使用前，请到“系统设置 → 隐私与安全性 → 辅助功能”中允许该二进制文件。"