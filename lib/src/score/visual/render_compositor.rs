use crate::{
    drawable::drawable_element::DrawableElement,
    geometry::xy::XY,
    score::{
        layout_ctx::Visibility,
        visual::{
            chord::Chord, part::Part, part_group::PartGroup, part_measure::PartMeasure,
            render_pass::RenderPass, score::Score, section::Section, staff::Staff, system::System,
        },
    },
    smufl::smufl_font::SmuflFont,
};

pub struct RenderCompositor {
    pub pass: Box<dyn RenderPass>,
}

/// One page's slice of [`RenderCompositor::walk_pages`] output: the page's
/// drawable elements together with where the page sits and how large it is.
///
/// Every value here -- `origin`, `width`, `height`, and the coordinates inside
/// `elements` -- is in MusicXML tenths and in the score's global coordinate
/// space (the same coordinates [`RenderCompositor::walk`] produces). A sink that
/// emits one physical page at a time (PDF) must therefore translate `elements`
/// by `-origin` to bring the page back to its own origin.
pub struct RenderedPage<'a> {
    pub origin: XY,
    pub width: f32,
    pub height: f32,
    pub elements: Vec<DrawableElement<'a>>,
}

impl RenderCompositor {
    pub fn walk<'a>(&self, score: &Score, font: &'a SmuflFont) -> Vec<DrawableElement<'a>> {
        let mut out: Vec<DrawableElement<'a>> =
            Vec::with_capacity(Self::estimate_element_count(score));

        for page in score.pages.values() {
            self.pass.render_page(page, font, &mut out);

            for system in page.systems.values() {
                self.walk_system(system, font, &mut out);
            }
        }

        out
    }

    /// Like [`walk`](Self::walk), but keeps each page's elements in their own
    /// [`RenderedPage`] instead of concatenating every page into one stream.
    /// Used by per-page sinks such as the PDF writer, which needs one content
    /// stream and media box per physical page.
    pub fn walk_pages<'a>(&self, score: &Score, font: &'a SmuflFont) -> Vec<RenderedPage<'a>> {
        score
            .pages
            .values()
            .map(|page| {
                let mut elements: Vec<DrawableElement<'a>> = Vec::new();
                self.pass.render_page(page, font, &mut elements);

                for system in page.systems.values() {
                    self.walk_system(system, font, &mut elements);
                }

                RenderedPage {
                    origin: page.xy,
                    width: page.width,
                    height: page.height,
                    elements,
                }
            })
            .collect()
    }

    fn walk_system<'a>(
        &self,
        system: &System,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_system(system, font, out);

        for measure in system.measures.values() {
            self.pass.render_system_measure(measure, font, out);
        }

        for section in system.sections.values() {
            self.walk_section(section, font, out);
        }
    }

    fn walk_section<'a>(
        &self,
        section: &Section,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_section(section, font, out);

        if section.shows_bracket() {
            self.pass.render_bracket(&section.bracket, font, out);
        }

        for measure in section.measures.values() {
            self.pass.render_section_measure(measure, font, out);
        }

        for group in section.part_groups.values() {
            self.walk_part_group(group, font, out);
        }
    }

    fn walk_part_group<'a>(
        &self,
        group: &PartGroup,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_part_group(group, font, out);

        if group.shows_brace() {
            self.pass.render_brace(&group.brace, font, out);
        }

        for measure in group.measures.values() {
            self.pass.render_part_group_measure(measure, font, out);
        }

        for part in group.parts.values() {
            if part.visibility == Visibility::Hidden {
                continue;
            }

            self.walk_part(part, font, out);
        }
    }

    fn walk_part<'a>(&self, part: &Part, font: &'a SmuflFont, out: &mut Vec<DrawableElement<'a>>) {
        self.pass.render_part(part, font, out);

        if part.shows_brace() {
            self.pass.render_brace(&part.brace, font, out);
        }

        for staff in part.staves.values() {
            if staff.hidden {
                continue;
            }

            self.walk_staff(staff, font, out);
        }

        for measure in part.measures.values() {
            self.walk_part_measure(measure, font, out);
        }
    }

    fn walk_staff<'a>(
        &self,
        staff: &Staff,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_staff(staff, font, out);

        for measure in staff.measures.values() {
            self.pass.render_staff_measure(measure, font, out);

            if let Some(ref clef) = measure.clef_start {
                self.pass.render_clef(clef, font, out);
            }

            if let Some(ref time_signature) = measure.time_signature_start {
                self.pass.render_time_signature(time_signature, font, out);
            }

            self.pass
                .render_key_signature(&measure.key_signature_start, font, out);

            for (_, accidental) in measure.key_signature_start.accidentals.iter() {
                self.pass.render_accidental(accidental, font, out);
            }

            if let Some(ref time_signature) = measure.time_signature_end {
                self.pass.render_time_signature(time_signature, font, out);
            }

            if let Some(ref clef) = measure.clef_end {
                self.pass.render_clef(clef, font, out);
            }

            for rest in measure.rests.iter() {
                self.pass.render_rest(rest, font, out);

                if let Some(ref clef) = rest.clef_change {
                    self.pass.render_clef(clef, font, out);
                }
            }
        }
    }

    fn walk_part_measure<'a>(
        &self,
        measure: &PartMeasure,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_part_measure(measure, font, out);

        for chords in measure.chords.values() {
            for chord in chords.iter() {
                self.walk_chord(chord, font, out);
            }
        }
    }

    fn walk_chord<'a>(
        &self,
        chord: &Chord,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_chord(chord, font, out);

        for note in chord.notes.iter() {
            self.pass.render_note(note, font, out);

            if let Some(ref accidental) = note.accidental {
                self.pass.render_accidental(accidental, font, out);
            }
        }

        if let Some(ref stem) = chord.stem {
            self.pass.render_stem(stem, font, out);

            if let Some(ref flag) = stem.flag {
                self.pass.render_flag(flag, font, out);
            }
        }

        for clef_change in chord.clef_change.values() {
            self.pass.render_clef(clef_change, font, out);
        }
    }

    /// Cheap, pass-agnostic upper-bound-ish estimate of how many `DrawableElement`s
    /// this walk will produce, used only to size `out` up front so it doesn't have
    /// to repeatedly reallocate/copy itself as it grows. Only sums `BTreeMap`/`Vec`
    /// lengths (all O(1)) while descending the same structure `walk` visits, so its
    /// cost stays proportional to the score's structural size rather than to
    /// per-note rendering work (no glyph lookups, no element construction).
    fn estimate_element_count(score: &Score) -> usize {
        score
            .pages
            .values()
            .map(|page| {
                1 + page
                    .systems
                    .values()
                    .map(Self::estimate_system)
                    .sum::<usize>()
            })
            .sum()
    }

    fn estimate_system(system: &System) -> usize {
        1 + system.measures.len() // system line + system measure lines
            + system
                .sections
                .values()
                .map(Self::estimate_section)
                .sum::<usize>()
    }

    fn estimate_section(section: &Section) -> usize {
        1 + section.measures.len() // bracket/section + section measure lines
            + section
                .part_groups
                .values()
                .map(Self::estimate_part_group)
                .sum::<usize>()
    }

    fn estimate_part_group(group: &PartGroup) -> usize {
        1 + group.measures.len() // brace + part group measure lines
            + group
                .parts
                .values()
                .map(Self::estimate_part)
                .sum::<usize>()
    }

    fn estimate_part(part: &Part) -> usize {
        let staves: usize = part.staves.values().map(Self::estimate_staff).sum();
        let measures: usize = part
            .measures
            .values()
            .map(Self::estimate_part_measure)
            .sum();

        1 + staves + measures // brace + staves + measures
    }

    fn estimate_staff(staff: &Staff) -> usize {
        5 + staff.measures.len() * 3 // 5 staff lines + clef/time sig/key sig, roughly
    }

    fn estimate_part_measure(measure: &PartMeasure) -> usize {
        let chords = measure.chords.values().flatten();

        // notehead + stem per chord, plus headroom for accidentals/flags/extra notes.
        measure.beams.len()
            + measure.ledgers.len()
            + chords.clone().count() * 2
            + chords.map(|c| c.notes.len()).sum::<usize>()
    }
}
