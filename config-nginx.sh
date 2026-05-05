#!/usr/bin/env bash
# config-nginx.sh
# 在树莓派上配置 nginx，新建 server 监听 2654 端口，serving PWA dist 目录
# 用法：./config-nginx.sh

set -euo pipefail

REMOTE_USER="flannian"
REMOTE_HOST="10.66.66.1"
REMOTE_DIR="/home/${REMOTE_USER}/vscode-copilot-agent-remote"
NGINX_PORT="2654"
TMP_SCRIPT="/tmp/_config_nginx_$$.sh"

echo "▶ 配置树莓派 nginx（端口 ${NGINX_PORT}）..."

# ── Step 1: 上传执行脚本到 /tmp（不需要 sudo）────────────────────────────────
ssh "${REMOTE_USER}@${REMOTE_HOST}" "cat > ${TMP_SCRIPT}" << ENDSSH
#!/usr/bin/env bash
set -euo pipefail

CONF="/etc/nginx/sites-available/vscode-copilot-agent-remote"

sudo tee "\${CONF}" > /dev/null << 'EOF'
server {
    listen ${NGINX_PORT};
    server_name _;

    root ${REMOTE_DIR}/dist;
    index index.html;

    # SPA fallback: history mode 下所有路径返回 index.html
    location / {
        try_files \$uri \$uri/ /index.html;
    }

    # 始终重新校验应用外壳与 service worker，避免 PWA 被旧缓存卡住
    location = /index.html {
        expires -1;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }

    location = /sw.js {
        expires -1;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }

    location = /manifest.webmanifest {
        expires -1;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }

    # 缓存静态资源
    location ~* \.(js|css|png|svg|ico|webmanifest|woff2?)$ {
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}
EOF

ENABLED="/etc/nginx/sites-enabled/vscode-copilot-agent-remote"
if [ ! -L "\${ENABLED}" ]; then
  sudo ln -s "\${CONF}" "\${ENABLED}"
fi

sudo nginx -t
sudo systemctl reload nginx
echo "✓ nginx 重载完成，监听端口 ${NGINX_PORT}"
ENDSSH

# ── Step 2: 用 -t 分配 TTY 执行，sudo 可正常弹出密码提示 ─────────────────────
ssh -t "${REMOTE_USER}@${REMOTE_HOST}" "bash ${TMP_SCRIPT}; rm -f ${TMP_SCRIPT}"

echo ""
echo "✓ nginx 配置完成！访问地址："
echo "   http://${REMOTE_HOST}:${NGINX_PORT}/"
