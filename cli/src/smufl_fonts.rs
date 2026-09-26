//! Where SMuFL fonts live on this machine, per the SMuFL specification's
//! "Font metadata locations".
//!
//! A font's metadata is installed as `SMuFL/Fonts/<fontname>/<fontname>.json`
//! under a per-user data directory and under one or more system-wide ones,
//! with the per-user location taking precedence. The font program itself is an
//! ordinary installed font and is not looked up here.
//!
//! Shared by the commands that read and write those locations, and owned by
//! none of them.

use std::env;
use std::path::{Path, PathBuf};

/// The data directories SMuFL metadata is looked up under, most preferred
/// first. Each holds a `SMuFL/Fonts` directory, or none.
#[derive(Debug, Clone)]
pub struct SmuflRoots {
    /// The per-user data directory, where `opus font install` writes. `None`
    /// when the environment names no home directory.
    pub user: Option<PathBuf>,
    /// The system-wide data directories, in the order they are searched.
    pub system: Vec<PathBuf>,
}

/// One font whose metadata was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledFont {
    /// The font's name, as its directory spells it.
    pub name: String,
    /// Its metadata file.
    pub metadata: PathBuf,
}

impl SmuflRoots {
    /// The locations the SMuFL specification names for this operating system.
    pub fn for_this_system() -> SmuflRoots {
        if cfg!(target_os = "macos") {
            SmuflRoots {
                user: home().map(|h| h.join("Library/Application Support")),
                system: vec![PathBuf::from("/Library/Application Support")],
            }
        } else if cfg!(target_os = "windows") {
            SmuflRoots {
                user: env_path("LOCALAPPDATA"),
                system: env_path("COMMONPROGRAMFILES").into_iter().collect(),
            }
        } else {
            // XDG Base Directory defaults apply when the variables are unset.
            let user = env_path("XDG_DATA_HOME").or_else(|| home().map(|h| h.join(".local/share")));
            let system = env::var("XDG_DATA_DIRS")
                .ok()
                .filter(|dirs| !dirs.is_empty())
                .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string())
                .split(':')
                .filter(|dir| !dir.is_empty())
                .map(PathBuf::from)
                .collect();
            SmuflRoots { user, system }
        }
    }

    /// Every root, per-user first.
    pub fn all(&self) -> impl Iterator<Item = &PathBuf> {
        self.user.iter().chain(self.system.iter())
    }
}

/// The directory a font's metadata is installed in under `root`.
pub fn font_dir(root: &Path, name: &str) -> PathBuf {
    root.join("SMuFL").join("Fonts").join(name)
}

/// Every font with metadata under `roots`. Where two roots carry a font of the
/// same name, the one found first -- the per-user one -- wins.
///
/// A font directory counts when it holds `<dirname>.json`, matched without
/// regard to case; anything else under `SMuFL/Fonts` is ignored.
pub fn discover(roots: &SmuflRoots) -> Vec<InstalledFont> {
    let mut found: Vec<InstalledFont> = Vec::new();

    for root in roots.all() {
        let Ok(entries) = root.join("SMuFL").join("Fonts").read_dir() else {
            continue;
        };

        let mut here: Vec<InstalledFont> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                let metadata = metadata_file(&entry.path(), &name)?;
                Some(InstalledFont { name, metadata })
            })
            .filter(|font| {
                !found
                    .iter()
                    .any(|f| f.name.eq_ignore_ascii_case(&font.name))
            })
            .collect();
        here.sort_by(|a, b| a.name.cmp(&b.name));

        found.append(&mut here);
    }

    found
}

/// The directory per-user fonts are installed in on this operating system, or
/// `None` when the environment names no home directory.
pub fn user_font_dir() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        home().map(|h| h.join("Library/Fonts"))
    } else if cfg!(target_os = "windows") {
        env_path("LOCALAPPDATA").map(|d| d.join("Microsoft/Windows/Fonts"))
    } else {
        env_path("XDG_DATA_HOME")
            .or_else(|| home().map(|h| h.join(".local/share")))
            .map(|d| d.join("fonts"))
    }
}

fn metadata_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let wanted = format!("{name}.json");
    dir.read_dir()
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.eq_ignore_ascii_case(&wanted))
        })
}

fn home() -> Option<PathBuf> {
    env_path("HOME").or_else(|| env_path("USERPROFILE"))
}

fn env_path(var: &str) -> Option<PathBuf> {
    env::var_os(var)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
