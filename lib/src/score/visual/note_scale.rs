use crate::score::core::note_kind::NoteKind;
use crate::score::visual::layoutable::LayoutParams;

/// Everything the size of one note-ish element (a notehead, rest, stem, flag or
/// augmentation dot) is derived from, split by when it can be known.
///
/// `content_scale` comes from the document and is settled the moment the element
/// is built. `kind` stands in for a factor that is *not* settled then:
/// `<note-size>` is a layout decision a caller can override, and the document
/// walk is cached and re-arranged for whatever layout a later render asks for.
/// Multiplying the factor in during the walk would freeze it at whatever the
/// first render happened to ask for.
///
/// So the two travel together unresolved, and [`resolve`](Self::resolve) turns
/// them into the single factor the geometry uses, once per `measure`.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct NoteScale {
    /// The staff's own content scaling, from `<staff-details><staff-size>`.
    pub content_scale: f32,
    /// Whether this element belongs to a normal, grace or cue note.
    pub kind: NoteKind,
}

impl NoteScale {
    pub fn new(content_scale: f32, kind: NoteKind) -> Self {
        Self {
            content_scale,
            kind,
        }
    }

    /// The factor the element is actually drawn at under `params`.
    pub fn resolve(&self, params: LayoutParams<'_>) -> f32 {
        self.content_scale * params.note_size(self.kind)
    }
}
