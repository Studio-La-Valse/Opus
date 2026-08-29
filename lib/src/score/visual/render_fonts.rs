use crate::drawable::elements::text::FontSpec;
use crate::smufl::smufl_font::SmuflFont;

/// The fonts a render pass draws text in: the SMuFL music font (for glyphs)
/// plus the caller's chosen title / lyric text faces. Passed to every
/// [`RenderPass`](super::render_pass::RenderPass) method in place of a bare
/// `&SmuflFont` so that text-emitting passes (titles, lyrics, tempo marks, ...)
/// have a face to reach for without another signature change.
pub struct RenderFonts<'a> {
    pub smufl: &'a SmuflFont,
    /// The music font as a [`FontSpec`] -- `smufl.meta.font`, regular/upright.
    pub music: FontSpec<'a>,
    pub title: FontSpec<'a>,
    pub lyric: FontSpec<'a>,
}

impl<'a> RenderFonts<'a> {
    /// Builds the set from the music font plus explicit title / lyric specs.
    fn new(smufl: &'a SmuflFont, title: FontSpec<'a>, lyric: FontSpec<'a>) -> RenderFonts<'a> {
        RenderFonts {
            music: FontSpec::plain(&smufl.meta.font),
            smufl,
            title,
            lyric,
        }
    }

    /// Builds the set from the music font and the (already default-resolved)
    /// title / lyric families.
    pub fn create(
        smufl: &'a SmuflFont,
        title_font: &'a str,
        lyric_font: &'a str,
    ) -> RenderFonts<'a> {
        RenderFonts::new(
            smufl,
            FontSpec::plain(title_font),
            FontSpec::plain(lyric_font),
        )
    }

    /// Convenience for callers that only render music glyphs: title and lyric
    /// both fall back to the music font.
    pub fn music_only(smufl: &'a SmuflFont) -> RenderFonts<'a> {
        let music = FontSpec::plain(&smufl.meta.font);
        RenderFonts {
            music,
            smufl,
            title: music,
            lyric: music,
        }
    }
}
