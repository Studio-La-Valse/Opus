#[cfg(test)]
mod tests {
    use cli::commands::render::paths::OutputTarget;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A fresh, empty scratch directory under the OS temp dir.
    fn scratch_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("opus-render-paths-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn explicit_out_directory_is_used_with_the_input_stem() {
        let target = OutputTarget::plan(Some("build/out"), "scores/sonata.musicxml");

        assert_eq!(target.single("pdf"), PathBuf::from("build/out/sonata.pdf"));
        assert_eq!(
            target.page(1, "svg"),
            PathBuf::from("build/out/sonata-p1.svg")
        );
        assert_eq!(
            target.page(12, "svg"),
            PathBuf::from("build/out/sonata-p12.svg")
        );
    }

    #[test]
    fn omitted_out_falls_back_to_the_input_files_directory() {
        let target = OutputTarget::plan(None, "scores/nested/sonata.musicxml");

        assert_eq!(
            target.page(3, "svg"),
            PathBuf::from("scores/nested/sonata-p3.svg")
        );
    }

    #[test]
    fn omitted_out_for_a_bare_filename_writes_to_the_current_directory() {
        let target = OutputTarget::plan(None, "sonata.musicxml");

        assert_eq!(target.single("pdf"), PathBuf::from("./sonata.pdf"));
    }

    #[test]
    fn a_path_without_a_usable_stem_falls_back_to_score() {
        let target = OutputTarget::plan(Some("out"), "..");

        assert_eq!(target.single("pdf"), PathBuf::from("out/score.pdf"));
    }

    #[test]
    fn write_creates_a_file_that_did_not_exist() {
        let dir = scratch_dir("new");
        let target = OutputTarget::resolve(dir.to_str(), "score.musicxml", false);
        let path = target.single("pdf");

        target.write(&path, b"pdf-bytes".as_slice());

        assert_eq!(fs::read(&path).unwrap(), b"pdf-bytes");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[should_panic(expected = "already exists")]
    fn write_refuses_to_replace_an_existing_file_without_overwrite() {
        let dir = scratch_dir("guard");
        let target = OutputTarget::resolve(dir.to_str(), "score.musicxml", false);
        let path = target.single("pdf");
        fs::write(&path, b"original").unwrap();

        target.write(&path, b"replacement".as_slice());
    }

    #[test]
    fn write_replaces_an_existing_file_when_overwrite_is_set() {
        let dir = scratch_dir("overwrite");
        let target = OutputTarget::resolve(dir.to_str(), "score.musicxml", true);
        let path = target.single("pdf");
        fs::write(&path, b"original").unwrap();

        target.write(&path, b"replacement".as_slice());

        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        fs::remove_dir_all(&dir).ok();
    }
}
