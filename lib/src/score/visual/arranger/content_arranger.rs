//! Places every note, rest, ledger line and opening/closing clef, key and
//! time signature in the tree, once the container pass ([`PageArranger`])
//! has placed every page, system, section, part-group, part, staff and
//! measure.
//!
//! Container geometry never reads content geometry -- a measure's width comes
//! from the walk and the `measure` pass, a staff's height from its line count,
//! a part's height from its staves -- so the container pass can run to
//! completion on its own, and content placement can be pulled out into its own
//! pass rather than being threaded through the container one.

use crate::drawable::elements::line::Line;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::chord::Chord;
use crate::score::visual::clef::ClefAnchor;
use crate::score::visual::dot::Dot;
use crate::score::visual::key_signature::KeySignature;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::Note;
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::rest::Rest;
use crate::score::visual::score::Score;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::staff_measure::{MeasureStartPaddings, StaffMeasure};
use crate::score::visual::stem::UpDown;
use crate::score::visual::system::System;
use crate::score::visual::time_signature::TimeSignature;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// Gap in tenths between a notehead's left edge and the right edge of its
/// accidental. Scaled with the note, like everything else it owns.
const ACCIDENTAL_GAP: f32 = 2.;

/// Staff-line index of the top staff line; notes with a lower index sit above the
/// staff and need ledger lines. The top line is where a staff is anchored, so
/// this holds however many lines it has.
const LEDGER_ABOVE_STAFF_LINE: i32 = 0;

/// Rebuilds every content element's position from the containers the
/// [`PageArranger`](crate::score::visual::arranger::PageArranger) has already
/// placed.
///
/// The second of [`SCORE_ARRANGERS`](crate::score::visual::arranger::SCORE_ARRANGERS):
/// content placement needs every container's final position, and
/// [`BeamArranger`](crate::score::visual::arranger::BeamArranger),
/// [`TieArranger`](crate::score::visual::arranger::TieArranger) and
/// [`ClefChangeArranger`](crate::score::visual::arranger::ClefChangeArranger)
/// need the content it places.
///
/// One method per element, each delegating to the methods of the elements it
/// owns, so arranging a [`Chord`] arranges its notes and so on down to the dots.
/// Holds no state: what an arrangement needs is handed to each call, and
/// sizing is settled by the time it runs, so the only layout parameters it
/// reads are the [`MeasureStartPaddings`] resolved once per pass.
pub struct ContentArranger;

impl ScoreArranger for ContentArranger {
    fn arrange(&self, score: &mut Score, params: LayoutParams<'_>) {
        let paddings = MeasureStartPaddings::resolve(params);

        for page in score.pages.values_mut() {
            for system in page.systems.values_mut() {
                self.walk_system(system, &paddings);
            }
        }
    }
}

// ---- element placement ----

impl ContentArranger {
    /// Lines the elements that open a measure -- the clef, the key signature and
    /// the time signature -- up into three columns shared by every drawn staff
    /// of this system.
    ///
    /// Left to itself a staff places these one after the other from its own left
    /// edge, so a staff whose key signature is narrower than its neighbour's
    /// starts its time signature further left than theirs. That shows in any
    /// score with transposing instruments: a score in C flat major carries seven
    /// flats, the trumpets in B flat five, and an unpitched percussion staff
    /// none at all, which is three different time-signature positions down one
    /// system.
    ///
    /// A column, like a measure width, is a fact about the whole system that no
    /// single staff can work out on its own -- so, like
    /// [`consolidate_measure_width`](System::consolidate_measure_width), the
    /// system settles it. Only the *start* of each column is shared: what a
    /// staff draws there is still its own, so a narrower key signature simply
    /// leaves more air before the next column.
    ///
    /// Run once the staves have been placed and the measures have the left edge
    /// these offsets are measured from. Only drawn staves take part, which is
    /// also all the compositor visits.
    pub fn arrange_system_measure_starts(
        &self,
        system: &mut System,
        paddings: &MeasureStartPaddings,
    ) {
        let mut by_measure: BTreeMap<u32, Vec<&mut StaffMeasure>> = BTreeMap::new();
        for staff in system.visible_staves_mut() {
            for (number, measure) in staff.measures.iter_mut() {
                by_measure.entry(*number).or_default().push(measure);
            }
        }

        for measures in by_measure.values_mut() {
            // How far right each staff has reached so far, from the measure's
            // own left edge. The columns resolve outwards from here, one at a
            // time, because each is measured from where the previous one left
            // the widest staff.
            let mut edges = vec![0.; measures.len()];

            self.arrange_column(
                measures,
                &mut edges,
                paddings.clef,
                Self::arrange_staff_measure_clef_start,
            );
            self.arrange_column(
                measures,
                &mut edges,
                paddings.key_signature,
                Self::arrange_staff_measure_key_signature_start,
            );
            self.arrange_column(
                measures,
                &mut edges,
                paddings.time_signature,
                Self::arrange_staff_measure_time_signature_start,
            );
        }
    }

    /// Places everything this measure draws: run once every measure in the part
    /// has its own `origin` from
    /// [`PageArranger`](crate::score::visual::arranger::PageArranger).
    pub fn arrange_part_measure_content(
        &self,
        measure: &mut PartMeasure,
        staff_ctx: &BTreeMap<StaffIdx, StaffCtx>,
    ) {
        self.arrange_part_measure_chords(measure, staff_ctx);
        self.arrange_part_measure_ledgers(measure, staff_ctx);
    }

    /// Places the opening clef `dx` right of this measure's left edge, and
    /// answers how far right its ink then reaches -- or `None` when the measure
    /// opens without a clef, which is every measure but the first of a system.
    ///
    /// The three `arrange_staff_measure_*_start` methods share this shape so that
    /// [`arrange_system_measure_starts`](Self::arrange_system_measure_starts)
    /// can place each of them the same way: it decides the offset, they report
    /// what the next column has to clear.
    pub fn arrange_staff_measure_clef_start(
        &self,
        measure: &mut StaffMeasure,
        dx: f32,
    ) -> Option<f32> {
        let origin = measure.xy;
        let scaling = measure.scale;

        let clef = measure.clef_start.as_mut()?;
        clef.place(ClefAnchor::LeftEdgeAt(origin.x + dx), origin.y, scaling);

        // The columns are worked out as offsets from the measure's own left
        // edge, so the edge reported back is one too.
        Some(dx + clef.width)
    }

    /// Places the opening key signature. Always answers an edge, even for a
    /// staff carrying no accidentals: an empty key signature is zero wide, so
    /// the time signature still lands a padding right of the shared column
    /// rather than crowding whatever came before it.
    pub fn arrange_staff_measure_key_signature_start(
        &self,
        measure: &mut StaffMeasure,
        dx: f32,
    ) -> Option<f32> {
        self.arrange_key_signature(&mut measure.key_signature_start, &measure.xy.mv(dx, 0.));

        Some(dx + measure.key_signature_start.width)
    }

    /// Places the opening time signature, which only the measures that open a
    /// score or announce a change carry.
    pub fn arrange_staff_measure_time_signature_start(
        &self,
        measure: &mut StaffMeasure,
        dx: f32,
    ) -> Option<f32> {
        let xy = measure.xy;

        let time_signature = measure.time_signature_start.as_mut()?;
        self.arrange_time_signature(time_signature, &xy.mv(dx, 0.));

        Some(dx + time_signature.width)
    }

    /// Places everything this measure carries other than its three opening
    /// columns, which
    /// [`arrange_system_measure_starts`](Self::arrange_system_measure_starts)
    /// places once every staff measure of the system is placed. Run after the
    /// container pass has placed this measure itself.
    pub fn arrange_staff_measure_content(&self, measure: &mut StaffMeasure) {
        self.arrange_staff_measure_time_signature_end(measure);
        self.arrange_staff_measure_clef_end(measure);
        self.arrange_staff_measure_rests(measure);
    }

    /// here, origin is the origin of the part measure.
    pub fn arrange_chord_ctx(
        &self,
        chord: &mut Chord,
        origin: &XY,
        staff_ctx: &BTreeMap<StaffIdx, StaffCtx>,
    ) {
        chord.xy = *origin;

        self.arrange_chord_notes(chord, staff_ctx);
        chord.place_stem(staff_ctx);
        self.arrange_chord_dots(chord, staff_ctx);

        // rearrange the accidentals so that they don't overlap.
        self.rearrange_chord_accidentals(chord);
    }

    /// here, origin is the origin of the staff measure, so adjust y coordinate for staff distance.
    pub fn arrange_rest_ctx(&self, rest: &mut Rest, origin: &XY, staff_ctx: &StaffCtx) {
        self.arrange_rest_glyph(rest, origin, staff_ctx);
        self.arrange_rest_dots(rest, staff_ctx);
    }

    pub fn arrange_note_ctx(&self, note: &mut Note, origin: &XY, staff_ctx: &StaffCtx) {
        let staff_top = origin.mv(0., staff_ctx.distance_from_top);
        let note_dy =
            note.staff_line as f32 * ((Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling);
        let note_top = staff_top.mv(0., note_dy);
        note.xy = note_top.mv(note.default_x, 0.);

        self.arrange_note_accidental(note);
    }

    /// `origin` is the fully-resolved centre of this dot, worked out by the
    /// owning note or rest.
    pub fn arrange_dot(&self, dot: &mut Dot, origin: &XY) {
        dot.xy = *origin;
    }

    /// Provided origin is the right origin of the accidental.
    pub fn arrange_accidental(&self, accidental: &mut Accidental, origin: &XY) {
        accidental.xy = origin.mv(-accidental.width, 0.);
    }

    pub fn arrange_key_signature(&self, key_signature: &mut KeySignature, origin: &XY) {
        key_signature.xy = *origin;

        let origin = key_signature.xy;
        let spacing = key_signature.accidental_spacing();
        let half_space = key_signature.line_space() / 2.;

        let mut x = origin.x;
        for (line, acc) in key_signature.accidentals.iter_mut() {
            let y = origin.y + half_space * (*line as f32);
            let xy = XY { x, y }.mv(acc.width, 0.);
            self.arrange_accidental(acc, &xy);

            x += acc.width + spacing;
        }
    }

    pub fn arrange_time_signature(&self, time_signature: &mut TimeSignature, origin: &XY) {
        time_signature.xy = *origin;
    }
}

// ---- internals ----

impl ContentArranger {
    /// Places one of the three opening columns across the staves of a single
    /// measure.
    ///
    /// Every staff draws at the same offset -- a padding clear of the furthest
    /// right any of them has reached -- and `edges` then advances to wherever each
    /// staff's own element ended. A staff with nothing to draw in this column
    /// answers `None` and keeps the edge it had, so it neither widens the column nor
    /// carries a gap for an element it does not have.
    fn arrange_column(
        &self,
        measures: &mut [&mut StaffMeasure],
        edges: &mut [f32],
        padding: f32,
        place: fn(&ContentArranger, &mut StaffMeasure, f32) -> Option<f32>,
    ) {
        let column = measures
            .iter()
            .zip(edges.iter())
            .map(|(measure, edge)| edge + padding * measure.scale)
            .fold(0., f32::max);

        for (measure, edge) in measures.iter_mut().zip(edges.iter_mut()) {
            if let Some(right) = place(self, measure, column) {
                *edge = right;
            }
        }
    }

    fn arrange_staff_measure_time_signature_end(&self, measure: &mut StaffMeasure) {
        if let Some(ref mut prepare_time_signature) = measure.time_signature_end {
            let pos = measure
                .xy
                .mv(measure.width - prepare_time_signature.width - 5., 0.);
            self.arrange_time_signature(prepare_time_signature, &pos);
        }
    }

    fn arrange_staff_measure_clef_end(&self, measure: &mut StaffMeasure) {
        let origin = measure.xy;
        let scaling = measure.scale;
        let measure_right = origin.x + measure.width;

        if let Some(clef) = measure.clef_end.as_mut() {
            clef.place(ClefAnchor::GapBefore(measure_right), origin.y, scaling);
        }
    }

    fn arrange_staff_measure_rests(&self, measure: &mut StaffMeasure) {
        let staff_ctx = StaffCtx {
            hidden: false,
            distance_from_top: 0.,
            scaling: measure.scale,
            lines: measure.lines,
        };
        for rest in measure.rests.iter_mut() {
            // A whole-measure rest carries no position of its own and is centred
            // in whatever width the measure ended up with.
            let dx: f32 = rest.default_x.unwrap_or(measure.width / 2.);

            let glyph_origin = measure.xy.mv(dx, 0.);
            self.arrange_rest_ctx(rest, &glyph_origin, &staff_ctx);
        }
    }

    fn arrange_part_measure_chords(
        &self,
        measure: &mut PartMeasure,
        staff_ctx: &BTreeMap<StaffIdx, StaffCtx>,
    ) {
        for chord in measure.chords.values_mut().flatten() {
            self.arrange_chord_ctx(chord, &measure.origin, staff_ctx);
        }
    }

    fn arrange_part_measure_ledgers(
        &self,
        measure: &mut PartMeasure,
        staff_ctx: &BTreeMap<StaffIdx, StaffCtx>,
    ) {
        measure.ledgers.clear();

        for chord in measure.chords.values().flatten() {
            for (idx, staff_ctx) in staff_ctx.iter() {
                // A staff drawn without any lines has nothing for a ledger line
                // to extend, so notes on it get none.
                if staff_ctx.lines == 0 {
                    continue;
                }

                let each_line = (Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling;
                let below_staff_line = self.ledger_below_staff_line(staff_ctx.lines);
                let key = |n: &&Note| OrderedFloat(n.xy.y);

                if let Some(note) = chord
                    .notes
                    .iter()
                    .filter(|n| n.staff == *idx)
                    .min_by_key(key)
                    && note.staff_line < LEDGER_ABOVE_STAFF_LINE
                {
                    let lines = self.part_measure_ledger_lines(
                        measure,
                        note,
                        LedgerSide::Above,
                        each_line,
                        below_staff_line,
                    );
                    measure.ledgers.extend(lines);
                }

                if let Some(note) = chord
                    .notes
                    .iter()
                    .filter(|n| n.staff == *idx)
                    .max_by_key(key)
                    && note.staff_line > below_staff_line
                {
                    let lines = self.part_measure_ledger_lines(
                        measure,
                        note,
                        LedgerSide::Below,
                        each_line,
                        below_staff_line,
                    );
                    measure.ledgers.extend(lines);
                }
            }
        }
    }

    /// The ledger lines for a single note that sits `side` of its staff: one
    /// short horizontal line on every even staff-line index between the note and
    /// the staff edge, stepping `each_line` back towards the staff each line.
    fn part_measure_ledger_lines(
        &self,
        measure: &PartMeasure,
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
                    stroke_width: measure.ledger_thickness,
                    stroke_color: measure.color,
                });
            }
            dy += step;
        }
        out
    }

    fn arrange_chord_notes(&self, chord: &mut Chord, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        for note in chord.notes.iter_mut() {
            let ctx = staff_ctx.get(&note.staff).unwrap();
            self.arrange_note_ctx(note, &chord.xy, ctx);
        }
    }

    /// Places the augmentation dots of every note in the chord. Dots of all
    /// notes align to one x column (just right of the widest notehead), and a
    /// dot that would land on a staff line is nudged half a space in the
    /// chord's stem direction - up if the stem points up, down if it points
    /// down, up by default when the chord has no stem.
    fn arrange_chord_dots(&self, chord: &mut Chord, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        let dir_sign = match &chord.stem {
            Some(stem) => match stem.direction {
                UpDown::Up => -1.,
                UpDown::Down => 1.,
            },
            None => -1.,
        };

        let column_x = chord
            .notes
            .iter()
            .map(|note| note.xy.x + note.width)
            .fold(f32::MIN, f32::max);

        for note in chord.notes.iter_mut() {
            if note.dots.is_empty() {
                continue;
            }

            let ctx = staff_ctx.get(&note.staff).unwrap();
            let on_staff_line = note.staff_line.rem_euclid(2) == 0;
            let dy = if on_staff_line {
                dir_sign * (Staff::DEFAULT_SPACE_SIZE / 2.) * ctx.scaling
            } else {
                0.
            };

            let base = XY {
                x: column_x,
                y: note.xy.y + dy,
            };
            for (i, dot) in note.dots.iter_mut().enumerate() {
                let center = base.mv((i as f32 + 1.) * note.dot_spacing, 0.);
                self.arrange_dot(dot, &center);
            }
        }
    }

    fn rearrange_chord_accidentals(&self, chord: &mut Chord) {
        let mut accidentals: Vec<&mut Accidental> = chord
            .notes
            .iter_mut()
            .filter_map(|v| v.accidental.as_mut())
            .collect();

        self.rearrange_accidentals(&mut accidentals)
    }

    fn arrange_note_accidental(&self, note: &mut Note) {
        let gap = ACCIDENTAL_GAP * note.scale;

        if let Some(accidental) = &mut note.accidental {
            self.arrange_accidental(accidental, &note.xy.mv(-gap, 0.));
        }
    }

    /// Rests carry no stem, so a dot landing on a staff line is nudged up (the
    /// default direction per engraving convention).
    fn arrange_rest_dots(&self, rest: &mut Rest, staff_ctx: &StaffCtx) {
        let on_staff_line = rest.staff_line.rem_euclid(2) == 0;
        let dy = if on_staff_line {
            -(Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling
        } else {
            0.
        };

        let base = rest.xy.mv(rest.width, dy);
        for (i, dot) in rest.dots.iter_mut().enumerate() {
            let center = base.mv((i as f32 + 1.) * rest.dot_spacing, 0.);
            self.arrange_dot(dot, &center);
        }
    }

    fn arrange_rest_glyph(&self, rest: &mut Rest, origin: &XY, staff_ctx: &StaffCtx) {
        let mut dy = staff_ctx.distance_from_top;
        dy += rest.staff_line as f32 * ((Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling);
        rest.xy = XY {
            x: origin.x,
            y: origin.y + dy,
        };
    }

    fn walk_system(&self, system: &mut System, paddings: &MeasureStartPaddings) {
        for section in system.sections.values_mut() {
            for group in section.part_groups.values_mut() {
                for part in group.parts.values_mut() {
                    self.walk_part(part);
                }
            }
        }

        // Last: the shared opening columns need every staff measure placed.
        self.arrange_system_measure_starts(system, paddings);
    }

    fn walk_part(&self, part: &mut Part) {
        let staff_ctx = part.create_staff_ctx();

        for staff in part.staves.values_mut() {
            if staff.hidden {
                continue;
            }

            for measure in staff.measures.values_mut() {
                self.arrange_staff_measure_content(measure);
            }
        }

        // Not filtered on visibility, unlike every other content walk: a hidden
        // part's measures are still arranged today (`PageArranger::arrange_part_clear_of` has
        // no such check) and nothing draws them, so this pass must keep leaving
        // them arranged rather than stale.
        for measure in part.measures.values_mut() {
            self.arrange_part_measure_content(measure, &staff_ctx);
        }
    }

    /// Staff-line index just below the bottom staff line of a staff of `lines`
    /// lines; notes with a higher index sit below the staff and need ledger lines.
    ///
    /// Indices count half-spaces down from the top line, so the bottom line of a
    /// five-line staff is 8 and this is 9 -- the half-space between it and the first
    /// note that needs a ledger.
    fn ledger_below_staff_line(&self, lines: usize) -> i32 {
        2 * (lines.saturating_sub(1) as i32) + 1
    }

    /// Rearranges accidentals in-place from top to bottom, moving each accidental
    /// left by the exact minimal amount to nest into cutouts of accidentals above it.
    fn rearrange_accidentals(&self, accidentals: &mut Vec<&mut Accidental>) {
        if accidentals.is_empty() {
            return;
        }

        // 1. Sort top to bottom (descending Y coordinate)
        accidentals.sort_by(|a, b| {
            a.xy.y
                .partial_cmp(&b.xy.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 2. Process top to bottom
        for i in 0..accidentals.len() {
            let mut shift_for_i: f32 = 0.0;

            // Find max shift required relative to all accidentals placed above it
            for j in 0..i {
                let shift = accidentals[i].required_left_shift(accidentals[j]);
                if shift > shift_for_i {
                    shift_for_i = shift;
                }
            }

            if shift_for_i > 0.0 {
                accidentals[i].xy.x -= shift_for_i + 1.5;
            }
        }
    }
}

/// Which side of the staff a note (and therefore its ledger lines) sits on.
enum LedgerSide {
    Above,
    Below,
}
