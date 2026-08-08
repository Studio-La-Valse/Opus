use crate::drawable::drawable_element::DrawableElement;
use crate::{
    color::Color,
    drawable::elements::{line::Line, rect::Rect},
    smufl::smufl_glyph::SmuflGlyph,
    visual::{
        accidental::Accidental,
        brace::Brace,
        bracket::Bracket,
        chord::Chord,
        clef::Clef,
        key_signature::KeySignature,
        note::Note,
        page::Page,
        part::Part,
        part_group::PartGroup,
        part_group_measure::PartGroupMeasure,
        part_measure::PartMeasure,
        rest::Rest,
        section::Section,
        section_measure::SectionMeasure,
        staff::Staff,
        staff_measure::StaffMeasure,
        stem::{Stem, UpDown},
        system::System,
        system_measure::SystemMeasure,
        time_signature::TimeSignature,
    },
    xy::XY,
};

pub trait RenderPass {
    fn render_page(&self, _page: &Page, _out: &mut Vec<DrawableElement>) {}

    fn render_system(&self, _system: &System, _out: &mut Vec<DrawableElement>) {}

    fn render_section(&self, _section: &Section, _out: &mut Vec<DrawableElement>) {}

    fn render_bracket(&self, _bracket: &Bracket, _out: &mut Vec<DrawableElement>) {}

    fn render_part_group(&self, _part_group: &PartGroup, _out: &mut Vec<DrawableElement>) {}

    fn render_brace(&self, _brace: &Brace, _out: &mut Vec<DrawableElement>) {}

    fn render_part(&self, _part: &Part, _out: &mut Vec<DrawableElement>) {}

    fn render_staff(&self, _staff: &Staff, _out: &mut Vec<DrawableElement>) {}

    fn render_system_measure(
        &self,
        _system_measure: &SystemMeasure,
        _out: &mut Vec<DrawableElement>,
    ) {
    }

    fn render_section_measure(
        &self,
        _section_measure: &SectionMeasure,
        _out: &mut Vec<DrawableElement>,
    ) {
    }

    fn render_part_group_measure(
        &self,
        _part_group_measure: &PartGroupMeasure,
        _out: &mut Vec<DrawableElement>,
    ) {
    }

    fn render_part_measure(&self, _part_measure: &PartMeasure, _out: &mut Vec<DrawableElement>) {}

    fn render_staff_measure(&self, _staff_measure: &StaffMeasure, _out: &mut Vec<DrawableElement>) {
    }

    fn render_clef(&self, _clef: &Clef, _out: &mut Vec<DrawableElement>) {}

    fn render_key_signature(&self, _key_signature: &KeySignature, _out: &mut Vec<DrawableElement>) {
    }

    fn render_accidental(&self, _accidental: &Accidental, _out: &mut Vec<DrawableElement>) {}

    fn render_time_signature(
        &self,
        _time_signature: &TimeSignature,
        _out: &mut Vec<DrawableElement>,
    ) {
    }

    fn render_rest(&self, _rest: &Rest, _out: &mut Vec<DrawableElement>) {}

    fn render_chord(&self, _chord: &Chord, _out: &mut Vec<DrawableElement>) {}

    fn render_note(&self, _note: &Note, _out: &mut Vec<DrawableElement>) {}

    fn render_stem(&self, _stem: &Stem, _out: &mut Vec<DrawableElement>) {}
}

pub struct RenderPasses {
    pub passes: Vec<Box<dyn RenderPass>>,
}

impl RenderPass for RenderPasses {
    fn render_page(&self, page: &Page, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_page(page, out);
        }
    }

    fn render_system(&self, system: &System, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_system(system, out);
        }
    }

    fn render_section(&self, section: &Section, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_section(section, out);
        }
    }

    fn render_bracket(&self, bracket: &Bracket, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_bracket(bracket, out);
        }
    }

    fn render_part_group(&self, part_group: &PartGroup, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_part_group(part_group, out);
        }
    }

    fn render_brace(&self, brace: &Brace, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_brace(brace, out);
        }
    }

    fn render_part(&self, part: &Part, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_part(part, out);
        }
    }

    fn render_staff(&self, staff: &Staff, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_staff(staff, out);
        }
    }

    fn render_system_measure(
        &self,
        system_measure: &SystemMeasure,
        out: &mut Vec<DrawableElement>,
    ) {
        for pass in self.passes.iter() {
            pass.render_system_measure(system_measure, out);
        }
    }

    fn render_section_measure(
        &self,
        section_measure: &SectionMeasure,
        out: &mut Vec<DrawableElement>,
    ) {
        for pass in self.passes.iter() {
            pass.render_section_measure(section_measure, out);
        }
    }

    fn render_part_group_measure(
        &self,
        part_group_measure: &PartGroupMeasure,
        out: &mut Vec<DrawableElement>,
    ) {
        for pass in self.passes.iter() {
            pass.render_part_group_measure(part_group_measure, out);
        }
    }

    fn render_part_measure(&self, part_measure: &PartMeasure, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_part_measure(part_measure, out);
        }
    }

    fn render_staff_measure(&self, staff_measure: &StaffMeasure, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_staff_measure(staff_measure, out);
        }
    }

    fn render_clef(&self, clef: &Clef, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_clef(clef, out);
        }
    }

    fn render_key_signature(&self, key_signature: &KeySignature, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_key_signature(key_signature, out);
        }
    }

    fn render_accidental(&self, accidental: &Accidental, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_accidental(accidental, out);
        }
    }

    fn render_time_signature(
        &self,
        time_signature: &TimeSignature,
        out: &mut Vec<DrawableElement>,
    ) {
        for pass in self.passes.iter() {
            pass.render_time_signature(time_signature, out);
        }
    }

    fn render_rest(&self, rest: &Rest, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_rest(rest, out);
        }
    }

    fn render_chord(&self, chord: &Chord, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_chord(chord, out);
        }
    }

    fn render_note(&self, note: &Note, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_note(note, out);
        }
    }

    fn render_stem(&self, stem: &Stem, out: &mut Vec<DrawableElement>) {
        for pass in self.passes.iter() {
            pass.render_stem(stem, out);
        }
    }
}

pub struct BaseRenderer {}

impl RenderPass for BaseRenderer {
    fn render_page(&self, page: &Page, out: &mut Vec<DrawableElement>) {
        let rect = Rect {
            xy: page.xy,
            width: page.width,
            height: page.height,
            color: page.color,
            stroke_color: Some(page.foreground),
            stroke_width: Some(1.),
        };
        out.push(rect.into());
    }

    fn render_system(&self, system: &System, out: &mut Vec<DrawableElement>) {
        let stroke_color = system.color;
        let stroke_width = system.line_width;

        let left_line = Line {
            start: system.xy,
            end: system.xy.mv(0., system.height),
            stroke_width,
            stroke_color,
        };
        out.push(left_line.into());
    }

    fn render_bracket(&self, bracket: &Bracket, out: &mut Vec<DrawableElement>) {
        let text = bracket
            .bracket_top
            .as_text(bracket.color, bracket.xy, bracket.scale);
        out.push(text.into());

        let text = bracket.bracket_bottom.as_text(
            bracket.color,
            bracket.xy.mv(0., bracket.height),
            bracket.scale,
        );
        out.push(text.into());

        let rect = Rect {
            xy: bracket.xy.mv(0., -1.),
            width: 5.,
            height: bracket.height + 2.,
            color: bracket.color,
            stroke_color: None,
            stroke_width: None,
        };
        out.push(rect.into());
    }

    fn render_brace(&self, brace: &Brace, out: &mut Vec<DrawableElement>) {
        let def_height = Staff::SPACES as f32 * Staff::DEFAULT_SPACE_SIZE;
        let scale = brace.height / def_height;

        let text = brace.brace.as_text(brace.color, brace.xy, scale);
        out.push(text.into());
    }

    fn render_staff(&self, staff: &Staff, out: &mut Vec<DrawableElement>) {
        if staff.hidden {
            return;
        }

        let mut start = XY {
            x: staff.xy.x - (staff.barline_thickness_light / 2.),
            y: staff.xy.y,
        };
        let mut end = XY {
            x: staff.xy.x + staff.width + (staff.barline_thickness_light / 2.),
            y: staff.xy.y,
        };

        let stroke_color = staff.color;
        let stroke_width = staff.line_thickness * staff.scale;

        for _i in 0..5 {
            let line = Line {
                start,
                end,
                stroke_color,
                stroke_width,
            };

            out.push(line.into());

            start = XY {
                x: start.x,
                y: start.y + staff.line_space(),
            };
            end = XY {
                x: end.x,
                y: end.y + staff.line_space(),
            };
        }
    }

    fn render_section_measure(
        &self,
        section_measure: &SectionMeasure,
        out: &mut Vec<DrawableElement>,
    ) {
        let stroke_color = section_measure.color;
        let stroke_width = section_measure.line_width;

        let right_line = Line {
            start: section_measure.xy.mv(section_measure.width, 0.),
            end: section_measure
                .xy
                .mv(section_measure.width, section_measure.height),
            stroke_width,
            stroke_color,
        };
        out.push(right_line.into());
    }

    fn render_part_measure(&self, part_measure: &PartMeasure, out: &mut Vec<DrawableElement>) {
        for beam in &part_measure.beams {
            let line: DrawableElement = beam.clone().into();
            out.push(line);
        }
        for line in &part_measure.ledgers {
            let line: DrawableElement = (*line).into();
            out.push(line);
        }
    }

    fn render_clef(&self, clef: &Clef, out: &mut Vec<DrawableElement>) {
        let text = clef.clef.as_text(clef.color, clef.xy, clef.scale);
        out.push(text.into());
    }

    fn render_time_signature(
        &self,
        time_signature: &TimeSignature,
        out: &mut Vec<DrawableElement>,
    ) {
        let pos_num = time_signature.xy.mv(0., time_signature.height / 4.);
        out.push(
            time_signature
                .num
                .as_text(time_signature.color, pos_num, 1.)
                .into(),
        );

        let pos_denom = time_signature.xy.mv(0., time_signature.height / 4. * 3.);
        out.push(
            time_signature
                .denom
                .as_text(time_signature.color, pos_denom, 1.)
                .into(),
        );
    }

    fn render_accidental(&self, accidental: &Accidental, out: &mut Vec<DrawableElement>) {
        let glyph = &accidental.glyph;
        let text = glyph.as_text(accidental.color, accidental.xy, accidental.scale);
        out.push(text.into());
    }

    fn render_rest(&self, rest: &Rest, out: &mut Vec<DrawableElement>) {
        let glyph = &rest.glyph;
        let text = glyph.as_text(rest.color, rest.xy, rest.scale);
        out.push(text.into());
    }

    fn render_note(&self, note: &Note, out: &mut Vec<DrawableElement>) {
        let glyph = &note.glyph;
        let text = glyph.as_text(note.color, note.xy, note.scale);
        out.push(text.into());
    }

    fn render_stem(&self, stem: &Stem, out: &mut Vec<DrawableElement>) {
        let thickness = stem.thickness * stem.scale;

        let line: DrawableElement = Line {
            start: stem.xy,
            end: stem.tip(),
            stroke_color: stem.color,
            stroke_width: thickness,
        }
        .into();
        out.push(line);

        if let Some(flag) = &stem.flag {
            let stem_anchor = match stem.direction {
                UpDown::Up => stem.nw(),
                UpDown::Down => stem.sw(),
            };
            out.push(display(&stem_anchor, &2., &Color::RED).into());

            let flag_anchor = flag.stem_anchor;
            let flag_anchor = scale_pt(&flag_anchor, &stem_anchor, stem.scale);
            out.push(display(&flag_anchor, &2.5, &Color::GREEN).into());

            let delta = stem_anchor - flag_anchor;

            let final_anchor = stem_anchor + delta;
            out.push(display(&final_anchor, &3., &Color::BLUE).into());

            let flag: DrawableElement = flag.as_text(stem.color, final_anchor, stem.scale).into();

            out.push(flag);
        }

        fn display(xy: &XY, size: &f32, color: &Color) -> Rect {
            Rect {
                xy: XY {
                    x: xy.x - size / 2.,
                    y: xy.y - size / 2.,
                },
                width: *size,
                height: *size,
                color: Color::TRANSPARENT,
                stroke_color: Some(*color),
                stroke_width: Some(0.25),
            }
        }

        /// Scales a normalized point to current position and scale.
        fn scale_pt(flag_anchor: &XY, stem_anchor: &XY, scale: f32) -> XY {
            let scaled = XY {
                x: flag_anchor.x * (Staff::DEFAULT_SPACE_SIZE * scale),
                y: flag_anchor.y * (Staff::DEFAULT_SPACE_SIZE * scale),
            };

            scaled + *stem_anchor
        }
    }
}

pub struct DebugRenderer {}

impl RenderPass for DebugRenderer {
    fn render_page(&self, page: &Page, out: &mut Vec<DrawableElement>) {
        let stroke_color = Color {
            a: 1.,
            ..Color::RED
        };
        let stroke_width = 1.;
        let left = Line {
            start: page.xy.mv(page.margins.left, 0.),
            end: page.xy.mv(page.margins.left, page.height),
            stroke_width,
            stroke_color,
        };
        out.push(left.into());

        let right = Line {
            start: page.xy.mv(page.width - page.margins.right, 0.),
            end: page.xy.mv(page.width - page.margins.right, page.height),
            stroke_width,
            stroke_color,
        };
        out.push(right.into());

        let top = Line {
            start: page.xy.mv(0., page.margins.top),
            end: page.xy.mv(page.width, page.margins.top),
            stroke_width,
            stroke_color,
        };
        out.push(top.into());

        let bottom = Line {
            start: page.xy.mv(0., page.height - page.margins.bottom),
            end: page.xy.mv(page.width, page.height - page.margins.bottom),
            stroke_width,
            stroke_color,
        };
        out.push(bottom.into());
    }

    fn render_accidental(&self, accidental: &Accidental, out: &mut Vec<DrawableElement>) {
        let glyph = &accidental.glyph;
        let bbox = accidental.glyph_bbox(&glyph.bbox);

        let rect: Rect = Rect {
            xy: bbox.xy,
            width: bbox.width(),
            height: bbox.height(),
            color: Color::TRANSPARENT,
            stroke_width: Some(0.25),
            stroke_color: Some(Color::RED),
        };

        out.push(rect.into());

        let origin = Line {
            start: accidental.xy,
            end: accidental.xy.mv(accidental.width, 0.),
            stroke_color: Color {
                a: 1.,
                r: 255,
                g: 0,
                b: 0,
            },
            stroke_width: 0.2,
        };
        out.push(origin.into());

        if let Some(ref cutouts) = glyph.cutouts {
            for cutout in [cutouts.nw, cutouts.ne, cutouts.se, cutouts.sw]
                .into_iter()
                .flatten()
            {
                let bbox = accidental.glyph_bbox(&cutout);

                let rect = Rect {
                    xy: bbox.xy,
                    width: bbox.width(),
                    height: bbox.height(),
                    color: Color::TRANSPARENT,
                    stroke_width: Some(0.2),
                    stroke_color: Some(Color::RED),
                };

                out.push(rect.into());
            }
        }
    }

    fn render_rest(&self, rest: &Rest, out: &mut Vec<DrawableElement>) {
        let glyph = &rest.glyph;
        let bbox = rest.scale_box(&glyph.bbox);

        let rect = Rect {
            xy: bbox.xy,
            width: bbox.size.x,
            height: bbox.size.y,
            color: Color::TRANSPARENT,
            stroke_width: Some(0.25),
            stroke_color: Some(Color {
                a: 1.,
                r: 255,
                g: 0,
                b: 0,
            }),
        };
        out.push(rect.into());

        let origin = Line {
            start: rest.xy,
            end: rest.xy.mv(rest.width, 0.),
            stroke_color: Color {
                a: 1.,
                r: 255,
                g: 0,
                b: 0,
            },
            stroke_width: 0.2,
        };
        out.push(origin.into());
    }

    fn render_note(&self, note: &Note, out: &mut Vec<DrawableElement>) {
        let glyph = &note.glyph;
        let bbox = note.scale_box(&glyph.bbox);

        let rect = Rect {
            xy: bbox.xy,
            width: bbox.width(),
            height: bbox.height(),
            color: Color::TRANSPARENT,
            stroke_width: Some(0.25),
            stroke_color: Some(Color::RED),
        };
        out.push(rect.into());

        let origin = Line {
            start: note.xy,
            end: note.xy.mv(note.width, 0.),
            stroke_color: Color {
                a: 1.,
                r: 255,
                g: 0,
                b: 0,
            },
            stroke_width: 0.2,
        };
        out.push(origin.into());

        for cutout in [
            glyph.cutouts.nw,
            glyph.cutouts.ne,
            glyph.cutouts.se,
            glyph.cutouts.sw,
        ]
        .into_iter()
        .flatten()
        {
            let bbox = note.scale_box(&cutout);

            let rect = Rect {
                xy: bbox.xy,
                width: bbox.width(),
                height: bbox.height(),
                color: Color::TRANSPARENT,
                stroke_width: Some(0.2),
                stroke_color: Some(Color::RED),
            };
            out.push(rect.into());
        }
    }
}
