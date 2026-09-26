use crate::score::visual::accidental::Accidental;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::chord::Chord;
use crate::score::visual::clef::Clef;
use crate::score::visual::dot::Dot;
use crate::score::visual::flag::Flag;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::key_signature::KeySignature;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::Note;
use crate::score::visual::page::Page;
use crate::score::visual::part::Part;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::part_group_measure::PartGroupMeasure;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::placed::Placed;
use crate::score::visual::rest::Rest;
use crate::score::visual::score::Score;
use crate::score::visual::section::Section;
use crate::score::visual::section_measure::SectionMeasure;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::visual::stem::Stem;
use crate::score::visual::system::System;
use crate::score::visual::system_measure::SystemMeasure;
use crate::score::visual::time_signature::TimeSignature;
use crate::score::walk_cursor::Visibility;

/// Sizes every element of a visual score: the pass that runs after
/// [`resolve_layout`](crate::score::visual::score::Score::resolve_layout) and
/// before anything is placed.
///
/// One method per element, each delegating to the methods of the elements it
/// owns, so measuring a [`Chord`] measures its notes and so on down to the
/// dots. Holds no state: sizing is settled by the elements' own resolved
/// appearance, so it ignores the [`LayoutParams`] its
/// [`ScoreArranger`] impl is handed.
///
/// An element that can find its own size takes nothing but itself: a page, a
/// system or a glyph-based element like a [`Clef`] is sized from what it owns
/// or draws. Only an element whose size depends on its surroundings is handed
/// a target, and only the dimension it actually depends on: the measures
/// spanning a run of staves take that run's `height`, a group symbol or name
/// the `span` it binds. Nothing is ever handed a width, because along the
/// staff every element's width follows from its content.
///
/// The first of [`SCORE_ARRANGERS`](crate::score::visual::arranger::SCORE_ARRANGERS):
/// page *placement* is a separate pass, [`PageArranger`](crate::score::visual::arranger::PageArranger).
pub struct ScoreMeasurement;

impl ScoreArranger for ScoreMeasurement {
    /// Sizes every page.
    fn arrange(&self, score: &mut Score, _params: LayoutParams<'_>) {
        for page in score.pages.values_mut() {
            self.measure_page(page);
        }
    }
}

impl ScoreMeasurement {
    pub fn measure_page(&self, page: &mut Page) {
        for system in page.systems.values_mut() {
            self.measure_system(system);
        }
    }

    pub fn measure_system(&self, system: &mut System) {
        system.width = 0.;
        system.height = 0.;

        if let Some(staff) = system.first_visible_staff() {
            staff.distance_final = 0.;
        }

        for section in system.sections.values_mut() {
            self.measure_section(section);
            system.height += section.height;
        }

        for measure in system.measures.values_mut() {
            self.measure_system_measure(measure, system.height);
            system.width += measure.width;
        }
    }

    /// `height` is the height of the system the measure belongs to.
    pub fn measure_system_measure(&self, measure: &mut SystemMeasure, height: f32) {
        measure.height = height;
    }

    pub fn measure_section(&self, section: &mut Section) {
        section.width = 0.;
        section.height = 0.;

        for pg in section.part_groups.values_mut() {
            self.measure_part_group(pg);
            section.height += pg.height;
        }

        let first_visible_staff_distance = section.first_visible_staff_distance();
        let staves_height = section.height - first_visible_staff_distance;

        for measure in section.measures.values_mut() {
            self.measure_section_measure(measure, staves_height);
            section.width += measure.width;
        }

        // Measured whatever it turns out to draw: a symbol that has been sized
        // and placed on every pass cannot go stale, and `shows_symbol` is then
        // a question about the current pass rather than about which branch last
        // ran. Only the compositor decides whether to draw it.
        self.measure_group_symbol(&mut section.symbol, staves_height);
    }

    /// `height` is the height of the staves the measure spans.
    pub fn measure_section_measure(&self, measure: &mut SectionMeasure, height: f32) {
        measure.height = height;
    }

    pub fn measure_part_group(&self, group: &mut PartGroup) {
        group.width = 0.;
        group.height = 0.;

        for part in group.parts.values_mut() {
            self.measure_part(part);
            group.height += part.height;
        }

        let first_visible_staff_distance = group.first_visible_staff_distance();
        let staves_height = group.height - first_visible_staff_distance;

        for measure in group.measures.values_mut() {
            self.measure_part_group_measure(measure, staves_height);
            group.width += measure.width;
        }

        // Sized on every pass whatever it draws; see `measure_section`. The
        // name is sized with the same staves height the symbol is.
        self.measure_group_symbol(&mut group.symbol, staves_height);
        self.measure_group_name(&mut group.name, staves_height);
    }

    /// `height` is the height of the staves the measure spans, like a
    /// [`SectionMeasure`]'s: it starts at the group's first visible staff.
    pub fn measure_part_group_measure(&self, measure: &mut PartGroupMeasure, height: f32) {
        measure.height = height;
    }

    pub fn measure_part(&self, part: &mut Part) {
        part.width = 0.;
        part.height = 0.;

        if part.visibility == Visibility::Hidden {
            return;
        }

        for staff in part.staves.values_mut() {
            // a staff knows its own size (sum of measure widths, staff height)
            self.measure_staff(staff);

            if staff.hidden {
                continue;
            }

            part.height += staff.height();
            part.height += staff.distance_final;
        }

        for measure in part.measures.values_mut() {
            self.measure_part_measure(measure, part.height);
            part.width += measure.width;
        }

        let first_visible_staff_distance = part.first_visible_staff_distance();
        let staves_height = part.height - first_visible_staff_distance;

        // Sized on every pass whatever it draws; see `measure_section`. The
        // name is sized with the same staves height the symbol is.
        self.measure_group_symbol(&mut part.symbol, staves_height);
        self.measure_group_name(&mut part.name, staves_height);
    }

    /// `height` is the height of the whole part, distance above its first
    /// visible staff included: unlike the section and part group measures, this
    /// one starts at the top of the part, because it is the frame its chords
    /// are placed in.
    pub fn measure_part_measure(&self, measure: &mut PartMeasure, height: f32) {
        measure.height = height;

        for chord in measure.chords.values_mut().flatten() {
            self.measure_chord(chord);
        }
    }

    pub fn measure_staff(&self, staff: &mut Staff) {
        staff.height = staff.height();
        staff.width = 0.;

        if staff.hidden {
            staff.height = 0.;
        }

        for measure in staff.measures.values_mut() {
            self.measure_staff_measure(measure, staff.height);
            staff.width += measure.width;
        }
    }

    /// Sizes the measure's elements, scaling each to the staff first: a clef or
    /// time signature drawn at anything other than full size has to be the
    /// right size *here*, because its width is what the system's shared opening
    /// columns are worked out from, and a stale one would put every staff's
    /// elements in the wrong place rather than just its own.
    ///
    /// `height` is the height of the staff the measure belongs to.
    pub fn measure_staff_measure(&self, measure: &mut StaffMeasure, height: f32) {
        measure.height = height;

        if let Some(ref mut clef) = measure.clef_start {
            clef.rescale(measure.scale);
            self.measure_clef(clef);
        }

        if let Some(ref mut time_signature) = measure.time_signature_start {
            time_signature.rescale(measure.scale);
            self.measure_time_signature(time_signature, height);
        }

        measure.key_signature_start.rescale(measure.scale);
        self.measure_key_signature(&mut measure.key_signature_start);

        if let Some(ref mut prepare_time_signature) = measure.time_signature_end {
            prepare_time_signature.rescale(measure.scale);
            self.measure_time_signature(prepare_time_signature, height);
        }

        if let Some(ref mut clef) = measure.clef_end {
            clef.rescale(measure.scale * Clef::COURTESY_SCALE);
            self.measure_clef(clef);
        }

        for rest in measure.rests.iter_mut() {
            self.measure_rest(rest);
        }
    }

    pub fn measure_chord(&self, chord: &mut Chord) {
        for note in chord.notes.iter_mut() {
            self.measure_note(note);
        }

        if let Some(stem) = chord.stem.as_mut() {
            self.measure_stem(stem);
        }
    }

    pub fn measure_note(&self, note: &mut Note) {
        note.height = Staff::DEFAULT_SPACE_SIZE * note.scale;

        let bbox = note.scale_box(&note.glyph().bbox);

        note.width = bbox.width();

        if let Some(accidental) = &mut note.accidental {
            // Sized with the note, not the staff: a cue or grace note's
            // accidental is reduced by the same factor its notehead is.
            accidental.rescale(note.scale);
            self.measure_accidental(accidental);
        }

        for dot in &mut note.dots {
            self.measure_dot(dot);
        }
    }

    pub fn measure_rest(&self, rest: &mut Rest) {
        rest.height = Staff::DEFAULT_SPACE_SIZE * rest.scale;

        let bbox = rest.scale_box(&rest.glyph().bbox);

        for dot in &mut rest.dots {
            self.measure_dot(dot);
        }

        rest.width = bbox.width();
    }

    pub fn measure_stem(&self, stem: &mut Stem) {
        if let Some(flag) = stem.flag.as_mut() {
            self.measure_flag(flag);
        }
    }

    pub fn measure_flag(&self, flag: &mut Flag) {
        let bbox = flag.scale_box(&flag.glyph().bbox);
        flag.width = bbox.width();
        flag.height = bbox.height();
    }

    pub fn measure_dot(&self, _dot: &mut Dot) {}

    pub fn measure_accidental(&self, accidental: &mut Accidental) {
        let bbox = accidental.world_bbox();

        accidental.width = bbox.width();
        accidental.height = bbox.height();
    }

    pub fn measure_clef(&self, clef: &mut Clef) {
        let bbox = clef.scaled_box();
        clef.width = bbox.width();
        clef.height = bbox.height();
    }

    pub fn measure_key_signature(&self, key_signature: &mut KeySignature) {
        key_signature.width = 0.;
        for (_, accidental) in key_signature.accidentals.iter_mut() {
            self.measure_accidental(accidental);
            key_signature.width += accidental.width;
        }

        if key_signature.accidentals.len() > 1 {
            key_signature.width +=
                (key_signature.accidentals.len() - 1) as f32 * key_signature.accidental_spacing();
        }
    }

    /// `height` is the height of the staff the time signature is drawn on; its
    /// width follows from its digits.
    pub fn measure_time_signature(&self, time_signature: &mut TimeSignature, height: f32) {
        time_signature.height = height;

        let unit = time_signature.unit();
        time_signature.width =
            (time_signature.num().advance() * unit).max(time_signature.denom().advance() * unit);
    }

    /// `span` is how tall a run of staves this symbol binds. There is no width
    /// to hand it: a symbol's width follows from its shape.
    pub fn measure_group_symbol(&self, symbol: &mut GroupSymbol, span: f32) {
        symbol.span = span.max(0.);
    }

    /// Sizes the name against a run of staves: `span` is how tall that run is,
    /// the same span a [`GroupSymbol`] is measured with. There is no width to
    /// hand it -- the box's width follows from where the symbol ended up.
    pub fn measure_group_name(&self, name: &mut GroupName, span: f32) {
        name.span = span.max(0.);
    }
}
