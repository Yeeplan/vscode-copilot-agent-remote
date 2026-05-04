# vscode-remote-control

A lightweight macOS HTTP server (written in Rust) that lets you remotely focus a VS Code window, open GitHub Copilot Chat, and paste text into the chat input — all via a simple REST API.

## How it works

The server uses AppleScript (`osascript`) to:
1. Raise and focus a VS Code window matching a partial title string.
2. Open the Copilot Chat panel with **Cmd+Shift+I**.
3. Write the desired text to the system clipboard and paste it into the chat input.

## Requirements

- macOS (AppleScript + `pbcopy` required)
- [Rust toolchain](https://rustup.rs/)
- VS Code or VS Code Insiders with GitHub Copilot Chat installed
- Accessibility permissions granted to the terminal / binary in **System Preferences → Privacy & Security → Accessibility**

## Build

```bash
cargo build --release
```

The binary will be at `target/release/vscode-remote-control`.

## Run

```bash
# Default port: 3030
./target/release/vscode-remote-control

# Custom port
PORT=8080 ./target/release/vscode-remote-control
```

## API

### `GET /health`

Returns `{"status":"ok"}`.

---

### `GET /api/windows`

List all window titles of a running application.

| Query param | Default | Description |
|---|---|---|
| `app_name` | `Code - Insiders` | Process name as shown in Activity Monitor |

**Example**

```bash
curl "http://127.0.0.1:3030/api/windows?app_name=Code%20-%20Insiders"
```

---

### `POST /api/focus`

Focus a window, open Copilot Chat, and optionally paste text.

**Request body (JSON)**

| Field | Type | Default | Description |
|---|---|---|---|
| `app_name` | string | `"Code - Insiders"` | Process name |
| `window_name` | string | *(required)* | Partial window title to match |
| `open_chat` | bool | `true` | Open Copilot Chat after focusing |
| `chat_content` | string \| null | `null` | Text to paste into the chat input |
| `step_delay_ms` | number | `400` | Delay (ms) between automation steps |

**Example**

```bash
curl -X POST http://127.0.0.1:3030/api/focus \
  -H "Content-Type: application/json" \
  -d '{
    "window_name": "my-project",
    "chat_content": "Please review this code for security issues."
  }'
```

**Response**

```json
{ "success": true, "message": "OK" }
```

## Security notes

- `window_name` and `app_name` are validated to reject `"` and `\` characters, preventing AppleScript injection.
- `chat_content` is written to the clipboard via `pbcopy` and never embedded in AppleScript strings.
- The server binds to `127.0.0.1` only and is not intended to be exposed to the network.

## License

MIT
