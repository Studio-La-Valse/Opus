use crate::drawable::elements::line::Line;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::xy::XY;
use crate::score::core::group_symbol::GroupSymbol as Kind;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::chord::Chord;
use crate::score::visual::clef::{Clef, ClefAnchor};
use crate::score::visual::dot::Dot;
use crate::score::visual::flag::Flag;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::{Glyphs, GroupSymbol};
use crate::score::visual::key_signature::KeySignature;
use crate::score::visual::note::Note;
use crate::score::visual::page::Page;
use crate::score::visual::part::Part;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::part_group_measure::PartGroupMeasure;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::placed::Placed;
use crate::score::visual::rest::Rest;
use crate::score::visual::section::Section;
use crate::score::visual::section_measure::SectionMeasure;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::staff_measure::{MeasureStartPaddings, StaffMeasure};
use crate::score::visual::stem::{Stem, UpDown};
use crate::score::visual::system::System;
use crate::score::visual::system_measure::SystemMeasure;
use crate::score::visual::time_signature::TimeSignature;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// Default stem length (in tenths) used when the source has no explicit stem
/// `default-y`; negative points up, positive points down.
const DEFAULT_STEM_LENGTH: f32 = 30.;

/// Gap in tenths between a notehead's left edge and the right edge of its
/// accidental. Scaled with the note, like everything else it owns.
const ACCIDENTAL_GAP: f32 = 2.;

/// Staff-line index of the top staff line; notes with a lower index sit above the
/// staff and need ledger lines. The top line is where a staff is anchored, so
/// this holds however many lines it has.
const LEDGER_ABOVE_STAFF_LINE: i32 = 0;

/// Places every element of a measured visual score.
///
/// One method per element, each delegating to the methods of the elements it
/// owns, so arranging a [`Chord`] arranges its notes and so on down to the dots.
/// Holds no state: what an arrangement needs is handed to each call, and it
/// takes no [`LayoutParams`](crate::score::visual::layoutable::LayoutParams)
/// because sizing is settled by the time it runs.
pub struct ArrangeMachine;

impl ArrangeMachine {
    /// Places this page's systems below its margins.
    pub fn arrange_page(&self, page: &mut Page, origin: &XY) {
        let m_left = page.margins.left;
        let m_top = page.margins.top;

        // top left of available space after margins
        let mut origin = origin.mv(m_left, m_top);

        let mut first = true;
        for system in page.systems.values_mut() {
            let s_m_left = system.m_left;
            let s_left = origin.x + s_m_left;

            // space on top of system is either its margin to previous if any,
            // else the distance to top of margins
            let mut s_m_top = system.distance;
            if first {
                s_m_top = system.top;
                first = false
            }

            let s_top = origin.y + s_m_top;

            let s_origin = XY {
                x: s_left,
                y: s_top,
            };
            self.arrange_system(system, &s_origin);

            origin = origin.mv(0., system.height + s_m_top);
        }
    }

    pub fn arrange_system(&self, system: &mut System, origin: &XY) {
        system.xy = *origin;

        let mut _origin = system.xy;
        for measure in system.measures.values_mut() {
            self.arrange_system_measure(measure, &_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        // The page's left margin, in world coordinates: `page.rs` places a
        // system at `page_margin + system.m_left`, so undoing the system's own
        // margin lands back on it. This is where a name's box reaches left to.
        let margin_left = system.xy.x - system.m_left;

        let mut _origin = system.xy;
        for section in system.sections.values_mut() {
            self.arrange_section_within(section, &_origin, margin_left);
            _origin = _origin.mv(0., section.height);
        }
    }

    pub fn arrange_system_measure(&self, measure: &mut SystemMeasure, _origin: &XY) {
        measure.xy = *_origin;
    }

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
    /// Run by
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger),
    /// once the staves have been placed and the measures have the left edge
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

    /// Places this section, additionally carrying the page's left margin
    /// (`margin_left`) down the chain so a part / part-group name knows how far
    /// left its box reaches. A section has no name of its own -- it is the
    /// (usually unnamed) bracket around a run of part-groups.
    ///
    /// [`arrange_section`](Self::arrange_section) delegates here with `origin.x`
    /// for the margin, the way each level's `arrange` delegates today.
    pub fn arrange_section_within(&self, section: &mut Section, origin: &XY, margin_left: f32) {
        section.xy = *origin;

        let first_visible_staff_distance = section.first_visible_staff_distance();
        let mut measure_origin = section.xy.mv(0., first_visible_staff_distance);

        for measure in section.measures.values_mut() {
            self.arrange_section_measure(measure, &measure_origin);
            measure_origin = measure_origin.mv(measure.width, 0.);
        }

        // Arranged before the part-groups, because where they put their own
        // symbols depends on how far left this one reached. A section's symbol
        // is the innermost of the three, so it is the only one measured from
        // the system itself.
        self.arrange_group_symbol(
            &mut section.symbol,
            &section.xy.mv(0., first_visible_staff_distance),
        );
        let clear_of = self.section_symbol_left_edge(section);

        let mut part_group_origin = section.xy;
        for part_group in section.part_groups.values_mut() {
            self.arrange_part_group_clear_of(part_group, &part_group_origin, clear_of, margin_left);
            part_group_origin = part_group_origin.mv(0., part_group.height);
        }
    }

    pub fn arrange_section(&self, section: &mut Section, origin: &XY) {
        self.arrange_section_within(section, origin, origin.x);
    }

    pub fn arrange_section_measure(&self, measure: &mut SectionMeasure, origin: &XY) {
        measure.xy = *origin;
    }

    /// Places this group, with `clear_of` the left edge of whatever the
    /// enclosing section drew and `margin_left` the page's left margin. This
    /// group's symbol sits its own gap further out than `clear_of`, its name
    /// ends a padding left of the symbol and reaches back to `margin_left`, and
    /// its parts keep clear of the symbol in turn -- so the three levels stack
    /// outward from the system without any of them knowing how wide the others
    /// are.
    pub fn arrange_part_group_clear_of(
        &self,
        group: &mut PartGroup,
        origin: &XY,
        clear_of: f32,
        margin_left: f32,
    ) {
        group.xy = *origin;

        let first_visible_staff_distance = group.first_visible_staff_distance();

        let mut _origin = group.xy;
        for measure in group.measures.values_mut() {
            self.arrange_part_group_measure(measure, &_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        // Before the parts, whose own symbols keep clear of this one.
        let symbol_top = XY {
            x: clear_of,
            y: group.xy.y + first_visible_staff_distance,
        };
        self.arrange_group_symbol(&mut group.symbol, &symbol_top);
        let clear_of = self.part_group_symbol_left_edge(group, clear_of);

        self.arrange_group_name_between(&mut group.name, symbol_top, clear_of, margin_left);

        let mut _origin = group.xy;
        for part in group.parts.values_mut() {
            self.arrange_part_clear_of(part, &_origin, clear_of, margin_left);
            _origin = _origin.mv(0., part.height);
        }
    }

    /// Places this group as `arrange_part_group_clear_of` normally does, with
    /// its enclosing section's symbol already at `clear_of`. Used on its own when
    /// there is nothing to keep clear of.
    pub fn arrange_part_group(&self, group: &mut PartGroup, origin: &XY) {
        self.arrange_part_group_clear_of(group, origin, origin.x, origin.x);
    }

    pub fn arrange_part_group_measure(&self, measure: &mut PartGroupMeasure, origin: &XY) {
        measure.xy = *origin;
    }

    /// Places this part, with `clear_of` the left edge of whatever its
    /// part-group drew and `margin_left` the page's left margin. A part's
    /// symbol is the outermost of the three, so nothing keeps clear of it in
    /// turn; its name ends a padding left of it and reaches back to the margin.
    pub fn arrange_part_clear_of(
        &self,
        part: &mut Part,
        origin: &XY,
        clear_of: f32,
        margin_left: f32,
    ) {
        part.xy = *origin;

        let mut _origin = part.xy;
        for staff in part.staves.values_mut() {
            if staff.hidden {
                continue;
            }

            _origin = _origin.mv(0., staff.distance_final);

            self.arrange_staff(staff, &_origin);
            _origin = _origin.mv(0., staff.height());
        }

        let mut _origin = part.xy;
        for measure in part.measures.values_mut() {
            self.arrange_part_measure(measure, &_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let first_visible_staff_distance = part.first_visible_staff_distance();
        let symbol_top = XY {
            x: clear_of,
            y: part.xy.y + first_visible_staff_distance,
        };
        self.arrange_group_symbol(&mut part.symbol, &symbol_top);

        let name_right = self.part_symbol_left_edge(part, clear_of);
        self.arrange_group_name_between(&mut part.name, symbol_top, name_right, margin_left);
    }

    /// Places this part as [`arrange_part_clear_of`](Self::arrange_part_clear_of)
    /// does, with nothing to its left to keep clear of.
    pub fn arrange_part(&self, part: &mut Part, origin: &XY) {
        self.arrange_part_clear_of(part, origin, origin.x, origin.x);
    }

    pub fn arrange_part_measure(&self, measure: &mut PartMeasure, origin: &XY) {
        measure.origin = *origin;
    }

    /// Places everything this measure draws: run by
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger)
    /// once every measure in the part has its own `origin` from
    /// [`arrange_part_measure`](Self::arrange_part_measure).
    pub fn arrange_part_measure_content(
        &self,
        measure: &mut PartMeasure,
        staff_ctx: &BTreeMap<StaffIdx, StaffCtx>,
    ) {
        self.arrange_part_measure_chords(measure, staff_ctx);
        self.arrange_part_measure_ledgers(measure, staff_ctx);
    }

    pub fn arrange_staff(&self, staff: &mut Staff, origin: &XY) {
        staff.xy = *origin;

        let mut _origin = staff.xy;
        for measure in staff.measures.values_mut() {
            self.arrange_staff_measure(measure, &_origin);

            _origin = _origin.mv(measure.width, 0.)
        }
    }

    /// Places this measure's own position. Everything it draws is placed
    /// afterwards by
    /// [`arrange_staff_measure_content`](Self::arrange_staff_measure_content),
    /// which the container pass does not call: content placement is
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger)'s
    /// job, run once every container in the tree -- this measure's opening
    /// columns included -- has its final position.
    pub fn arrange_staff_measure(&self, measure: &mut StaffMeasure, origin: &XY) {
        measure.xy = *origin;
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
        self.place_clef(
            clef,
            ClefAnchor::LeftEdgeAt(origin.x + dx),
            origin.y,
            scaling,
        );

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
    /// places once every staff measure of the system is placed. Run by
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger)
    /// after the container pass has placed this measure itself.
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
        self.arrange_chord_stem(chord, staff_ctx);
        self.arrange_chord_dots(chord, staff_ctx);

        // rearrange the accidentals so that they don't overlap.
        self.rearrange_chord_accidentals(chord);
    }

    /// Positions this chord's stem and computes its natural length from the
    /// notes' already-arranged positions and `default_y` (or the engine's own
    /// default when the document gives none).
    ///
    /// Called once from [`arrange_chord_ctx`](Self::arrange_chord_ctx) during the
    /// ordinary arrange, and a second time by
    /// [`BeamArranger`](crate::score::visual::arranger::BeamArranger)
    /// before it fits a beam ray: resetting the stem to this natural length
    /// undoes whatever an earlier beam pass adjusted it to, which is what
    /// keeps beaming idempotent. Pure in the notes and the staff context, so
    /// calling it again always reproduces the same length.
    pub fn arrange_chord_stem(&self, chord: &mut Chord, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        if let Some(stem) = chord.stem.as_mut() {
            let staff_top = staff_ctx.get(&stem.staff).unwrap().distance_from_top;

            let key = |n: &&Note| OrderedFloat(n.xy.y);

            let lowest_note = chord.notes.iter().max_by_key(key);
            let highest_note = chord.notes.iter().min_by_key(key);

            let tail_note = match stem.direction {
                UpDown::Up => lowest_note,
                UpDown::Down => highest_note,
            }
            .unwrap();

            let tail_anchor = match stem.direction {
                UpDown::Up => tail_note.glyph.stem_anchor_right,
                UpDown::Down => tail_note.glyph.stem_anchor_left,
            }
            .unwrap();
            let tail_anchor = tail_note.scale_pt(&tail_anchor);
            self.arrange_stem(stem, &tail_anchor);

            let default_y: f32 = if let Some(def_y) = &stem.default_y {
                *def_y
            } else {
                let default_length = match stem.direction {
                    UpDown::Up => -DEFAULT_STEM_LENGTH,
                    UpDown::Down => DEFAULT_STEM_LENGTH,
                };

                let tip_note = match stem.direction {
                    UpDown::Up => highest_note,
                    UpDown::Down => lowest_note,
                }
                .unwrap();

                let tip_anchor = (match stem.direction {
                    UpDown::Up => tip_note.glyph.stem_anchor_right,
                    UpDown::Down => tip_note.glyph.stem_anchor_left,
                })
                .unwrap();
                let tip_anchor = tip_note.scale_pt(&tip_anchor);

                let tip = &tip_anchor.mv(0., default_length);
                let staff_m_origin = &chord.xy.mv(0., staff_top);
                staff_m_origin.y - tip.y
            };

            let length = ((chord.xy.y + staff_top) - default_y) - tail_anchor.y;
            stem.length = length;

            self.arrange_stem_flag(stem);
        }
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

    pub fn arrange_stem(&self, stem: &mut Stem, origin: &XY) {
        let thickness = stem.thickness * stem.scale;
        let canvas_offset = match stem.direction {
            UpDown::Down => thickness / 2.,
            UpDown::Up => -thickness / 2.,
        };

        stem.xy = XY {
            x: origin.x + canvas_offset,
            y: origin.y,
        }
    }

    /// Positions the flag (if any) against this stem's own terminal corner.
    /// Must be called once `length` is finalized - stem geometry is only
    /// complete once the caller
    /// ([`arrange_chord_stem`](Self::arrange_chord_stem)) has computed and set
    /// `length`, so this can't happen inside
    /// [`arrange_stem`](Self::arrange_stem), which runs before `length` is known.
    pub fn arrange_stem_flag(&self, stem: &mut Stem) {
        let anchor = match stem.direction {
            UpDown::Up => stem.nw(),
            UpDown::Down => stem.sw(),
        };

        if let Some(flag) = stem.flag.as_mut() {
            self.arrange_flag(flag, &anchor);
        }
    }

    /// Supplied origin is the point on the stem (its nw/sw corner) that the
    /// flag's own SMuFL stem-attachment anchor should land on - not the
    /// flag's own top-left corner.
    pub fn arrange_flag(&self, flag: &mut Flag, origin: &XY) {
        flag.xy = *origin - scaled_stem_anchor(flag);
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

    /// Supplied origin x coordinate is left of clef, y coordinate is the line in the staff.
    pub fn arrange_clef(&self, clef: &mut Clef, origin: &XY) {
        clef.xy = *origin;
    }

    /// Places this clef against `anchor`, sitting on whichever line its own clef
    /// names measured down from `staff_top`.
    ///
    /// The one rule for every clef the engine draws. Only the anchor differs
    /// between them: an opening clef takes the column
    /// [`arrange_system_measure_starts`](Self::arrange_system_measure_starts)
    /// hands it, a clef at a barline the right edge of its own measure, a
    /// mid-measure change the left edge of the note or rest it precedes.
    ///
    /// Reads `width`, so the clef has to be sized before it is placed --
    /// [`rescale`](Clef::rescale) if it was built after the measure pass, as the
    /// ones
    /// [`ClefChangeArranger`](crate::score::visual::arranger::ClefChangeArranger) builds
    /// are.
    /// A clef straight out of [`new`](Clef::new) has no width at all.
    ///
    /// `staff_scaling` is the *staff's* factor and not the clef's own, which for
    /// a courtesy clef is already reduced by [`Clef::COURTESY_SCALE`]: the line
    /// it sits on is a fact about the staff, not about how large the glyph is
    /// drawn.
    ///
    /// Reports nothing back. A caller that has to say how far right the ink
    /// reaches -- only `arrange_staff_measure_clef_start` does, for the next
    /// column -- works it out from the offset it passed in, exactly as the key
    /// and time signature beside it do.
    pub fn place_clef(
        &self,
        clef: &mut Clef,
        anchor: ClefAnchor,
        staff_top: f32,
        staff_scaling: f32,
    ) {
        let x = match anchor {
            ClefAnchor::LeftEdgeAt(x) => x,
            ClefAnchor::GapBefore(x) => x - Clef::COURTESY_GAP - clef.width,
        };

        let dy = clef.clef.line as f32 * (Staff::DEFAULT_SPACE_SIZE / 2.) * staff_scaling;

        self.arrange_clef(
            clef,
            &XY {
                x,
                y: staff_top + dy,
            },
        );
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

    /// `origin.x` is the left edge this symbol sits clear of, and `origin.y` the
    /// top line of the first staff it spans. Neither is a position for the
    /// symbol itself: it steps left by its own gap from there.
    ///
    /// What that edge is depends on the level. A section's is the system's own
    /// left edge, since a section symbol is the innermost of the three. A
    /// part-group's is whatever its section drew, and a part's is whatever its
    /// part-group drew, so the three stack outward without any of them knowing
    /// how wide the others are -- which they could not know, a brace's width
    /// following the span it covers.
    pub fn arrange_group_symbol(&self, symbol: &mut GroupSymbol, origin: &XY) {
        symbol.anchor = origin.mv(-symbol.gap, 0.);

        // One statement for both, so the box always describes the ink.
        let (shape, bounds) = if symbol.span <= 0. {
            symbol.empty_shape()
        } else {
            match (&symbol.glyphs, symbol.kind) {
                (Glyphs::Brace(glyph), Kind::Brace) => symbol.brace_shape(glyph),
                (Glyphs::Bracket(top, bottom), Kind::Bracket) => symbol.bracket_shape(top, bottom),
                (_, Kind::Line) => symbol.line_shape(),
                (_, Kind::Square) => symbol.square_shape(),
                // `None`, and any shape whose glyphs failed to resolve.
                _ => symbol.empty_shape(),
            }
        };

        symbol.shape = shape;
        symbol.bounds = bounds;
    }

    /// Places the box in one statement, so it always describes what is reserved.
    ///
    /// `top` is the top line of the first staff the name covers, `right` the
    /// left edge of the level's symbol (or where that symbol would have been,
    /// when nothing is drawn), and `margin_left` the page's left margin. The
    /// box's right edge is always `right` less the padding -- that is where a
    /// right-aligned run ends. Its left edge is the margin when there is room,
    /// and the right edge itself when there is not: a name with no room to its
    /// left still ends in the right place and simply overflows past the margin,
    /// since nothing in the horizontal layout moves to make space for it.
    pub fn arrange_group_name_between(
        &self,
        name: &mut GroupName,
        top: XY,
        right: f32,
        margin_left: f32,
    ) {
        let box_right = right - name.padding;
        let box_left = margin_left.min(box_right);
        name.bounds = BoundingBox {
            xy: XY {
                x: box_left,
                y: top.y,
            },
            size: XY {
                x: box_right - box_left,
                y: name.span,
            },
        };
    }

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
        place: fn(&ArrangeMachine, &mut StaffMeasure, f32) -> Option<f32>,
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
            self.place_clef(
                clef,
                ClefAnchor::GapBefore(measure_right),
                origin.y,
                scaling,
            );
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
                let below_staff_line = ledger_below_staff_line(staff_ctx.lines);
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

        rearrange_accidentals(&mut accidentals)
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

    /// How far left this section's own ink reaches, which is what the symbols
    /// inside it keep clear of. The system's left edge when nothing is drawn --
    /// an undrawn symbol must not push its part-groups outward by a gap that
    /// nothing occupies.
    fn section_symbol_left_edge(&self, section: &Section) -> f32 {
        if section.shows_symbol() {
            section.symbol.bounds().x_min()
        } else {
            section.xy.x
        }
    }

    /// How far left this group's own ink reaches, which is what its name aligns
    /// against and what its parts keep clear of: the symbol's left edge when it
    /// draws, and the incoming `clear_of` when it does not -- an undrawn symbol
    /// must not push what is outside it away by a gap nothing occupies. The
    /// counterpart of [`section_symbol_left_edge`](Self::section_symbol_left_edge).
    fn part_group_symbol_left_edge(&self, group: &PartGroup, clear_of: f32) -> f32 {
        if group.shows_symbol() {
            group.symbol.bounds().x_min()
        } else {
            clear_of
        }
    }

    /// How far left this part's own ink reaches, which is what its name aligns
    /// against: the symbol's left edge when it draws, and the incoming
    /// `clear_of` when it does not. The counterpart of
    /// [`section_symbol_left_edge`](Self::section_symbol_left_edge).
    fn part_symbol_left_edge(&self, part: &Part, clear_of: f32) -> f32 {
        if part.shows_symbol() {
            part.symbol.bounds().x_min()
        } else {
            clear_of
        }
    }
}

/// Which side of the staff a note (and therefore its ledger lines) sits on.
enum LedgerSide {
    Above,
    Below,
}

/// Staff-line index just below the bottom staff line of a staff of `lines`
/// lines; notes with a higher index sit below the staff and need ledger lines.
///
/// Indices count half-spaces down from the top line, so the bottom line of a
/// five-line staff is 8 and this is 9 -- the half-space between it and the first
/// note that needs a ledger.
fn ledger_below_staff_line(lines: usize) -> i32 {
    2 * (lines.saturating_sub(1) as i32) + 1
}

/// The offset from a flag's glyph origin to its stem-attachment anchor, which is
/// what `arrange_flag` subtracts to put the origin where the anchor lands on the
/// stem.
fn scaled_stem_anchor(flag: &Flag) -> XY {
    flag.glyph.stem_anchor.scale(flag.unit())
}

/// Rearranges accidentals in-place from top to bottom, moving each accidental
/// left by the exact minimal amount to nest into cutouts of accidentals above it.
fn rearrange_accidentals(accidentals: &mut Vec<&mut Accidental>) {
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
