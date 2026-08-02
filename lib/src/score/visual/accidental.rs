use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::drawable_content::Drawable;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::layoutable::Layoutable;
use crate::smufl::glyphs::accidental::Accidental as SmuflAccidental;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::score_element::ScoreElement;
use crate::xy::XY;

pub struct Accidental {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,
    pub color: Color,

    pub accidental: SmuflAccidental,
}

impl Accidental {
    pub fn new(accidental: SmuflAccidental) -> Self {
        Self {
            xy: XY::ZERO,
            width: 10.,
            height: 10.,

            scale: 1.,
            color: Color::BLACK,

            accidental,
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            xy: self.xy,
            size: XY {
                x: self.width,
                y: self.height,
            },
        }
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
        // todo: Measure from bounding box
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Drawable for Accidental {
    fn content(&self) -> Vec<&dyn Drawable> {
        vec![]
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![
            self.accidental
                .as_text(self.color, self.xy, self.scale)
                .into(),
        ]
    }
}
