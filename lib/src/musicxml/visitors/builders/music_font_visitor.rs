use crate::musicxml::utils::NodeUtils;
use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use roxmltree::Node;

/// Records the music font the document asks for: `<defaults><music-font
/// font-family>`, a comma-separated list in order of preference.
///
/// Only records the names. Which of them is available, and whether the user
/// overrides them, is decided once the walk is done -- see
/// [`choose_music_font`](crate::smufl::font_choice::choose_music_font).
pub struct MusicFontVisitor {}

impl<'a> Visitor<WalkerCtx<'a>> for MusicFontVisitor {
    fn enter_defaults(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        let Some(family) = element
            .children()
            .find(|n| n.has_tag("music-font"))
            .and_then(|n| n.attribute("font-family"))
        else {
            return;
        };

        ctx.layout.music_font = family_list(family);
    }
}

/// Splits a MusicXML `font-family` into its names, dropping the whitespace and
/// any quotes around each one.
fn family_list(family: &str) -> Vec<String> {
    family
        .split(',')
        .map(|name| name.trim().trim_matches(['"', '\'']).trim())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}
