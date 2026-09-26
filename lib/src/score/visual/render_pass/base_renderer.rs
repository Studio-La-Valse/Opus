use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::drawable::elements::{circle::Circle, line::Line, polygon::Polygon, rect::Rect};
use crate::geometry::xy::XY;
use crate::score::visual::render_fonts::RenderFonts;
use crate::score::visual::render_pass::RenderPass;
use crate::score::visual::{
    accidental::Accidental,
    clef::Clef,
    dot::Dot,
    flag::Flag,
    group_name::GroupName,
    group_symbol::{GroupSymbol, Shape},
    note::Note,
    page::Page,
    part_measure::PartMeasure,
    rest::Rest,
    section_measure::SectionMeasure,
    staff::Staff,
    stem::Stem,
    system::System,
    tie::TieSegment,
    time_signature::TimeSignature,
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
            xy: XY::ZERO,
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
        let stroke_width = system.light_barline;

        let (top, bottom) = system.barline_span();
        let left_line = Line {
            start: system.xy.mv(0., top),
            end: system.xy.mv(0., bottom),
            stroke_width,
            stroke_color,
        };
        out.push(left_line.into());
    }

    /// Draws whatever shape the symbol resolved to. Everything here is already
    /// placed -- the renderer works out no geometry of its own, so what is drawn
    /// and what the symbol reports as its bounds cannot disagree.
    fn render_group_symbol<'a>(
        &self,
        symbol: &GroupSymbol,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let color = symbol.color();

        match symbol.shape() {
            Shape::Nothing => {}

            Shape::Brace { glyph, xy, scale } => {
                out.push(glyph.as_glyph(fonts.smufl, color, *xy, *scale).into());
            }

            Shape::Bracket {
                stroke,
                top,
                bottom,
                scale,
            } => {
                // Tips first: the stroke is drawn over them, which is what
                // closes the seam where they meet.
                out.push(top.0.as_glyph(fonts.smufl, color, top.1, *scale).into());
                out.push(
                    bottom
                        .0
                        .as_glyph(fonts.smufl, color, bottom.1, *scale)
                        .into(),
                );
                out.push((*stroke).into());
            }

            Shape::Line { stroke } => out.push((*stroke).into()),

            Shape::Square { stroke, arms } => {
                out.push((*stroke).into());
                for arm in arms {
                    out.push((*arm).into());
                }
            }
        }
    }

    /// One [`Text`] laid out in the reserved box the name reports, drawn from
    /// its right-middle point so the run ends against the symbol and is centred
    /// on the staves. No background: whatever is under the box shows through, as
    /// every text does today.
    fn render_group_name<'a>(
        &self,
        name: &'a GroupName,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let text = Text {
            text: name.text(),
            color: name.color(),
            font_size: name.font_size(),
            font: fonts.group_name,
            bounds: name.bounds(),
            vertical_alignment: VerticalAlign::Middle,
            horizontal_alignment: HorizontalAlign::Right,
            background: None,
        };
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

        // The lines start at the top of the block the staff occupies, save for
        // a one-line staff, whose single line stands in for a five-line staff's
        // middle and is drawn two spaces down. See `Staff::top_line_offset`.
        let top = staff.xy.y + staff.top_line_offset();
        let mut start = XY {
            x: staff.xy.x - (staff.light_barline / 2.),
            y: top,
        };
        let mut end = XY {
            x: staff.xy.x + staff.width + (staff.light_barline / 2.),
            y: top,
        };

        let stroke_color = staff.color;
        let stroke_width = staff.line_width * staff.scale;

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
        let stroke_width = section_measure.light_barline;

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
        for line in &part_measure.ledgers {
            let line: DrawableElement = (*line).into();
            out.push(line);
        }
    }

    fn render_beam<'a>(
        &self,
        beam: &Polygon,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        out.push(beam.clone().into());
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
        let glyph = clef
            .glyph()
            .as_glyph(fonts.smufl, clef.color, clef.xy, clef.scale);
        out.push(glyph.into());
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
            let glyph = digit.as_glyph(fonts.smufl, time_signature.color, xy, time_signature.scale);
            out.push(glyph.into());
        }
    }

    fn render_accidental<'a>(
        &self,
        accidental: &Accidental,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = accidental.glyph().as_glyph(
            fonts.smufl,
            accidental.color,
            accidental.xy,
            accidental.scale,
        );
        out.push(glyph.into());
    }

    fn render_rest<'a>(
        &self,
        rest: &Rest,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = rest
            .glyph()
            .as_glyph(fonts.smufl, rest.color, rest.xy, rest.scale);
        out.push(glyph.into());
    }

    fn render_note<'a>(
        &self,
        note: &Note,
        fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let glyph = note
            .glyph()
            .as_glyph(fonts.smufl, note.color, note.xy, note.scale);
        out.push(glyph.into());
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
        let glyph = flag
            .glyph()
            .as_glyph(fonts.smufl, flag.color, flag.xy, flag.scale);
        out.push(glyph.into());
    }
}
