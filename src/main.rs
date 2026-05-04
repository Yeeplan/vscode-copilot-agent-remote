use axum::{
    extract::{Json, Query},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    net::SocketAddr,
    process::{Command, Stdio},
};
use tokio::time::{sleep, Duration};

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
struct WindowsResponse {
    success: bool,
    windows: Vec<String>,
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
                message: format!(
                    "'{label}' contains a disallowed character (\" or \\)"
                ),
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

/// Run an AppleScript snippet via `osascript -e`.
fn run_applescript(script: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to spawn osascript: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Write `content` to the system clipboard using `pbcopy`.
/// This avoids embedding arbitrary text inside AppleScript string literals.
fn set_clipboard(content: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn pbcopy: {e}"))?;

    child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to pbcopy: {e}"))?;

    child
        .wait()
        .map_err(|e| format!("pbcopy wait failed: {e}"))?;

    Ok(())
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

    // ── Step 1: raise & focus the target window ───────────────────────────────
    let focus_script = format!(
        r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set targetWin to (first window of vsProc whose title contains "{window_name}")
  perform action "AXRaise" of targetWin
  set frontmost of vsProc to true
end tell"#,
        app_name = req.app_name,
        window_name = req.window_name,
    );

    if let Err(e) = run_applescript(&focus_script) {
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
            app_name = req.app_name,
        );

        if let Err(e) = run_applescript(&open_chat_script) {
            server_err!(format!("Open chat failed: {e}"));
        }
    }

    // ── Step 3: paste chat content via clipboard ──────────────────────────────
    if let Some(ref content) = req.chat_content {
        sleep(Duration::from_millis(req.step_delay_ms)).await;

        if let Err(e) = set_clipboard(content) {
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
            app_name = req.app_name,
        );

        if let Err(e) = run_applescript(&paste_script) {
            server_err!(format!("Paste failed: {e}"));
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
                windows: vec![e.message],
            }),
        );
    }

    let script = format!(
        r#"tell application "System Events"
  set vsProc to first process whose name is "{app_name}"
  set output to ""
  repeat with w in (every window of vsProc)
    try
      set t to title of w
      set output to output & t & linefeed
    end try
  end repeat
  return output
end tell"#,
        app_name = q.app_name,
    );

    match run_applescript(&script) {
        Ok(output) => {
            let windows: Vec<String> = output
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect();
            (
                StatusCode::OK,
                Json(WindowsResponse {
                    success: true,
                    windows,
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(WindowsResponse {
                success: false,
                windows: vec![e],
            }),
        ),
    }
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

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/windows", get(handle_list_windows))
        .route("/api/focus", post(handle_focus));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("vscode-remote-control listening on http://{addr}");
    println!();
    println!("Endpoints:");
    println!("  GET  /health");
    println!("  GET  /api/windows?app_name=<name>");
    println!("  POST /api/focus   (JSON body)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
