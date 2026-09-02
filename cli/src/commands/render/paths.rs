use std::fs;
use std::path::{Path, PathBuf};

/// Where `render` writes its output, and how the files are named.
///
/// `--out`, when given, is a directory (created if it doesn't exist). When
/// omitted, output is written next to the input `--file`. Output filenames
/// reuse the input file's stem: `score.musicxml` produces `score.pdf`, or
/// `score-p1.svg`, `score-p2.svg`, ... for the per-page SVG format.
///
/// [`write`](Self::write) refuses to replace a file that already exists unless
/// `--overwrite` was passed, so a stray `--out` (or the default, which writes
/// beside the input) can't silently clobber an unrelated file.
pub struct OutputTarget {
    dir: PathBuf,
    stem: String,
    overwrite: bool,
}

impl OutputTarget {
    /// Directory + stem resolution, no filesystem access.
    ///
    /// The directory is `out` when given, otherwise the input file's parent
    /// (or `.` when it has none). The stem is the input file's stem, falling
    /// back to `score` for a path with no usable stem.
    fn new(out: Option<&str>, file: &str, overwrite: bool) -> Self {
        let input = Path::new(file);
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("score")
            .to_string();

        let dir = match out {
            Some(dir) => PathBuf::from(dir),
            None => input
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
        };

        Self {
            dir,
            stem,
            overwrite,
        }
    }

    /// Pure name resolution with `overwrite` left off -- for testing the
    /// derived paths without touching the filesystem. Use
    /// [`resolve`](Self::resolve) for the real render path.
    pub fn plan(out: Option<&str>, file: &str) -> Self {
        Self::new(out, file, false)
    }

    /// Resolves the target, ensures its directory exists, and records whether
    /// [`write`](Self::write) may overwrite existing files.
    pub fn resolve(out: Option<&str>, file: &str, overwrite: bool) -> Self {
        let target = Self::new(out, file, overwrite);

        fs::create_dir_all(&target.dir).unwrap_or_else(|err| {
            panic!(
                "failed to create output directory '{}': {err}",
                target.dir.display()
            )
        });

        target
    }

    /// `<dir>/<stem>.<ext>` -- for a format that emits a single file (PDF).
    pub fn single(&self, ext: &str) -> PathBuf {
        self.dir.join(format!("{}.{ext}", self.stem))
    }

    /// `<dir>/<stem>-p{n}.<ext>` -- one file per page (SVG); `n` is 1-based.
    pub fn page(&self, n: u32, ext: &str) -> PathBuf {
        self.dir.join(format!("{}-p{n}.{ext}", self.stem))
    }

    /// Writes `contents` to `path`. Panics if `path` already exists and this
    /// target was resolved without `overwrite`, so an accidental run can't
    /// replace a file it didn't create.
    pub fn write(&self, path: &Path, contents: impl AsRef<[u8]>) {
        if !self.overwrite && path.exists() {
            panic!(
                "'{}' already exists; pass --overwrite to replace it",
                path.display()
            );
        }
        fs::write(path, contents)
            .unwrap_or_else(|err| panic!("failed to write '{}': {err}", path.display()));
    }
}
