// Claude Sessions — Tauri backend
//
// Two responsibilities:
//   1. `list_sessions`  — scan ~/.claude/projects/**/*.jsonl and extract session metadata
//   2. `run_command`    — spawn a configurable, detached shell command (the "Resume" action)
//
// Everything is read-only on the Claude data; we never mutate the .jsonl transcripts.

use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// One Claude Code session, flattened for the UI.
/// All timestamps are epoch milliseconds (i64) so the frontend can sort/format freely.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Session {
    id: String,           // session UUID (the .jsonl filename)
    title: String,        // custom title > last prompt > first user message > "Untitled"
    project: String,      // absolute cwd, e.g. "D:\\www\\claude-sessions"
    project_name: String, // last path segment, e.g. "claude-sessions"
    messages: u32,        // count of real user/assistant turns
    updated: i64,         // last message timestamp (fallback: file mtime)
    created: i64,         // first message timestamp (fallback: file mtime)
    modified: i64,        // file modification time
    path: String,         // absolute path to the .jsonl file
}

/// Parse an ISO-8601 timestamp (e.g. "2026-06-22T14:45:56.689Z") to epoch millis.
fn iso_to_millis(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

/// File mtime as epoch millis, or 0 if unavailable.
fn mtime_millis(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Extract a plain-text preview from a message `content` value, which may be
/// either a JSON string or an array of content blocks.
fn extract_text(content: &serde_json::Value) -> Option<String> {
    if let Some(s) = content.as_str() {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        for block in arr {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

/// Collapse whitespace and trim a candidate title to a sane single line.
fn clean_title(raw: &str) -> String {
    let one_line: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = one_line.trim();
    if trimmed.chars().count() > 120 {
        let cut = trimmed
            .char_indices()
            .nth(120)
            .map(|(i, _)| i)
            .unwrap_or(trimmed.len());
        format!("{}…", &trimmed[..cut])
    } else {
        trimmed.to_string()
    }
}

/// Parse a single .jsonl transcript into a Session, or None if it looks empty/invalid.
fn parse_session(path: &Path) -> Option<Session> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let id = path.file_stem()?.to_string_lossy().to_string();

    let mut custom_title: Option<String> = None;
    let mut last_prompt: Option<String> = None;
    let mut first_user_text: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut messages: u32 = 0;
    let mut first_ts: Option<i64> = None;
    let mut last_ts: Option<i64> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            _ => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let kind = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match kind {
            "custom-title" => {
                if let Some(t) = v.get("customTitle").and_then(|t| t.as_str()) {
                    custom_title = Some(t.to_string());
                }
            }
            "last-prompt" => {
                if let Some(p) = v.get("lastPrompt").and_then(|p| p.as_str()) {
                    last_prompt = Some(p.to_string());
                }
            }
            "user" | "assistant" => {
                // Skip meta rows and subagent sidechains when counting turns / picking a title,
                // but still let their timestamps and cwd through below.
                let is_meta = v.get("isMeta").and_then(|b| b.as_bool()).unwrap_or(false);
                let is_sidechain = v.get("isSidechain").and_then(|b| b.as_bool()).unwrap_or(false);
                if !is_meta && !is_sidechain {
                    messages += 1;
                    if kind == "user" && first_user_text.is_none() {
                        if let Some(content) = v.get("message").and_then(|m| m.get("content")) {
                            if let Some(txt) = extract_text(content) {
                                let t = txt.trim();
                                // Ignore command wrappers / reminders as a title source.
                                if !t.is_empty()
                                    && !t.starts_with("<system-reminder>")
                                    && !t.starts_with("<command-")
                                {
                                    first_user_text = Some(txt);
                                }
                            }
                        }
                    }
                }

                if cwd.is_none() {
                    if let Some(c) = v.get("cwd").and_then(|c| c.as_str()) {
                        cwd = Some(c.to_string());
                    }
                }
                if let Some(ts) =
                    v.get("timestamp").and_then(|t| t.as_str()).and_then(iso_to_millis)
                {
                    if first_ts.is_none() {
                        first_ts = Some(ts);
                    }
                    last_ts = Some(ts);
                }
            }
            _ => {
                // Pick up cwd from any row that carries it (system rows, etc.).
                if cwd.is_none() {
                    if let Some(c) = v.get("cwd").and_then(|c| c.as_str()) {
                        cwd = Some(c.to_string());
                    }
                }
            }
        }
    }

    // A transcript with no real turns is not worth listing.
    if messages == 0 {
        return None;
    }

    let modified = mtime_millis(path);
    let title = custom_title
        .or(last_prompt)
        .or(first_user_text)
        .map(|t| clean_title(&t))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Untitled session".to_string());

    let project = cwd.unwrap_or_else(|| "Unknown".to_string());
    let project_name = Path::new(&project)
        .components()
        .last()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| project.clone());

    Some(Session {
        id,
        title,
        project,
        project_name,
        messages,
        updated: last_ts.unwrap_or(modified),
        created: first_ts.unwrap_or(modified),
        modified,
        path: path.to_string_lossy().to_string(),
    })
}

fn projects_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("projects"))
}

/// Scan every project folder for .jsonl transcripts and return parsed sessions.
#[tauri::command]
fn list_sessions() -> Result<Vec<Session>, String> {
    let root = projects_dir().ok_or_else(|| "Could not resolve home directory".to_string())?;
    if !root.exists() {
        return Ok(vec![]);
    }

    let mut out = Vec::new();
    let entries = fs::read_dir(&root).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let files = match fs::read_dir(&dir) {
            Ok(f) => f,
            Err(_) => continue,
        };
        for f in files.flatten() {
            let p = f.path();
            if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                if let Some(session) = parse_session(&p) {
                    out.push(session);
                }
            }
        }
    }
    Ok(out)
}

/// Spawn a detached shell command (the "Resume" action). The frontend builds the
/// full command line from an editable template, so any launcher works without
/// recompiling: Warp, Windows Terminal, PowerShell, custom, etc.
#[tauri::command]
fn run_command(command: String, cwd: Option<String>) -> Result<(), String> {
    if command.trim().is_empty() {
        return Err("Empty command".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: don't flash a console for the transient `cmd` host.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/C").raw_arg(&command).creation_flags(CREATE_NO_WINDOW);
        if let Some(dir) = cwd.as_deref().filter(|d| !d.is_empty()) {
            cmd.current_dir(dir);
        }
        cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(&command);
        if let Some(dir) = cwd.as_deref().filter(|d| !d.is_empty()) {
            cmd.current_dir(dir);
        }
        cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_sessions, run_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
