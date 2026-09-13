use crate::drawable::elements::line::Line;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::voice::Voice;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::chord::Chord;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::Note;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// Staff-line index of the top staff line; notes with a lower index sit above the
/// staff and need ledger lines. The top line is where a staff is anchored, so
/// this holds however many lines it has.
const LEDGER_ABOVE_STAFF_LINE: i32 = 0;

/// Staff-line index just below the bottom staff line of a staff of `lines`
/// lines; notes with a higher index sit below the staff and need ledger lines.
///
/// Indices count half-spaces down from the top line, so the bottom line of a
/// five-line staff is 8 and this is 9 -- the half-space between it and the first
/// note that needs a ledger.
fn ledger_below_staff_line(lines: usize) -> i32 {
    2 * (lines.saturating_sub(1) as i32) + 1
}

/// Which side of the staff a note (and therefore its ledger lines) sits on.
enum LedgerSide {
    Above,
    Below,
}

#[derive(Default)]
pub struct PartMeasure {
    pub part_id: String,
    pub number: u32,

    pub specified_width: Option<f32>,
    pub final_width: f32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,

    pub chords: BTreeMap<Voice, Vec<Chord>>,
    pub ledgers: Vec<Line>,

    pub color: Color,

    pub ledger_thickness: f32,
    pub ledger_width: f32,
}

impl PartMeasure {
    pub fn new(part_id: String, number: u32) -> Self {
        Self {
            part_id,
            number,
            ..Default::default()
        }
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for grace in [false, true] {
            let chord_groups = collect_voices(&mut self.chords, grace);

            for mut chords in chord_groups {
                strategy.rebeam(&mut chords)
            }
        }
    }

    pub fn arrange(&mut self, origin: &XY) {
        self.origin = *origin;
    }

    /// Places everything this measure draws: run by
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger)
    /// once every measure in the part has its own `origin` from
    /// [`arrange`](Self::arrange).
    pub fn arrange_content(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.arrange_chords(staff_ctx);
        self.arrange_ledgers(staff_ctx);
    }

    fn arrange_chords(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        for chord in self.chords.values_mut().flatten() {
            chord.arrange_ctx(&self.origin, staff_ctx);
        }
    }

    fn arrange_ledgers(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.ledgers.clear();

        for chord in self.chords.values().flatten() {
            for (idx, staff_ctx) in staff_ctx.iter() {
                // A staff drawn without any lines has nothing for a ledger line
                // to extend, so notes on it get none.
                if staff_ctx.lines == 0 {
                    continue;
                }

                let each_line = (Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling;
                let below_staff_line = ledger_below_staff_line(staff_ctx.lines);
                let key = |n: &&Note| OrderedFloat(n.xy.y);

                if let Some(note) = chord
                    .notes
                    .iter()
                    .filter(|n| n.staff == *idx)
                    .min_by_key(key)
                    && note.staff_line < LEDGER_ABOVE_STAFF_LINE
                {
                    let lines =
                        self.ledger_lines(note, LedgerSide::Above, each_line, below_staff_line);
                    self.ledgers.extend(lines);
                }

                if let Some(note) = chord
                    .notes
                    .iter()
                    .filter(|n| n.staff == *idx)
                    .max_by_key(key)
                    && note.staff_line > below_staff_line
                {
                    let lines =
                        self.ledger_lines(note, LedgerSide::Below, each_line, below_staff_line);
                    self.ledgers.extend(lines);
                }
            }
        }
    }

    /// The ledger lines for a single note that sits `side` of its staff: one
    /// short horizontal line on every even staff-line index between the note and
    /// the staff edge, stepping `each_line` back towards the staff each line.
    fn ledger_lines(
        &self,
        note: &Note,
        side: LedgerSide,
        each_line: f32,
        below_staff_line: i32,
    ) -> Vec<Line> {
        let ledger_width = note.width + 5.;
        let anchor = note.xy.mv(note.width / 2., 0.);
        let left = anchor.mv(ledger_width / -2., 0.);
        let right = anchor.mv(ledger_width / 2., 0.);

        // Staff-line indices from the note inward to the staff edge, plus the
        // per-line dy step (towards the staff, so away from the note).
        let (lines, step): (Vec<i32>, f32) = match side {
            LedgerSide::Above => (
                (note.staff_line..=LEDGER_ABOVE_STAFF_LINE - 1).collect(),
                each_line,
            ),
            LedgerSide::Below => (
                (below_staff_line + 1..=note.staff_line).rev().collect(),
                -each_line,
            ),
        };

        let mut out = Vec::new();
        let mut dy = 0.;
        for line in lines {
            if line % 2 == 0 {
                out.push(Line {
                    start: left.mv(0., dy),
                    end: right.mv(0., dy),
                    stroke_width: self.ledger_thickness,
                    stroke_color: self.color,
                });
            }
            dy += step;
        }
        out
    }
}

impl PartMeasure {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.ledger_thickness = user_layout
            .staff_line_width
            .or(score_defaults.appearance.staff)
            .unwrap_or(app_defaults.staff_line_width);

        for chord in self.chords.values_mut().flatten() {
            chord.resolve_layout(params);
        }
    }
}

impl PartMeasure {
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.height = available.y;

        for chord in self.chords.values_mut().flatten() {
            chord.measure(available, params);
        }
    }
}

fn collect_voices(
    chord_groups: &mut BTreeMap<Voice, Vec<Chord>>,
    grace: bool,
) -> Vec<Vec<&mut Chord>> {
    let mut result: Vec<Vec<&mut Chord>> = Vec::new();

    for chords in chord_groups.values_mut() {
        let mut group: Vec<&mut Chord> = Vec::new();

        for chord in chords {
            if chord.grace != grace {
                continue;
            }

            group.push(chord);
        }

        result.push(group);
    }

    result
}
