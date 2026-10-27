// Move — relocate a Session, with everything it owns, either to another Project
// of its Profile or to the same Project in another Profile.
//
// A Session is not one file. Inside its Profile it owns:
//   - `projects/<enc>/<id>.jsonl` and the `<id>/` sidecar next to it (subagents,
//     workflows, tool results…), possibly in several `<enc>` folders (a session
//     started in a worktree leaves stubs under the main repo's folder too), plus
//     discarded `<id>.orphaned-*` / `<id>.jsonl.superseded-*` transcripts;
//   - `file-history/<id>/` (the /rewind checkpoints), `session-env/<id>/` and a
//     few other per-session folders;
//   - `plans/<slug>.md` for the plan-mode slug its rows carry;
//   - its lines in `history.jsonl` (prompt recall), and the `paste-cache/`
//     files only those lines reference.
//
// A move is all or nothing: everything is copied to the target first (paths
// rewritten, see docs/adr/0001), history lines are moved, and only then is the
// source deleted. Any failure before that point removes what was copied and
// leaves the source untouched.
//
// A Session open in a running Claude Code process is never moved: Claude would
// keep appending to the old file.

use crate::profiles::{self, Profile};
use crate::{parse_session, retention};
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf, Prefix};
use std::time::SystemTime;

/// Per-session folders at the root of a Profile, named after the session id.
const SESSION_DIRS: &[&str] = &[
    "file-history",
    "session-env",
    "image-cache",
    "uploads",
    "dev-mods",
];
/// Root folders holding per-session files whose names start with the session id.
const SESSION_FILE_DIRS: &[&str] = &["todos", "debug"];

// --- path rewriting ----------------------------------------------------------

/// The parts of an absolute path: an optional drive letter and its segments.
fn split_path(p: &Path) -> (Option<char>, Vec<String>) {
    let mut drive = None;
    let mut segs = Vec::new();
    for c in p.components() {
        match c {
            Component::Prefix(pre) => {
                if let Prefix::Disk(d) | Prefix::VerbatimDisk(d) = pre.kind() {
                    drive = Some((d as char).to_ascii_uppercase());
                }
            }
            Component::Normal(s) => segs.push(s.to_string_lossy().to_string()),
            _ => {}
        }
    }
    (drive, segs)
}

/// Rewrites, inside raw JSON text, every spelling of a path that moved.
///
/// Claude writes the same path three ways — `C:\\Users\\…` (escaped in JSON),
/// `C:/Users/…` and `/c/Users/…` — so each rule matches any of them, and the
/// replacement keeps the spelling it found. A rule only fires on a whole path
/// segment: `…\.claude\projects` never matches inside `…\.claude-perso\…`.
pub(crate) struct Rewriter {
    rules: Vec<(Regex, PathBuf)>,
    cwd: Option<(Regex, String)>,
}

impl Rewriter {
    pub fn new() -> Rewriter {
        Rewriter {
            rules: Vec::new(),
            cwd: None,
        }
    }

    /// Rewrite `from` (and anything under it) to `to`.
    pub fn path(&mut self, from: &Path, to: &Path) {
        let (drive, segs) = split_path(from);
        if segs.is_empty() {
            return;
        }
        let sep = r"(?:\\+|/)";
        let mut pat = String::new();
        if cfg!(windows) {
            pat.push_str("(?i)");
        }
        match drive {
            Some(d) => pat.push_str(&format!(r"(?P<drive>{}:|/{})", d, d.to_ascii_lowercase())),
            None => pat.push_str("(?P<drive>)"),
        }
        for (i, s) in segs.iter().enumerate() {
            if i == 0 {
                pat.push_str(r"(?P<sep>\\+|/)");
            } else {
                pat.push_str(sep);
            }
            pat.push_str(&regex::escape(s));
        }
        // The segment must end here: no `-perso`, no longer id.
        pat.push_str(r"(?P<tail>[^A-Za-z0-9_\-]|$)");
        if let Ok(re) = Regex::new(&pat) {
            self.rules.push((re, to.to_path_buf()));
        }
    }

    /// Rewrite the `cwd` field of every row from `from` to `to`.
    pub fn cwd(&mut self, from: &str, to: &str) {
        let (Ok(f), Ok(t)) = (serde_json::to_string(from), serde_json::to_string(to)) else {
            return;
        };
        if let Ok(re) = Regex::new(&format!(r#""cwd"\s*:\s*{}"#, regex::escape(&f))) {
            self.cwd = Some((re, format!(r#""cwd":{t}"#)));
        }
    }

    pub fn apply(&self, text: &str) -> String {
        let mut out = text.to_string();
        for (re, to) in &self.rules {
            if !re.is_match(&out) {
                continue;
            }
            let (drive, segs) = split_path(to);
            out = re
                .replace_all(&out, |c: &regex::Captures| {
                    let found_drive = c.name("drive").map_or("", |m| m.as_str());
                    let sep = c.name("sep").map_or("/", |m| m.as_str());
                    let mut s = String::new();
                    if let Some(d) = drive {
                        if found_drive.starts_with('/') {
                            s.push('/');
                            s.push(d.to_ascii_lowercase());
                        } else {
                            s.push(d);
                            s.push(':');
                        }
                    }
                    for seg in &segs {
                        s.push_str(sep);
                        s.push_str(seg);
                    }
                    s.push_str(c.name("tail").map_or("", |m| m.as_str()));
                    s
                })
                .into_owned();
        }
        if let Some((re, to)) = &self.cwd {
            out = re.replace_all(&out, regex::NoExpand(to)).into_owned();
        }
        out
    }
}

// --- plan ----------------------------------------------------------------------

/// One file or folder to relocate.
struct Item {
    from: PathBuf,
    to: PathBuf,
    /// Transcript-like files under it get their paths rewritten.
    rewrite: bool,
}

#[derive(Default)]
struct Plan {
    items: Vec<Item>,
    /// `<enc>` folders of the source `projects/` the session had files in.
    source_encs: Vec<PathBuf>,
    /// The target already holds this session's transcript (identical): keep the
    /// target's copy of anything that exists on both sides.
    dedupe: bool,
}

fn name_of(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// True for the session's own entries in a folder: `<id>`, `<id>.jsonl`,
/// `<id>.orphaned-….jsonl`, `<id>.jsonl.superseded-…`.
fn belongs(name: &str, id: &str) -> bool {
    name == id || name.starts_with(&format!("{id}."))
}

/// Every `projects/<enc>/<id>.jsonl` of a Profile.
fn transcripts_of(projects: &Path, id: &str) -> Vec<PathBuf> {
    let Ok(encs) = fs::read_dir(projects) else {
        return vec![];
    };
    encs.flatten()
        .map(|e| e.path().join(format!("{id}.jsonl")))
        .filter(|p| p.is_file())
        .collect()
}

/// Plan-mode slugs carried by a transcript's rows.
fn slugs_of(transcript: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    let Ok(text) = fs::read_to_string(transcript) else {
        return out;
    };
    for line in text.lines().filter(|l| l.contains("\"slug\"")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(s) = v.get("slug").and_then(|s| s.as_str()) {
                if !s.is_empty() && !s.contains(['/', '\\', '.']) {
                    out.insert(s.to_string());
                }
            }
        }
    }
    out
}

/// What a move to another Profile carries, folder names unchanged.
fn plan_profile_move(src: &Profile, dst: &Profile, main: &Path, id: &str) -> Plan {
    let (sr, dr) = (src.root(), dst.root());
    let mut plan = Plan::default();

    if let Ok(encs) = fs::read_dir(src.projects_dir()) {
        for enc in encs.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
            let Ok(entries) = fs::read_dir(&enc) else {
                continue;
            };
            let mut touched = false;
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if belongs(&name, id) {
                    plan.items.push(Item {
                        from: e.path(),
                        to: dst.projects_dir().join(name_of(&enc)).join(&name),
                        rewrite: true,
                    });
                    touched = true;
                }
            }
            if touched {
                plan.source_encs.push(enc);
            }
        }
    }

    for d in SESSION_DIRS {
        let p = sr.join(d).join(id);
        if p.exists() {
            plan.items.push(Item {
                to: dr.join(d).join(id),
                from: p,
                rewrite: false,
            });
        }
    }
    for d in SESSION_FILE_DIRS {
        let Ok(entries) = fs::read_dir(sr.join(d)) else {
            continue;
        };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(id) {
                plan.items.push(Item {
                    from: e.path(),
                    to: dr.join(d).join(&name),
                    rewrite: false,
                });
            }
        }
    }

    let slugs = slugs_of(main);
    if !slugs.is_empty() {
        if let Ok(entries) = fs::read_dir(sr.join("plans")) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                // `<slug>.md`, and the `<slug>-agent-<agentId>.md` of its subagents.
                let owned = slugs.iter().any(|s| {
                    name == format!("{s}.md")
                        || (name.starts_with(&format!("{s}-agent-")) && name.ends_with(".md"))
                });
                if owned {
                    plan.items.push(Item {
                        from: e.path(),
                        to: dr.join("plans").join(&name),
                        rewrite: false,
                    });
                }
            }
        }
    }
    plan
}

/// What a move to another Project of the same Profile carries: the transcript,
/// its sidecar and discarded copies, from its own `<enc>` folder only. Stubs in
/// other folders belong to those folders' projects and stay.
fn plan_project_move(main: &Path, target_dir: &Path, id: &str) -> Plan {
    let mut plan = Plan::default();
    let Some(enc) = main.parent() else {
        return plan;
    };
    if let Ok(entries) = fs::read_dir(enc) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if belongs(&name, id) {
                plan.items.push(Item {
                    from: e.path(),
                    to: target_dir.join(&name),
                    rewrite: true,
                });
            }
        }
    }
    plan.source_encs.push(enc.to_path_buf());
    plan
}

// --- transaction ---------------------------------------------------------------

/// What a move created, so a failure can take it back.
#[derive(Default)]
struct Created {
    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
}

impl Created {
    /// Create `dir` and its missing parents, remembering each one.
    fn mkdir(&mut self, dir: &Path) -> Result<(), String> {
        let mut missing = Vec::new();
        let mut cur = Some(dir);
        while let Some(d) = cur {
            if d.exists() {
                break;
            }
            missing.push(d.to_path_buf());
            cur = d.parent();
        }
        for d in missing.into_iter().rev() {
            fs::create_dir(&d).map_err(|e| format!("{}: {e}", d.display()))?;
            self.dirs.push(d);
        }
        Ok(())
    }

    /// Remove what was created. Folders go only if empty: nothing that existed
    /// before the move is ever deleted.
    fn rollback(&self) {
        for f in self.files.iter().rev() {
            let _ = fs::remove_file(f);
        }
        for d in self.dirs.iter().rev() {
            let _ = fs::remove_dir(d);
        }
    }
}

fn is_transcript_like(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|e| e.to_str()),
        Some("jsonl") | Some("json")
    )
}

fn copy_file(
    from: &Path,
    to: &Path,
    rewrite: Option<&Rewriter>,
    dedupe: bool,
    created: &mut Created,
) -> Result<(), String> {
    let text = match rewrite {
        Some(rw) if is_transcript_like(from) => fs::read_to_string(from).ok().map(|t| rw.apply(&t)),
        _ => None,
    };
    if to.exists() {
        // Same content on both sides is not a conflict; the target's copy stays.
        let same = match &text {
            Some(t) => fs::read(to).is_ok_and(|b| b == t.as_bytes()),
            None => same_bytes(from, to),
        };
        if same || dedupe {
            return Ok(());
        }
        return Err(format!("{} already exists in the target", to.display()));
    }
    if let Some(parent) = to.parent() {
        created.mkdir(parent)?;
    }
    let expected = match &text {
        Some(t) => {
            fs::write(to, t).map_err(|e| format!("{}: {e}", to.display()))?;
            t.len() as u64
        }
        None => fs::copy(from, to).map_err(|e| format!("{}: {e}", to.display()))?,
    };
    created.files.push(to.to_path_buf());
    // Verify before anything is deleted at the source.
    let written = fs::metadata(to).map(|m| m.len()).unwrap_or(u64::MAX);
    if written != expected {
        return Err(format!("{}: incomplete copy", to.display()));
    }
    Ok(())
}

fn copy_tree(
    from: &Path,
    to: &Path,
    rewrite: Option<&Rewriter>,
    dedupe: bool,
    created: &mut Created,
) -> Result<(), String> {
    if from.is_dir() {
        created.mkdir(to)?;
        for e in fs::read_dir(from).map_err(|e| e.to_string())?.flatten() {
            copy_tree(&e.path(), &to.join(e.file_name()), rewrite, dedupe, created)?;
        }
        Ok(())
    } else {
        copy_file(from, to, rewrite, dedupe, created)
    }
}

pub(crate) fn same_bytes(a: &Path, b: &Path) -> bool {
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(ma), Ok(mb)) if ma.len() == mb.len() => {
            matches!((fs::read(a), fs::read(b)), (Ok(x), Ok(y)) if x == y)
        }
        _ => false,
    }
}

// --- history.jsonl ---------------------------------------------------------------

/// Rewrite a file Claude may append to at any moment. The new content is
/// written aside and swapped in only if the file did not change since it was
/// read; otherwise it is read again. Gives up after a few attempts rather than
/// lose a line another Claude process just wrote.
fn rewrite_shared<F>(path: &Path, f: F) -> Result<(), String>
where
    F: Fn(&str) -> Option<String>,
{
    let stamp = |p: &Path| -> Option<(u64, SystemTime)> {
        let m = fs::metadata(p).ok()?;
        Some((m.len(), m.modified().ok()?))
    };
    for _ in 0..3 {
        let before = stamp(path);
        let content = fs::read_to_string(path).unwrap_or_default();
        let Some(next) = f(&content) else {
            return Ok(());
        };
        let tmp = path.with_extension("jsonl.claude-sessions.tmp");
        fs::write(&tmp, &next).map_err(|e| e.to_string())?;
        if stamp(path) != before {
            let _ = fs::remove_file(&tmp);
            continue;
        }
        match fs::rename(&tmp, path) {
            Ok(()) => return Ok(()),
            Err(_) => {
                let _ = fs::remove_file(&tmp);
            }
        }
    }
    Err(format!(
        "{} kept changing (Claude Code is writing to it); try again",
        path.display()
    ))
}

fn row_session_id(line: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    v.get("sessionId")?.as_str().map(str::to_string)
}

fn row_timestamp(line: &str) -> Option<i64> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    v.get("timestamp")?.as_i64()
}

fn is_row_of(line: &str, id: &str) -> bool {
    line.contains(id) && row_session_id(line).as_deref() == Some(id)
}

fn join_lines(lines: &[&str]) -> String {
    let mut s = lines.join("\n");
    if !s.is_empty() {
        s.push('\n');
    }
    s
}

/// Insert `moved` (in time order) among `content`'s rows by timestamp, the
/// existing rows keeping their order. Rows already present are skipped.
pub(crate) fn merge_rows(content: &str, moved: &[String]) -> String {
    let existing: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    let present: HashSet<&str> = existing.iter().copied().collect();
    let mut incoming: Vec<(i64, &str)> = moved
        .iter()
        .filter(|l| !present.contains(l.as_str()))
        .map(|l| (row_timestamp(l).unwrap_or(0), l.as_str()))
        .collect();
    incoming.sort_by_key(|(t, _)| *t);

    let mut out: Vec<&str> = Vec::with_capacity(existing.len() + incoming.len());
    let mut next = incoming.into_iter().peekable();
    let mut last_ts = i64::MIN;
    for line in existing {
        let ts = row_timestamp(line).unwrap_or(last_ts);
        last_ts = ts;
        while let Some((t, l)) = next.peek() {
            if *t < ts {
                out.push(l);
                next.next();
            } else {
                break;
            }
        }
        out.push(line);
    }
    out.extend(next.map(|(_, l)| l));
    join_lines(&out)
}

/// Content hashes of the pastes a history row references.
fn paste_hashes(line: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return vec![];
    };
    let Some(map) = v.get("pastedContents").and_then(|p| p.as_object()) else {
        return vec![];
    };
    map.values()
        .filter_map(|p| p.get("contentHash").and_then(|h| h.as_str()))
        .filter(|h| !h.is_empty() && !h.contains(['/', '\\', '.']))
        .map(str::to_string)
        .collect()
}

// --- source cleanup ----------------------------------------------------------------

/// Drop the session from a project folder's legacy `sessions-index.json`.
fn drop_from_index(enc: &Path, id: &str) {
    let p = enc.join("sessions-index.json");
    let Ok(text) = fs::read_to_string(&p) else {
        return;
    };
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    let Some(entries) = v.get_mut("entries").and_then(|e| e.as_array_mut()) else {
        return;
    };
    let before = entries.len();
    entries.retain(|e| e.get("sessionId").and_then(|s| s.as_str()) != Some(id));
    if entries.len() == before {
        return;
    }
    if let Ok(body) = serde_json::to_string_pretty(&v) {
        let tmp = p.with_extension("json.tmp");
        if fs::write(&tmp, body).is_ok() {
            let _ = fs::rename(&tmp, &p);
        }
    }
}

fn remove_path(p: &Path) -> Result<(), String> {
    let r = if p.is_dir() {
        fs::remove_dir_all(p)
    } else {
        fs::remove_file(p)
    };
    r.map_err(|e| format!("{}: {e}", p.display()))
}

// --- live sessions -------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn pid_alive(pid: u64) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    match std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains(&format!("\"{pid}\"")),
        // Can't tell: assume it runs rather than move a session under its feet.
        Err(_) => true,
    }
}

#[cfg(not(target_os = "windows"))]
fn pid_alive(pid: u64) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(true)
}

/// Refuse when a running Claude Code process of this Profile has the session
/// open (`sessions/<pid>.json` names it and that pid is alive).
pub(crate) fn ensure_not_live(profile: &Profile, id: &str) -> Result<(), String> {
    let Ok(entries) = fs::read_dir(profile.root().join("sessions")) else {
        return Ok(());
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&p) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if v.get("sessionId").and_then(|s| s.as_str()) != Some(id) {
            continue;
        }
        let pid = v.get("pid").and_then(|n| n.as_u64()).or_else(|| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.parse().ok())
        });
        if pid.is_some_and(pid_alive) {
            return Err(
                "This session is open in a running Claude Code process. Close it first, then move it."
                    .to_string(),
            );
        }
    }
    Ok(())
}

// --- command ----------------------------------------------------------------------

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum MoveOutcome {
    /// Done. `path` is the transcript's new location; `warnings` lists source
    /// leftovers that could not be removed.
    #[serde(rename_all = "camelCase")]
    Moved { path: String, warnings: Vec<String> },
    /// The target Profile already holds this exact session: ask, then call
    /// again with `dedupe` to drop the source copy.
    Duplicate,
}

/// Move a session to another Project of its Profile (`target_dir` names the
/// project folder, `target_cwd` its working directory) or to the same Project
/// in another Profile (`target_profile` differs, no `target_dir`).
#[tauri::command(async)]
pub fn move_session(
    profile: String,
    path: String,
    target_profile: String,
    target_dir: Option<String>,
    target_cwd: Option<String>,
    dedupe: bool,
) -> Result<MoveOutcome, String> {
    let src = profiles::resolve(&profile)?;
    let dst = profiles::resolve(&target_profile)?;
    move_between(
        &src,
        &dst,
        PathBuf::from(&path),
        target_dir.map(PathBuf::from),
        target_cwd,
        dedupe,
    )
}

/// `move_session` on resolved Profiles.
fn move_between(
    src: &Profile,
    dst: &Profile,
    main: PathBuf,
    target_dir: Option<PathBuf>,
    target_cwd: Option<String>,
    dedupe: bool,
) -> Result<MoveOutcome, String> {
    let path = main.to_string_lossy().to_string();
    if !main.is_file() {
        return Err("Session file not found".to_string());
    }
    let id = main
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .ok_or_else(|| "Invalid session path".to_string())?;
    let old_cwd = parse_session(&main).map(|s| s.project).unwrap_or_default();
    ensure_not_live(src, &id)?;

    let cross = src.id != dst.id;
    let mut rewriter = Rewriter::new();
    let (mut plan, new_main, new_cwd) = if cross {
        if !dst.exists {
            return Err(format!(
                "The folder of profile \"{}\" no longer exists",
                dst.name
            ));
        }
        let existing = transcripts_of(&dst.projects_dir(), &id);
        let mut new_main = dst
            .projects_dir()
            .join(name_of(main.parent().unwrap_or(Path::new(""))))
            .join(name_of(&main));
        let mut dedupe_now = false;
        if !existing.is_empty() {
            match existing.iter().find(|f| same_bytes(f, &main)) {
                Some(twin) if dedupe => {
                    new_main = twin.clone();
                    dedupe_now = true;
                }
                Some(_) => return Ok(MoveOutcome::Duplicate),
                None => {
                    return Err(format!(
                        "A different session with this id already exists in \"{}\"",
                        dst.name
                    ))
                }
            }
        }
        let mut plan = plan_profile_move(src, dst, &main, &id);
        plan.dedupe = dedupe_now;
        (plan, new_main, old_cwd.clone())
    } else {
        let dir = target_dir.ok_or_else(|| "No target project".to_string())?;
        let cwd = target_cwd.ok_or_else(|| "No target project".to_string())?;
        if !dir.is_dir() || dir.parent() != Some(src.projects_dir().as_path()) {
            return Err("Target project folder not found".to_string());
        }
        let new_main = dir.join(name_of(&main));
        if new_main == main {
            return Ok(MoveOutcome::Moved {
                path,
                warnings: vec![],
            });
        }
        if new_main.exists() {
            return Err("A session with this id already exists in the target project".to_string());
        }
        rewriter.cwd(&old_cwd, &cwd);
        (plan_project_move(&main, &dir, &id), new_main, cwd)
    };
    if plan.items.is_empty() {
        return Err("Nothing to move".to_string());
    }
    for it in &plan.items {
        rewriter.path(&it.from, &it.to);
    }

    // History rows of the session, and the pastes only they can recall.
    let (sr, dr) = (src.root(), dst.root());
    let src_history = sr.join("history.jsonl");
    let moved_rows: Vec<String> = if cross {
        fs::read_to_string(&src_history)
            .unwrap_or_default()
            .lines()
            .filter(|l| is_row_of(l, &id))
            .map(str::to_string)
            .collect()
    } else {
        vec![]
    };
    let mut hashes: Vec<String> = moved_rows.iter().flat_map(|l| paste_hashes(l)).collect();
    hashes.sort();
    hashes.dedup();
    for h in &hashes {
        let f = sr.join("paste-cache").join(format!("{h}.txt"));
        if f.is_file() {
            plan.items.push(Item {
                to: dr.join("paste-cache").join(format!("{h}.txt")),
                from: f,
                rewrite: false,
            });
        }
    }

    // 1. Copy everything to the target. Nothing at the source is touched yet.
    let mut created = Created::default();
    let skip_projects = plan.dedupe;
    for it in &plan.items {
        // Deduplicating: the target already holds the transcript and its sidecar.
        if skip_projects && it.rewrite {
            continue;
        }
        let rw = it.rewrite.then_some(&rewriter);
        if let Err(e) = copy_tree(&it.from, &it.to, rw, plan.dedupe, &mut created) {
            created.rollback();
            return Err(format!("Move cancelled, nothing changed: {e}"));
        }
    }

    // 2. Move the history rows: into the target first, then out of the source.
    let mut warnings = Vec::new();
    if cross && !moved_rows.is_empty() {
        let dst_history = dr.join("history.jsonl");
        if let Err(e) = rewrite_shared(&dst_history, |c| Some(merge_rows(c, &moved_rows))) {
            created.rollback();
            return Err(format!("Move cancelled, nothing changed: {e}"));
        }
        let drop_rows = |c: &str| {
            let kept: Vec<&str> = c
                .lines()
                .filter(|l| !l.trim().is_empty() && !is_row_of(l, &id))
                .collect();
            Some(join_lines(&kept))
        };
        if let Err(e) = rewrite_shared(&src_history, drop_rows) {
            // Take the rows back out of the target; the source still has them.
            let _ = rewrite_shared(&dst_history, |c| {
                let mut extra: Vec<&str> = moved_rows.iter().map(String::as_str).collect();
                let kept: Vec<&str> = c
                    .lines()
                    .filter(|l| match extra.iter().position(|m| m == l) {
                        Some(i) => {
                            extra.remove(i);
                            false
                        }
                        None => !l.trim().is_empty(),
                    })
                    .collect();
                Some(join_lines(&kept))
            });
            created.rollback();
            return Err(format!("Move cancelled, nothing changed: {e}"));
        }
    } else if !cross && !old_cwd.is_empty() {
        // Same Profile: the rows stay, but recall them under the new project.
        let (Ok(from), Ok(to)) = (
            serde_json::to_string(&old_cwd),
            serde_json::to_string(&new_cwd),
        ) else {
            return Err("Invalid project path".to_string());
        };
        let (from, to) = (format!("\"project\":{from}"), format!("\"project\":{to}"));
        let r = rewrite_shared(&src_history, |c| {
            if !c.lines().any(|l| is_row_of(l, &id) && l.contains(&from)) {
                return None;
            }
            let lines: Vec<String> = c
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| {
                    if is_row_of(l, &id) {
                        l.replace(&from, &to)
                    } else {
                        l.to_string()
                    }
                })
                .collect();
            Some(join_lines(
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            ))
        });
        if let Err(e) = r {
            warnings.push(format!("prompt history not updated: {e}"));
        }
    }

    // 3. The target is complete: remove the source.
    for it in &plan.items {
        // A paste another session still recalls stays where it is.
        if it.from.starts_with(sr.join("paste-cache")) {
            let h = it
                .from
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let still_used = fs::read_to_string(&src_history)
                .unwrap_or_default()
                .lines()
                .any(|l| l.contains(&h));
            if still_used {
                continue;
            }
        }
        if let Err(e) = remove_path(&it.from) {
            warnings.push(e);
        }
    }
    for enc in &plan.source_encs {
        drop_from_index(enc, &id);
        // `remove_dir` only succeeds on an empty folder: a project still holding
        // other sessions (or its memory) stays.
        let _ = fs::remove_dir(enc);
    }

    // Tests run on temp folders only: never let them reach the real registry.
    if !cfg!(test) {
        retention::relocate(&src.id, &dst.id, &main, &new_main, &new_cwd);
    }

    Ok(MoveOutcome::Moved {
        path: new_main.to_string_lossy().to_string(),
        warnings,
    })
}

// --- tests -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A fresh temp directory, removed on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let d = std::env::temp_dir().join(format!("cs-mover-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&d);
            fs::create_dir_all(&d).expect("temp dir");
            Scratch(d)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const ID: &str = "11111111-2222-3333-4444-555555555555";
    const CWD: &str = "D:\\www\\app";

    fn profile(id: &str, root: &Path) -> Profile {
        fs::create_dir_all(root.join("projects")).unwrap();
        Profile {
            id: id.to_string(),
            name: id.to_string(),
            path: root.to_string_lossy().to_string(),
            color: None,
            hidden: false,
            manual: true,
            is_default: false,
            exists: true,
            resume_prefix: None,
        }
    }

    fn write(p: &Path, body: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn rows(values: &[serde_json::Value]) -> String {
        values.iter().map(|v| format!("{v}\n")).collect()
    }

    /// A path as it appears inside a JSON string.
    fn escaped(p: &Path) -> String {
        let s = serde_json::to_string(&p.to_string_lossy()).unwrap();
        s.trim_matches('"').to_string()
    }

    fn lines(p: &Path) -> Vec<String> {
        fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// A session with everything a real one can own, plus a neighbour.
    fn seed(src: &Path) -> PathBuf {
        let enc = src.join("projects").join("D--www-app");
        let side = enc.join(ID);
        let tool = side.join("tool-results").join("a.txt");
        let main = enc.join(format!("{ID}.jsonl"));
        let skill = src.join("skills").join("x").join("SKILL.md");
        write(
            &main,
            &rows(&[
                json!({"type":"user","cwd":CWD,"sessionId":ID,"slug":"brave-blue-fox",
                       "timestamp":"2026-01-01T00:00:00Z","message":{"role":"user","content":"hello"}}),
                json!({"type":"user","cwd":CWD,"sessionId":ID,"timestamp":"2026-01-01T00:01:00Z",
                       "toolUseResult":{"persistedOutputPath": tool.to_string_lossy()},
                       "message":{"role":"user","content":"x"}}),
                json!({"type":"file-history-snapshot","snapshot":{"trackedFileBackups":
                       {skill.to_string_lossy(): {"backupFileName":"abc@v1"}}}}),
                json!({"type":"assistant","cwd":CWD,"sessionId":ID,"timestamp":"2026-01-01T00:02:00Z",
                       "message":{"role":"assistant","content":"done"}}),
            ]),
        );
        write(&tool, "output");
        write(
            &side.join("subagents").join("agent-1.jsonl"),
            &rows(&[json!({"see": tool.to_string_lossy().replace('\\', "/")})]),
        );
        write(
            &src.join("projects")
                .join("D--www-wt")
                .join(format!("{ID}.jsonl")),
            &rows(&[json!({"type":"custom-title","customTitle":"T","sessionId":ID})]),
        );
        write(&enc.join("other.jsonl"), "{}\n");
        write(
            &enc.join("sessions-index.json"),
            &json!({"version":1,"entries":[{"sessionId":ID},{"sessionId":"other"}]}).to_string(),
        );
        write(&src.join("file-history").join(ID).join("abc@v1"), "backup");
        fs::create_dir_all(src.join("session-env").join(ID)).unwrap();
        write(&src.join("plans").join("brave-blue-fox.md"), "# plan");
        write(
            &src.join("history.jsonl"),
            &rows(&[
                json!({"display":"a","timestamp":1,"project":CWD,"sessionId":"other","pastedContents":{}}),
                json!({"display":"b","timestamp":2,"project":CWD,"sessionId":ID,
                       "pastedContents":{"1":{"contentHash":"h1"}}}),
                json!({"display":"c","timestamp":3,"project":CWD,"sessionId":"other",
                       "pastedContents":{"1":{"contentHash":"h2"}}}),
            ]),
        );
        write(&src.join("paste-cache").join("h1.txt"), "p1");
        write(&src.join("paste-cache").join("h2.txt"), "p2");
        // A process that recorded the session, long gone.
        write(
            &src.join("sessions").join("4000000000.json"),
            &json!({"pid": 4_000_000_000u64, "sessionId": ID}).to_string(),
        );
        main
    }

    #[test]
    fn a_profile_move_carries_everything_and_leaves_the_source_clean() {
        let s = Scratch::new("profile-move");
        let (sr, dr) = (s.0.join("src"), s.0.join("dst"));
        let (src, dst) = (profile("a", &sr), profile("b", &dr));
        let main = seed(&sr);
        write(
            &dr.join("history.jsonl"),
            &rows(&[json!({"display":"z","timestamp":1,"project":CWD,"sessionId":"x"})]),
        );

        let out = move_between(&src, &dst, main.clone(), None, None, false).unwrap();
        let new_main = dr
            .join("projects")
            .join("D--www-app")
            .join(format!("{ID}.jsonl"));
        match out {
            MoveOutcome::Moved { path, warnings } => {
                assert_eq!(PathBuf::from(path), new_main);
                assert!(warnings.is_empty(), "{warnings:?}");
            }
            MoveOutcome::Duplicate => panic!("not a duplicate"),
        }

        // Target: every artifact, session-owned paths pointing into it.
        let text = fs::read_to_string(&new_main).unwrap();
        let dst_tool = dr
            .join("projects")
            .join("D--www-app")
            .join(ID)
            .join("tool-results");
        assert!(text.contains(&escaped(&dst_tool.join("a.txt"))));
        assert!(!text.contains(&escaped(&sr.join("projects"))));
        assert!(
            text.contains(&escaped(&sr.join("skills").join("x").join("SKILL.md"))),
            "paths to files that stay must not be rewritten"
        );
        let sub = fs::read_to_string(
            dr.join("projects")
                .join("D--www-app")
                .join(ID)
                .join("subagents")
                .join("agent-1.jsonl"),
        )
        .unwrap();
        assert!(sub.contains(&dst_tool.to_string_lossy().replace('\\', "/")));
        assert!(dr
            .join("projects")
            .join("D--www-wt")
            .join(format!("{ID}.jsonl"))
            .is_file());
        assert!(dr.join("file-history").join(ID).join("abc@v1").is_file());
        assert!(dr.join("session-env").join(ID).is_dir());
        assert!(dr.join("plans").join("brave-blue-fox.md").is_file());
        assert!(dr.join("paste-cache").join("h1.txt").is_file());
        let h = lines(&dr.join("history.jsonl"));
        assert_eq!(h.len(), 2);
        assert!(
            h[0].contains("\"z\"") && h[1].contains(ID),
            "merged by time: {h:?}"
        );

        // Source: nothing of the session left, neighbours untouched.
        assert!(!main.exists());
        assert!(!sr.join("projects").join("D--www-app").join(ID).exists());
        assert!(
            !sr.join("projects").join("D--www-wt").exists(),
            "emptied folder removed"
        );
        assert!(sr
            .join("projects")
            .join("D--www-app")
            .join("other.jsonl")
            .is_file());
        assert!(!sr.join("file-history").join(ID).exists());
        assert!(!sr.join("session-env").join(ID).exists());
        assert!(!sr.join("plans").join("brave-blue-fox.md").exists());
        assert!(!sr.join("paste-cache").join("h1.txt").exists());
        assert!(
            sr.join("paste-cache").join("h2.txt").is_file(),
            "still recalled by another row"
        );
        let h = lines(&sr.join("history.jsonl"));
        assert_eq!(h.len(), 2);
        assert!(h.iter().all(|l| !l.contains(ID)));
        let index = fs::read_to_string(
            sr.join("projects")
                .join("D--www-app")
                .join("sessions-index.json"),
        )
        .unwrap();
        assert!(!index.contains(ID) && index.contains("other"));
    }

    #[test]
    fn an_identical_session_in_the_target_is_deduplicated_on_request() {
        let s = Scratch::new("dedupe");
        let (sr, dr) = (s.0.join("src"), s.0.join("dst"));
        let (src, dst) = (profile("a", &sr), profile("b", &dr));
        let main = seed(&sr);
        let twin = dr
            .join("projects")
            .join("D--elsewhere")
            .join(format!("{ID}.jsonl"));
        write(&twin, &fs::read_to_string(&main).unwrap());

        assert!(matches!(
            move_between(&src, &dst, main.clone(), None, None, false).unwrap(),
            MoveOutcome::Duplicate
        ));
        assert!(main.exists(), "nothing happens before the user agrees");

        match move_between(&src, &dst, main.clone(), None, None, true).unwrap() {
            MoveOutcome::Moved { path, .. } => assert_eq!(PathBuf::from(path), twin),
            MoveOutcome::Duplicate => panic!("dedupe was asked"),
        }
        assert!(!main.exists());
        assert!(!dr
            .join("projects")
            .join("D--www-app")
            .join(format!("{ID}.jsonl"))
            .exists());
        assert_eq!(transcripts_of(&dr.join("projects"), ID), vec![twin]);
    }

    #[test]
    fn a_different_session_with_the_same_id_blocks_the_move() {
        let s = Scratch::new("conflict");
        let (sr, dr) = (s.0.join("src"), s.0.join("dst"));
        let (src, dst) = (profile("a", &sr), profile("b", &dr));
        let main = seed(&sr);
        write(
            &dr.join("projects").join("D--x").join(format!("{ID}.jsonl")),
            "{}\n",
        );
        assert!(move_between(&src, &dst, main.clone(), None, None, false).is_err());
        assert!(main.exists());
    }

    #[test]
    fn a_live_session_is_never_moved() {
        let s = Scratch::new("live");
        let (sr, dr) = (s.0.join("src"), s.0.join("dst"));
        let (src, dst) = (profile("a", &sr), profile("b", &dr));
        let main = seed(&sr);
        write(
            &sr.join("sessions").join("1.json"),
            &json!({"pid": std::process::id(), "sessionId": ID}).to_string(),
        );
        assert!(move_between(&src, &dst, main.clone(), None, None, false).is_err());
        assert!(main.exists());
        assert!(!dr.join("projects").join("D--www-app").exists());
    }

    #[test]
    fn a_project_move_rewrites_cwd_paths_and_prompt_history() {
        let s = Scratch::new("project-move");
        let sr = s.0.join("src");
        let src = profile("a", &sr);
        let main = seed(&sr);
        let target = sr.join("projects").join("D--www-new");
        fs::create_dir_all(&target).unwrap();
        let new_cwd = "D:\\www\\new";

        move_between(
            &src,
            &src,
            main.clone(),
            Some(target.clone()),
            Some(new_cwd.to_string()),
            false,
        )
        .unwrap();

        let new_main = target.join(format!("{ID}.jsonl"));
        let text = fs::read_to_string(&new_main).unwrap();
        assert!(text.contains(r#""cwd":"D:\\www\\new""#));
        assert!(!text.contains(r#""cwd":"D:\\www\\app""#));
        let tool = target.join(ID).join("tool-results").join("a.txt");
        assert!(text.contains(&escaped(&tool)));
        assert!(tool.is_file());
        assert!(!main.exists());
        // The stub belongs to the other project's folder and stays there.
        assert!(sr
            .join("projects")
            .join("D--www-wt")
            .join(format!("{ID}.jsonl"))
            .is_file());
        let h = lines(&sr.join("history.jsonl"));
        assert!(h
            .iter()
            .any(|l| l.contains(ID) && l.contains(r#""project":"D:\\www\\new""#)));
        assert!(h
            .iter()
            .filter(|l| !l.contains(ID))
            .all(|l| l.contains(r#""project":"D:\\www\\app""#)));
    }

    #[cfg(windows)]
    #[test]
    fn rewrites_every_spelling_of_a_moved_path() {
        let mut rw = Rewriter::new();
        rw.path(
            Path::new(r"C:\Users\o\.claude\projects\D--www-app\abc"),
            Path::new(r"C:\Users\o\.claude-perso\projects\D--www-app\abc"),
        );
        let raw = r#"{"a":"C:\\Users\\o\\.claude\\projects\\D--www-app\\abc\\tool-results\\x.txt","b":"C:/Users/o/.claude/projects/D--www-app/abc.jsonl","c":"/c/Users/o/.claude/projects/D--www-app/abc/sub"}"#;
        let out = rw.apply(raw);
        assert!(out.contains(
            r#""C:\\Users\\o\\.claude-perso\\projects\\D--www-app\\abc\\tool-results\\x.txt""#
        ));
        assert!(out.contains(r#""C:/Users/o/.claude-perso/projects/D--www-app/abc.jsonl""#));
        assert!(out.contains(r#""/c/Users/o/.claude-perso/projects/D--www-app/abc/sub""#));
    }

    #[cfg(windows)]
    #[test]
    fn leaves_paths_that_did_not_move_alone() {
        let mut rw = Rewriter::new();
        rw.path(
            Path::new(r"C:\Users\o\.claude\projects\D--www-app\abc"),
            Path::new(r"C:\Users\o\.claude-perso\projects\D--www-app\abc"),
        );
        let raw = r#"{"skill":"C:\\Users\\o\\.claude\\skills\\x\\SKILL.md","other":"C:\\Users\\o\\.claude\\projects\\D--www-app\\abcdef.jsonl","perso":"C:\\Users\\o\\.claude-perso\\projects\\D--www-app\\abc.jsonl"}"#;
        assert_eq!(rw.apply(raw), raw);
    }

    #[test]
    fn rewrites_the_cwd_field_only() {
        let mut rw = Rewriter::new();
        rw.cwd(r"D:\www\old", r"D:\www\new");
        let raw = r#"{"cwd":"D:\\www\\old","text":"D:\\www\\old"}"#;
        assert_eq!(
            rw.apply(raw),
            r#"{"cwd":"D:\\www\\new","text":"D:\\www\\old"}"#
        );
    }

    #[test]
    fn history_rows_are_merged_in_time_order_without_duplicates() {
        let content =
            "{\"timestamp\":10,\"sessionId\":\"a\"}\n{\"timestamp\":30,\"sessionId\":\"a\"}\n";
        let moved = vec![
            "{\"timestamp\":20,\"sessionId\":\"b\"}".to_string(),
            "{\"timestamp\":40,\"sessionId\":\"b\"}".to_string(),
            "{\"timestamp\":10,\"sessionId\":\"a\"}".to_string(),
        ];
        assert_eq!(
            merge_rows(content, &moved),
            "{\"timestamp\":10,\"sessionId\":\"a\"}\n{\"timestamp\":20,\"sessionId\":\"b\"}\n{\"timestamp\":30,\"sessionId\":\"a\"}\n{\"timestamp\":40,\"sessionId\":\"b\"}\n"
        );
    }

    #[test]
    fn session_entries_are_recognised_by_name() {
        assert!(belongs("abc", "abc"));
        assert!(belongs("abc.jsonl", "abc"));
        assert!(belongs("abc.orphaned-1.jsonl", "abc"));
        assert!(!belongs("abcd.jsonl", "abc"));
        assert!(!belongs("memory", "abc"));
    }

    #[test]
    fn paste_hashes_come_from_pasted_contents() {
        let line = r#"{"pastedContents":{"1":{"contentHash":"deadbeef"},"2":{"content":"inline"}},"sessionId":"a"}"#;
        assert_eq!(paste_hashes(line), vec!["deadbeef".to_string()]);
    }
}
