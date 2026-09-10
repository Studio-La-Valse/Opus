use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::{line::Line, rect::Rect};
use crate::geometry::{color::Color, xy::XY};
use crate::score::visual::placed::Placed;
use crate::score::visual::render_fonts::RenderFonts;
use crate::score::visual::render_pass::RenderPass;
use crate::score::visual::{
    accidental::Accidental,
    clef::Clef,
    dot::Dot,
    flag::Flag,
    group_name::GroupName,
    group_symbol::GroupSymbol,
    note::Note,
    page::Page,
    rest::Rest,
    stem::{Stem, UpDown},
    time_signature::TimeSignature,
};

/// An overlay renderer used with `--debug`: draws page margins, glyph bounding
/// boxes, anchor points and origin markers in red / green on top of the normal
/// output.
pub struct DebugRenderer {}

impl RenderPass for DebugRenderer {
    fn render_page<'a>(
        &self,
        page: &Page,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let stroke_color = Color::RED;
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

    /// The box a group symbol reports, plus its anchor.
    ///
    /// Worth drawing even though the symbol is one of the simplest shapes on the
    /// page: this box is what instrument and part-group names are aligned
    /// against, so being able to see it is the point of it being exact. A
    /// bracket's box reaching right of the anchor, past the system's left edge,
    /// is correct -- its tips flare that way.
    fn render_group_symbol<'a>(
        &self,
        symbol: &GroupSymbol,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let bbox = symbol.bounds();

        let rect = Rect {
            xy: bbox.xy,
            width: bbox.width(),
            height: bbox.height(),
            color: Color::TRANSPARENT,
            stroke_width: Some(0.25),
            stroke_color: Some(Color::RED),
        };
        out.push(rect.into());

        out.push(display_xy(&symbol.anchor(), &2.5, &Color::GREEN).into());
    }

    /// The reserved box a name reports, plus the point its run is drawn from.
    fn render_group_name<'a>(
        &self,
        name: &'a GroupName,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let bbox = name.bounds();

        let rect = Rect {
            xy: bbox.xy,
            width: bbox.width(),
            height: bbox.height(),
            color: Color::TRANSPARENT,
            stroke_width: Some(0.25),
            stroke_color: Some(Color::RED),
        };
        out.push(rect.into());

        out.push(display_xy(&name.anchor(), &2.5, &Color::GREEN).into());
    }

    fn render_clef<'a>(
        &self,
        clef: &Clef,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let bbox = clef.scaled_box();

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
            start: clef.xy,
            end: clef.xy.mv(clef.width, 0.),
            stroke_color: Color::RED,
            stroke_width: 0.2,
        };
        out.push(origin.into());
    }

    fn render_time_signature<'a>(
        &self,
        time_signature: &TimeSignature,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let digits = time_signature
            .num_digits()
            .chain(time_signature.denom_digits());

        for (digit, xy) in digits {
            let bbox = time_signature.digit_box(digit, xy);

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
                start: xy,
                end: xy.mv(bbox.width(), 0.),
                stroke_color: Color::RED,
                stroke_width: 0.2,
            };
            out.push(origin.into());
        }
    }

    fn render_accidental<'a>(
        &self,
        accidental: &Accidental,
        _fonts: &RenderFonts<'a>,
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
            stroke_color: Color::RED,
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
        _fonts: &RenderFonts<'a>,
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
            stroke_color: Some(Color::RED),
        };
        out.push(rect.into());

        let origin = Line {
            start: rest.xy,
            end: rest.xy.mv(rest.width, 0.),
            stroke_color: Color::RED,
            stroke_width: 0.2,
        };
        out.push(origin.into());
    }

    fn render_note<'a>(
        &self,
        note: &Note,
        _fonts: &RenderFonts<'a>,
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
            stroke_color: Color::RED,
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
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        match stem.direction {
            UpDown::Down => {
                out.push(display_xy(&stem.nw(), &2.5, &Color::RED).into());
                out.push(display_xy(&stem.sw(), &2.5, &Color::RED).into());
            }
            UpDown::Up => {
                out.push(display_xy(&stem.nw(), &2.5, &Color::RED).into());
                out.push(display_xy(&stem.se(), &2.5, &Color::RED).into());
            }
        }
    }

    fn render_flag<'a>(
        &self,
        flag: &Flag,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let bbox = flag.scale_box(&flag.glyph.bbox);

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
            start: flag.xy,
            end: flag.xy.mv(flag.width, 0.),
            stroke_color: Color::RED,
            stroke_width: 0.2,
        };
        out.push(origin.into());

        out.push(display_xy(&flag.stem_anchor_world(), &1., &Color::GREEN).into());
    }

    fn render_dot<'a>(
        &self,
        dot: &Dot,
        _fonts: &RenderFonts<'a>,
        out: &mut Vec<DrawableElement<'a>>,
    ) {
        let rect = Rect {
            xy: XY {
                x: dot.xy.x - dot.radius,
                y: dot.xy.y - dot.radius,
            },
            width: dot.radius * 2.,
            height: dot.radius * 2.,
            color: Color::TRANSPARENT,
            stroke_width: Some(0.2),
            stroke_color: Some(Color::RED),
        };
        out.push(rect.into());
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
