#[cfg(test)]
mod tests {
    use cli::commands::render::layout_file;
    use lib::geometry::color::Color;
    use lib::score::core::group_symbol::GroupSymbol;
    use lib::score::layout_options::{PageLayout, TieLayout, UserLayout};

    /// A layout file names only what it changes: the tables and keys it leaves
    /// out stay `None`, to fall through to the document and the app default.
    #[test]
    fn a_partial_file_sets_only_what_it_names() {
        let layout = layout_file::parse(
            r##"
            [page]
            color = "#112233"

            [tie]
            height_max = 14

            [section]
            symbol = "square"
            "##,
        )
        .expect("a valid partial layout");

        assert_eq!(layout.page.color.expect("page.color").to_hex(), "#112233FF");
        assert_eq!(layout.tie.height_max, Some(14.));
        assert_eq!(layout.section.symbol, Some(GroupSymbol::Square));

        assert_eq!(layout.tie.height_min, None, "a key left out stays unset");
        assert_eq!(layout.beam.thickness, None, "a table left out stays unset");
    }

    #[test]
    fn an_empty_file_is_an_empty_layout() {
        let layout = layout_file::parse("").expect("an empty file is valid");
        assert_eq!(layout.page.color.map(|c| c.to_hex()), None);
        assert_eq!(layout.tie.height_max, None);
    }

    /// A typo must not quietly do nothing: an unknown key or table is an error
    /// that names it.
    #[test]
    fn unknown_keys_and_tables_are_rejected() {
        let key = layout_file::parse("[tie]\nheight_maximum = 14\n")
            .expect_err("an unknown key must be rejected");
        assert!(key.to_string().contains("height_maximum"), "{key}");

        let table = layout_file::parse("[ties]\nheight_max = 14\n")
            .expect_err("an unknown table must be rejected");
        assert!(table.to_string().contains("ties"), "{table}");
    }

    /// Flat keys are the CLI's spelling, not the file's: a file groups them.
    #[test]
    fn flat_keys_are_rejected() {
        assert!(layout_file::parse("tie_height_max = 14\n").is_err());
    }

    #[test]
    fn a_wrongly_typed_value_is_rejected() {
        assert!(layout_file::parse("[tie]\nheight_max = \"tall\"\n").is_err());
        assert!(layout_file::parse("[section]\nsymbol = \"curly\"\n").is_err());
        assert!(layout_file::parse("[page]\ncolor = \"white\"\n").is_err());
    }

    /// The CLI's precedence: the file first, then the flags on top of it,
    /// option by option -- a flag replaces only the option it names, and
    /// leaves the file's other options in the same group alone.
    #[test]
    fn overrides_win_option_by_option() {
        let file = UserLayout {
            page: PageLayout {
                color: Some(Color::RED),
            },
            tie: TieLayout {
                height_min: Some(4.),
                height_max: Some(14.),
                ..Default::default()
            },
            ..Default::default()
        };
        let flags = UserLayout {
            tie: TieLayout {
                height_max: Some(20.),
                note_gap: Some(3.),
                ..Default::default()
            },
            ..Default::default()
        };

        let layout = file.overlay(flags);

        assert_eq!(layout.tie.height_max, Some(20.), "the flag wins");
        assert_eq!(
            layout.tie.height_min,
            Some(4.),
            "the file's sibling survives"
        );
        assert_eq!(
            layout.tie.note_gap,
            Some(3.),
            "a flag the file lacks applies"
        );
        assert_eq!(
            layout.page.color.map(|c| c.to_hex()),
            Some(Color::RED.to_hex()),
            "another group the flags leave alone survives"
        );
        assert_eq!(layout.beam.thickness, None, "neither sets it");
    }
}
