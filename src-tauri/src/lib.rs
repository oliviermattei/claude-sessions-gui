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
    color: Option<String>, // Claude "agent-color" (red/blue/…) or None
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
    let mut agent_color: Option<String> = None;
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
            // Claude writes the session's accent color as its own row; last wins.
            "agent-color" => {
                if let Some(c) = v.get("agentColor").and_then(|c| c.as_str()) {
                    agent_color = if c == "default" { None } else { Some(c.to_string()) };
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
        color: agent_color,
    })
}

// --- Transcript reading (iteration 3: "Lire") -------------------------------

/// A single content block inside a message. Tagged by `kind` for the frontend's
/// discriminated union.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Block {
    Text {
        text: String,
    },
    Thinking {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        #[serde(rename = "toolUseId")]
        tool_use_id: String,
        text: String,
        #[serde(rename = "isError")]
        is_error: bool,
    },
}

#[derive(Serialize)]
struct Message {
    role: String, // "user" | "assistant"
    ts: i64,      // timestamp ms (0 if missing)
    blocks: Vec<Block>,
}

#[derive(Serialize)]
struct Subagent {
    id: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Transcript {
    messages: Vec<Message>,
    subagents: Vec<Subagent>,
}

/// Flatten a message `content` (string or array of blocks) into UI blocks.
fn content_to_blocks(content: &serde_json::Value) -> Vec<Block> {
    let mut blocks = Vec::new();

    if let Some(s) = content.as_str() {
        if !s.trim().is_empty() {
            blocks.push(Block::Text { text: s.to_string() });
        }
        return blocks;
    }

    let Some(arr) = content.as_array() else {
        return blocks;
    };
    for b in arr {
        match b.get("type").and_then(|t| t.as_str()).unwrap_or("") {
            "text" => {
                if let Some(t) = b.get("text").and_then(|x| x.as_str()) {
                    blocks.push(Block::Text { text: t.to_string() });
                }
            }
            "thinking" => {
                if let Some(t) = b.get("thinking").and_then(|x| x.as_str()) {
                    blocks.push(Block::Thinking { text: t.to_string() });
                }
            }
            "tool_use" => {
                blocks.push(Block::ToolUse {
                    id: b.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    name: b.get("name").and_then(|x| x.as_str()).unwrap_or("tool").to_string(),
                    input: b.get("input").cloned().unwrap_or(serde_json::Value::Null),
                });
            }
            "tool_result" => {
                blocks.push(Block::ToolResult {
                    tool_use_id: b
                        .get("tool_use_id")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    text: tool_result_text(b.get("content")),
                    is_error: b.get("is_error").and_then(|x| x.as_bool()).unwrap_or(false),
                });
            }
            _ => {}
        }
    }
    blocks
}

/// A tool_result's `content` is usually a string, sometimes an array of text blocks.
fn tool_result_text(content: Option<&serde_json::Value>) -> String {
    match content {
        Some(v) if v.is_string() => v.as_str().unwrap_or("").to_string(),
        Some(v) if v.is_array() => {
            let mut s = String::new();
            for b in v.as_array().unwrap() {
                if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(t) = b.get("text").and_then(|x| x.as_str()) {
                        s.push_str(t);
                        s.push('\n');
                    }
                }
            }
            s.trim_end().to_string()
        }
        _ => String::new(),
    }
}

/// Parse a transcript .jsonl into ordered messages (real user/assistant turns only).
fn parse_transcript_file(path: &Path) -> Vec<Message> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();

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
        if kind != "user" && kind != "assistant" {
            continue;
        }
        if v.get("isMeta").and_then(|b| b.as_bool()).unwrap_or(false) {
            continue;
        }

        let content = match v.get("message").and_then(|m| m.get("content")) {
            Some(c) => c,
            None => continue,
        };
        let blocks = content_to_blocks(content);
        if blocks.is_empty() {
            continue;
        }

        out.push(Message {
            role: kind.to_string(),
            ts: v
                .get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(iso_to_millis)
                .unwrap_or(0),
            blocks,
        });
    }
    out
}

/// Read a full session transcript plus any subagent sidechains stored alongside it
/// at `<parent>/<session-id>/subagents/*.jsonl`.
#[tauri::command]
fn get_session_messages(path: String) -> Result<Transcript, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("Session file not found".to_string());
    }

    let messages = parse_transcript_file(&p);

    let mut subagents = Vec::new();
    if let (Some(parent), Some(stem)) = (p.parent(), p.file_stem()) {
        let sdir = parent.join(stem).join("subagents");
        if sdir.is_dir() {
            if let Ok(entries) = fs::read_dir(&sdir) {
                let mut files: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|pp| pp.extension().and_then(|e| e.to_str()) == Some("jsonl"))
                    .collect();
                files.sort();
                for f in files {
                    let id = f
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let msgs = parse_transcript_file(&f);
                    if !msgs.is_empty() {
                        subagents.push(Subagent { id, messages: msgs });
                    }
                }
            }
        }
    }

    Ok(Transcript { messages, subagents })
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

/// Move a session .jsonl (and its `<id>/` sidecar dir, if any) into another
/// project's folder, rewriting the `cwd` field on every row so that
/// `claude --resume <id>` resolves the session in the new project.
///
/// `target_dir` is the destination project folder (parent of its sessions);
/// `target_cwd` is the absolute working directory written into the transcript.
/// Returns the new absolute .jsonl path.
#[tauri::command]
fn move_session(path: String, target_dir: String, target_cwd: String) -> Result<String, String> {
    let src = PathBuf::from(&path);
    if !src.exists() {
        return Err("Session file not found".to_string());
    }
    let dst_dir = PathBuf::from(&target_dir);
    if !dst_dir.is_dir() {
        return Err("Target project folder not found".to_string());
    }
    let file_name = src
        .file_name()
        .ok_or_else(|| "Invalid session path".to_string())?;
    let dst = dst_dir.join(file_name);
    if dst == src {
        return Ok(path); // already there
    }
    if dst.exists() {
        return Err("A session with this id already exists in the target project".to_string());
    }

    // Rewrite cwd on every row that carries one, then write the new file.
    let content = fs::read_to_string(&src).map_err(|e| e.to_string())?;
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(mut v) => {
                if v.get("cwd").is_some() {
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert(
                            "cwd".to_string(),
                            serde_json::Value::String(target_cwd.clone()),
                        );
                    }
                }
                out.push_str(&serde_json::to_string(&v).map_err(|e| e.to_string())?);
                out.push('\n');
            }
            // Preserve non-JSON lines verbatim rather than dropping data.
            Err(_) => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    fs::write(&dst, out).map_err(|e| e.to_string())?;

    // Move the `<id>/` sidecar directory (subagents, etc.) alongside it.
    if let (Some(parent), Some(stem)) = (src.parent(), src.file_stem()) {
        let side = parent.join(stem);
        if side.is_dir() {
            let dst_side = dst_dir.join(stem);
            if !dst_side.exists() {
                // Rename works within the same filesystem; ignore failures so a
                // cross-device sidecar doesn't abort the whole move.
                let _ = fs::rename(&side, &dst_side);
            }
        }
    }

    fs::remove_file(&src).map_err(|e| e.to_string())?;
    Ok(dst.to_string_lossy().to_string())
}

/// True if a transcript has zero real user/assistant turns (a stray/empty file
/// that `list_sessions` never surfaces). Cheap: bails on the first real turn.
fn is_empty_session(path: &Path) -> bool {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            _ => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let kind = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if kind == "user" || kind == "assistant" {
            let is_meta = v.get("isMeta").and_then(|b| b.as_bool()).unwrap_or(false);
            let is_side = v.get("isSidechain").and_then(|b| b.as_bool()).unwrap_or(false);
            if !is_meta && !is_side {
                return false; // found a real turn → not empty
            }
        }
    }
    true
}

/// Scan every project folder for transcripts with no real turns. Returns their
/// absolute paths (candidates for "Clean empty sessions").
#[tauri::command]
fn find_empty_sessions() -> Result<Vec<String>, String> {
    let root = projects_dir().ok_or_else(|| "Could not resolve home directory".to_string())?;
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let Ok(files) = fs::read_dir(&dir) else { continue };
        for f in files.flatten() {
            let p = f.path();
            if p.extension().and_then(|e| e.to_str()) == Some("jsonl") && is_empty_session(&p) {
                out.push(p.to_string_lossy().to_string());
            }
        }
    }
    Ok(out)
}

/// Send many sessions to the trash in one call (each with its `<id>/` sidecar).
/// Returns how many .jsonl files were trashed.
#[tauri::command]
fn trash_sessions(paths: Vec<String>) -> Result<usize, String> {
    let mut n = 0;
    for path in paths {
        let p = PathBuf::from(&path);
        if !p.exists() {
            continue;
        }
        if let (Some(parent), Some(stem)) = (p.parent(), p.file_stem()) {
            let side = parent.join(stem);
            if side.is_dir() {
                let _ = trash::delete(&side);
            }
        }
        if trash::delete(&p).is_ok() {
            n += 1;
        }
    }
    Ok(n)
}

/// Send a session .jsonl (and its `<id>/` sidecar dir, if any) to the OS trash.
/// Recoverable — nothing is permanently deleted.
#[tauri::command]
fn delete_session(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("Session file not found".to_string());
    }
    if let (Some(parent), Some(stem)) = (p.parent(), p.file_stem()) {
        let side = parent.join(stem);
        if side.is_dir() {
            let _ = trash::delete(&side);
        }
    }
    trash::delete(&p).map_err(|e| e.to_string())
}

/// Append JSONL rows to a transcript, one per line, without gluing onto a last
/// line that lacks a trailing newline. Claude reads the last matching row, so
/// appending is how it sets/overrides title, color, etc.
fn append_rows(p: &Path, rows: &[serde_json::Value]) -> Result<(), String> {
    use std::io::{Read, Seek, SeekFrom, Write};

    let needs_nl = {
        let mut f = fs::File::open(p).map_err(|e| e.to_string())?;
        let len = f.metadata().map_err(|e| e.to_string())?.len();
        if len == 0 {
            false
        } else {
            f.seek(SeekFrom::End(-1)).map_err(|e| e.to_string())?;
            let mut b = [0u8; 1];
            f.read_exact(&mut b).map_err(|e| e.to_string())?;
            b[0] != b'\n'
        }
    };

    let mut out = String::new();
    if needs_nl {
        out.push('\n');
    }
    for r in rows {
        out.push_str(&serde_json::to_string(r).map_err(|e| e.to_string())?);
        out.push('\n');
    }

    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(p)
        .map_err(|e| e.to_string())?;
    f.write_all(out.as_bytes()).map_err(|e| e.to_string())
}

fn session_id_of(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Set a session's accent color by appending a Claude-native `agent-color` row.
/// `list_sessions` reads the last such row, so this both sets and overrides.
/// Pass "default" to clear the color.
#[tauri::command]
fn set_session_color(path: String, color: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("Session file not found".to_string());
    }
    let id = session_id_of(&p);
    append_rows(
        &p,
        &[serde_json::json!({
            "type": "agent-color",
            "agentColor": color,
            "sessionId": id,
        })],
    )
}

/// Rename a session by appending the two Claude-native rows it writes on rename:
/// `custom-title` (drives the displayed title) and `agent-name`. Both carry the
/// same value so Claude's own UI stays consistent.
#[tauri::command]
fn set_session_title(path: String, title: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("Session file not found".to_string());
    }
    let title = title.trim();
    if title.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    let id = session_id_of(&p);
    append_rows(
        &p,
        &[
            serde_json::json!({
                "type": "custom-title",
                "customTitle": title,
                "sessionId": id,
            }),
            serde_json::json!({
                "type": "agent-name",
                "agentName": title,
                "sessionId": id,
            }),
        ],
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            get_session_messages,
            run_command,
            move_session,
            delete_session,
            set_session_color,
            set_session_title,
            find_empty_sessions,
            trash_sessions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
