- [x] 找出本机的vscode insider打开了valuego-web工程的chat输入窗口，并进行激活
- [x] 创建readme
- [x] 创建.gitignore
- [x] 提交github
- [x] 创建一个pwa文件夹，在其中创建一个基于Vite+Vue3的PWA单页应用，iOS APP风格，面向iPhone手机，支持调用http api接口获取mac下所有的VSCode Insider窗口然后用一级导航的形式显示。点击一级导航后进入二级页面，提供表单让用户输入聊天内容，点击发送按钮后，调用http api向Copilot Chat发送聊天内容。可用的http api参考readme
- [x] 请让rust程序的http api适配跨域访问
Access to fetch at 'http://127.0.0.1:3030/api/windows' from origin 'http://localhost:5173' has been blocked by CORS policy: No 'Access-Control-Allow-Origin' header is present on the requested resource.
127.0.0.1:3030/api/windows:1  Failed to load resource: net::ERR_FAILED
(index):1 Access to fetch at 'http://127.0.0.1:3030/api/windows' from origin 'http://localhost:5173' has been blocked by CORS policy: No 'Access-Control-Allow-Origin' header is present on the requested resource.
WindowsView.vue:67  GET http://127.0.0.1:3030/api/windows net::ERR_FAILED 200 (OK)
loadWindows @ WindowsView.vue:67
callWithErrorHandling @ runtime-core.esm-bundler.js:199
callWithAsyncErrorHandling @ runtime-core.esm-bundler.js:206
(anonymous) @ runtime-dom.esm-bundler.js:745
- [x] rust程序的/api/focus接口，发送聊天内容后，还需要在停留3秒后，自动跟一个回车键，以触发agent响应
- [x] rust程序应该监听0.0.0.0
- [x] pwa程序，npm run preview默认不要监听localhost，应该以本机ip监听
- [x] pwa程序，npm run preview运行时能否显示一个二维码，内容为Network地址，这样手机扫码就能打开
- [x] 请提供将pwa程序部署到树莓派的脚本deploy-pwa.sh，此脚本可以执行编译并把dist文件夹通过ssh -l flannian 192.168.1.6部署到树莓派，树莓派已经安装了nginx，请配置其新建server监听2654端口，文件夹选择~/vscode-copilot-agent-remote
- [x] 创建部署rust程序的脚本deploy-rust.sh，部署前自动编译，部署到本机~/tools/vscode-copilot-agent-remote文件夹，部署后检查是否已配置为服务，如果没有则将其配置为服务
- [x] rust程序部署为服务后，调用http://192.168.1.14:3030/api/focus，发送{"window_name":"Visual Studio Code - Insiders","open_chat":true,"chat_content":"你是谁？"}，能够激活窗口，但是不会自动填写内容了。直接用terminal运行则无此问题。
- [x] deploy-rust.sh的方式对于普通用户还是太复杂，能否生成一个dmg安装器
- [x] rust程序需求既支持VSCode Insider，也支持VSCode
- [x] pwa更新后需保证被浏览器缓存的旧版能够失效
- [x] rust程序需要提供支持关闭窗口的接口，对应的pwa需提供关闭窗口的按钮，关闭前需用户再确认避免误点击
- [x] pwa程序点击关闭窗口按钮并确认后，出现错误提示但是几秒后自动消失了，来不及复制
- [x] rust程序需要提供列出窗口会话列表的接口，对应的pwa需提供列出会话的按钮，点击后在窗口详情页（即最末一级页面）列出所有会话列表。
- [x] rust程序需要提供列出窗口会话详情的接口，对应的pwa需支持点击会话列表中的一个会话项，进入会话详情页面，类似vscode渲染Chat的方式渲染会话详情。
- [x] main.rs中，list_sessions_for_app函数的逻辑有问题。以Code - Insiders为例，应该打开/Users/yingshen/Library/Application Support/Code - Insiders/User/globalStorage/storage.json，读取windowsState.openedWindows的数组，与传入的_window_name参数比对，_window_name的"— "之后的内容为文件夹名称，可以与windowsState.openedWindows数组项的folder属性中的最末一级文件夹名称匹配，由此可以得到_window_name对应窗口的workspace文件夹路径，从backupPath属性可以提取最后一个/后的内容为workspace的uuid。然后以sqllite格式读取/Users/yingshen/Library/Application Support/Code - Insiders/User/workspaceStorage/{{workspace_uuid}}/state.vscdb下key为"chat.ChatSessionStore.index"的JOSN值，提取并枚举entries对象作为指定窗口的session列表。"chat.ChatSessionStore.index"的JOSN值示例为：{"version":1,"entries":{"838f950d-db82-41c7-b189-f98a4580dc21":{"sessionId":"838f950d-db82-41c7-b189-f98a4580dc21","title":"Error message debugging information","lastMessageDate":1778075292909,"timing":{"created":1778075224825,"lastRequestStarted":1778075292909,"lastRequestEnded":1778075743172},"initialLocation":"panel","hasPendingEdits":false,"isEmpty":false,"isExternal":false,"lastResponseState":1,"permissionLevel":"autoApprove"},"19da4dfc-5a2f-4691-b688-8fc1845680cc":{"sessionId":"19da4dfc-5a2f-4691-b688-8fc1845680cc","title":"New Chat","lastMessageDate":1778245590635,"timing":{"created":1778245590635},"initialLocation":"panel","hasPendingEdits":false,"isEmpty":true,"isExternal":false,"lastResponseState":1,"permissionLevel":"default"},"0f23fa70-6744-4bfb-ba62-c652f50277b3":{"sessionId":"0f23fa70-6744-4bfb-ba62-c652f50277b3","title":"New Chat","lastMessageDate":1778733052682,"timing":{"created":1778733052682},"initialLocation":"panel","hasPendingEdits":false,"isEmpty":true,"isExternal":false,"lastResponseState":1,"permissionLevel":"default"},"e2a37943-797c-469a-890f-eeb146711aed":{"sessionId":"e2a37943-797c-469a-890f-eeb146711aed","title":"New Chat","lastMessageDate":1778889726269,"timing":{"created":1778889726269},"initialLocation":"panel","hasPendingEdits":false,"isEmpty":true,"isExternal":false,"lastResponseState":1,"permissionLevel":"default"},"7155dca1-b187-4a81-af7a-ee36d5a541e5":{"sessionId":"7155dca1-b187-4a81-af7a-ee36d5a541e5","title":"New Chat","lastMessageDate":1778889936447,"timing":{"created":1778889936447},"initialLocation":"panel","hasPendingEdits":false,"isEmpty":true,"isExternal":false,"lastResponseState":1,"permissionLevel":"default"}}}