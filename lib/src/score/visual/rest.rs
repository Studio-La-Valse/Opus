use crate::app_defaults::AppDefaults;
use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::drawable_content::Drawable;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::score::core::staff_idx::StaffIdx;
use crate::smufl::glyphs::rest::Rest as SmuflRest;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::user_layout::UserLayout;
use crate::visual::clef::Clef;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff::Staff;
use crate::visual::staff_ctx::StaffCtx;

pub struct Rest {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub is_measure: bool,

    pub default_x: Option<f32>,
    pub staff_line: i32,
    pub staff: StaffIdx,

    pub scale: f32,

    pub color: Color,

    pub glyph: SmuflRest,

    pub clef_change: Option<Clef>,
}

impl Rest {
    pub fn new(
        glyph: SmuflRest,
        is_measure: bool,
        default_x: Option<f32>,
        staff: StaffIdx,
        staff_line: i32,
        scale: f32,
    ) -> Self {
        Rest {
            glyph,

            is_measure,

            default_x,
            staff,
            staff_line,
            scale,

            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            color: Color::default(),

            clef_change: None,
        }
    }

    /// here, origin is the origin of the staff measure, so adjust y coordinate for staff distance.
    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &StaffCtx) {
        self.arrange_glyph(origin, staff_ctx);
        self.arrange_clef_changes(origin, staff_ctx);
    }

    fn arrange_glyph(&mut self, origin: &XY, staff_ctx: &StaffCtx) {
        let mut dy = staff_ctx.distance_from_top;
        dy += self.staff_line as f32 * ((Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling);
        self.xy = XY {
            x: origin.x,
            y: origin.y + dy,
        };
    }

    fn arrange_clef_changes(&mut self, origin: &XY, ctx: &StaffCtx) {
        if let Some(ref mut clef) = self.clef_change {
            let dx = -5. - clef.width;
            let dy = ctx.distance_from_top
                + clef.clef.line as f32 * Staff::DEFAULT_SPACE_SIZE / 2. * ctx.scaling;

            let origin = origin.mv(dx, dy);

            clef.arrange(&origin);

            clef.scale = ctx.scaling * 0.8;
        }
    }

    /// Scales a (smufl-like-) normalized bounding box to current position and scale.
    pub fn scale_box(&self, bbox: &BoundingBox) -> BoundingBox {
        let scaled = BoundingBox {
            x_min: bbox.x_min * (Staff::DEFAULT_SPACE_SIZE * self.scale),
            y_min: bbox.y_min * (Staff::DEFAULT_SPACE_SIZE * self.scale),
            x_max: bbox.x_max * (Staff::DEFAULT_SPACE_SIZE * self.scale),
            y_max: bbox.y_max * (Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        BoundingBox {
            x_min: scaled.x_min + self.xy.x,
            y_min: scaled.y_min + self.xy.y,
            x_max: scaled.x_max + self.xy.x,
            y_max: scaled.y_max + self.xy.y,
        }
    }

    /// Scales a normalized point to current position and scale.
    pub fn scale_pt(&self, xy: &XY) -> XY {
        let scaled = XY {
            x: xy.x * (Staff::DEFAULT_SPACE_SIZE * self.scale),
            y: xy.y * (Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        XY {
            x: scaled.x + self.xy.x,
            y: scaled.y + self.xy.y,
        }
    }
}

impl ScoreElement for Rest {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut children: Vec<&mut dyn ScoreElement> = Vec::new();

        if let Some(clef) = self.clef_change.as_mut() {
            children.push(clef)
        }

        children
    }

    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Rest {
    fn measure(&mut self, _available: &XY) {
        self.height = Staff::DEFAULT_SPACE_SIZE * self.scale;

        let glyph = &self.glyph;
        let bbox = self.scale_box(&glyph.bbox);

        if let Some(clef) = self.clef_change.as_mut() {
            clef.measure(_available);
        }

        self.width = bbox.width();
    }

    fn arrange(&mut self, _origin: &XY) {
        todo!("use arrange_ctx instead")
    }
}

impl Drawable for Rest {
    fn content(&self) -> Vec<&dyn Drawable> {
        let mut content: Vec<&dyn Drawable> = Vec::new();

        if let Some(ref clef) = self.clef_change {
            content.push(clef)
        }

        content
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut result: Vec<DrawableElement> = Vec::new();

        let glyph = &self.glyph;
        let text = glyph.as_text(self.color, self.xy, self.scale);
        result.push(text.into());

        let bbox = self.scale_box(&glyph.bbox);

        let rect = Rect {
            xy: XY {
                x: bbox.x_min,
                y: bbox.y_min,
            },
            width: bbox.x_max - bbox.x_min,
            height: bbox.y_max - bbox.y_min,
            color: Color::TRANSPARENT,
            stroke_width: Some(0.25),
            stroke_color: Some(Color {
                a: 1.,
                r: 255,
                g: 0,
                b: 0,
            }),
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

        result
    }
}
