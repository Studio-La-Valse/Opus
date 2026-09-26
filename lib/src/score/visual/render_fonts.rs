use crate::drawable::elements::text::FontSpec;
use crate::score::layout_options::{APP_DEFAULTS, UserLayout};
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
    /// The face part / part-group names are drawn in.
    pub group_name: FontSpec<'a>,
}

impl<'a> RenderFonts<'a> {
    /// Builds the set from the music font and `user_layout`'s text faces, each
    /// resolved user -> the font's `textFontFamily` -> app.
    ///
    /// The font's tier is a whole CSS family list rather than one name -- its
    /// recommended faces are rarely installed, so the renderer is handed every
    /// one of them to fall through. See
    /// [`UserLayout::from_engraving_defaults`].
    pub fn resolve(smufl: &'a SmuflFont, user_layout: &'a UserLayout) -> RenderFonts<'a> {
        let font = &smufl.layout;
        let resolve = |user: &'a Option<String>, font: &'a Option<String>, app: &'static str| {
            FontSpec::plain(user.as_deref().or(font.as_deref()).unwrap_or(app))
        };

        RenderFonts {
            music: FontSpec::plain(&smufl.meta.font),
            smufl,
            title: resolve(
                &user_layout.title.font,
                &font.title.font,
                APP_DEFAULTS.title.font,
            ),
            lyric: resolve(
                &user_layout.lyric.font,
                &font.lyric.font,
                APP_DEFAULTS.lyric.font,
            ),
            group_name: resolve(
                &user_layout.group_name.font,
                &font.group_name.font,
                APP_DEFAULTS.group_name.font,
            ),
        }
    }

    /// Convenience for callers that only render music glyphs: every text face
    /// falls back to the music font.
    pub fn music_only(smufl: &'a SmuflFont) -> RenderFonts<'a> {
        let music = FontSpec::plain(&smufl.meta.font);
        RenderFonts {
            music,
            smufl,
            title: music,
            lyric: music,
            group_name: music,
        }
    }
}
