use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::{circle::Circle, line::Line, rect::Rect};
use crate::geometry::xy::XY;
use crate::score::visual::render_fonts::RenderFonts;
use crate::score::visual::render_pass::RenderPass;
use crate::score::visual::{
    accidental::Accidental, brace::Brace, bracket::Bracket, clef::Clef, dot::Dot, flag::Flag,
    note::Note, page::Page, part_measure::PartMeasure, rest::Rest, section_measure::SectionMeasure,
    staff::Staff, stem::Stem, system::System, tie::TieSegment, time_signature::TimeSignature,
};
use crate::smufl::smufl_glyph::SmuflGlyph;

/// The normal score renderer: staves, stems, beams, and the SMuFL glyphs for
/// clefs, noteheads, accidentals, rests, time signatures, braces and brackets.
pub struct BaseRenderer {}

impl RenderPass for BaseRenderer {
    fn render_page<'a>(
        &self,
        page: &Page,
        _fonts: &RenderFonts<'a>,
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
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let stroke_color = system.color;
        let stroke_width = system.line_width;

        let (top, bottom) = system.barline_span();
        let left_line = Line {
            start: system.xy.mv(0., top),
            end: system.xy.mv(0., bottom),
            stroke_width,
            stroke_color,
        };
        out.push(left_line.into());
    }

    fn render_bracket<'a>(
        &self,
        bracket: &Bracket,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let text =
            bracket
                .bracket_top
                .as_text(fonts.smufl, bracket.color, bracket.xy, bracket.scale);
        out.push(text.into());

        let text = bracket.bracket_bottom.as_text(
            fonts.smufl,
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
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let def_height = Staff::SPACES as f32 * Staff::DEFAULT_SPACE_SIZE;
        let scale = brace.height / def_height;

        let text = brace
            .brace
            .as_text(fonts.smufl, brace.color, brace.xy, scale);
        out.push(text.into());
    }

    fn render_staff<'a>(
        &self,
        staff: &Staff,
        _fonts: &RenderFonts<'a>,
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

        for _i in 0..staff.lines {
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
        _fonts: &RenderFonts<'a>,
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
        _fonts: &RenderFonts<'a>,
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

    fn render_tie<'a>(
        &self,
        tie: &TieSegment,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        out.push(tie.shape.clone().into());
    }

    fn render_clef<'a>(
        &self,
        clef: &Clef,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let text = clef
            .clef
            .as_text(fonts.smufl, clef.color, clef.xy, clef.scale);
        out.push(text.into());
    }

    fn render_time_signature<'a>(
        &self,
        time_signature: &TimeSignature,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        // One text per digit: `12` is two glyphs, and the font gives no glyph
        // for a whole multi-digit number.
        let digits = time_signature
            .num_digits()
            .chain(time_signature.denom_digits());

        for (digit, xy) in digits {
            let text = digit.as_text(fonts.smufl, time_signature.color, xy, time_signature.scale);
            out.push(text.into());
        }
    }

    fn render_accidental<'a>(
        &self,
        accidental: &Accidental,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = &accidental.glyph;
        let text = glyph.as_text(
            fonts.smufl,
            accidental.color,
            accidental.xy,
            accidental.scale,
        );
        out.push(text.into());
    }

    fn render_rest<'a>(
        &self,
        rest: &Rest,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = &rest.glyph;
        let text = glyph.as_text(fonts.smufl, rest.color, rest.xy, rest.scale);
        out.push(text.into());
    }

    fn render_note<'a>(
        &self,
        note: &Note,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = &note.glyph;
        let text = glyph.as_text(fonts.smufl, note.color, note.xy, note.scale);
        out.push(text.into());
    }

    fn render_dot<'a>(
        &self,
        dot: &Dot,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let circle: DrawableElement = Circle {
            xy: dot.xy,
            radius: dot.radius,
            color: dot.color,
            stroke_color: None,
            stroke_width: None,
        }
        .into();
        out.push(circle);
    }

    fn render_stem<'a>(
        &self,
        stem: &Stem,
        _fonts: &RenderFonts<'a>,
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
    }

    fn render_flag<'a>(
        &self,
        flag: &Flag,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = &flag.glyph;
        let text = glyph.as_text(fonts.smufl, flag.color, flag.xy, flag.scale);
        out.push(text.into());
    }
}
