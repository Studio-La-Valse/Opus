use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::core::note_kind::NoteKind;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;

/// The three layout-config sources every element resolves its own appearance
/// from, threaded through the [`Layoutable::measure`] pass: the document's
/// declared defaults, the caller's overrides, and the hard-coded fallbacks.
#[derive(Clone, Copy)]
pub struct LayoutParams<'a> {
    pub score_defaults: &'a ScoreDefaults,
    pub user_layout: &'a UserLayout,
    pub app_defaults: &'a AppDefaults,
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
