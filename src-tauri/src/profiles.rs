// Profiles — the Claude Code configuration directories this app browses.
//
// A Profile is one `CLAUDE_CONFIG_DIR`: `~/.claude` (the Default Profile, what
// Claude Code uses when the variable is unset) and any other directory the user
// runs Claude with, e.g. `~/.claude-perso`. Profiles are detected — every
// `~/.claude*` folder holding a `projects/` dir, plus `$CLAUDE_CONFIG_DIR` — and
// the user can rename, colour or hide them, or add any other directory.
//
// Every Profile ever seen is recorded in `profiles.json` next to the vault, so
// its id stays stable: the id keys the retention registry and the vault layout.
// A Profile whose directory disappears is kept (its retained sessions and vault
// copies survive) and simply reported as missing.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

pub(crate) const DEFAULT_ID: &str = "default";

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
struct Stored {
    id: String,
    path: String,
    /// User-chosen name; None keeps the one derived from the folder.
    name: Option<String>,
    color: Option<String>,
    hidden: bool,
    /// Added by hand rather than detected; only these can be removed.
    manual: bool,
    /// Overrides the shell prefix put in front of a copied resume command.
    resume_prefix: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Store {
    version: u32,
    profiles: Vec<Stored>,
}

/// One Profile, as the UI sees it.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub color: Option<String>,
    pub hidden: bool,
    pub manual: bool,
    pub is_default: bool,
    /// False when the directory no longer exists.
    pub exists: bool,
    pub resume_prefix: Option<String>,
}

impl Profile {
    pub fn root(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    pub fn projects_dir(&self) -> PathBuf {
        self.root().join("projects")
    }
}

// --- storage ---------------------------------------------------------------

fn store_path() -> Option<PathBuf> {
    crate::retention::app_dir().map(|d| d.join("profiles.json"))
}

impl Store {
    fn load() -> Store {
        store_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<Store>(&s).ok())
            .unwrap_or_default()
    }

    fn save(&mut self) -> Result<(), String> {
        self.version = 1;
        let p = store_path().ok_or_else(|| "Could not resolve the app data dir".to_string())?;
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let body = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp = p.with_extension("json.tmp");
        fs::write(&tmp, body).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &p).map_err(|e| e.to_string())
    }
}

static STORE_LOCK: Mutex<()> = Mutex::new(());

fn lock_store() -> MutexGuard<'static, ()> {
    STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

// --- detection -------------------------------------------------------------

/// Comparable form of a directory path: no trailing separator, and
/// case-insensitive where the filesystem is.
fn norm(p: &str) -> String {
    let t = p.trim_end_matches(['/', '\\']).replace('\\', "/");
    if cfg!(windows) {
        t.to_lowercase()
    } else {
        t
    }
}

fn default_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

/// `~/.claude-*` folders that look like a Claude config dir, plus the one
/// `CLAUDE_CONFIG_DIR` points to.
fn detect() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(home) = dirs::home_dir() {
        if let Ok(entries) = fs::read_dir(&home) {
            let mut found: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(".claude-"))
                        && p.join("projects").is_dir()
                })
                .collect();
            found.sort();
            out.extend(found);
        }
    }
    if let Some(env) = std::env::var_os("CLAUDE_CONFIG_DIR") {
        let p = PathBuf::from(env);
        if p.join("projects").is_dir() {
            out.push(p);
        }
    }
    out
}

/// Name derived from the folder: `.claude-perso` → "perso".
fn derived_name(path: &Path, is_default: bool) -> String {
    if is_default {
        return "Default".to_string();
    }
    let n = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let n = n.strip_prefix(".claude-").unwrap_or(&n);
    let n = n.trim_start_matches('.');
    if n.is_empty() {
        path.to_string_lossy().to_string()
    } else {
        n.to_string()
    }
}

/// A fresh id derived from the folder name, unique among `taken`.
fn fresh_id(path: &Path, taken: &[String]) -> String {
    let base: String = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let base = base.trim_matches('-');
    let base = if base.is_empty() { "profile" } else { base };
    let mut id = base.to_string();
    let mut n = 2;
    while id == DEFAULT_ID || taken.contains(&id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

/// Load the store and fold in the Default Profile and every detected folder.
/// Returns the store and whether it changed (so the caller saves it).
fn load_merged() -> (Store, bool) {
    let mut store = Store::load();
    let mut dirty = false;

    if let Some(root) = default_root() {
        let root_s = root.to_string_lossy().to_string();
        match store.profiles.iter_mut().find(|p| p.id == DEFAULT_ID) {
            Some(p) if norm(&p.path) != norm(&root_s) => {
                p.path = root_s;
                dirty = true;
            }
            Some(_) => {}
            None => {
                store.profiles.insert(
                    0,
                    Stored {
                        id: DEFAULT_ID.to_string(),
                        path: root_s,
                        ..Default::default()
                    },
                );
                dirty = true;
            }
        }
    }

    for dir in detect() {
        let s = dir.to_string_lossy().to_string();
        if store.profiles.iter().any(|p| norm(&p.path) == norm(&s)) {
            continue;
        }
        let taken: Vec<String> = store.profiles.iter().map(|p| p.id.clone()).collect();
        store.profiles.push(Stored {
            id: fresh_id(&dir, &taken),
            path: s,
            ..Default::default()
        });
        dirty = true;
    }
    (store, dirty)
}

fn to_profile(s: &Stored) -> Profile {
    let root = PathBuf::from(&s.path);
    let is_default = s.id == DEFAULT_ID;
    Profile {
        id: s.id.clone(),
        name: s
            .name
            .clone()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| derived_name(&root, is_default)),
        path: s.path.clone(),
        color: s.color.clone(),
        hidden: s.hidden,
        manual: s.manual,
        is_default,
        exists: root.is_dir(),
        resume_prefix: s.resume_prefix.clone(),
    }
}

/// Every known Profile, the Default one first.
pub(crate) fn all() -> Vec<Profile> {
    let _guard = lock_store();
    let (mut store, dirty) = load_merged();
    if dirty {
        let _ = store.save();
    }
    store.profiles.iter().map(to_profile).collect()
}

/// Resolve a Profile id to its Profile.
pub(crate) fn resolve(id: &str) -> Result<Profile, String> {
    all()
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("Unknown profile \"{id}\""))
}

// --- commands --------------------------------------------------------------

#[tauri::command(async)]
pub fn list_profiles() -> Vec<Profile> {
    all()
}

/// The user-editable part of a Profile.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileEdit {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub hidden: bool,
    pub resume_prefix: Option<String>,
}

#[tauri::command(async)]
pub fn update_profile(edit: ProfileEdit) -> Result<Profile, String> {
    let _guard = lock_store();
    let (mut store, _) = load_merged();
    let p = store
        .profiles
        .iter_mut()
        .find(|p| p.id == edit.id)
        .ok_or_else(|| format!("Unknown profile \"{}\"", edit.id))?;
    p.name = edit
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    p.color = edit.color.filter(|c| !c.is_empty());
    // The Default Profile is always there to fall back on.
    p.hidden = edit.hidden && p.id != DEFAULT_ID;
    p.resume_prefix = edit.resume_prefix.filter(|r| !r.trim().is_empty());
    let out = to_profile(p);
    store.save()?;
    Ok(out)
}

/// Register any directory as a Profile.
#[tauri::command(async)]
pub fn add_profile(path: String) -> Result<Profile, String> {
    let dir = PathBuf::from(path.trim());
    if !dir.is_dir() {
        return Err("This folder does not exist".to_string());
    }
    let _guard = lock_store();
    let (mut store, _) = load_merged();
    let s = dir.to_string_lossy().to_string();
    if let Some(existing) = store
        .profiles
        .iter_mut()
        .find(|p| norm(&p.path) == norm(&s))
    {
        // Adding a known (maybe hidden) folder again just brings it back.
        existing.hidden = false;
        let out = to_profile(existing);
        store.save()?;
        return Ok(out);
    }
    let taken: Vec<String> = store.profiles.iter().map(|p| p.id.clone()).collect();
    let stored = Stored {
        id: fresh_id(&dir, &taken),
        path: s,
        manual: true,
        ..Default::default()
    };
    let out = to_profile(&stored);
    store.profiles.push(stored);
    store.save()?;
    Ok(out)
}

/// Forget a Profile added by hand. Detected ones can only be hidden: they would
/// be detected again on the next scan anyway. Its retained sessions and vault
/// copies are kept.
#[tauri::command(async)]
pub fn remove_profile(id: String) -> Result<(), String> {
    let _guard = lock_store();
    let (mut store, _) = load_merged();
    let idx = store
        .profiles
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| format!("Unknown profile \"{id}\""))?;
    if !store.profiles[idx].manual {
        return Err(
            "Only profiles added by hand can be removed; hide this one instead".to_string(),
        );
    }
    store.profiles.remove(idx);
    store.save()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_derived_from_the_folder() {
        assert_eq!(derived_name(Path::new("/home/u/.claude"), true), "Default");
        assert_eq!(
            derived_name(Path::new("/home/u/.claude-perso"), false),
            "perso"
        );
        assert_eq!(
            derived_name(Path::new("/srv/work-claude"), false),
            "work-claude"
        );
    }

    #[test]
    fn ids_are_unique_and_never_default() {
        let taken = vec!["claude-perso".to_string()];
        assert_eq!(
            fresh_id(Path::new("/x/.claude-perso"), &taken),
            "claude-perso-2"
        );
        assert_eq!(fresh_id(Path::new("/x/default"), &[]), "default-2");
        assert_eq!(fresh_id(Path::new("/x/.claude-work"), &[]), "claude-work");
    }

    #[test]
    fn paths_compare_without_trailing_separator() {
        assert_eq!(norm("/a/b/"), norm("/a/b"));
    }
}
