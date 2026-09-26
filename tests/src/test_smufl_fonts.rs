//! Finding installed SMuFL fonts, and `opus font install` putting them where
//! they are found. Every test works in a scratch directory of its own, laid out
//! the way the SMuFL specification lays out a real data directory.

#[cfg(test)]
mod tests {
    use cli::commands::font::install::install;
    use cli::smufl_fonts::{InstalledFont, SmuflRoots, discover};
    use std::fs;
    use std::path::{Path, PathBuf};

    const LELAND: &str = "assets/smufl/Leland-main";
    const MAESTRO: &str = "assets/smufl/Maestro-main";

    /// A fresh, empty directory unique to `name` and this test run.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("opus-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn asset(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(relative)
    }

    /// Writes an (empty-bodied) metadata file for `font` under `root`, spelling
    /// the file name as `file`.
    fn put_metadata(root: &Path, font: &str, file: &str) -> PathBuf {
        let dir = root.join("SMuFL/Fonts").join(font);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(file);
        fs::write(&path, "{}").unwrap();
        path
    }

    fn names(fonts: &[InstalledFont]) -> Vec<&str> {
        fonts.iter().map(|f| f.name.as_str()).collect()
    }

    // ------------------------------------------------------------ discovery

    #[test]
    fn a_font_is_found_by_its_directory_and_matching_json() {
        let root = scratch("discover-one");
        let metadata = put_metadata(&root, "Leland", "Leland.json");

        let roots = SmuflRoots {
            user: Some(root),
            system: vec![],
        };
        assert_eq!(
            discover(&roots),
            vec![InstalledFont {
                name: "Leland".into(),
                metadata
            }]
        );
    }

    /// The file name is matched regardless of case; a directory without a
    /// matching file is not a font.
    #[test]
    fn only_a_directory_with_its_own_json_counts() {
        let root = scratch("discover-case");
        put_metadata(&root, "Bravura", "bravura.json");
        put_metadata(&root, "Stray", "something-else.json");

        let roots = SmuflRoots {
            user: None,
            system: vec![root],
        };
        assert_eq!(names(&discover(&roots)), vec!["Bravura"]);
    }

    #[test]
    fn user_level_metadata_wins_over_system_level() {
        let user = scratch("discover-user");
        let system = scratch("discover-system");
        let mine = put_metadata(&user, "Bravura", "Bravura.json");
        put_metadata(&system, "Bravura", "Bravura.json");
        put_metadata(&system, "Leland", "Leland.json");

        let fonts = discover(&SmuflRoots {
            user: Some(user),
            system: vec![system],
        });
        assert_eq!(names(&fonts), vec!["Bravura", "Leland"]);
        assert_eq!(fonts[0].metadata, mine);
    }

    #[test]
    fn missing_roots_find_nothing() {
        let roots = SmuflRoots {
            user: Some(scratch("discover-empty").join("nowhere")),
            system: vec![],
        };
        assert!(discover(&roots).is_empty());
    }

    // ------------------------------------------------------------ install

    /// Finale Maestro ships its metadata as `Finale Maestro.json`; installed,
    /// it is named after the `fontName` inside, which is where discovery looks.
    #[test]
    fn an_installed_font_is_discovered_under_its_font_name() {
        let root = scratch("install-maestro");
        let fonts = root.join("fonts");

        let installed = install(&asset(MAESTRO), &root, &fonts, false).unwrap();
        assert_eq!(installed.metadata.len(), 1);
        assert_eq!(installed.fonts.len(), 5);
        assert!(fonts.join("FinaleMaestro.otf").is_file());

        let found = discover(&SmuflRoots {
            user: Some(root.clone()),
            system: vec![],
        });
        assert_eq!(names(&found), vec!["Finale Maestro"]);
        assert_eq!(
            found[0].metadata,
            root.join("SMuFL/Fonts/Finale Maestro/Finale Maestro.json")
        );
    }

    #[test]
    fn installing_twice_needs_overwrite() {
        let root = scratch("install-twice");
        let fonts = root.join("fonts");

        install(&asset(LELAND), &root, &fonts, false).unwrap();
        fs::write(fonts.join("Leland.otf"), b"another version").unwrap();

        let again = install(&asset(LELAND), &root, &fonts, false);
        assert!(again.unwrap_err().contains("--overwrite"));

        install(&asset(LELAND), &root, &fonts, true).unwrap();
    }

    /// Reinstalling what is already there, byte for byte, changes nothing and
    /// needs no `--overwrite` -- the usual case of a font installed by hand
    /// before its metadata.
    #[test]
    fn identical_files_already_installed_are_left_alone() {
        let root = scratch("install-identical");
        let fonts = root.join("fonts");

        install(&asset(LELAND), &root, &fonts, false).unwrap();
        install(&asset(LELAND), &root, &fonts, false).unwrap();
    }

    #[test]
    fn a_folder_without_fonts_is_an_error() {
        let empty = scratch("install-empty");
        let root = scratch("install-empty-dest");

        let result = install(&empty, &root, &root.join("fonts"), false);
        assert!(result.is_err());
    }
}
