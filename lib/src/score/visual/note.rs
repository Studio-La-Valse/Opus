use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::layout_options::APP_DEFAULTS;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::dot::Dot;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note_scale::NoteScale;
use crate::score::visual::placed::Placed;
use crate::score::visual::stem::UpDown;
use crate::score::visual::system::SystemKey;
use crate::smufl::glyphs::notehead::Notehead;

/// Identifies one [`Note`] -- or one [`Rest`](crate::score::visual::rest::Rest),
/// which is a `<note>` in MusicXML and counted as one here -- within a
/// [`Score`](crate::score::visual::score::Score).
///
/// The visual tree is a pure containment hierarchy, so it cannot express a
/// relation between two notes that sit in different branches of it -- which is
/// exactly what a tie is. Ids let [`Tie`](crate::score::visual::tie::Tie) name
/// its two endpoints without needing a pointer into the tree, and
/// [`ClefChange`](crate::score::visual::clef::ClefChange) name the note or rest
/// it is drawn in front of.
///
/// Handed out by
/// [`WalkCursor`](crate::score::walk_cursor::WalkCursor), so that every visitor
/// in the chain agrees on which note it is looking at. They are assigned during
/// the cached `walk_document` half of the pipeline, so they stay stable across
/// the repeated `arrange_score` calls the wasm render path makes. Only
/// uniqueness is meaningful -- never read anything into the values themselves.
#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct NoteId(u32);

impl NoteId {
    /// The next id in sequence.
    pub fn next(self) -> Self {
        NoteId(self.0 + 1)
    }
}

impl From<u32> for NoteId {
    fn from(value: u32) -> Self {
        NoteId(value)
    }
}

/// A note's finished geometry, copied out of the tree so a whole-score pass
/// can borrow the score immutably while it resolves relations between notes,
/// then mutably while it writes the result back onto the tree. Built by
/// [`Score::note_anchors`](crate::score::visual::score::Score::note_anchors);
/// the only consumer today is
/// [`split_tie`](crate::score::visual::tie::split_tie), but nothing here is
/// tie-specific.
#[derive(Copy, Clone, Debug)]
pub struct NoteAnchor {
    pub key: SystemKey,
    /// Left edge of the notehead at its vertical centre -- exactly what
    /// `Note::xy` is, per `ContentArranger::arrange_note_ctx` and
    /// `ContentArranger::part_measure_ledger_lines`.
    pub left: XY,
    pub width: f32,
    /// Right edge of the `PartMeasure` this note sits in. A tie broken across a
    /// system runs its opening fragment out to here, since "the space available
    /// to the tie" is the remainder of its own measure.
    pub measure_right: f32,
    /// The note's own scale factor, so grace notes get proportionate ties.
    pub scale: f32,
    pub staff_line: i32,
    /// The owning chord's stem direction, for
    /// [`TieSide::infer`](crate::score::visual::tie::TieSide::infer).
    pub stem: Option<UpDown>,
    pub color: Color,
}

pub struct Note {
    pub id: NoteId,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub default_x: f32,
    pub staff_line: i32,
    pub staff: StaffIdx,

    /// What this note's size is derived from: the staff's content scaling and
    /// whether the note is normal, grace or cue. Kept unresolved so that the
    /// note-size factor can be re-decided on every render; see [`NoteScale`].
    pub size: NoteScale,
    /// The factor `size` resolved to, written by `resolve_layout` and so only
    /// meaningful once that pass has run.
    pub scale: f32,

    pub color: Color,

    pub dots: Vec<Dot>,
    /// Resolved centre-to-centre dot step (and notehead-edge-to-first-dot gap),
    /// in world units; see [`DotDefaults::spacing`](crate::score::layout_options::DotDefaults).
    pub dot_spacing: f32,

    /// The SMuFL name of the notehead this note draws, which the walk settles
    /// without knowing the font.
    pub glyph_name: &'static str,
    /// `glyph_name` looked up in the font, written by `resolve_layout`.
    glyph: Option<Notehead>,
    pub accidental: Option<Accidental>,
}

impl Note {
    pub fn new(
        id: NoteId,
        glyph_name: &'static str,
        default_x: f32,
        staff: StaffIdx,
        staff_line: i32,
        size: NoteScale,
        dots: u8,
    ) -> Self {
        Note {
            id,
            glyph_name,
            glyph: None,
            accidental: None,

            default_x,
            staff,
            staff_line,

            dots: (0..dots).map(|_| Dot::new(size)).collect(),
            dot_spacing: 0.,

            size,
            scale: size.content_scale,

            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            color: Color::default(),
        }
    }

    /// The notehead glyph in the font the score is arranged with.
    ///
    /// # Panics
    ///
    /// Before `resolve_layout` has run: the walk that builds the note does not
    /// know the font.
    pub fn glyph(&self) -> &Notehead {
        self.glyph
            .as_ref()
            .expect("note glyph read before resolve_layout")
    }
}

impl Placed for Note {
    fn xy(&self) -> XY {
        self.xy
    }

    fn scale(&self) -> f32 {
        self.scale
    }
}

impl Note {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        self.scale = self.size.resolve(params);
        self.glyph = Some(params.font.notehead(self.glyph_name));

        self.color = params.foreground_color();
        self.dot_spacing = params
            .user_layout
            .dot
            .spacing
            .unwrap_or(APP_DEFAULTS.dot.spacing);

        if let Some(accidental) = self.accidental.as_mut() {
            accidental.resolve_layout(params);
        }

        for dot in self.dots.iter_mut() {
            dot.resolve_layout(params);
        }
    }
}
