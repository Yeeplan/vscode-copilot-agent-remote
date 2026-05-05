#!/usr/bin/env bash

set -euo pipefail

BINARY_NAME="${BINARY_NAME:-vscode-remote-control}"
DEPLOY_DIR="${DEPLOY_DIR:-${HOME}/tools/vscode-copilot-agent-remote}"
PLIST_LABEL="${PLIST_LABEL:-com.vscode-copilot-agent-remote}"
PLIST_FILE="${PLIST_FILE:-${HOME}/Library/LaunchAgents/${PLIST_LABEL}.plist}"
CERT_NAME="${CERT_NAME:-vscode-remote-control-codesign}"

log() {
  printf '%s\n' "$*"
}

ensure_codesign_cert() {
  log "▶ 检查代码签名证书..."

  if security find-identity -v -p codesigning 2>/dev/null | grep -qF "\"${CERT_NAME}\""; then
    log "✓ 代码签名证书已存在"
    return
  fi

  log "  未找到证书，正在自动创建（首次安装执行一次）..."
  tmp_base="${TMPDIR:-/tmp}/vrc_codesign_$$"

  cat > "${tmp_base}.cnf" << 'CNFEOF'
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
    -keyout "${tmp_base}.key" -out "${tmp_base}.crt" \
    -days 3650 -nodes -config "${tmp_base}.cnf" 2>/dev/null

  /usr/bin/openssl pkcs12 -export \
    -out "${tmp_base}.p12" -inkey "${tmp_base}.key" -in "${tmp_base}.crt" \
    -passout pass:vrc_tmp 2>/dev/null

  security import "${tmp_base}.p12" \
    -k ~/Library/Keychains/login.keychain-db \
    -P vrc_tmp -T /usr/bin/codesign -A 2>/dev/null

  security add-trusted-cert -r trustRoot -p codeSign \
    -k ~/Library/Keychains/login.keychain-db "${tmp_base}.crt"

  rm -f "${tmp_base}.cnf" "${tmp_base}.key" "${tmp_base}.crt" "${tmp_base}.p12"
  log "✓ 代码签名证书已创建：${CERT_NAME}"
}

sign_binary() {
  local binary_path="$1"

  log "▶ 对二进制文件进行代码签名..."
  codesign --force --sign "${CERT_NAME}" "${binary_path}"
  log "✓ 代码签名完成"
  log "  （辅助功能权限只需在首次安装后手动授权一次，后续更新可保持有效）"
}

write_launch_agent() {
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
}

reload_launch_agent() {
  local service_target="gui/$(id -u)/${PLIST_LABEL}"

  write_launch_agent

  # 停止旧实例（首次安装时可能不存在，忽略失败）
  if launchctl bootout "${service_target}" 2>/dev/null; then
    log "  已停止旧服务实例"
  fi

  # 注册并启动服务
  log "▶ 启动服务 ${PLIST_LABEL}..."
  if ! launchctl bootstrap "gui/$(id -u)" "${PLIST_FILE}" 2>/dev/null; then
    launchctl load -w "${PLIST_FILE}"
  fi
  launchctl enable "${service_target}" 2>/dev/null || true

  # 等待进程启动，最多 5 秒
  local i=0
  while (( i < 10 )); do
    if launchctl print "${service_target}" 2>/dev/null | grep -q "state = running"; then
      log "✓ 服务已成功重启并正在运行"
      return 0
    fi
    sleep 0.5
    (( i++ )) || true
  done

  log "⚠ 服务已加载，但 5 秒内未检测到 running 状态（可能正在初始化，请稍后检查）"
  log "   检查命令：launchctl print ${service_target}"
}

install_binary() {
  local source_binary="$1"
  local target_binary="${DEPLOY_DIR}/${BINARY_NAME}"

  if [[ ! -f "${source_binary}" ]]; then
    log "未找到待安装二进制：${source_binary}"
    return 1
  fi

  log "▶ 部署到 ${DEPLOY_DIR}..."
  mkdir -p "${DEPLOY_DIR}"
  cp "${source_binary}" "${target_binary}"
  chmod +x "${target_binary}"
  log "✓ 文件部署完成：${target_binary}"

  ensure_codesign_cert
  sign_binary "${target_binary}"
  reload_launch_agent
}

print_post_install_notes() {
  log ""
  log "🎉 安装完成！"
  log "   二进制：${DEPLOY_DIR}/${BINARY_NAME}"
  log "   服务：${PLIST_LABEL}"
  log "   日志：${HOME}/Library/Logs/${PLIST_LABEL}.log"
  log ""
  log "   如需首次授权，请在“系统设置 → 隐私与安全性 → 辅助功能”中添加："
  log "   ${DEPLOY_DIR}/${BINARY_NAME}"
}