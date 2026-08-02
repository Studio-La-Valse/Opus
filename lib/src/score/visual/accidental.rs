use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::drawable_content::Drawable;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
use crate::drawable::layoutable::Layoutable;
use crate::smufl::glyphs::accidental::Accidental as SmuflAccidental;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff::Staff;
use crate::xy::XY;

pub struct Accidental {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,
    pub color: Color,

    pub glyph: SmuflAccidental,
}

impl Accidental {
    pub fn new(glyph: SmuflAccidental) -> Self {
        Self {
            xy: XY::ZERO,
            width: 0.,
            height: 0.,

            scale: 1.,
            color: Color::BLACK,

            glyph,
        }
    }

    /// Scales a (smufl-like-) normalized bounding box to current position and scale.
    pub fn glyph_bbox(&self, bbox: &BoundingBox) -> BoundingBox {
        let scaled: BoundingBox = BoundingBox {
            xy: bbox.xy.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
            size: bbox.size.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        scaled.mv(self.xy.x, self.xy.y)
    }
}

impl ScoreElement for Accidental {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn _apply_layout(
        &mut self,
        _layout: &crate::layout::Layout,
        user_layout: &crate::user_layout::UserLayout,
        app_defaults: &crate::app_defaults::AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Accidental {
    fn measure(&mut self, _available: &XY) {
        let glyph = &self.glyph;
        let bbox = self.glyph_bbox(&glyph.bbox);

        self.width = bbox.width();
        self.height = bbox.height();
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = origin.mv(-self.width, 0.);
    }
}

impl Drawable for Accidental {
    fn content(&self) -> Vec<&dyn Drawable> {
        vec![]
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut result: Vec<DrawableElement> = Vec::new();

        let glyph = &self.glyph;
        let text = glyph.as_text(self.color, self.xy, self.scale);
        result.push(text.into());

        let bbox = self.glyph_bbox(&glyph.bbox);

        let rect: Rect = Rect {
            xy: bbox.xy,
            width: bbox.width(),
            height: bbox.height(),
            color: Color::TRANSPARENT,
            stroke_width: Some(0.25),
            stroke_color: Some(Color::RED),
        };

        result.push(rect.into());

        let origin = Line {
            start: self.xy,
            end: self.xy.mv(self.width, 0.),
            stroke_color: Color {
                a: 1.,
                r: 255,
                g: 0,
                b: 0,
            },
            stroke_width: 0.2,
        };
        result.push(origin.into());

        for cutout in [
            glyph.cutouts.nw,
            glyph.cutouts.ne,
            glyph.cutouts.se,
            glyph.cutouts.sw,
        ]
        .into_iter()
        .flatten()
        {
            let bbox = self.glyph_bbox(&cutout);

            let rect = Rect {
                xy: bbox.xy,
                width: bbox.width(),
                height: bbox.height(),
                color: Color::TRANSPARENT,
                stroke_width: Some(0.2),
                stroke_color: Some(Color::RED),
            };

            result.push(rect.into());
        }

        result
    }
}
