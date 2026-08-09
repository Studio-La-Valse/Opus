use crate::{
    drawable::drawable_element::DrawableElement,
    score::{
        layout_ctx::Visibility,
        visual::{render_pass::RenderPass, score::Score},
    },
    smufl::smufl_font::SmuflFont,
};

pub struct RenderCompositor {
    pub pass: Box<dyn RenderPass>,
}

impl RenderCompositor {
    pub fn walk<'a>(&self, score: &Score, font: &'a SmuflFont) -> Vec<DrawableElement<'a>> {
        let mut out: Vec<DrawableElement<'a>> = Vec::with_capacity(Self::estimate_element_count(score));

        for page in score.pages.values() {
            self.pass.render_page(page, font, &mut out);

            for system in page.systems.values() {
                self.pass.render_system(system, font, &mut out);

                for measure in system.measures.values() {
                    self.pass.render_system_measure(measure, font, &mut out);
                }

                for section in system.sections.values() {
                    self.pass.render_section(section, font, &mut out);

                    if section.shows_bracket() {
                        self.pass.render_bracket(&section.bracket, font, &mut out);
                    }

                    for measure in section.measures.values() {
                        self.pass.render_section_measure(measure, font, &mut out);
                    }

                    for group in section.part_groups.values() {
                        self.pass.render_part_group(group, font, &mut out);

                        if group.shows_brace() {
                            self.pass.render_brace(&group.brace, font, &mut out);
                        }

                        for measure in group.measures.values() {
                            self.pass.render_part_group_measure(measure, font, &mut out);
                        }

                        for part in group.parts.values() {
                            if part.visibility == Visibility::Hidden {
                                continue;
                            }

                            self.pass.render_part(part, font, &mut out);

                            if part.shows_brace() {
                                self.pass.render_brace(&part.brace, font, &mut out);
                            }

                            for staff in part.staves.values() {
                                if staff.hidden {
                                    continue;
                                }

                                self.pass.render_staff(staff, font, &mut out);

                                for measure in staff.measures.values() {
                                    self.pass.render_staff_measure(measure, font, &mut out);

                                    if let Some(ref clef) = measure.clef_start {
                                        self.pass.render_clef(clef, font, &mut out);
                                    }

                                    if let Some(ref time_signature) = measure.time_signature_start {
                                        self.pass.render_time_signature(
                                            time_signature,
                                            font,
                                            &mut out,
                                        );
                                    }

                                    self.pass.render_key_signature(
                                        &measure.key_signature_start,
                                        font,
                                        &mut out,
                                    );

                                    for (_, accidental) in
                                        measure.key_signature_start.accidentals.iter()
                                    {
                                        self.pass.render_accidental(accidental, font, &mut out);
                                    }

                                    if let Some(ref time_signature) = measure.time_signature_end {
                                        self.pass.render_time_signature(
                                            time_signature,
                                            font,
                                            &mut out,
                                        );
                                    }

                                    if let Some(ref clef) = measure.clef_end {
                                        self.pass.render_clef(clef, font, &mut out);
                                    }

                                    for rest in measure.rests.iter() {
                                        self.pass.render_rest(rest, font, &mut out);

                                        if let Some(ref clef) = rest.clef_change {
                                            self.pass.render_clef(clef, font, &mut out);
                                        }
                                    }
                                }
                            }

                            for measure in part.measures.values() {
                                self.pass.render_part_measure(measure, font, &mut out);

                                for chords in measure.chords.values() {
                                    for chord in chords.iter() {
                                        self.pass.render_chord(chord, font, &mut out);

                                        for note in chord.notes.iter() {
                                            self.pass.render_note(note, font, &mut out);

                                            if let Some(ref accidental) = note.accidental {
                                                self.pass
                                                    .render_accidental(accidental, font, &mut out);
                                            }
                                        }

                                        if let Some(ref stem) = chord.stem {
                                            self.pass.render_stem(stem, font, &mut out);
                                        }

                                        for clef_change in chord.clef_change.values() {
                                            self.pass.render_clef(clef_change, font, &mut out);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        out
    }

    /// Cheap, pass-agnostic upper-bound-ish estimate of how many `DrawableElement`s
    /// this walk will produce, used only to size `out` up front so it doesn't have
    /// to repeatedly reallocate/copy itself as it grows. Only sums `BTreeMap`/`Vec`
    /// lengths (all O(1)) while descending the same structure `walk` visits, so its
    /// cost stays proportional to the score's structural size rather than to
    /// per-note rendering work (no glyph lookups, no element construction).
    fn estimate_element_count(score: &Score) -> usize {
        let mut count = 0;

        for page in score.pages.values() {
            count += 1; // page

            for system in page.systems.values() {
                count += 1 + system.measures.len(); // system line + system measure lines

                for section in system.sections.values() {
                    count += 1 + section.measures.len(); // bracket/section + section measure lines

                    for group in section.part_groups.values() {
                        count += 1 + group.measures.len(); // brace + part group measure lines

                        for part in group.parts.values() {
                            count += 1; // brace

                            for staff in part.staves.values() {
                                count += 5; // 5 staff lines
                                count += staff.measures.len() * 3; // clef/time sig/key sig, roughly
                            }

                            for measure in part.measures.values() {
                                count += measure.beams.len() + measure.ledgers.len();

                                for chords in measure.chords.values() {
                                    // notehead + stem per chord, plus headroom for
                                    // accidentals/flags/extra notes per chord.
                                    count += chords.len() * 2;
                                    count += chords.iter().map(|c| c.notes.len()).sum::<usize>();
                                }
                            }
                        }
                    }
                }
            }
        }

        count
    }
}
