use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::line::Line;
use crate::layout::Layout;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;

#[derive(Default)]
pub struct SectionMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub line_width: f32,
}

impl SectionMeasure {}

impl ScoreElement for SectionMeasure {
    fn _apply_layout(
        &mut self,
        layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.line_width = user_layout
            .light_barline
            .or(layout.appearance.light_barline)
            .unwrap_or(app_defaults.barline_light)
    }
}

impl Layoutable for SectionMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl DrawableContent for SectionMeasure {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        vec![]
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut elements: Vec<DrawableElement> = Vec::new();

        let stroke_color = self.color;
        let stroke_width = self.line_width;

        let right_line = Line {
            start: self.xy.mv(self.width, 0.),
            end: self.xy.mv(self.width, self.height),
            stroke_width,
            stroke_color,
        };
        elements.push(right_line.into());

        elements
    }
}
