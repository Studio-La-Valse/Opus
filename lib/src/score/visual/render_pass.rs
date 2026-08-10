use crate::drawable::drawable_element::DrawableElement;
use crate::smufl::smufl_font::SmuflFont;
use crate::{
    drawable::elements::{line::Line, rect::Rect},
    geometry::{color::Color, xy::XY},
    score::visual::{
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
    smufl::smufl_glyph::SmuflGlyph,
};

pub trait RenderPass {
    fn render_page<'a>(
        &self,
        _page: &Page,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_system<'a>(
        &self,
        _system: &System,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_section<'a>(
        &self,
        _section: &Section,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_bracket<'a>(
        &self,
        _bracket: &Bracket,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part_group<'a>(
        &self,
        _part_group: &PartGroup,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_brace<'a>(
        &self,
        _brace: &Brace,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part<'a>(
        &self,
        _part: &Part,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_staff<'a>(
        &self,
        _staff: &Staff,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_system_measure<'a>(
        &self,
        _system_measure: &SystemMeasure,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_section_measure<'a>(
        &self,
        _section_measure: &SectionMeasure,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part_group_measure<'a>(
        &self,
        _part_group_measure: &PartGroupMeasure,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part_measure<'a>(
        &self,
        _part_measure: &PartMeasure,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_staff_measure<'a>(
        &self,
        _staff_measure: &StaffMeasure,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_clef<'a>(
        &self,
        _clef: &Clef,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_key_signature<'a>(
        &self,
        _key_signature: &KeySignature,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_accidental<'a>(
        &self,
        _accidental: &Accidental,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_time_signature<'a>(
        &self,
        _time_signature: &TimeSignature,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_rest<'a>(
        &self,
        _rest: &Rest,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_chord<'a>(
        &self,
        _chord: &Chord,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_note<'a>(
        &self,
        _note: &Note,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_stem<'a>(
        &self,
        _stem: &Stem,
        _font: &'a SmuflFont,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }
}

pub struct RenderPasses {
    pub passes: Vec<Box<dyn RenderPass>>,
}

impl RenderPass for RenderPasses {
    fn render_page<'a>(
        &self,
        page: &Page,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_page(page, font, out);
        }
    }

    fn render_system<'a>(
        &self,
        system: &System,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_system(system, font, out);
        }
    }

    fn render_section<'a>(
        &self,
        section: &Section,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_section(section, font, out);
        }
    }

    fn render_bracket<'a>(
        &self,
        bracket: &Bracket,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_bracket(bracket, font, out);
        }
    }

    fn render_part_group<'a>(
        &self,
        part_group: &PartGroup,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_part_group(part_group, font, out);
        }
    }

    fn render_brace<'a>(
        &self,
        brace: &Brace,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_brace(brace, font, out);
        }
    }

    fn render_part<'a>(
        &self,
        part: &Part,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_part(part, font, out);
        }
    }

    fn render_staff<'a>(
        &self,
        staff: &Staff,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_staff(staff, font, out);
        }
    }

    fn render_system_measure<'a>(
        &self,
        system_measure: &SystemMeasure,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_system_measure(system_measure, font, out);
        }
    }

    fn render_section_measure<'a>(
        &self,
        section_measure: &SectionMeasure,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_section_measure(section_measure, font, out);
        }
    }

    fn render_part_group_measure<'a>(
        &self,
        part_group_measure: &PartGroupMeasure,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_part_group_measure(part_group_measure, font, out);
        }
    }

    fn render_part_measure<'a>(
        &self,
        part_measure: &PartMeasure,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_part_measure(part_measure, font, out);
        }
    }

    fn render_staff_measure<'a>(
        &self,
        staff_measure: &StaffMeasure,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_staff_measure(staff_measure, font, out);
        }
    }

    fn render_clef<'a>(
        &self,
        clef: &Clef,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_clef(clef, font, out);
        }
    }

    fn render_key_signature<'a>(
        &self,
        key_signature: &KeySignature,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_key_signature(key_signature, font, out);
        }
    }

    fn render_accidental<'a>(
        &self,
        accidental: &Accidental,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_accidental(accidental, font, out);
        }
    }

    fn render_time_signature<'a>(
        &self,
        time_signature: &TimeSignature,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_time_signature(time_signature, font, out);
        }
    }

    fn render_rest<'a>(
        &self,
        rest: &Rest,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_rest(rest, font, out);
        }
    }

    fn render_chord<'a>(
        &self,
        chord: &Chord,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_chord(chord, font, out);
        }
    }

    fn render_note<'a>(
        &self,
        note: &Note,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_note(note, font, out);
        }
    }

    fn render_stem<'a>(
        &self,
        stem: &Stem,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for pass in self.passes.iter() {
            pass.render_stem(stem, font, out);
        }
    }
}

pub struct BaseRenderer {}

impl RenderPass for BaseRenderer {
    fn render_page<'a>(
        &self,
        page: &Page,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

    fn render_system<'a>(
        &self,
        system: &System,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

    fn render_bracket<'a>(
        &self,
        bracket: &Bracket,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let text = bracket
            .bracket_top
            .as_text(font, bracket.color, bracket.xy, bracket.scale);
        out.push(text.into());

        let text = bracket.bracket_bottom.as_text(
            font,
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

    fn render_brace<'a>(
        &self,
        brace: &Brace,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let def_height = Staff::SPACES as f32 * Staff::DEFAULT_SPACE_SIZE;
        let scale = brace.height / def_height;

        let text = brace.brace.as_text(font, brace.color, brace.xy, scale);
        out.push(text.into());
    }

    fn render_staff<'a>(
        &self,
        staff: &Staff,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

            start = start.mv(0., staff.line_space());
            end = end.mv(0., staff.line_space());
        }
    }

    fn render_section_measure<'a>(
        &self,
        section_measure: &SectionMeasure,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
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

    fn render_part_measure<'a>(
        &self,
        part_measure: &PartMeasure,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for beam in &part_measure.beams {
            let line: DrawableElement = beam.clone().into();
            out.push(line);
        }
        for line in &part_measure.ledgers {
            let line: DrawableElement = (*line).into();
            out.push(line);
        }
    }

    fn render_clef<'a>(
        &self,
        clef: &Clef,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let text = clef.clef.as_text(font, clef.color, clef.xy, clef.scale);
        out.push(text.into());
    }

    fn render_time_signature<'a>(
        &self,
        time_signature: &TimeSignature,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        out.push(
            time_signature
                .num
                .as_text(
                    font,
                    time_signature.color,
                    time_signature.num_xy(),
                    time_signature.scale,
                )
                .into(),
        );

        out.push(
            time_signature
                .denom
                .as_text(
                    font,
                    time_signature.color,
                    time_signature.denom_xy(),
                    time_signature.scale,
                )
                .into(),
        );
    }

    fn render_accidental<'a>(
        &self,
        accidental: &Accidental,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = &accidental.glyph;
        let text = glyph.as_text(font, accidental.color, accidental.xy, accidental.scale);
        out.push(text.into());
    }

    fn render_rest<'a>(
        &self,
        rest: &Rest,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = &rest.glyph;
        let text = glyph.as_text(font, rest.color, rest.xy, rest.scale);
        out.push(text.into());
    }

    fn render_note<'a>(
        &self,
        note: &Note,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = &note.glyph;
        let text = glyph.as_text(font, note.color, note.xy, note.scale);
        out.push(text.into());
    }

    fn render_stem<'a>(
        &self,
        stem: &Stem,
        font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

            let flag_anchor = flag.stem_anchor;
            let flag_anchor = scale_pt(&flag_anchor, &stem_anchor, stem.scale);
            let delta = stem_anchor - flag_anchor;
            let final_anchor = stem_anchor + delta;

            let flag: DrawableElement = flag
                .as_text(font, stem.color, final_anchor, stem.scale)
                .into();
            out.push(flag);
        }
    }
}

pub struct DebugRenderer {}

impl RenderPass for DebugRenderer {
    fn render_page<'a>(
        &self,
        page: &Page,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

    fn render_clef<'a>(
        &self,
        clef: &Clef,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let bbox = clef.scale_box(&clef.clef.bbox);

        let rect = Rect {
            xy: bbox.xy,
            width: bbox.width(),
            height: bbox.height(),
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
            start: clef.xy,
            end: clef.xy.mv(clef.width, 0.),
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

    fn render_time_signature<'a>(
        &self,
        time_signature: &TimeSignature,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        for (glyph, xy) in [
            (&time_signature.num, time_signature.num_xy()),
            (&time_signature.denom, time_signature.denom_xy()),
        ] {
            let bbox = time_signature
                .scale_box(&glyph.bbox)
                .mv(0., xy.y - time_signature.xy.y);

            let rect = Rect {
                xy: bbox.xy,
                width: bbox.width(),
                height: bbox.height(),
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
                start: xy,
                end: xy.mv(bbox.width(), 0.),
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
    }

    fn render_accidental<'a>(
        &self,
        accidental: &Accidental,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

    fn render_rest<'a>(
        &self,
        rest: &Rest,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

    fn render_note<'a>(
        &self,
        note: &Note,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
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

    fn render_stem<'a>(
        &self,
        stem: &Stem,
        _font: &'a SmuflFont,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        match stem.direction {
            UpDown::Down => {
                out.push(display_xy(&stem.nw(), &1., &Color::RED).into());
                out.push(display_xy(&stem.sw(), &2., &Color::RED).into());
            }
            UpDown::Up => {
                out.push(display_xy(&stem.nw(), &1., &Color::RED).into());
                out.push(display_xy(&stem.se(), &1., &Color::RED).into());
            }
        }

        // if let Some(flag) = &stem.flag {
        // let stem_anchor = match stem.direction {
        //     UpDown::Up => stem.nw(),
        //     UpDown::Down => stem.se(),
        // };
        // out.push(display(&stem_anchor, &2., &Color::RED).into());

        // let flag_anchor = flag.stem_anchor;
        // let flag_anchor = scale_pt(&flag_anchor, &stem_anchor, stem.scale);
        // out.push(display(&flag_anchor, &2.5, &Color::GREEN).into());

        // let delta = stem_anchor - flag_anchor;
        // let final_anchor = stem_anchor + delta;
        // out.push(display(&final_anchor, &3., &Color::BLUE).into());
        // }
    }
}

fn display_xy(xy: &XY, size: &f32, color: &Color) -> Rect {
    Rect {
        xy: XY {
            x: xy.x - size / 2.,
            y: xy.y - size / 2.,
        },
        width: *size,
        height: *size,
        color: *color,
        stroke_color: None,
        stroke_width: None,
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
