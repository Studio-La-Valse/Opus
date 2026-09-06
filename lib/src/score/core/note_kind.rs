/// How prominently one note or rest is drawn: MusicXML's `<grace>` and `<cue>`
/// markers, or neither.
///
/// Carried by the drawable elements (`Note`, `Rest`, `Stem`, `Flag`, `Dot`)
/// **instead of** the size factor it stands for. The factor comes from
/// `<defaults><appearance><note-size>` and can be overridden by the caller, and
/// the two halves of the pipeline resolve at different times: the document walk
/// is cached and re-arranged for whatever layout a later render asks for, so a
/// factor baked in during the walk would be stuck at whatever the first render
/// happened to ask for. The kind is a fact about the note and never changes; the
/// factor is a layout decision, resolved per render by
/// [`LayoutParams::note_size`](crate::score::visual::layoutable::LayoutParams::note_size).
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub enum NoteKind {
    #[default]
    Normal,
    Grace,
    Cue,
}
