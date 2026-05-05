#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="VSCode Remote Control Installer"
APP_BUNDLE="${APP_NAME}.app"
DMG_NAME="vscode-remote-control-installer"
BUILD_ROOT="${SCRIPT_DIR}/build/dmg"
STAGE_DIR="${BUILD_ROOT}/stage"
APP_DIR="${STAGE_DIR}/${APP_BUNDLE}"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
OUTPUT_DMG="${SCRIPT_DIR}/${DMG_NAME}.dmg"

echo "▶ 编译 Rust 程序（release 模式）..."
cd "${SCRIPT_DIR}"
cargo build --release
echo "✓ 编译完成"

echo "▶ 准备 Installer.app..."
rm -rf "${BUILD_ROOT}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

cp "${SCRIPT_DIR}/scripts/gui-install.sh" "${MACOS_DIR}/install"
cp "${SCRIPT_DIR}/scripts/install-common.sh" "${RESOURCES_DIR}/install-common.sh"
cp "${SCRIPT_DIR}/target/release/vscode-remote-control" "${RESOURCES_DIR}/vscode-remote-control"
chmod +x "${MACOS_DIR}/install" "${RESOURCES_DIR}/vscode-remote-control"

cat > "${CONTENTS_DIR}/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>zh_CN</string>
  <key>CFBundleExecutable</key>
  <string>install</string>
  <key>CFBundleIdentifier</key>
  <string>com.vscode-copilot-agent-remote.installer</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>VSCode Remote Control Installer</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
</dict>
</plist>
EOF

cat > "${STAGE_DIR}/README.txt" <<'EOF'
1. 双击 VSCode Remote Control Installer.app 开始安装。
2. 安装完成后，到“系统设置 -> 隐私与安全性 -> 辅助功能”中允许：
   ~/tools/vscode-copilot-agent-remote/vscode-remote-control
3. 如果需要重新启动服务，可执行：
   launchctl kickstart -k gui/$(id -u)/com.vscode-copilot-agent-remote
EOF

echo "✓ Installer.app 已生成"

echo "▶ 创建 DMG..."
rm -f "${OUTPUT_DMG}"
hdiutil create \
  -volname "VSCode Remote Control Installer" \
  -srcfolder "${STAGE_DIR}" \
  -ov \
  -format UDZO \
  "${OUTPUT_DMG}" >/dev/null

echo "✓ DMG 已生成：${OUTPUT_DMG}"