use crate::{
    drawable::drawable_element::DrawableElement,
    layout_ctx::Visibility,
    visual::{
        render_pass::{RenderPass, RenderPasses},
        score::Score,
    },
};

pub struct RenderCompositor {
    pub pass: RenderPasses,
}

impl RenderCompositor {
    pub fn walk(&self, score: &Score) -> Vec<DrawableElement> {
        let mut out: Vec<DrawableElement> = vec![];

        for page in score.pages.values() {
            self.pass.render_page(page, &mut out);

            for system in page.systems.values() {
                self.pass.render_system(system, &mut out);

                for measure in system.measures.values() {
                    self.pass.render_system_measure(measure, &mut out);
                }

                for section in system.sections.values() {
                    self.pass.render_section(section, &mut out);

                    if section.shows_bracket() {
                        self.pass.render_bracket(&section.bracket, &mut out);
                    }

                    for measure in section.measures.values() {
                        self.pass.render_section_measure(measure, &mut out);
                    }

                    for group in section.part_groups.values() {
                        self.pass.render_part_group(group, &mut out);

                        if group.shows_brace() {
                            self.pass.render_brace(&group.brace, &mut out);
                        }

                        for measure in group.measures.values() {
                            self.pass.render_part_group_measure(measure, &mut out);
                        }

                        for part in group.parts.values() {
                            if part.visibility == Visibility::Hidden {
                                continue;
                            }

                            self.pass.render_part(part, &mut out);

                            if part.shows_brace() {
                                self.pass.render_brace(&part.brace, &mut out);
                            }

                            for staff in part.staves.values() {
                                if staff.hidden {
                                    continue;
                                }

                                self.pass.render_staff(staff, &mut out);

                                for measure in staff.measures.values() {
                                    self.pass.render_staff_measure(measure, &mut out);

                                    if let Some(ref clef) = measure.clef_start {
                                        self.pass.render_clef(clef, &mut out);
                                    }

                                    if let Some(ref time_signature) = measure.time_signature_start {
                                        self.pass.render_time_signature(time_signature, &mut out);
                                    }

                                    self.pass.render_key_signature(
                                        &measure.key_signature_start,
                                        &mut out,
                                    );

                                    for (_, accidental) in
                                        measure.key_signature_start.accidentals.iter()
                                    {
                                        self.pass.render_accidental(accidental, &mut out);
                                    }

                                    if let Some(ref time_signature) = measure.time_signature_end {
                                        self.pass.render_time_signature(time_signature, &mut out);
                                    }

                                    if let Some(ref clef) = measure.clef_end {
                                        self.pass.render_clef(clef, &mut out);
                                    }

                                    for rest in measure.rests.iter() {
                                        self.pass.render_rest(rest, &mut out);

                                        if let Some(ref clef) = rest.clef_change {
                                            self.pass.render_clef(clef, &mut out);
                                        }
                                    }
                                }
                            }

                            for measure in part.measures.values() {
                                self.pass.render_part_measure(measure, &mut out);

                                for chords in measure.chords.values() {
                                    for chord in chords.iter() {
                                        self.pass.render_chord(chord, &mut out);

                                        for note in chord.notes.iter() {
                                            self.pass.render_note(note, &mut out);

                                            if let Some(ref accidental) = note.accidental {
                                                self.pass.render_accidental(accidental, &mut out);
                                            }
                                        }

                                        if let Some(ref stem) = chord.stem {
                                            self.pass.render_stem(stem, &mut out);
                                        }

                                        for clef_change in chord.clef_change.values() {
                                            self.pass.render_clef(clef_change, &mut out);
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
}
