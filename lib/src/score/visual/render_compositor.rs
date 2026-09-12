use crate::{
    drawable::drawable_element::DrawableElement,
    geometry::xy::XY,
    score::{
        visual::{
            chord::Chord,
            page::Page,
            part::Part,
            part_group::PartGroup,
            part_measure::PartMeasure,
            render_fonts::RenderFonts,
            render_pass::{BaseRenderer, DebugRenderer, RenderPass},
            score::Score,
            section::Section,
            staff::Staff,
            system::System,
        },
        walk_cursor::Visibility,
    },
};

pub struct RenderCompositor {
    pub pass: Box<dyn RenderPass>,
}

/// One page's slice of [`RenderCompositor::walk_pages`] output: the page's
/// drawable elements together with where the page sits and how large it is.
///
/// Every value here -- `origin`, `width`, `height`, and the coordinates inside
/// `elements` -- is in MusicXML tenths and in the score's global coordinate
/// space: each page is placed at its own global offset and the elements carry
/// absolute coordinates, not page-local ones. A sink that emits one physical
/// page at a time (PDF) must therefore translate `elements` by `-origin` to
/// bring the page back to its own origin.
pub struct RenderedPage<'a> {
    /// 1-based position of this page in the walk, for page-numbered output
    /// (filenames, the flat-buffer page table). This is the page's position in
    /// the walk, not [`Page::number`](crate::score::visual::page::Page): the two
    /// agree for a document whose pages run 1..n without gaps, which is the only
    /// shape the cursor produces today, but only this one is guaranteed dense.
    pub number: u32,
    pub origin: XY,
    pub width: f32,
    pub height: f32,
    pub elements: Vec<DrawableElement<'a>>,
}

impl RenderCompositor {
    pub fn new(pass: Box<dyn RenderPass>) -> Self {
        Self { pass }
    }

    /// Compositor for the normal score render (staves, glyphs, stems, beams).
    pub fn base() -> Self {
        Self::new(Box::new(BaseRenderer {}))
    }

    /// Compositor for the debug overlay (bounding boxes, anchors, guides).
    pub fn debug() -> Self {
        Self::new(Box::new(DebugRenderer {}))
    }

    /// The whole render in one call: the base pass, plus the debug overlay
    /// merged into each page when `debug` is set. Every consumer wants exactly
    /// this, so the base/overlay zip lives here rather than being repeated at
    /// each call site.
    ///
    /// The overlay is appended after a page's base elements, so it draws on top
    /// of them; `zip` stops at the shorter side, which is a no-op in practice
    /// because both passes walk the same pages.
    pub fn compose<'a>(
        score: &'a Score,
        fonts: &RenderFonts<'a>,
        debug: bool,
    ) -> Vec<RenderedPage<'a>> {
        let mut pages = Self::base().walk_pages(score, fonts);

        if debug {
            for (page, overlay) in pages.iter_mut().zip(Self::debug().walk_pages(score, fonts)) {
                page.elements.extend(overlay.elements);
            }
        }

        pages
    }

    /// Every drawable element for the score, one [`RenderedPage`] per laid-out
    /// page, in page order. The single compositor entry point: sinks that want
    /// one continuous stream (SVG per file, the flat buffer) concatenate the
    /// pages themselves; sinks that emit one surface per page (PDF) drive each
    /// [`RenderedPage`] separately.
    pub fn walk_pages<'a>(
        &self,
        score: &'a Score,
        fonts: &RenderFonts<'a>,
    ) -> Vec<RenderedPage<'a>> {
        score
            .pages
            .values()
            .enumerate()
            .map(|(index, page)| {
                let mut elements: Vec<DrawableElement<'a>> =
                    Vec::with_capacity(Self::estimate_page(page));
                self.pass.render_page(page, fonts, &mut elements);

                for system in page.systems.values() {
                    self.walk_system(system, fonts, &mut elements);
                }

                RenderedPage {
                    number: index as u32 + 1,
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
        system: &'a System,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_system(system, fonts, out);

        for measure in system.measures.values() {
            self.pass.render_system_measure(measure, fonts, out);
        }

        for section in system.sections.values() {
            self.walk_section(section, fonts, out);
        }

        // Mid-measure clef changes are filed on the system for the same reason
        // beams and ties are, but they are ordinary staff glyphs and sit in the
        // gap before their note, so nothing about them wants to be painted over
        // the music.
        for clef in system.clef_changes.iter() {
            self.pass.render_clef(clef, fonts, out);
        }

        // Last, so beams and ties paint on top. Elements are drawn in walk order
        // and the staff lines they cross come out of `render_staff`, part-way
        // through the section walk above.
        //
        // Beams used to draw inside their own part, between its staff lines and
        // its noteheads; from here they draw over both. A beam sits at the stem
        // tip and so never overlaps a notehead, which is why the move is
        // invisible -- but it is a real change in paint order.
        for beam in system.beams.iter() {
            self.pass.render_beam(beam, fonts, out);
        }

        for tie in system.ties.iter() {
            self.pass.render_tie(tie, fonts, out);
        }
    }

    fn walk_section<'a>(
        &self,
        section: &'a Section,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_section(section, fonts, out);

        if section.shows_symbol() {
            self.pass.render_group_symbol(&section.symbol, fonts, out);
        }

        for measure in section.measures.values() {
            self.pass.render_section_measure(measure, fonts, out);
        }

        for group in section.part_groups.values() {
            self.walk_part_group(group, fonts, out);
        }
    }

    fn walk_part_group<'a>(
        &self,
        group: &'a PartGroup,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_part_group(group, fonts, out);

        if group.shows_symbol() {
            self.pass.render_group_symbol(&group.symbol, fonts, out);
        }

        if group.shows_name() {
            self.pass.render_group_name(&group.name, fonts, out);
        }

        for measure in group.measures.values() {
            self.pass.render_part_group_measure(measure, fonts, out);
        }

        for part in group.parts.values() {
            if part.visibility == Visibility::Hidden {
                continue;
            }

            self.walk_part(part, fonts, out);
        }
    }

    fn walk_part<'a>(
        &self,
        part: &'a Part,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_part(part, fonts, out);

        if part.shows_symbol() {
            self.pass.render_group_symbol(&part.symbol, fonts, out);
        }

        if part.shows_name() {
            self.pass.render_group_name(&part.name, fonts, out);
        }

        for staff in part.staves.values() {
            if staff.hidden {
                continue;
            }

            self.walk_staff(staff, fonts, out);
        }

        for measure in part.measures.values() {
            self.walk_part_measure(measure, fonts, out);
        }
    }

    fn walk_staff<'a>(
        &self,
        staff: &Staff,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_staff(staff, fonts, out);

        for measure in staff.measures.values() {
            self.pass.render_staff_measure(measure, fonts, out);

            if let Some(ref clef) = measure.clef_start {
                self.pass.render_clef(clef, fonts, out);
            }

            if let Some(ref time_signature) = measure.time_signature_start {
                self.pass.render_time_signature(time_signature, fonts, out);
            }

            self.pass
                .render_key_signature(&measure.key_signature_start, fonts, out);

            for (_, accidental) in measure.key_signature_start.accidentals.iter() {
                self.pass.render_accidental(accidental, fonts, out);
            }

            if let Some(ref time_signature) = measure.time_signature_end {
                self.pass.render_time_signature(time_signature, fonts, out);
            }

            if let Some(ref clef) = measure.clef_end {
                self.pass.render_clef(clef, fonts, out);
            }

            for rest in measure.rests.iter() {
                self.pass.render_rest(rest, fonts, out);

                for dot in rest.dots.iter() {
                    self.pass.render_dot(dot, fonts, out);
                }
            }
        }
    }

    fn walk_part_measure<'a>(
        &self,
        measure: &PartMeasure,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_part_measure(measure, fonts, out);

        for chords in measure.chords.values() {
            for chord in chords.iter() {
                self.walk_chord(chord, fonts, out);
            }
        }
    }

    fn walk_chord<'a>(
        &self,
        chord: &Chord,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        self.pass.render_chord(chord, fonts, out);

        for note in chord.notes.iter() {
            self.pass.render_note(note, fonts, out);

            if let Some(ref accidental) = note.accidental {
                self.pass.render_accidental(accidental, fonts, out);
            }

            for dot in note.dots.iter() {
                self.pass.render_dot(dot, fonts, out);
            }
        }

        if let Some(ref stem) = chord.stem {
            self.pass.render_stem(stem, fonts, out);

            if let Some(ref flag) = stem.flag {
                self.pass.render_flag(flag, fonts, out);
            }
        }
    }

    /// Cheap, pass-agnostic upper-bound-ish estimate of how many `DrawableElement`s
    /// one page's walk will produce, used only to size the page's element `Vec` up
    /// front so it doesn't have to repeatedly reallocate/copy itself as it grows.
    /// Only sums `BTreeMap`/`Vec` lengths (all O(1)) while descending the same
    /// structure `walk_pages` visits, so its cost stays proportional to the
    /// page's structural size rather than to per-note rendering work (no glyph
    /// lookups, no element construction).
    fn estimate_page(page: &Page) -> usize {
        1 + page
            .systems
            .values()
            .map(Self::estimate_system)
            .sum::<usize>()
    }

    fn estimate_system(system: &System) -> usize {
        1 + system.measures.len() // system line + system measure lines
            + system.beams.len()
            + system.ties.len()
            + system.clef_changes.len()
            + system
                .sections
                .values()
                .map(Self::estimate_section)
                .sum::<usize>()
    }

    fn estimate_section(section: &Section) -> usize {
        // The symbol counts as one, though a bracket is three elements and a
        // square four. Only sizes a Vec up front, so an undercount costs at most
        // one reallocation.
        1 + section.measures.len() // group symbol + section measure lines
            + section
                .part_groups
                .values()
                .map(Self::estimate_part_group)
                .sum::<usize>()
    }

    fn estimate_part_group(group: &PartGroup) -> usize {
        1 + group.measures.len() // group symbol + part group measure lines
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

        1 + staves + measures // group symbol + staves + measures
    }

    fn estimate_staff(staff: &Staff) -> usize {
        5 + staff.measures.len() * 3 // 5 staff lines + clef/time sig/key sig, roughly
    }

    fn estimate_part_measure(measure: &PartMeasure) -> usize {
        let chords = measure.chords.values().flatten();

        // notehead + stem per chord, plus headroom for accidentals/flags/extra notes.
        measure.ledgers.len()
            + chords.clone().count() * 2
            + chords.map(|c| c.notes.len()).sum::<usize>()
    }
}
