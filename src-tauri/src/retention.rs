// Retention — keep chosen sessions alive past Claude Code's automatic cleanup.
//
// Claude Code purges transcripts on launch: it walks ~/.claude/projects and
// unlinks every file whose mtime is older than `cleanupPeriodDays`, sidecar
// `<id>/subagents/*.jsonl` included, then rmdir's what's left empty. The mtime
// is the *only* criterion, and when the period comes from managed policy the
// CLI doesn't even spare currently-open sessions.
//
// So two mechanisms, applied together:
//
//   1. keep-alive — refresh the mtime of retained transcripts (file + sidecar)
//      so the cutoff never reaches them. The session stays exactly where it is,
//      still listed by `claude --resume`. Only holds if this app is opened at
//      least once per retention period.
//   2. archive — mirror the transcript into our own vault, outside the scanned
//      tree, so a session survives even if nobody opens this app for weeks.
//      A vault copy can be restored back into ~/.claude/projects on demand.
//
// The registry (which sessions are retained, plus their true last-activity
// mtime) lives next to the vault in the OS app-data dir. Both are keyed by
// Profile: every Profile has its own cleanup period and its own transcripts,
// and the same session id can exist in two of them.

use crate::profiles::{self, Profile, DEFAULT_ID};
use crate::{mtime_millis, parse_session, Session};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// --- paths -----------------------------------------------------------------

/// `%APPDATA%/claude-sessions` (Windows), `~/.local/share/claude-sessions`, …
/// Deliberately inside the user profile: transcripts can hold confidential
/// material, so nothing lands in a shared or cloud-synced location.
pub(crate) fn app_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("claude-sessions"))
}

pub(crate) fn vault_dir() -> Option<PathBuf> {
    app_dir().map(|d| d.join("vault"))
}

fn registry_path() -> Option<PathBuf> {
    app_dir().map(|d| d.join("retention.json"))
}

// --- registry --------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Entry {
    pub id: String,
    /// Id of the Profile the session belongs to.
    pub profile: String,
    /// Name of the `<profile>/projects/<…>` folder the session belongs to.
    pub project_dir: String,
    /// Last known live path under `<profile>/projects` (may no longer exist).
    pub path: String,
    /// Session cwd, kept so a restore can be described without reading the file.
    pub cwd: String,
    /// True last-activity mtime, i.e. what the file would say if we never touched it.
    pub real_modified: i64,
    /// The mtime our last keep-alive wrote. `mtime != touched_to` means Claude
    /// (or this app) really wrote to the transcript since then.
    pub touched_to: i64,
    /// `real_modified` as of the last vault copy; 0 when never archived.
    pub archived_modified: i64,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Registry {
    version: u32,
    /// Keyed by `key(profile, id)`.
    entries: BTreeMap<String, Entry>,
}

const REGISTRY_VERSION: u32 = 2;

fn key(profile: &str, id: &str) -> String {
    format!("{profile}/{id}")
}

impl Registry {
    fn load() -> Registry {
        let Some(p) = registry_path() else {
            return Registry::default();
        };
        // A missing or corrupt registry is not an error: nothing is retained yet.
        let mut reg = fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str::<Registry>(&s).ok())
            .unwrap_or_default();
        if reg.version < REGISTRY_VERSION && !reg.entries.is_empty() {
            reg.migrate_to_profiles();
            let _ = reg.save();
        }
        reg
    }

    /// Version 1 knew only `~/.claude`: its entries, keyed by bare id, belong to
    /// the Default Profile, and their vault copies move under its folder.
    fn migrate_to_profiles(&mut self) {
        let old = std::mem::take(&mut self.entries);
        for (_, mut e) in old {
            if e.profile.is_empty() {
                e.profile = DEFAULT_ID.to_string();
                if let (Some(vault), Some((nf, nside))) =
                    (vault_dir(), vault_paths(&e.profile, &e.project_dir, &e.id))
                {
                    let root = vault.join(&e.project_dir);
                    let (of, oside) = (root.join(format!("{}.jsonl", e.id)), root.join(&e.id));
                    if let Some(parent) = nf.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if of.is_file() {
                        let _ = fs::rename(&of, &nf);
                    }
                    if oside.is_dir() {
                        let _ = fs::rename(&oside, &nside);
                    }
                    let _ = fs::remove_dir(&root);
                }
            }
            self.entries.insert(key(&e.profile, &e.id), e);
        }
    }

    fn save(&mut self) -> Result<(), String> {
        self.version = REGISTRY_VERSION;
        let p = registry_path().ok_or_else(|| "Could not resolve the app data dir".to_string())?;
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let body = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        // Write-then-rename so an interrupted save can't leave a half file that
        // would read back as "nothing is retained".
        let tmp = p.with_extension("json.tmp");
        fs::write(&tmp, body).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &p).map_err(|e| e.to_string())
    }
}

/// Commands run off the main thread, so a keep-alive pass can overlap a user
/// action. Every load → mutate → save of the registry holds this lock, otherwise
/// the last writer would silently drop the other's changes.
static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

fn lock_registry() -> MutexGuard<'static, ()> {
    // A panic mid-update leaves nothing worse than what is on disk: keep going.
    REGISTRY_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

// --- mtime helpers ---------------------------------------------------------

fn set_mtime(p: &Path, t: SystemTime) -> Result<(), String> {
    fs::OpenOptions::new()
        .write(true)
        .open(p)
        .map_err(|e| e.to_string())?
        .set_modified(t)
        .map_err(|e| e.to_string())
}

fn millis_to_time(ms: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(ms.max(0) as u64)
}

/// The `<id>/` sidecar directory sitting next to a transcript, if any.
fn sidecar_of(jsonl: &Path) -> Option<PathBuf> {
    let (parent, stem) = (jsonl.parent()?, jsonl.file_stem()?);
    let d = parent.join(stem);
    d.is_dir().then_some(d)
}

fn set_mtime_recursive(dir: &Path, t: SystemTime) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            set_mtime_recursive(&p, t);
        } else {
            // Best-effort: one unwritable sidecar must not abort the keep-alive.
            let _ = set_mtime(&p, t);
        }
    }
}

/// Push a transcript's mtime (and its sidecar's) to `t`. Returns the mtime the
/// filesystem actually stored, read back so later comparisons are exact.
fn touch_tree(jsonl: &Path, t: SystemTime) -> Result<i64, String> {
    set_mtime(jsonl, t)?;
    if let Some(side) = sidecar_of(jsonl) {
        set_mtime_recursive(&side, t);
    }
    Ok(mtime_millis(jsonl))
}

// --- vault -----------------------------------------------------------------

pub(crate) fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for e in fs::read_dir(src).map_err(|e| e.to_string())?.flatten() {
        let from = e.path();
        let to = dst.join(e.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn vault_paths(profile: &str, project_dir: &str, id: &str) -> Option<(PathBuf, PathBuf)> {
    let root = vault_dir()?.join(profile).join(project_dir);
    Some((root.join(format!("{id}.jsonl")), root.join(id)))
}

/// Mirror a transcript (and its sidecar) into the vault, overwriting any
/// previous copy.
fn archive(jsonl: &Path, profile: &str, project_dir: &str, id: &str) -> Result<(), String> {
    let (dst, dst_side) = vault_paths(profile, project_dir, id)
        .ok_or_else(|| "Could not resolve the vault dir".to_string())?;
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(jsonl, &dst).map_err(|e| e.to_string())?;
    if let Some(side) = sidecar_of(jsonl) {
        let _ = fs::remove_dir_all(&dst_side);
        copy_dir(&side, &dst_side)?;
    }
    Ok(())
}

fn drop_vault(profile: &str, project_dir: &str, id: &str) {
    if let Some((f, side)) = vault_paths(profile, project_dir, id) {
        let _ = fs::remove_file(&f);
        let _ = fs::remove_dir_all(&side);
        // Leave no empty project (or profile) folder behind in the vault.
        if let Some(parent) = f.parent() {
            let _ = fs::remove_dir(parent);
            if let Some(profile_dir) = parent.parent() {
                let _ = fs::remove_dir(profile_dir);
            }
        }
    }
}

/// Folder name of the `<profile>/projects/<…>` directory holding a transcript.
fn project_dir_of(jsonl: &Path) -> String {
    jsonl
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn id_of(jsonl: &Path) -> String {
    jsonl
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

// --- annotation (used by `list_sessions`) ----------------------------------

/// Flag retained sessions of `profile`, hide the keep-alive from the UI's
/// "modified" value, and append its sessions that now exist only in the vault.
pub(crate) fn annotate(sessions: &mut Vec<Session>, profile: &str) {
    let _guard = lock_registry();
    let mut reg = Registry::load();
    if reg.entries.is_empty() {
        return;
    }
    let mut dirty = false;

    for s in sessions.iter_mut() {
        let Some(e) = reg.entries.get_mut(&key(profile, &s.id)) else {
            continue;
        };
        s.retained = true;

        // The file moved (or was re-listed from a new path) — follow it.
        if e.path != s.path {
            e.path = s.path.clone();
            e.project_dir = project_dir_of(Path::new(&s.path));
            e.cwd = s.project.clone();
            dirty = true;
        }
        // An mtime we didn't write means real activity: adopt it as the truth.
        if s.modified != e.touched_to {
            e.real_modified = s.modified;
            dirty = true;
        }
        // Report the true last-activity time, never our keep-alive stamp. When
        // parse_session fell back to the mtime for updated/created, correct
        // those too rather than showing a fabricated date.
        if s.updated == s.modified {
            s.updated = e.real_modified;
        }
        if s.created == s.modified {
            s.created = e.real_modified;
        }
        s.modified = e.real_modified;
    }

    // Entries with no live transcript: surface the vault copy instead, so a
    // session cleaned up while the app was closed is still there, read from
    // the archive and restorable.
    let live: std::collections::HashSet<String> = sessions.iter().map(|s| s.id.clone()).collect();
    let orphans: Vec<Entry> = reg
        .entries
        .values()
        .filter(|e| e.profile == profile && !live.contains(&e.id))
        .cloned()
        .collect();

    for e in orphans {
        let Some((vf, _)) = vault_paths(&e.profile, &e.project_dir, &e.id) else {
            continue;
        };
        if !vf.is_file() {
            continue;
        }
        if let Some(mut s) = parse_session(&vf) {
            s.profile = profile.to_string();
            s.retained = true;
            s.archived = true;
            if s.updated == s.modified {
                s.updated = e.real_modified;
            }
            if s.created == s.modified {
                s.created = e.real_modified;
            }
            s.modified = e.real_modified;
            sessions.push(s);
        }
    }

    if dirty {
        let _ = reg.save();
    }
}

// --- commands --------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    /// Effective `cleanupPeriodDays`.
    pub days: u32,
    /// True when the value comes from managed/remote settings, i.e. the user
    /// cannot raise it locally.
    pub managed: bool,
    /// Where the vault lives, so the UI can point at it.
    pub vault: String,
}

fn read_period(p: &Path) -> Option<u32> {
    let body = fs::read_to_string(p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    v.get("cleanupPeriodDays")?.as_u64().map(|n| n as u32)
}

fn managed_settings_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    return PathBuf::from("C:\\ProgramData\\ClaudeCode\\managed-settings.json");
    #[cfg(target_os = "macos")]
    return PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json");
    #[cfg(all(unix, not(target_os = "macos")))]
    return PathBuf::from("/etc/claude-code/managed-settings.json");
}

/// The retention window a Profile's sessions are actually judged against.
/// Precedence follows Claude Code's own: machine-wide managed policy, then the
/// Profile's org remote settings, then its settings.json, else the 30-day default.
#[tauri::command(async)]
pub fn retention_policy(profile: String) -> RetentionPolicy {
    let vault = vault_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if let Some(days) = read_period(&managed_settings_path()) {
        return RetentionPolicy {
            days,
            managed: true,
            vault,
        };
    }
    if let Ok(p) = profiles::resolve(&profile) {
        let root = p.root();
        if let Some(days) = read_period(&root.join("remote-settings.json")) {
            return RetentionPolicy {
                days,
                managed: true,
                vault,
            };
        }
        if let Some(days) = read_period(&root.join("settings.json")) {
            return RetentionPolicy {
                days,
                managed: false,
                vault,
            };
        }
    }
    RetentionPolicy {
        days: 30,
        managed: false,
        vault,
    }
}

/// Mark a session retained (keep-alive + archive) or release it.
///
/// Releasing restores the transcript's true mtime, so the org's retention
/// applies again exactly as if the session had never been retained.
#[tauri::command(async)]
pub fn set_session_retained(profile: String, path: String, retained: bool) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let id = id_of(&p);
    if id.is_empty() {
        return Err("Invalid session path".to_string());
    }
    let _guard = lock_registry();
    let mut reg = Registry::load();

    if !retained {
        if let Some(e) = reg.entries.remove(&key(&profile, &id)) {
            drop_vault(&e.profile, &e.project_dir, &e.id);
            if p.is_file() && e.real_modified > 0 {
                let _ = touch_tree(&p, millis_to_time(e.real_modified));
            }
            reg.save()?;
        }
        return Ok(());
    }

    if !p.is_file() {
        return Err("Session file not found".to_string());
    }
    let project_dir = project_dir_of(&p);
    let real_modified = mtime_millis(&p);
    let cwd = parse_session(&p).map(|s| s.project).unwrap_or_default();

    archive(&p, &profile, &project_dir, &id)?;
    let touched_to = touch_tree(&p, SystemTime::now())?;

    reg.entries.insert(
        key(&profile, &id),
        Entry {
            id,
            profile,
            project_dir,
            path,
            cwd,
            real_modified,
            touched_to,
            archived_modified: real_modified,
        },
    );
    reg.save()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshReport {
    /// Retained sessions whose mtime was pushed past the cutoff.
    pub refreshed: u32,
    /// Vault copies rewritten because the transcript had changed.
    pub archived: u32,
    /// Retained sessions no longer in their Profile's projects (served from the vault).
    pub missing: u32,
}

/// Keep-alive pass over every Profile: run it on every app start, and
/// periodically while it runs. Each retained transcript gets a fresh mtime and,
/// when it changed since the last pass, a fresh vault copy.
#[tauri::command(async)]
pub fn refresh_retained() -> Result<RefreshReport, String> {
    let _guard = lock_registry();
    let mut reg = Registry::load();
    let mut report = RefreshReport {
        refreshed: 0,
        archived: 0,
        missing: 0,
    };
    if reg.entries.is_empty() {
        return Ok(report);
    }

    let now = SystemTime::now();
    let mut dead: Vec<String> = Vec::new();

    for (k, e) in reg.entries.iter_mut() {
        let p = PathBuf::from(&e.path);
        if !p.is_file() {
            // Gone from the scanned tree. Keep the entry as long as the vault
            // still holds it; otherwise there is nothing left to point at.
            match vault_paths(&e.profile, &e.project_dir, &e.id) {
                Some((vf, _)) if vf.is_file() => report.missing += 1,
                _ => dead.push(k.clone()),
            }
            continue;
        }

        let mtime = mtime_millis(&p);
        if mtime != e.touched_to {
            e.real_modified = mtime;
        }

        let vault_stale = match vault_paths(&e.profile, &e.project_dir, &e.id) {
            Some((vf, _)) => !vf.is_file() || e.archived_modified != e.real_modified,
            None => false,
        };
        if vault_stale && archive(&p, &e.profile, &e.project_dir, &e.id).is_ok() {
            e.archived_modified = e.real_modified;
            report.archived += 1;
        }

        if let Ok(touched) = touch_tree(&p, now) {
            e.touched_to = touched;
            report.refreshed += 1;
        }
    }

    for k in dead {
        reg.entries.remove(&k);
    }
    reg.save()?;
    Ok(report)
}

/// Copy a vault-only session back into its Profile's projects so the CLI can
/// resume it again. Returns its new live path.
#[tauri::command(async)]
pub fn restore_retained(profile: String, id: String) -> Result<String, String> {
    let target: Profile = profiles::resolve(&profile)?;
    let _guard = lock_registry();
    let mut reg = Registry::load();
    let k = key(&profile, &id);
    let e = reg
        .entries
        .get(&k)
        .cloned()
        .ok_or_else(|| "This session is not retained".to_string())?;
    let (vf, vside) = vault_paths(&e.profile, &e.project_dir, &e.id)
        .ok_or_else(|| "Could not resolve the vault dir".to_string())?;
    if !vf.is_file() {
        return Err("No archived copy to restore".to_string());
    }

    let dir = target.projects_dir().join(&e.project_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dst = dir.join(format!("{}.jsonl", e.id));
    if dst.exists() {
        return Err("A live session with this id already exists".to_string());
    }

    fs::copy(&vf, &dst).map_err(|e| e.to_string())?;
    if vside.is_dir() {
        copy_dir(&vside, &dir.join(&e.id))?;
    }

    let entry = reg.entries.get_mut(&k).expect("checked above");
    entry.path = dst.to_string_lossy().to_string();
    entry.touched_to = touch_tree(&dst, SystemTime::now())?;
    reg.save()?;
    Ok(dst.to_string_lossy().to_string())
}

/// Drop a session from the registry and delete its vault copy. Called when a
/// session is deleted for real, so it can't come back as an archived ghost.
pub(crate) fn forget(profile: &str, jsonl: &Path) {
    let id = id_of(jsonl);
    if id.is_empty() {
        return;
    }
    let _guard = lock_registry();
    let mut reg = Registry::load();
    if let Some(e) = reg.entries.remove(&key(profile, &id)) {
        drop_vault(&e.profile, &e.project_dir, &e.id);
        let _ = reg.save();
    }
}

/// Follow a retained session that just moved to another project folder or
/// another Profile: it stays retained at its new place.
pub(crate) fn relocate(from_profile: &str, to_profile: &str, old: &Path, new: &Path, cwd: &str) {
    let id = id_of(old);
    if id.is_empty() {
        return;
    }
    let _guard = lock_registry();
    let mut reg = Registry::load();
    let Some(mut e) = reg.entries.remove(&key(from_profile, &id)) else {
        return;
    };
    let (old_profile, old_project_dir) = (e.profile.clone(), e.project_dir.clone());
    e.profile = to_profile.to_string();
    e.path = new.to_string_lossy().to_string();
    e.project_dir = project_dir_of(new);
    e.cwd = cwd.to_string();
    // The vault mirrors the profile/project layout, so re-archive at the new place.
    if e.profile != old_profile || e.project_dir != old_project_dir {
        drop_vault(&old_profile, &old_project_dir, &id);
        e.archived_modified = 0;
    }
    if archive(new, &e.profile, &e.project_dir, &id).is_ok() {
        e.archived_modified = e.real_modified;
    }
    if let Ok(touched) = touch_tree(new, SystemTime::now()) {
        e.touched_to = touched;
    }
    reg.entries.insert(key(to_profile, &id), e);
    let _ = reg.save();
}

// --- tests -----------------------------------------------------------------
//
// The whole feature rests on being able to write a file's mtime, so that is what
// these cover, on explicit temp paths only — never the real registry or vault.

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh temp directory, removed by `Scratch::drop`.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let d =
                std::env::temp_dir().join(format!("cs-retention-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&d);
            fs::create_dir_all(&d).expect("temp dir");
            Scratch(d)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    // A known past instant, well outside any plausible cleanup cutoff.
    const OLD_MS: i64 = 1_600_000_000_000;

    #[test]
    fn touch_tree_rewrites_the_transcript_and_its_sidecar() {
        let s = Scratch::new("touch");
        let jsonl = s.path().join("abc.jsonl");
        fs::write(&jsonl, "{}\n").unwrap();
        let side = s.path().join("abc").join("subagents");
        fs::create_dir_all(&side).unwrap();
        let sub = side.join("sub.jsonl");
        fs::write(&sub, "{}\n").unwrap();

        // Age everything, the way a session Claude hasn't touched in weeks looks.
        let returned = touch_tree(&jsonl, millis_to_time(OLD_MS)).unwrap();
        assert_eq!(returned, OLD_MS, "returned mtime must be what was stored");
        assert_eq!(mtime_millis(&jsonl), OLD_MS);
        assert_eq!(mtime_millis(&sub), OLD_MS, "sidecars are purged separately");

        // Keep-alive: push it back over the cutoff.
        let fresh = touch_tree(&jsonl, SystemTime::now()).unwrap();
        assert!(fresh > OLD_MS);
        assert_eq!(mtime_millis(&jsonl), fresh);
        assert_eq!(mtime_millis(&sub), fresh);
    }

    #[test]
    fn copy_dir_mirrors_nested_files() {
        let s = Scratch::new("copy");
        let src = s.path().join("src").join("subagents");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.jsonl"), "a").unwrap();

        let dst = s.path().join("dst");
        copy_dir(&s.path().join("src"), &dst).unwrap();
        assert_eq!(
            fs::read_to_string(dst.join("subagents").join("a.jsonl")).unwrap(),
            "a"
        );
    }

    #[test]
    fn read_period_only_reports_an_explicit_value() {
        let s = Scratch::new("period");
        let with = s.path().join("with.json");
        fs::write(&with, r#"{"cleanupPeriodDays": 14, "model": "opus"}"#).unwrap();
        assert_eq!(read_period(&with), Some(14));

        let without = s.path().join("without.json");
        fs::write(&without, r#"{"model": "opus"}"#).unwrap();
        assert_eq!(read_period(&without), None);

        assert_eq!(read_period(&s.path().join("missing.json")), None);
    }

    #[test]
    fn paths_are_derived_from_the_transcript_location() {
        let p = Path::new("/home/u/.claude/projects/D--www-app/1234.jsonl");
        assert_eq!(project_dir_of(p), "D--www-app");
        assert_eq!(id_of(p), "1234");
    }
}
