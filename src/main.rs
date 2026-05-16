use axum::{
    extract::{Json, Query},
    http::{Method, StatusCode},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tower_http::cors::{Any, CorsLayer};

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FocusRequest {
    /// Process name of the application, e.g. "Code - Insiders"
    #[serde(default = "default_app_name")]
    app_name: String,

    /// Partial string matched against window titles (case-sensitive)
    window_name: String,

    /// Whether to open Copilot Chat with Cmd+Shift+I after focusing (default: true)
    #[serde(default = "default_true")]
    open_chat: bool,

    /// Text to paste into the chat input box after it opens
    chat_content: Option<String>,

    /// Milliseconds to wait between each step (default: 400)
    #[serde(default = "default_delay_ms")]
    step_delay_ms: u64,
}

#[derive(Debug, Deserialize)]
struct CloseWindowRequest {
    /// Process name of the application, e.g. "Code - Insiders"
    #[serde(default = "default_app_name")]
    app_name: String,

    /// Partial string matched against window titles (case-sensitive)
    window_name: String,
}

#[derive(Debug, Deserialize)]
struct ListSessionsRequest {
    /// Process name of the application, e.g. "Code - Insiders"
    #[serde(default = "default_app_name")]
    app_name: String,

    /// Partial string matched against window titles (case-sensitive)
    window_name: String,
}

#[derive(Debug, Deserialize)]
struct ListWindowsQuery {
    #[serde(default = "default_app_name")]
    app_name: String,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct WindowEntry {
    app_name: String,
    title: String,
}

#[derive(Serialize)]
struct WindowsResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    windows: Vec<WindowEntry>,
}

#[derive(Serialize)]
struct SessionInfo {
    id: String,
    title: String,
}

#[derive(Serialize)]
struct SessionsResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    sessions: Vec<SessionInfo>,
}

#[derive(Debug, Deserialize)]
struct SessionDetailRequest {
    /// Process name of the application, e.g. "Code - Insiders"
    #[serde(default = "default_app_name")]
    app_name: String,
    /// UUID of the session (filename stem of the .jsonl file)
    session_id: String,
}

#[derive(Serialize, Clone)]
struct ChatMessage {
    role: String,  // "user" | "assistant"
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,
}

#[derive(Serialize)]
struct SessionDetailResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    title: String,
    messages: Vec<ChatMessage>,
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

fn default_app_name() -> String {
    "Code - Insiders".to_string()
}

fn default_true() -> bool {
    true
}

fn default_delay_ms() -> u64 {
    400
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate strings that will be embedded inside AppleScript double-quoted
/// string literals.  Reject characters that could escape the literal and
/// allow arbitrary AppleScript execution.
fn validate_as_string_arg(label: &str, value: &str) -> Result<(), ApiResponse> {
    for ch in value.chars() {
        if ch == '"' || ch == '\\' {
            return Err(ApiResponse {
                success: false,
                message: format!("'{label}' contains a disallowed character (\" or \\)"),
            });
        }
    }
    if value.is_empty() {
        return Err(ApiResponse {
            success: false,
            message: format!("'{label}' must not be empty"),
        });
    }
    Ok(())
}

/// Run an AppleScript snippet via `osascript -e`, with a 10-second timeout.
///
/// macOS TCC checks the *responsible process* hierarchy.  Because
/// `vscode-remote-control` is the parent that spawns `osascript`, TCC grants
/// access when this binary is listed in
/// System Settings → Privacy & Security → Accessibility.
/// This mirrors how Terminal.app → osascript works.
async fn run_applescript(script: &str) -> Result<String, String> {
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output(),
    )
    .await
    .map_err(|_| {
        "osascript timed out (10s) — ensure the binary is added to \
         System Settings → Privacy & Security → Accessibility, \
         then restart the service"
            .to_string()
    })?
    .map_err(|e| format!("Failed to spawn osascript: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Write `content` to the system clipboard via a temp file + osascript.
async fn set_clipboard(content: &str) -> Result<(), String> {
    let tmp_path = "/tmp/vscode_rc_clipboard.txt";
    std::fs::write(tmp_path, content).map_err(|e| format!("Failed to write temp file: {e}"))?;

    let script = format!(
        r#"set fileContent to (read POSIX file "{tmp_path}" as «class utf8»)
set the clipboard to fileContent"#,
        tmp_path = tmp_path
    );

    let result = run_applescript(&script).await;
    let _ = std::fs::remove_file(tmp_path);
    result.map(|_| ())
}

async fn app_process_exists(app_name: &str) -> Result<bool, String> {
    let script = format!(
        r#"tell application "System Events"
  return (count of (every process whose name is "{app_name}")) > 0
end tell"#,
        app_name = app_name,
    );

    let output = run_applescript(&script).await?;
    Ok(output.trim() == "true")
}

fn alternate_app_name(app_name: &str) -> Option<&'static str> {
    match app_name {
        "Code - Insiders" => Some("Code"),
        "Code" => Some("Code - Insiders"),
        _ => None,
    }
}

fn candidate_app_names(app_name: &str) -> Vec<&str> {
    let mut names = vec![app_name];
    if let Some(alternate) = alternate_app_name(app_name) {
        names.push(alternate);
    }
    names
}

async fn resolve_running_app_names(app_name: &str) -> Result<Vec<String>, ApiResponse> {
    let mut running = Vec::new();

    for candidate in candidate_app_names(app_name) {
        if app_process_exists(candidate)
            .await
            .map_err(|e| ApiResponse {
                success: false,
                message: format!("检查进程状态失败：{e}"),
            })?
        {
            running.push(candidate.to_string());
        }
    }

    if !running.is_empty() {
        return Ok(running);
    }

    Err(ApiResponse {
        success: false,
        message: format!(
            "未找到进程 '{}'，也未找到 '{}'。请确保已启动 VS Code 或 VS Code Insiders。",
            app_name,
            alternate_app_name(app_name).unwrap_or("Code 或 Code - Insiders")
        ),
    })
}

async fn list_windows_for_app(app_name: &str) -> Result<Vec<String>, String> {
    let script = format!(
        r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set output to ""
  repeat with w in (every window of vsProc)
    try
      set t to title of w
      if t is not "" then
        set output to output & t & linefeed
      end if
    end try
  end repeat
  return output
end tell"#,
        app_name = app_name,
    );

    let output = run_applescript(&script).await?;
    Ok(output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect())
}

async fn resolve_window_app_name(
    preferred_app_name: &str,
    window_name: &str,
) -> Result<String, ApiResponse> {
    let running_apps = resolve_running_app_names(preferred_app_name).await?;

    for app_name in &running_apps {
        let windows = list_windows_for_app(app_name)
            .await
            .map_err(|e| ApiResponse {
                success: false,
                message: format!("读取窗口列表失败：{e}"),
            })?;

        if windows.iter().any(|title| title.contains(window_name)) {
            return Ok(app_name.clone());
        }
    }

    Err(ApiResponse {
        success: false,
        message: format!(
            "未在 {} 中找到包含 '{}' 的窗口。",
            running_apps.join(" / "),
            window_name
        ),
    })
}

async fn close_window_for_app(app_name: &str, window_name: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set targetWin to (first window of vsProc whose title contains "{window_name}")
    perform action "AXRaise" of targetWin
    set frontmost of vsProc to true
    keystroke "w" using {{command down}}
end tell"#,
        app_name = app_name,
        window_name = window_name,
    );

    run_applescript(&script).await.map(|_| ())
}

async fn list_sessions_for_app(
    app_name: &str,
    window_name: &str,
) -> Result<Vec<SessionInfo>, String> {
    use std::path::Path;

    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/unknown".to_string());

    let user_data = if app_name.to_lowercase().contains("insiders") {
        format!("{home}/Library/Application Support/Code - Insiders/User")
    } else {
        format!("{home}/Library/Application Support/Code/User")
    };

    let ws_storage = format!("{user_data}/workspaceStorage");
    if !Path::new(&ws_storage).is_dir() {
        return Err(format!("VS Code workspaceStorage not found: {ws_storage}"));
    }

    // VS Code window titles (as returned by macOS AppleScript) have the form:
    //   "folder-name"               (folder-only window)
    //   "file.rs — folder-name"     (file open in workspace)
    // Try every segment as a potential folder name.
    let candidates: Vec<&str> = window_name.split(" \u{2014} ").collect();

    // Scan workspaceStorage dirs; each has a workspace.json with the folder URL.
    // Match the last URL segment against our candidates to identify the workspace.
    let mut matched_uuid: Option<String> = None;
    'outer: for entry in std::fs::read_dir(&ws_storage)
        .map_err(|e| format!("Cannot read workspaceStorage: {e}"))?
        .flatten()
    {
        let ws_json_path = entry.path().join("workspace.json");
        if !ws_json_path.is_file() {
            continue;
        }
        let content = match std::fs::read_to_string(&ws_json_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let val: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        // folder is a file:// URL, e.g. "file:///Users/.../folder-name"
        let folder_url = match val.get("folder").and_then(|f| f.as_str()) {
            Some(f) => f,
            None => continue,
        };
        let last_seg = folder_url.trim_end_matches('/').rsplit('/').next().unwrap_or("");
        if last_seg.is_empty() {
            continue;
        }
        for candidate in &candidates {
            if *candidate == last_seg {
                matched_uuid = Some(entry.file_name().to_string_lossy().to_string());
                break 'outer;
            }
        }
    }

    let uuid = matched_uuid
        .ok_or_else(|| format!("No workspace storage found for window: {window_name}"))?;

    let db_path = format!("{ws_storage}/{uuid}/state.vscdb");

    // SQLite access is blocking; run on the thread pool.
    let result = tokio::task::spawn_blocking(move || -> Result<Vec<SessionInfo>, String> {
        let conn = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("Cannot open state.vscdb: {e}"))?;

        let value: String = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = 'chat.ChatSessionStore.index'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Cannot read chat sessions from DB: {e}"))?;

        let json: serde_json::Value = serde_json::from_str(&value)
            .map_err(|e| format!("Cannot parse chat session index JSON: {e}"))?;

        let entries = json
            .get("entries")
            .and_then(|e| e.as_object())
            .ok_or_else(|| "No 'entries' object in chat session index".to_string())?;

        let mut sessions: Vec<(i64, SessionInfo)> = entries
            .values()
            .filter_map(|entry| {
                let id = entry.get("sessionId")?.as_str()?.to_string();
                let title = entry
                    .get("title")
                    .and_then(|t| t.as_str())
                    .filter(|t| !t.is_empty())
                    .unwrap_or("New Chat")
                    .to_string();
                let ts = entry
                    .get("lastMessageDate")
                    .and_then(|t| t.as_i64())
                    .unwrap_or(0);
                Some((ts, SessionInfo { id, title }))
            })
            .collect();

        // Newest first.
        sessions.sort_by(|a, b| b.0.cmp(&a.0));

        Ok(sessions.into_iter().map(|(_, s)| s).collect())
    })
    .await
    .map_err(|e| format!("Thread pool error: {e}"))??;

    Ok(result)
}

/// Load all turns of a specific session from its JSONL snapshot file.
async fn get_session_detail(
    app_name: &str,
    session_id: &str,
) -> Result<SessionDetailResponse, String> {
    // Only allow UUID-shaped IDs (hex + hyphens) to prevent path traversal.
    if session_id.is_empty()
        || !session_id
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-')
    {
        return Err("Invalid session_id".to_string());
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/unknown".to_string());
    let user_data = if app_name.to_lowercase().contains("insiders") {
        format!("{home}/Library/Application Support/Code - Insiders/User")
    } else {
        format!("{home}/Library/Application Support/Code/User")
    };

    let ws_storage = format!("{user_data}/workspaceStorage");
    let ws_path = std::path::Path::new(&ws_storage);
    if !ws_path.is_dir() {
        return Err(format!("VS Code workspace storage not found: {ws_storage}"));
    }

    // Walk all workspace directories looking for a matching session file.
    let mut session_path: Option<std::path::PathBuf> = None;
    for ws_entry in std::fs::read_dir(ws_path)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let chat_dir = ws_entry.path().join("chatSessions");
        if !chat_dir.is_dir() {
            continue;
        }
        let jsonl = chat_dir.join(format!("{session_id}.jsonl"));
        if jsonl.exists() {
            session_path = Some(jsonl);
            break;
        }
        let json = chat_dir.join(format!("{session_id}.json"));
        if json.exists() {
            session_path = Some(json);
            break;
        }
    }

    let path = session_path
        .ok_or_else(|| format!("Session not found: {session_id}"))?;

    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // For JSONL pick the last kind=0 entry (most-recent full snapshot).
    let outer: serde_json::Value = if ext == "jsonl" {
        let mut best: Option<serde_json::Value> = None;
        for line in content.lines() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if v.get("kind").and_then(|k| k.as_u64()) == Some(0) {
                    best = Some(v);
                }
            }
        }
        best.ok_or_else(|| "No snapshot line found in session file".to_string())?
    } else {
        serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| e.to_string())?
    };

    let v = outer.get("v").unwrap_or(&outer);

    let title = v
        .get("title")
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .unwrap_or_else(|| session_id.to_string());

    let requests = v
        .get("requests")
        .and_then(|r| r.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);

    let mut messages: Vec<ChatMessage> = Vec::new();

    for req in requests {
        // -- User turn -------------------------------------------------------
        let user_text = req
            .get("message")
            .and_then(|m| {
                // Current VS Code: message is a plain string.
                m.as_str()
                    .map(|s| s.to_string())
                    // Older format: { text: "...", parts: [...] }.
                    .or_else(|| {
                        m.get("text")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string())
                    })
            })
            .unwrap_or_default();

        let ts = req.get("timestamp").and_then(|t| t.as_i64());

        if !user_text.is_empty() {
            messages.push(ChatMessage {
                role: "user".to_string(),
                text: user_text,
                timestamp: ts,
            });
        }

        // -- Assistant turn --------------------------------------------------
        // Response items that carry text have no "kind" field (or kind is absent).
        let assistant_text: String = req
            .get("response")
            .and_then(|r| r.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item.get("kind").is_none())
                    .filter_map(|item| item.get("value").and_then(|v| v.as_str()))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        if !assistant_text.is_empty() {
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                text: assistant_text,
                timestamp: None,
            });
        }
    }

    Ok(SessionDetailResponse {
        success: true,
        message: None,
        title,
        messages,
    })
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/focus
///
/// Focus a window by partial title match, optionally open Copilot Chat,
/// and optionally paste text into the chat input.
async fn handle_focus(Json(req): Json<FocusRequest>) -> (StatusCode, Json<ApiResponse>) {
    macro_rules! bad_req {
        ($msg:expr) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: $msg,
                }),
            )
        };
    }
    macro_rules! server_err {
        ($msg:expr) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: $msg,
                }),
            )
        };
    }

    // ── Validate inputs ──────────────────────────────────────────────────────
    if let Err(e) = validate_as_string_arg("app_name", &req.app_name) {
        bad_req!(e.message);
    }
    if let Err(e) = validate_as_string_arg("window_name", &req.window_name) {
        bad_req!(e.message);
    }

    let app_name = match resolve_window_app_name(&req.app_name, &req.window_name).await {
        Ok(name) => name,
        Err(err) => return (StatusCode::BAD_REQUEST, Json(err)),
    };

    // ── Step 1: raise & focus the target window ───────────────────────────────
    let focus_script = format!(
        r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set targetWin to (first window of vsProc whose title contains "{window_name}")
  perform action "AXRaise" of targetWin
  set frontmost of vsProc to true
end tell"#,
        app_name = app_name,
        window_name = req.window_name,
    );

    if let Err(e) = run_applescript(&focus_script).await {
        bad_req!(format!("Focus failed: {e}"));
    }

    // ── Step 2: open Copilot Chat (Cmd+Shift+I) ──────────────────────────────
    if req.open_chat || req.chat_content.is_some() {
        sleep(Duration::from_millis(req.step_delay_ms)).await;

        let open_chat_script = format!(
            r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set frontmost of vsProc to true
  keystroke "i" using {{command down, shift down}}
end tell"#,
            app_name = app_name,
        );

        if let Err(e) = run_applescript(&open_chat_script).await {
            server_err!(format!("Open chat failed: {e}"));
        }
    }

    // ── Step 3: paste chat content via clipboard ──────────────────────────────
    if let Some(ref content) = req.chat_content {
        sleep(Duration::from_millis(req.step_delay_ms)).await;

        if let Err(e) = set_clipboard(content).await {
            server_err!(format!("Clipboard error: {e}"));
        }

        // Small extra pause so the chat input is fully focused before paste
        sleep(Duration::from_millis(150)).await;

        let paste_script = format!(
            r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set frontmost of vsProc to true
  keystroke "v" using {{command down}}
end tell"#,
            app_name = app_name,
        );

        if let Err(e) = run_applescript(&paste_script).await {
            server_err!(format!("Paste failed: {e}"));
        }

        // Wait 3 seconds then press Enter to trigger agent response
        sleep(Duration::from_millis(3000)).await;

        let enter_script = format!(
            r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set frontmost of vsProc to true
  key code 36
end tell"#,
            app_name = app_name,
        );

        if let Err(e) = run_applescript(&enter_script).await {
            server_err!(format!("Enter key failed: {e}"));
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "OK".to_string(),
        }),
    )
}

/// POST /api/close-window
///
/// Close a VS Code window by partial title match.
async fn handle_close_window(
    Json(req): Json<CloseWindowRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    macro_rules! bad_req {
        ($msg:expr) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: $msg,
                }),
            )
        };
    }

    if let Err(e) = validate_as_string_arg("app_name", &req.app_name) {
        bad_req!(e.message);
    }
    if let Err(e) = validate_as_string_arg("window_name", &req.window_name) {
        bad_req!(e.message);
    }

    let app_name = match resolve_window_app_name(&req.app_name, &req.window_name).await {
        Ok(name) => name,
        Err(err) => return (StatusCode::BAD_REQUEST, Json(err)),
    };

    if let Err(e) = close_window_for_app(&app_name, &req.window_name).await {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: format!("Close window failed: {e}"),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "OK".to_string(),
        }),
    )
}

/// POST /api/list-sessions
///
/// List visible session entries for a VS Code window by opening the built-in
/// session picker and scraping its accessibility tree.
async fn handle_list_sessions(
    Json(req): Json<ListSessionsRequest>,
) -> (StatusCode, Json<SessionsResponse>) {
    macro_rules! bad_req {
        ($msg:expr) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SessionsResponse {
                    success: false,
                    message: Some($msg),
                    sessions: vec![],
                }),
            )
        };
    }

    if let Err(e) = validate_as_string_arg("app_name", &req.app_name) {
        bad_req!(e.message);
    }

    // Session files are read from disk; no open window is required.
    // Use the supplied app_name directly so that VS Code doesn't need to
    // be focused or have any particular window open.
    match list_sessions_for_app(&req.app_name, &req.window_name).await {
        Ok(sessions) => (
            StatusCode::OK,
            Json(SessionsResponse {
                success: true,
                message: None,
                sessions,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(SessionsResponse {
                success: false,
                message: Some(format!("List sessions failed: {e}")),
                sessions: vec![],
            }),
        ),
    }
}

/// POST /api/session-detail
///
/// Return the full conversation of a single chat session.
async fn handle_session_detail(
    Json(req): Json<SessionDetailRequest>,
) -> (StatusCode, Json<SessionDetailResponse>) {
    macro_rules! bad_req {
        ($msg:expr) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SessionDetailResponse {
                    success: false,
                    message: Some($msg),
                    title: String::new(),
                    messages: vec![],
                }),
            )
        };
    }

    if let Err(e) = validate_as_string_arg("app_name", &req.app_name) {
        bad_req!(e.message);
    }
    if req.session_id.is_empty() {
        bad_req!("session_id must not be empty".to_string());
    }

    match get_session_detail(&req.app_name, &req.session_id).await {
        Ok(detail) => (StatusCode::OK, Json(detail)),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(SessionDetailResponse {
                success: false,
                message: Some(e),
                title: String::new(),
                messages: vec![],
            }),
        ),
    }
}

/// GET /api/windows?app_name=Code%20-%20Insiders
///
/// List all window titles of a running application.
async fn handle_list_windows(
    Query(q): Query<ListWindowsQuery>,
) -> (StatusCode, Json<WindowsResponse>) {
    if let Err(e) = validate_as_string_arg("app_name", &q.app_name) {
        return (
            StatusCode::BAD_REQUEST,
            Json(WindowsResponse {
                success: false,
                message: Some(e.message),
                windows: vec![],
            }),
        );
    }

    let app_names = match resolve_running_app_names(&q.app_name).await {
        Ok(names) => names,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(WindowsResponse {
                    success: false,
                    message: Some(err.message),
                    windows: vec![],
                }),
            )
        }
    };

    let mut windows = Vec::new();
    for app_name in app_names {
        match list_windows_for_app(&app_name).await {
            Ok(app_windows) => {
                windows.extend(app_windows.into_iter().map(|title| WindowEntry {
                    app_name: app_name.clone(),
                    title,
                }));
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(WindowsResponse {
                        success: false,
                        message: Some(e),
                        windows: vec![],
                    }),
                )
            }
        }
    }

    windows.sort_by(|left, right| {
        left.title
            .cmp(&right.title)
            .then(left.app_name.cmp(&right.app_name))
    });

    (
        StatusCode::OK,
        Json(WindowsResponse {
            success: true,
            message: None,
            windows,
        }),
    )
}

/// GET /health
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/windows", get(handle_list_windows))
        .route("/api/focus", post(handle_focus))
        .route("/api/close-window", post(handle_close_window))
        .route("/api/list-sessions", post(handle_list_sessions))
        .route("/api/session-detail", post(handle_session_detail))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("vscode-remote-control listening on http://0.0.0.0:{port}");
    println!();
    println!("Endpoints:");
    println!("  GET  /health");
    println!("  GET  /api/windows?app_name=<name>");
    println!("  POST /api/focus   (JSON body)");
    println!("  POST /api/close-window   (JSON body)");
    println!("  POST /api/list-sessions  (JSON body)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
