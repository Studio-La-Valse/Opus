use std::fmt;

/// The music font used when neither the user nor the document names one that
/// is available.
pub const DEFAULT_MUSIC_FONT: &str = "Bravura";

/// Why no music font could be chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicFontError {
    /// The user asked for a font by name, and it is not among the available
    /// ones. Not quietly replaced: an explicit choice that silently turns into
    /// another font is harder to notice than a refusal.
    Unavailable(String),
    /// Nobody asked for anything available, and the default is not available
    /// either.
    NoneAvailable,
}

impl fmt::Display for MusicFontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicFontError::Unavailable(name) => {
                write!(f, "music font '{name}' is not available")
            }
            MusicFontError::NoneAvailable => write!(
                f,
                "no music font is available: the document names none that is, \
                 and the default '{DEFAULT_MUSIC_FONT}' is missing too"
            ),
        }
    }
}

/// Picks the music font to engrave with, out of `available`.
///
/// Precedence is `user`, then the document's `<music-font>` list in order,
/// then [`DEFAULT_MUSIC_FONT`]: the user always overrides the document. Names
/// match case-insensitively, and a few legacy names are mapped to the SMuFL
/// font that replaced them -- `Maestro` is `Finale Maestro`, as Finale itself
/// maps it on import. A document name that matches nothing, such as the
/// generic `engraved`, is skipped.
///
/// Returns the name as `available` spells it.
pub fn choose_music_font<'a>(
    user: Option<&str>,
    document: &[String],
    available: &'a [String],
) -> Result<&'a str, MusicFontError> {
    if let Some(user) = user {
        return find(user, available).ok_or_else(|| MusicFontError::Unavailable(user.to_string()));
    }

    document
        .iter()
        .find_map(|name| find(name, available))
        .or_else(|| find(DEFAULT_MUSIC_FONT, available))
        .ok_or(MusicFontError::NoneAvailable)
}

/// Legacy and alternative names, lower case, with the font each stands for.
const ALIASES: [(&str, &str); 1] = [("maestro", "Finale Maestro")];

fn find<'a>(name: &str, available: &'a [String]) -> Option<&'a str> {
    let name = name.trim();
    let name = ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
        .map_or(name, |(_, font)| font);

    available
        .iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(name))
        .map(String::as_str)
}
