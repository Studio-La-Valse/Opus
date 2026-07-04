use crate::app_defaults::AppDefaults;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::layout::Layout;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;

#[derive(Default)]
pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl StaffMeasure {}

impl ScoreElement for StaffMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
    }
}

impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Content for StaffMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        let result: Vec<&dyn Content> = Vec::new();

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
