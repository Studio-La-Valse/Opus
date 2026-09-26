use crate::smufl_fonts::{self, SmuflRoots};
use clap::Args;
use lib::smufl::smufl_metadata::SmuflMetadata;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// The font's folder, as it ships: searched, subfolders included, for SMuFL
    /// metadata files and `.otf` fonts. A folder of text fonts only, with no
    /// metadata, installs those.
    folder: String,

    /// Replace metadata and font files that are already installed.
    #[arg(long, action)]
    overwrite: bool,
}

/// What an install copied, source to destination.
#[derive(Debug, Default)]
pub struct Installed {
    /// Each metadata file, keyed by the `fontName` it declares.
    pub metadata: Vec<(String, PathBuf)>,
    pub fonts: Vec<PathBuf>,
}

pub fn run(args: InstallArgs) {
    let roots = SmuflRoots::for_this_system();
    let metadata_root = roots
        .user
        .unwrap_or_else(|| panic!("no per-user data directory: set HOME"));
    let font_dir = smufl_fonts::user_font_dir()
        .unwrap_or_else(|| panic!("no per-user font directory: set HOME"));

    let installed = install(
        Path::new(&args.folder),
        &metadata_root,
        &font_dir,
        args.overwrite,
    )
    .unwrap_or_else(|err| panic!("{err}"));

    for (name, path) in &installed.metadata {
        println!("Installed {name}: {}", path.display());
    }
    for path in &installed.fonts {
        println!("Installed font: {}", path.display());
    }
}

/// Installs the fonts found in `source`: each SMuFL metadata file as
/// `SMuFL/Fonts/<fontName>/<fontName>.json` under `metadata_root`, and each
/// `.otf` into `font_dir`.
///
/// Named after the `fontName` the metadata declares rather than the file it
/// came in, which is how the specification lays it out and what makes a
/// package like Finale Maestro's, with its `Finale Maestro.json`, findable.
/// Nothing is written unless everything can be: an existing destination
/// without `overwrite` fails the whole install -- unless it already holds
/// exactly the file being installed, as a font installed by hand before its
/// metadata was does.
pub fn install(
    source: &Path,
    metadata_root: &Path,
    font_dir: &Path,
    overwrite: bool,
) -> Result<Installed, String> {
    let files = files_under(source)
        .map_err(|err| format!("failed to read '{}': {err}", source.display()))?;

    let mut copies: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut installed = Installed::default();

    for file in &files {
        match extension(file).as_deref() {
            Some("json") => {
                let Some(name) = font_name(file) else {
                    continue;
                };
                let dest = smufl_fonts::font_dir(metadata_root, &name).join(format!("{name}.json"));
                copies.push((file.clone(), dest.clone()));
                installed.metadata.push((name, dest));
            }
            Some("otf") => {
                let dest = font_dir.join(file.file_name().unwrap());
                copies.push((file.clone(), dest.clone()));
                installed.fonts.push(dest);
            }
            _ => {}
        }
    }

    if copies.is_empty() {
        return Err(format!(
            "no SMuFL metadata or .otf fonts found in '{}'",
            source.display()
        ));
    }

    // An identical file is already installed; leaving it is the same result.
    copies.retain(|(from, to)| !same_contents(from, to));

    if !overwrite && let Some((_, dest)) = copies.iter().find(|(_, dest)| dest.exists()) {
        return Err(format!(
            "'{}' is already installed; pass --overwrite to replace it",
            dest.display()
        ));
    }

    for (from, to) in &copies {
        let parent = to.parent().unwrap();
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create '{}': {err}", parent.display()))?;
        fs::copy(from, to).map_err(|err| {
            format!(
                "failed to copy '{}' to '{}': {err}",
                from.display(),
                to.display()
            )
        })?;
    }

    Ok(installed)
}

/// The `fontName` of `path` if it is SMuFL font metadata, and `None` for any
/// other JSON file.
fn font_name(path: &Path) -> Option<String> {
    let json = fs::read_to_string(path).ok()?;
    let meta: SmuflMetadata = serde_json::from_str(&json).ok()?;
    Some(meta.font)
}

fn same_contents(a: &Path, b: &Path) -> bool {
    match (fs::read(a), fs::read(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Every file under `dir`, subfolders included, skipping hidden entries.
fn files_under(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let hidden = path
            .file_name()
            .and_then(|f| f.to_str())
            .is_some_and(|f| f.starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            files.extend(files_under(&path)?);
        } else {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
}
