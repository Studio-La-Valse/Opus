use crate::geometry::xy::XY;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::chord::Chord;
use crate::score::visual::clef::Clef;
use crate::score::visual::dot::Dot;
use crate::score::visual::flag::Flag;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::key_signature::KeySignature;
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
/// appearance, so it takes no [`LayoutParams`](crate::score::visual::layoutable::LayoutParams).
pub struct MeasureMachine;

impl MeasureMachine {
    /// Sizes every page. Page *placement* is a separate pass -- see
    /// [`PageArranger`](crate::score::visual::arranger::PageArranger),
    /// which is why there is no `arrange_score`.
    pub fn measure_score(&self, score: &mut Score, available: &XY) {
        for page in score.pages.values_mut() {
            self.measure_page(page, available);
        }
    }

    pub fn measure_page(&self, page: &mut Page, available: &XY) {
        for system in page.systems.values_mut() {
            self.measure_system(system, available);
        }
    }

    pub fn measure_system(&self, system: &mut System, _: &XY) {
        system.width = 0.;
        system.height = 0.;

        if let Some(staff) = system.first_visible_staff() {
            staff.distance_final = 0.;
        }

        for section in system.sections.values_mut() {
            let available = XY::INFINITE;
            self.measure_section(section, &available);
            system.height += section.height;
        }

        for measure in system.measures.values_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: system.height,
            };
            self.measure_system_measure(measure, &available);
            system.width += measure.width;
        }
    }

    pub fn measure_system_measure(&self, measure: &mut SystemMeasure, available: &XY) {
        measure.height = available.y;
    }

    pub fn measure_section(&self, section: &mut Section, _: &XY) {
        section.width = 0.;
        section.height = 0.;

        for pg in section.part_groups.values_mut() {
            let available = XY::INFINITE;
            self.measure_part_group(pg, &available);
            section.height += pg.height;
        }

        let first_visible_staff_distance = section.first_visible_staff_distance();
        let staves_height = section.height - first_visible_staff_distance;

        for measure in section.measures.values_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: staves_height,
            };
            self.measure_section_measure(measure, &available);
            section.width += measure.width;
        }

        // Measured whatever it turns out to draw: a symbol that has been sized
        // and placed on every pass cannot go stale, and `shows_symbol` is then
        // a question about the current pass rather than about which branch last
        // ran. Only the compositor decides whether to draw it.
        let avail = XY {
            x: f32::INFINITY,
            y: staves_height,
        };
        self.measure_group_symbol(&mut section.symbol, &avail);
    }

    pub fn measure_section_measure(&self, measure: &mut SectionMeasure, available: &XY) {
        measure.height = available.y;
    }

    pub fn measure_part_group(&self, group: &mut PartGroup, _: &XY) {
        group.width = 0.;
        group.height = 0.;

        for part in group.parts.values_mut() {
            let available = XY::INFINITE;
            self.measure_part(part, &available);
            group.height += part.height;
        }

        let first_visible_staff_distance = group.first_visible_staff_distance();
        let staves_height = group.height - first_visible_staff_distance;

        for measure in group.measures.values_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: group.height,
            };
            self.measure_part_group_measure(measure, &available);
            group.width += measure.width;
        }

        // Sized on every pass whatever it draws; see `measure_section`. The
        // name is sized with the same staves height the symbol is.
        let available = XY {
            x: f32::INFINITY,
            y: staves_height,
        };
        self.measure_group_symbol(&mut group.symbol, &available);
        self.measure_group_name(&mut group.name, &available);
    }

    pub fn measure_part_group_measure(&self, measure: &mut PartGroupMeasure, available: &XY) {
        measure.height = available.y;
    }

    pub fn measure_part(&self, part: &mut Part, _: &XY) {
        part.width = 0.;
        part.height = 0.;

        if part.visibility == Visibility::Hidden {
            return;
        }

        for staff in part.staves.values_mut() {
            let available = &XY::INFINITE;
            // a staff knows its own size (sum of measure widths, staff height)
            self.measure_staff(staff, available);

            if staff.hidden {
                continue;
            }

            part.height += staff.height();
            part.height += staff.distance_final;
        }

        for measure in part.measures.values_mut() {
            let available = &XY {
                x: f32::INFINITY,
                y: part.height,
            };
            self.measure_part_measure(measure, available);
            part.width += measure.width;
        }

        let first_visible_staff_distance = part.first_visible_staff_distance();
        let staves_height = part.height - first_visible_staff_distance;

        // Sized on every pass whatever it draws; see `measure_section`. The
        // name is sized with the same staves height the symbol is.
        let available = XY {
            x: f32::INFINITY,
            y: staves_height,
        };
        self.measure_group_symbol(&mut part.symbol, &available);
        self.measure_group_name(&mut part.name, &available);
    }

    pub fn measure_part_measure(&self, measure: &mut PartMeasure, available: &XY) {
        measure.height = available.y;

        for chord in measure.chords.values_mut().flatten() {
            self.measure_chord(chord, available);
        }
    }

    pub fn measure_staff(&self, staff: &mut Staff, _available: &XY) {
        staff.height = staff.height();
        staff.width = 0.;

        if staff.hidden {
            staff.height = 0.;
        }

        for measure in staff.measures.values_mut() {
            let available = &XY {
                x: f32::INFINITY,
                y: staff.height,
            };
            self.measure_staff_measure(measure, available);
            staff.width += measure.width;
        }
    }

    /// Sizes the measure's elements, scaling each to the staff first: a clef or
    /// time signature drawn at anything other than full size has to be the
    /// right size *here*, because its width is what the system's shared opening
    /// columns are worked out from, and a stale one would put every staff's
    /// elements in the wrong place rather than just its own.
    pub fn measure_staff_measure(&self, measure: &mut StaffMeasure, available: &XY) {
        measure.height = available.y;

        if let Some(ref mut clef) = measure.clef_start {
            clef.rescale(measure.scale);
            self.measure_clef(clef, available);
        }

        if let Some(ref mut time_signature) = measure.time_signature_start {
            time_signature.rescale(measure.scale);
            self.measure_time_signature(time_signature, available);
        }

        measure.key_signature_start.rescale(measure.scale);
        self.measure_key_signature(&mut measure.key_signature_start, available);

        if let Some(ref mut prepare_time_signature) = measure.time_signature_end {
            prepare_time_signature.rescale(measure.scale);
            self.measure_time_signature(prepare_time_signature, available);
        }

        if let Some(ref mut clef) = measure.clef_end {
            clef.rescale(measure.scale * Clef::COURTESY_SCALE);
            self.measure_clef(clef, available);
        }

        for rest in measure.rests.iter_mut() {
            self.measure_rest(rest, available);
        }
    }

    pub fn measure_chord(&self, chord: &mut Chord, available: &XY) {
        for note in chord.notes.iter_mut() {
            self.measure_note(note, available);
        }

        if let Some(stem) = chord.stem.as_mut() {
            self.measure_stem(stem, available);
        }
    }

    pub fn measure_note(&self, note: &mut Note, available: &XY) {
        note.height = Staff::DEFAULT_SPACE_SIZE * note.scale;

        let glyph = &note.glyph;
        let bbox = note.scale_box(&glyph.bbox);

        note.width = bbox.width();

        if let Some(accidental) = &mut note.accidental {
            // Sized with the note, not the staff: a cue or grace note's
            // accidental is reduced by the same factor its notehead is.
            accidental.rescale(note.scale);
            self.measure_accidental(accidental, available);
        }

        for dot in &mut note.dots {
            self.measure_dot(dot, available);
        }
    }

    pub fn measure_rest(&self, rest: &mut Rest, available: &XY) {
        rest.height = Staff::DEFAULT_SPACE_SIZE * rest.scale;

        let glyph = &rest.glyph;
        let bbox = rest.scale_box(&glyph.bbox);

        for dot in &mut rest.dots {
            self.measure_dot(dot, available);
        }

        rest.width = bbox.width();
    }

    pub fn measure_stem(&self, stem: &mut Stem, available: &XY) {
        if let Some(flag) = stem.flag.as_mut() {
            self.measure_flag(flag, available);
        }
    }

    pub fn measure_flag(&self, flag: &mut Flag, _available: &XY) {
        flag.measure_size();
    }

    pub fn measure_dot(&self, _dot: &mut Dot, _available: &XY) {}

    pub fn measure_accidental(&self, accidental: &mut Accidental, _available: &XY) {
        accidental.measure_size();
    }

    pub fn measure_clef(&self, clef: &mut Clef, _available: &XY) {
        clef.measure_size();
    }

    pub fn measure_key_signature(&self, key_signature: &mut KeySignature, available: &XY) {
        key_signature.width = 0.;
        for (_, accidental) in key_signature.accidentals.iter_mut() {
            self.measure_accidental(accidental, available);
            key_signature.width += accidental.width;
        }

        if key_signature.accidentals.len() > 1 {
            key_signature.width +=
                (key_signature.accidentals.len() - 1) as f32 * key_signature.accidental_spacing();
        }
    }

    pub fn measure_time_signature(&self, time_signature: &mut TimeSignature, available: &XY) {
        time_signature.height = available.y;
        time_signature.measure_width();
    }

    /// `available.y` is how tall a run of staves this symbol binds; `available.x`
    /// is ignored, because a symbol's width follows from its shape rather than
    /// being granted to it.
    pub fn measure_group_symbol(&self, symbol: &mut GroupSymbol, available: &XY) {
        symbol.span = available.y.max(0.);
    }

    /// Sizes the name against a run of staves: `available.y` is how tall that
    /// run is, the same span a [`GroupSymbol`] is measured with. `available.x` is ignored -- the box's width follows
    /// from where the symbol ended up, not from a width granted here.
    pub fn measure_group_name(&self, name: &mut GroupName, available: &XY) {
        name.span = available.y.max(0.);
    }
}
