use crate::score::core::clef::Clef;
use crate::score::core::duration_base::BaseDuration;
use crate::score::core::key::Key;
use crate::score::core::note_kind::NoteKind;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::voice::Voice;
use crate::score::visual::note::NoteId;
use std::collections::{BTreeMap, HashSet};

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Visibility {
    #[default]
    Unset,
    Hidden,
    Shown,
}

#[derive(Clone)]
pub struct PageInfo {
    pub page_number: u32,
}

#[derive(Clone)]
pub struct SystemInfo {
    pub index: u32,
    pub margin_left: Option<f32>,
    pub margin_right: Option<f32>,
    pub distance: Option<f32>,
    pub distance_top: Option<f32>,
}

#[derive(Clone)]
pub struct StaffInfo {
    pub number: StaffIdx,
    pub visibility: Visibility,

    pub explicitly_hidden: HashSet<StaffIdx>,
    pub explicitly_shown: HashSet<StaffIdx>,

    pub distances: BTreeMap<StaffIdx, f32>,
    pub staff_scaling: BTreeMap<StaffIdx, f32>,
    pub content_scaling: BTreeMap<StaffIdx, f32>,

    /// How many lines each staff is drawn with, from
    /// `<staff-details><staff-lines>`. A staff missing from the map is a normal
    /// five-line staff -- see [`Staff::DEFAULT_LINES`](crate::score::visual::staff::Staff).
    pub lines: BTreeMap<StaffIdx, usize>,

    /// Currently active clef, tracked across one part across staves.
    pub active_clef: BTreeMap<StaffIdx, Clef>,

    /// The opening clefs for each staff in this part in a system, reset on each new system.
    pub opening_clef: BTreeMap<StaffIdx, Clef>,

    /// The clef changes for each staff in this part in a single part measure.
    pub clef_changes: BTreeMap<StaffIdx, BTreeMap<u32, Clef>>,
}

impl StaffInfo {
    pub fn active_clef(&self, staff_idx: &StaffIdx, position: &u32) -> Clef {
        if let Some(clef_changes) = self.clef_changes.get(staff_idx) {
            let mut clef: Option<Clef> = None;
            for (clef_pos, clef_change) in clef_changes.iter() {
                if clef_pos > position {
                    break;
                }

                clef = Some(*clef_change);
            }

            if let Some(clef) = clef {
                return clef;
            }
        };

        if let Some(opening_clef) = self.opening_clef.get(staff_idx) {
            return *opening_clef;
        }

        Clef::Treble
    }
}

#[derive(Clone)]
pub struct MeasureInfo {
    pub number: u32,
    pub width: Option<f32>,
}

#[derive(Clone)]
pub struct WalkCursor {
    pub part_id: String,
    pub part_hidden_specified: Visibility,
    pub page: PageInfo,
    pub system: SystemInfo,
    pub staff: StaffInfo,
    pub measure: MeasureInfo,

    pub divisions: u32,
    pub duration: u32,
    pub beats: u8,
    pub beat_type: BaseDuration,
    pub position: u32,
    pub key: Key,
    pub voice: Voice,

    pub new_page: bool,
    pub new_system: bool,

    pub chord: bool,
    pub grace: bool,

    /// Whether the `<note>` currently being visited carries a `<cue>`.
    ///
    /// Sits beside `grace` because it is the same kind of fact -- a property of
    /// the note the walk is on, wanted by more than one visitor -- even though,
    /// unlike `grace`, it has no bearing on the position arithmetic. The two are
    /// mutually exclusive in the format, and a note carrying both is read as a
    /// grace note.
    pub cue: bool,

    /// Identity of the `<note>` currently being visited, handed out by
    /// [`WalkCursorVisitor`](crate::musicxml::visitors::walk_cursor_visitor::WalkCursorVisitor)
    /// on `enter_note`.
    ///
    /// Lives here rather than inside one visitor because more than one visitor
    /// needs it and they must agree: `ContentVisitor` stamps it onto the `Note`
    /// it builds, and `TieVisitor` uses it to name a tie's endpoints. Private
    /// counters in each would silently drift apart the moment one visitor's skip
    /// conditions changed.
    ///
    /// Deliberately **not** cleared by [`WalkCursor::reset`], which runs once per
    /// part -- resetting it there would make ids collide between parts. It just
    /// counts up for the life of the cursor, across both document walks; only
    /// uniqueness matters, not the actual values.
    pub note_id: NoteId,
}

impl WalkCursor {
    /// Assigns the next id and makes it current. Called once per `<note>`,
    /// before any other visitor in the chain sees the element.
    pub fn advance_note_id(&mut self) {
        self.note_id = self.note_id.next();
    }

    /// How prominently the `<note>` currently being visited should be drawn.
    /// `<grace>` and `<cue>` are mutually exclusive in the format, so a note
    /// carrying both is read as a grace note rather than compounding the two.
    pub fn note_kind(&self) -> NoteKind {
        if self.grace {
            NoteKind::Grace
        } else if self.cue {
            NoteKind::Cue
        } else {
            NoteKind::Normal
        }
    }

    pub fn reset(&mut self) {
        self.system.index = 0;

        self.page.page_number = 1;

        self.system.margin_left = None;
        self.system.margin_right = None;
        self.system.distance = None;
        self.system.distance_top = None;

        self.part_hidden_specified = Visibility::Unset;

        self.staff.number = 1.into();
        self.staff.distances.clear();
        self.staff.explicitly_hidden.clear();
        self.staff.explicitly_shown.clear();
        self.staff.staff_scaling.clear();
        self.staff.content_scaling.clear();
        self.staff.lines.clear();
        self.staff.active_clef.clear();
        self.staff.opening_clef.clear();

        self.measure.number = 0;
        self.measure.width = None;

        self.divisions = 8;
        self.beats = 4;
        self.beat_type = 4.into();
        self.voice = 1.into();
        self.key = Key::C_MAJOR;
        self.chord = false;
        self.grace = false;
        self.cue = false;

        self.new_page = true;
        self.new_system = true;
    }
}

impl Default for WalkCursor {
    fn default() -> Self {
        WalkCursor {
            part_id: "".to_string(),
            part_hidden_specified: Visibility::Unset,
            page: PageInfo { page_number: 1 },
            system: SystemInfo {
                index: 0,
                margin_left: None,
                margin_right: None,
                distance: None,
                distance_top: None,
            },
            staff: StaffInfo {
                number: 1.into(),
                distances: BTreeMap::new(),
                visibility: Visibility::Unset,
                explicitly_hidden: HashSet::new(),
                explicitly_shown: HashSet::new(),
                staff_scaling: BTreeMap::new(),
                content_scaling: BTreeMap::new(),
                lines: BTreeMap::new(),
                active_clef: BTreeMap::new(),
                opening_clef: BTreeMap::new(),
                clef_changes: BTreeMap::new(),
            },
            measure: MeasureInfo {
                number: 0,
                width: None,
            },

            duration: 0,
            divisions: 8,
            beats: 4,
            beat_type: 4.into(),
            position: 0,
            voice: 1.into(),
            key: Key::C_MAJOR,

            chord: false,
            grace: false,
            cue: false,
            note_id: NoteId::default(),

            new_page: true,
            new_system: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionError {
    /// Moving the position backwards would take it before the start of the measure.
    Underflow,
    /// Moving the position forwards overflowed the counter.
    Overflow,
}

impl WalkCursor {
    pub fn begin_measure(&mut self) {
        self.position = 0;
    }

    pub fn set_divisions(&mut self, divisions: u32) {
        self.divisions = divisions;
    }

    pub fn set_beats(&mut self, beats: u8) {
        self.beats = beats;
    }

    pub fn set_beat_type(&mut self, beat_type: BaseDuration) {
        self.beat_type = beat_type;
    }

    pub fn apply_backup(&mut self, duration: u32) -> Result<(), PositionError> {
        self.position = self
            .position
            .checked_sub(duration)
            .ok_or(PositionError::Underflow)?;
        Ok(())
    }

    pub fn apply_forward(&mut self, duration: u32) -> Result<(), PositionError> {
        self.position = self
            .position
            .checked_add(duration)
            .ok_or(PositionError::Overflow)?;
        Ok(())
    }

    /// Call when entering a `<note>`, after determining whether it's a chord
    /// note, a grace note and/or a cue note, and (for non-grace notes) its
    /// duration.
    pub fn enter_note(
        &mut self,
        duration: u32,
        is_chord: bool,
        is_grace: bool,
        is_cue: bool,
    ) -> Result<(), PositionError> {
        self.chord = is_chord;
        self.grace = is_grace;
        self.cue = is_cue;

        if is_chord {
            // move the position backwards (by the previous note duration), so that
            // this chord note starts at the same position as the note it's attached to.
            self.apply_backup(self.duration)?;
        }

        self.duration = if is_grace { 0 } else { duration };
        Ok(())
    }

    /// Call when exiting a `<note>`, moving the position forward by its duration.
    pub fn exit_note(&mut self) -> Result<(), PositionError> {
        self.apply_forward(self.duration)
    }

    /// Divisions are annotated per quarter note. For example, if duration = 1 and
    /// divisions = 2, this is an eighth note duration.
    pub fn position_exceeds_measure(&self) -> bool {
        let whole_beats = self.beat_type.as_int() as f32; // eg 4. for a 3/4 measure, 8. for a 7/8 measure.
        let quarter_beats = self.beats as f32 * (4. / whole_beats); // eg 1.5 for 3/8, 4. for 2/2.
        let divisions_in_measure = self.divisions as f32 * quarter_beats;

        self.position as f32 > divisions_in_measure
    }
}
