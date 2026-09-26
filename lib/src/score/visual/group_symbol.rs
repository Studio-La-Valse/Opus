use crate::drawable::elements::rect::Rect;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::group_symbol::{GroupLevel, GroupSymbol as Kind};
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::staff::Staff;
use crate::smufl::glyphs::brace::Brace as SmuflBrace;
use crate::smufl::glyphs::bracket::{BracketBottom, BracketTop};
use crate::smufl::smufl_glyph::staff_space;

/// Thickness in tenths of the vertical stroke SMuFL's bracket tip glyphs are
/// drawn against: Bravura's `engravingDefaults.bracketThickness`, 0.5 staff
/// spaces. A property of the font, not a preference -- it is what a drawn
/// bracket's serifs are in proportion to, so it is what a configured stroke
/// thickness is scaled against.
const BRACKET_GLYPH_STROKE: f32 = 5.;

/// How far in tenths a bracket's stroke runs past the tips it meets, at each
/// end. Purely a seam: the stroke is drawn over the glyphs, and without the
/// overlap a hairline of background shows through where they meet. Small enough
/// to sit well inside the tips' own flare, so it never reaches the reported box
/// and is not worth a knob.
const BRACKET_STROKE_SEAM: f32 = 1.;

/// What binds a run of staves together at the left of a system: a section's
/// bracket, a part-group's brace, the brace joining one part's own staves.
///
/// One element for all five shapes rather than one per shape, because which
/// shape is drawn is not known when the element is built -- it is resolved on
/// every layout pass from the document, the caller's overrides and the level's
/// own convention, and a caller may change its mind between two renders of the
/// same walked score.
///
/// [`shape`](Self::shape) and [`bounds`](Self::bounds) are written by the same
/// statement in
/// [`arrange_group_symbol`](crate::score::visual::arranger::PageArranger::arrange_group_symbol)
/// so the arrange pass cannot produce ink that falls outside the reported box.
/// That is the arrangement [`Glyph`](crate::drawable::elements::glyph::Glyph)
/// uses for a single glyph, kept for a composite of several pieces -- and the
/// box has to be exact, because instrument names are aligned against it.
#[derive(Clone)]
pub struct GroupSymbol {
    pub level: GroupLevel,
    pub declared: Option<Kind>,

    pub kind: Kind,
    pub color: Color,
    pub gap: f32,
    pub thickness: f32,
    pub arm: f32,
    pub span: f32,

    pub glyphs: Glyphs,

    pub anchor: XY,
    pub shape: Shape,
    pub bounds: BoundingBox,
}

/// The glyph metrics the resolved shape needs, read from the font on the layout
/// pass. Owned rather than borrowed, so the score tree stays font-free.
#[derive(Clone)]
pub enum Glyphs {
    None,
    Brace(SmuflBrace),
    Bracket(BracketTop, BracketBottom),
}

/// One symbol's ink, already placed. A renderer draws exactly what is here and
/// works nothing out for itself.
#[derive(Clone)]
pub enum Shape {
    /// `GroupSymbol::None`, and any symbol given no staves to span.
    Nothing,
    /// `xy` is the brace's *right* edge, not its origin; see
    /// [`SmuflGlyph for Brace`](crate::smufl::glyphs::brace::Brace).
    Brace {
        glyph: SmuflBrace,
        xy: XY,
        scale: f32,
    },
    /// The stroke is drawn last, over the tips, which is what the seam overlap
    /// is for.
    Bracket {
        stroke: Rect,
        top: (BracketTop, XY),
        bottom: (BracketBottom, XY),
        scale: f32,
    },
    Line {
        stroke: Rect,
    },
    Square {
        stroke: Rect,
        arms: [Rect; 2],
    },
}

impl GroupSymbol {
    /// A symbol for `level`, binding whatever staves it is later given.
    ///
    /// `declared` is what the document asked for: a `<part-group>`'s
    /// `<group-symbol>`, or an `<attributes><part-symbol>` for a part. `None`
    /// means the document named none and the level's default applies -- which is
    /// not the same as `Some(Kind::None)`, a declaration that nothing be drawn.
    pub fn new(level: GroupLevel, declared: Option<Kind>) -> Self {
        Self {
            level,
            declared,

            kind: Kind::None,
            color: Color::BLACK,
            gap: 0.,
            thickness: 0.,
            arm: 0.,
            span: 0.,

            glyphs: Glyphs::None,

            anchor: XY::ZERO,
            shape: Shape::Nothing,
            bounds: BoundingBox::ZERO,
        }
    }

    /// The shape resolved for this layout pass. `None` until the first
    /// [`measure_group_symbol`](crate::score::visual::arranger::ScoreMeasurement::measure_group_symbol).
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Where the symbol's right edge sits: the system's left edge, less the
    /// level's gap, at the top line of the first staff it spans.
    pub fn anchor(&self) -> XY {
        self.anchor
    }

    /// The colour every piece of this symbol is drawn in. The [`Rect`]s in
    /// [`shape`](Self::shape) already carry it; the glyphs take it at draw time.
    pub fn color(&self) -> Color {
        self.color
    }

    /// This symbol's ink, placed. [`Shape::Nothing`] when nothing is drawn.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Exactly what the ink covers, in world coordinates.
    ///
    /// For a bracket this reaches *right* of [`anchor`](Self::anchor) and may
    /// cross the system's left edge, because SMuFL's bracket tips flare toward
    /// the staff. That is correct engraving, not an overlap to be corrected.
    pub fn bounds(&self) -> BoundingBox {
        self.bounds
    }

    /// Whether this symbol draws anything: it has a shape to draw, and staves to
    /// draw it against.
    pub fn is_drawn(&self) -> bool {
        self.kind != Kind::None && self.span > 0.
    }

    /// A filled rectangle in this symbol's colour.
    fn filled(&self, xy: XY, width: f32, height: f32) -> Rect {
        Rect {
            xy,
            width,
            height,
            color: self.color,
            stroke_color: None,
            stroke_width: None,
        }
    }

    /// The brace, scaled so its ink spans exactly the staves it binds.
    ///
    /// The scale follows the glyph's own box rather than a nominal four spaces,
    /// so an alternate brace of a different height still comes out the right
    /// size. Placing it by `bbox.xy.y` rather than by the span's foot is the
    /// same point: a glyph whose ink does not start at its origin still lands
    /// where it should.
    pub fn brace_shape(&self, glyph: &SmuflBrace) -> (Shape, BoundingBox) {
        let scale = self.span / (glyph.bbox.height() * Staff::DEFAULT_SPACE_SIZE);
        let unit = staff_space(scale);

        // The one place the brace's right-edge placement is undone, mirroring
        // what `SmuflGlyph for Brace` does when it draws. Kept in step by
        // `the_reported_brace_box_is_the_drawable_glyph_box`.
        let origin = XY {
            x: self.anchor.x - glyph.advance * unit,
            y: self.anchor.y - glyph.bbox.y_min() * unit,
        };

        let shape = Shape::Brace {
            glyph: glyph.clone(),
            xy: XY {
                x: self.anchor.x,
                y: origin.y,
            },
            scale,
        };

        (shape, glyph.bbox.placed(origin, unit))
    }

    /// The bracket: a stroke down the left, with a tip glyph at each end.
    ///
    /// The tips are scaled with the stroke so a thickened bracket keeps its
    /// serifs in proportion -- at the default thickness that factor is exactly
    /// one, and the glyphs are drawn at nominal size.
    pub fn bracket_shape(&self, top: &BracketTop, bottom: &BracketBottom) -> (Shape, BoundingBox) {
        let scale = self.thickness / BRACKET_GLYPH_STROKE;
        let unit = staff_space(scale);

        // Both tips register against the stroke's left edge.
        let left = self.anchor.x - self.thickness;
        let top_xy = XY {
            x: left,
            y: self.anchor.y,
        };
        let bottom_xy = XY {
            x: left,
            y: self.anchor.y + self.span,
        };

        let stroke = self.filled(
            XY {
                x: left,
                y: self.anchor.y - BRACKET_STROKE_SEAM,
            },
            self.thickness,
            self.span + BRACKET_STROKE_SEAM * 2.,
        );

        let bounds = stroke_box(&stroke)
            .union(&top.bbox.placed(top_xy, unit))
            .union(&bottom.bbox.placed(bottom_xy, unit));

        let shape = Shape::Bracket {
            stroke,
            top: (top.clone(), top_xy),
            bottom: (bottom.clone(), bottom_xy),
            scale,
        };

        (shape, bounds)
    }

    /// A bare vertical line, spanning exactly the staves it binds.
    pub fn line_shape(&self) -> (Shape, BoundingBox) {
        let stroke = self.filled(
            XY {
                x: self.anchor.x - self.thickness,
                y: self.anchor.y,
            },
            self.thickness,
            self.span,
        );

        (Shape::Line { stroke }, stroke_box(&stroke))
    }

    /// A square bracket: a stroke with an arm at each end reaching toward the
    /// system.
    ///
    /// The arms sit outside the staves rather than over them, and the stroke
    /// runs the same distance so the corners are square. The arms' right ends
    /// are what the gap is measured to, since they are what comes nearest the
    /// staff.
    pub fn square_shape(&self) -> (Shape, BoundingBox) {
        let t = self.thickness;
        let left = self.anchor.x - self.arm - t;
        let top = self.anchor.y - t;

        let stroke = self.filled(XY { x: left, y: top }, t, self.span + t * 2.);
        let arms = [
            self.filled(XY { x: left, y: top }, self.arm + t, t),
            self.filled(
                XY {
                    x: left,
                    y: self.anchor.y + self.span,
                },
                self.arm + t,
                t,
            ),
        ];

        let bounds = stroke_box(&stroke)
            .union(&stroke_box(&arms[0]))
            .union(&stroke_box(&arms[1]));

        (Shape::Square { stroke, arms }, bounds)
    }

    /// Nothing drawn, but still a box: zero-width at the anchor, spanning the
    /// staves. A name aligned against a symbol-less group then lands where it
    /// would have had there been one.
    pub fn empty_shape(&self) -> (Shape, BoundingBox) {
        let bounds = BoundingBox {
            xy: self.anchor,
            size: XY {
                x: 0.,
                y: self.span,
            },
        };

        (Shape::Nothing, bounds)
    }
}

fn stroke_box(rect: &Rect) -> BoundingBox {
    BoundingBox {
        xy: rect.xy,
        size: XY {
            x: rect.width,
            y: rect.height,
        },
    }
}

impl GroupSymbol {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            font,
            ..
        } = params;

        self.kind = params.group_symbol(self.level, self.declared);
        self.gap = params.group_symbol_gap(self.level);
        self.thickness = params.group_symbol_thickness(self.kind);
        self.arm = params.group_square_arm();

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        // Read here rather than at construction: the walk that builds the tree
        // runs once, and the shape can change between two renders of it.
        self.glyphs = match self.kind {
            Kind::Brace => Glyphs::Brace(font.brace(None)),
            Kind::Bracket => Glyphs::Bracket(font.bracket_top(), font.bracket_bottom()),
            Kind::None | Kind::Line | Kind::Square => Glyphs::None,
        };
    }
}
