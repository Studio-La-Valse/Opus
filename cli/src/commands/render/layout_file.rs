use lib::score::layout_options::UserLayout;
use std::fs::read_to_string;

/// Reads the TOML layout file at `path`, panicking with the path and the reason
/// if it can't be read or isn't a valid layout -- the way an unreadable `--meta`
/// is reported.
pub fn read(path: &str) -> UserLayout {
    let source = read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read layout file '{path}': {err}"));

    parse(&source).unwrap_or_else(|err| panic!("Invalid layout file '{path}': {err}"))
}

/// Parses a TOML layout: one table per option group, every key optional, an
/// unknown group or key an error.
///
/// ```toml
/// [tie]
/// height_max = 14
///
/// [section]
/// symbol = "square"
/// ```
pub fn parse(source: &str) -> Result<UserLayout, toml::de::Error> {
    toml::from_str(source)
}
