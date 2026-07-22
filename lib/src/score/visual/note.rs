use crate::app_defaults::AppDefaults;
use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::score::core::staff_idx::StaffIdx;
use crate::smufl::glyphs::notehead::Notehead;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::user_layout::UserLayout;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff::Staff;
use crate::visual::staff_ctx::StaffCtx;

pub struct Note {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub default_x: f32,
    pub staff_line: i32,
    pub staff: StaffIdx,

    pub scale: f32,

    pub color: Color,

    pub glyph: Notehead,
}

impl Note {
    pub fn new(
        glyph: Notehead,
        default_x: f32,
        staff: StaffIdx,
        staff_line: i32,
        scale: f32,
    ) -> Self {
        Note {
            glyph,

            default_x,
            staff,
            staff_line,

            scale,

            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            color: Color::default(),
        }
    }

    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &StaffCtx) {
        let staff_top = origin.mv(0., staff_ctx.distance_from_top);
        let note_dy =
            self.staff_line as f32 * ((Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling);
        let note_top = staff_top.mv(0., note_dy);
        self.xy = note_top.mv(self.default_x, 0.);
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

impl ScoreElement for Note {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let children: Vec<&mut dyn ScoreElement> = Vec::new();
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

impl Layoutable for Note {
    fn measure(&mut self, _available: &XY) {
        self.height = Staff::DEFAULT_SPACE_SIZE * self.scale;

        let glyph = &self.glyph;
        let bbox = self.scale_box(&glyph.bbox);

        self.width = bbox.width();
    }

    /// here, origin is the origin of the part measure. Get the dy from the staff ctx.
    fn arrange(&mut self, _origin: &XY) {
        todo!("Use arrange_ctx instead")
    }
}

impl DrawableContent for Note {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        let content: Vec<&dyn DrawableContent> = Vec::new();
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
            let bbox = self.scale_box(&cutout);

            let rect = Rect {
                xy: XY {
                    x: bbox.x_min,
                    y: bbox.y_min,
                },
                width: bbox.x_max - bbox.x_min,
                height: bbox.y_max - bbox.y_min,
                color: Color::TRANSPARENT,
                stroke_width: Some(0.2),
                stroke_color: Some(Color::RED),
            };

            result.push(rect.into());
        }

        result
    }
}
