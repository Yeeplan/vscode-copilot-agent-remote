1. 双击 VSCode Remote Control Installer.app 开始安装。
2. 安装完成后，到“系统设置 -> 隐私与安全性 -> 辅助功能”中允许：
   ~/tools/vscode-copilot-agent-remote/vscode-remote-control
3. 如果需要重新启动服务，可执行：
   launchctl kickstart -k gui/$(id -u)/com.vscode-copilot-agent-remote
