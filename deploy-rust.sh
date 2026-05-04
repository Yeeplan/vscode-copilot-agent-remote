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
CERT_NAME="vscode-remote-control-codesign"   # 自签名代码签名证书名称

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

# ── Step 3: 确保代码签名证书存在（首次自动创建）──────────────────────────────
# 原理：TCC 存储的是签名证书的"指定要求"（Designated Requirement），
# 而不是二进制内容的哈希。用同一张证书重新签名后 DR 不变，TCC 无需重新授权。
# 无需完整磁盘访问权限，证书只存储在用户 login.keychain 中。
echo "▶ 检查代码签名证书..."
if ! security find-identity -v -p codesigning 2>/dev/null | grep -qF "\"${CERT_NAME}\""; then
  echo "  未找到证书，正在自动创建（首次部署执行一次）..."
  TMP="${TMPDIR:-/tmp}/vrc_codesign_$$"

  # 生成密钥和自签名证书（macOS 内置 LibreSSL，无需额外依赖）
  cat > "${TMP}.cnf" << 'CNFEOF'
[req]
distinguished_name = req_dn
x509_extensions    = v3_req
prompt             = no
[req_dn]
CN = vscode-remote-control-codesign
[v3_req]
keyUsage         = critical, digitalSignature
extendedKeyUsage = codeSigning
basicConstraints = CA:FALSE
CNFEOF

  /usr/bin/openssl req -x509 -newkey rsa:2048 -sha256 \
    -keyout "${TMP}.key" -out "${TMP}.crt" \
    -days 3650 -nodes -config "${TMP}.cnf" 2>/dev/null

  /usr/bin/openssl pkcs12 -export \
    -out "${TMP}.p12" -inkey "${TMP}.key" -in "${TMP}.crt" \
    -passout pass:vrc_tmp 2>/dev/null

  # 导入 login.keychain，预授权 codesign 无提示访问私钥
  security import "${TMP}.p12" \
    -k ~/Library/Keychains/login.keychain-db \
    -P vrc_tmp -T /usr/bin/codesign -A 2>/dev/null

  # 将证书设为用户级受信任的代码签名证书
  # （不加 -d 表示 user 域，无需 sudo）
  security add-trusted-cert -r trustRoot -p codeSign \
    -k ~/Library/Keychains/login.keychain-db "${TMP}.crt"

  rm -f "${TMP}".{cnf,key,crt,p12}
  echo "✓ 代码签名证书已创建：${CERT_NAME}"
else
  echo "✓ 代码签名证书已存在"
fi

# ── Step 4: 对二进制文件签名 ──────────────────────────────────────────────────
echo "▶ 对二进制文件进行代码签名..."
codesign --force --sign "${CERT_NAME}" "${DEPLOY_DIR}/${BINARY_NAME}"
echo "✓ 代码签名完成"
echo "  （辅助功能权限只需在首次部署后手动授权一次，后续 deploy 自动保持有效）"

# ── Step 5: 检查并配置 launchd 服务 ──────────────────────────────────────────
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
