use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::core::note_kind::NoteKind;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::smufl::smufl_font::SmuflFont;

/// What every element resolves its own appearance and size from, threaded
/// through the [`Layoutable::measure`] pass.
///
/// Three of these are layout config, in falling precedence: the caller's
/// overrides, the document's declared defaults, and the hard-coded fallbacks.
/// The fourth, `font`, is not config but a resource -- the glyph metrics an
/// element needs to size itself.
///
/// The font is here rather than being handed to elements at construction
/// because *which* glyph an element draws is not always settled by then. A
/// group symbol resolves its shape from the user layout, which is re-resolved on
/// every `arrange_score` -- the wasm bindings walk a document once and re-lay it
/// out per render -- so an element that had picked its glyph during the walk
/// would be frozen at whatever the first render asked for.
#[derive(Clone, Copy)]
pub struct LayoutParams<'a> {
    pub score_defaults: &'a ScoreDefaults,
    pub user_layout: &'a UserLayout,
    pub app_defaults: &'a AppDefaults,
    pub font: &'a SmuflFont,
}

impl LayoutParams<'_> {
    /// The fraction of full size a note of this kind is drawn at.
    ///
    /// Lives here rather than on [`NoteKind`] because it is the one place the
    /// precedence is decided, and everything a reduced note owns has to agree on
    /// it: the notehead, its dots, the stem, the flag, and the beams the
    /// enclosing [`PartMeasure`](crate::score::visual::part_measure::PartMeasure)
    /// draws. Resolving it separately in each of those is how a grace note ends
    /// up with beams reduced by a different number than its noteheads.
    pub fn note_size(&self, kind: NoteKind) -> f32 {
        match kind {
            NoteKind::Normal => 1.,
            NoteKind::Grace => self
                .user_layout
                .note_size_grace
                .or(self.score_defaults.appearance.note_size_grace)
                .unwrap_or(self.app_defaults.note_size_grace),
            NoteKind::Cue => self
                .user_layout
                .note_size_cue
                .or(self.score_defaults.appearance.note_size_cue)
                .unwrap_or(self.app_defaults.note_size_cue),
        }
    }
}

pub trait Layoutable {
    /// Resolves this element's appearance from `params` and sizes it (and its
    /// children). This is the single downward pass that used to be a separate
    /// `apply_layout` walk followed by a size-only `measure` walk.
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>);

    fn arrange(&mut self, origin: &XY);
}
